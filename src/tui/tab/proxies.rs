use super::dev::*;
use crate::functions::restful::proxies::{self, ProxiesResponse};
use crossterm::event::KeyEventKind;
use indexmap::IndexMap;
use ratatui::text::Line;
use ratatui::widgets::ListItem;
use std::collections::HashMap;

newtype_tab!(ProxiesTab(Tab<Proxies>));

// ── Key ──

enum Key {
    MoveUp,
    MoveDown,
    Parent,
    Expand,
    Select,
    CollapseAll,
    ExpandAll,
    ToggleSort,
}

impl TryFrom<&KeyEvent> for Key {
    type Error = ();

    fn try_from(ev: &KeyEvent) -> Result<Self, Self::Error> {
        if ev.kind != KeyEventKind::Press {
            return Err(());
        }
        match ev.code {
            KeyCode::Up | KeyCode::Char('k') => Ok(Key::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Ok(Key::MoveDown),
            KeyCode::Char('h') => Ok(Key::Parent),
            KeyCode::Char('l') => Ok(Key::Expand),
            KeyCode::Enter => Ok(Key::Select),
            KeyCode::Char('f') => Ok(Key::CollapseAll),
            KeyCode::Char('e') => Ok(Key::ExpandAll),
            KeyCode::Char('s') => Ok(Key::ToggleSort),
            _ => Err(()),
        }
    }
}

macro_rules! tri {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                crate::tui::widget::popmsg::Confirm::err(e);
                return do_nothing();
            }
        }
    };
    ($e:expr, or_cancel) => {
        match $e {
            Ok(v) => v,
            Err(_) => return do_nothing(),
        }
    };
    ($e:expr, or_set) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                return wrapper(move |content: &mut Self| {
                    content.error = Some(e.to_string());
                });
            }
        }
    };
}

// ── ProxyTree ──
//
// Three node types:
//   Folder — actual folder at its "real" position in the tree.
//            Can expand to show children (Links for sub-folders, Files for leaves).
//   Link   — a cross-reference to a Folder that appears elsewhere.
//            Enter jumps to the Folder position.
//   File   — leaf node (no children). Enter selects if inside a Selector.
//
// The flat Vec order IS the render order.

#[derive(Clone, PartialEq)]
enum NodeType {
    Folder,
    Link,
    File,
}

#[derive(Clone)]
struct NodeItem {
    name: String,
    depth: usize,
    node_type: NodeType,
    proxy_type: String,
    delay: Option<u64>,
    parent: Option<String>,
    expanded: bool,
    is_now: bool,
}

struct ProxyTree {
    nodes: Vec<NodeItem>,
    name_index: HashMap<String, usize>,
    sorted: bool,
}

impl Default for ProxyTree {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            name_index: HashMap::new(),
            sorted: false,
        }
    }
}

impl ProxyTree {
    fn build(response: ProxiesResponse) -> Self {
        let proxies = response.proxies;
        let mut tree = ProxyTree::default();
        tree.rebuild_from_proxies(&proxies);
        tree
    }

    fn rebuild_from_proxies(&mut self, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        // Collect expanded state before rebuild
        let expanded_map: HashMap<String, bool> = self
            .nodes
            .iter()
            .filter(|n| n.expanded && n.node_type == NodeType::Folder)
            .map(|n| (n.name.clone(), true))
            .collect();

        let mut nodes = Vec::new();

        // Top-level: only groups (directories with children)
        let mut top: Vec<&str> = proxies
            .iter()
            .filter(|(_, p)| {
                !p.hidden && p.all.as_ref().map(|a| !a.is_empty()).unwrap_or(false)
            })
            .map(|(name, _)| name.as_str())
            .collect();

        if self.sorted {
            top.sort();
        } else {
            if let Some(global) = proxies.get("GLOBAL") {
                if let Some(ref group_all) = global.all {
                    let sort_index: Vec<&str> = group_all.iter().map(|s| s.as_str()).collect();
                    top.sort_by_key(|name| {
                        if *name == "GLOBAL" {
                            usize::MAX
                        } else {
                            sort_index.iter().position(|&s| s == *name).unwrap_or(usize::MAX - 1)
                        }
                    });
                }
            }
        }

        for name in &top {
            Self::push_entry(&mut nodes, name, None, None, 0, proxies, &expanded_map);
        }

        self.nodes = nodes;
        self.rebuild_index();
    }

    /// Push a top-level entry (Folder if it has children, otherwise File).
    /// If expanded, push its children as Link (for sub-groups) or File (for leaves).
    fn push_entry(
        nodes: &mut Vec<NodeItem>,
        name: &str,
        parent: Option<String>,
        parent_now: Option<&str>,
        depth: usize,
        proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>,
        expanded_map: &HashMap<String, bool>,
    ) {
        let proxy = match proxies.get(name) {
            Some(p) => p,
            None => return,
        };
        if proxy.hidden {
            return;
        }
        let has_kids = proxy.all.as_ref().map(|a| !a.is_empty()).unwrap_or(false);
        let node_type = if has_kids { NodeType::Folder } else { NodeType::File };
        let expanded = expanded_map.get(name).copied().unwrap_or(false);

        nodes.push(NodeItem {
            name: name.to_owned(),
            depth,
            node_type,
            proxy_type: proxy.proxy_type.clone(),
            delay: proxy.history.last().map(|r| r.delay),
            parent,
            expanded,
            is_now: parent_now == Some(name),
        });

        if has_kids && expanded {
            if let Some(ref kids) = proxy.all {
                let my_now = proxy.now.as_deref();
                for kid in kids {
                    let is_group = proxies
                        .get(kid.as_str())
                        .map(|p| p.all.as_ref().map(|a| !a.is_empty()).unwrap_or(false))
                        .unwrap_or(false);
                    if is_group {
                        // Sub-group → Link
                        nodes.push(NodeItem {
                            name: kid.clone(),
                            depth: depth + 1,
                            node_type: NodeType::Link,
                            proxy_type: String::new(),
                            delay: None,
                            parent: Some(name.to_owned()),
                            expanded: false,
                            is_now: my_now == Some(kid.as_str()),
                        });
                    } else {
                        // Leaf → File
                        let kid_proxy = proxies.get(kid.as_str());
                        nodes.push(NodeItem {
                            name: kid.clone(),
                            depth: depth + 1,
                            node_type: NodeType::File,
                            proxy_type: kid_proxy.map(|p| p.proxy_type.clone()).unwrap_or_default(),
                            delay: kid_proxy.and_then(|p| p.history.last().map(|r| r.delay)),
                            parent: Some(name.to_owned()),
                            expanded: false,
                            is_now: my_now == Some(kid.as_str()),
                        });
                    }
                }
            }
        }
    }

    fn toggle_expand_at(&mut self, name: &str, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        if let Some(idx) = self.find_folder_index(name) {
            self.nodes[idx].expanded = !self.nodes[idx].expanded;
            self.rebuild_from_proxies(proxies);
        }
    }

    fn expand_at(&mut self, name: &str, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        if let Some(idx) = self.find_folder_index(name) {
            self.nodes[idx].expanded = true;
            self.rebuild_from_proxies(proxies);
        }
    }

    fn collapse_at(&mut self, name: &str, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        if let Some(idx) = self.find_folder_index(name) {
            self.nodes[idx].expanded = false;
            self.rebuild_from_proxies(proxies);
        }
    }

    fn collapse_all(&mut self, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        for n in &mut self.nodes {
            n.expanded = false;
        }
        self.rebuild_from_proxies(proxies);
    }

    fn expand_all(&mut self, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        for n in &mut self.nodes {
            if n.node_type == NodeType::Folder {
                n.expanded = true;
            }
        }
        self.rebuild_from_proxies(proxies);
    }

    fn find_folder_index(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.node_type == NodeType::Folder && n.name == name)
    }

    fn rebuild_index(&mut self) {
        self.name_index.clear();
        for (i, node) in self.nodes.iter().enumerate() {
            self.name_index.insert(node.name.clone(), i);
        }
    }

    fn node_at(&self, idx: usize) -> Option<&NodeItem> {
        self.nodes.get(idx)
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

// ── Proxies ──

#[derive(Default)]
struct Proxies {
    tree: ProxyTree,
    proxies: IndexMap<String, crate::functions::restful::proxies::Proxy>,
    error: Option<String>,
}

impl Proxies {
    fn dispatch_key(
        &mut self,
        key: Key,
        task_set: &mut FutureSet<Self>,
        state: &mut ListState,
    ) {
        let current = state.selected().unwrap_or(0);

        match key {
            Key::MoveUp => {
                if current > 0 {
                    state.select(Some(current - 1));
                }
            }
            Key::MoveDown => {
                if current + 1 < self.tree.len() {
                    state.select(Some(current + 1));
                }
            }
            Key::Parent => {
                let info = self.tree.node_at(current)
                    .map(|n| (n.name.clone(), n.node_type.clone(), n.parent.clone()));
                if let Some((name, ntype, parent)) = info {
                    match ntype {
                        NodeType::Folder => {
                            self.tree.collapse_at(&name, &self.proxies);
                            if let Some(idx) = self.tree.find_folder_index(&name) {
                                state.select(Some(idx));
                            }
                        }
                        _ => {
                            if let Some(ref parent) = parent {
                                self.tree.collapse_at(parent, &self.proxies);
                                if let Some(idx) = self.tree.find_folder_index(parent) {
                                    state.select(Some(idx));
                                }
                            }
                        }
                    }
                }
            }
            Key::Expand => {
                let info = self.tree.node_at(current)
                    .map(|n| (n.name.clone(), n.node_type.clone(), n.parent.clone()));
                if let Some((name, ntype, _parent)) = info {
                    match ntype {
                        NodeType::Folder => {
                            self.tree.expand_at(&name, &self.proxies);
                        }
                        NodeType::Link => {
                            if let Some(idx) = self.tree.find_folder_index(&name) {
                                state.select(Some(idx));
                            }
                        }
                        NodeType::File => {}
                    }
                }
            }
            Key::Select => {
                let info = self.tree.node_at(current)
                    .map(|n| (n.name.clone(), n.node_type.clone(), n.parent.clone()));
                if let Some((name, ntype, parent)) = info {
                    match ntype {
                        NodeType::Folder => {
                            self.tree.toggle_expand_at(&name, &self.proxies);
                        }
                        NodeType::Link => {
                            if let Some(ref parent) = parent {
                                if let Some(&p_idx) = self.tree.name_index.get(parent) {
                                    if self.tree.nodes[p_idx].proxy_type == "Selector" {
                                        Self::spawn_select_inline(
                                            parent.clone(),
                                            name.clone(),
                                            task_set,
                                        );
                                        return;
                                    }
                                }
                            }
                            if let Some(idx) = self.tree.find_folder_index(&name) {
                                state.select(Some(idx));
                            }
                        }
                        NodeType::File => {
                            if let Some(ref parent) = parent {
                                if let Some(&p_idx) = self.tree.name_index.get(parent) {
                                    if self.tree.nodes[p_idx].proxy_type == "Selector" {
                                        Self::spawn_select_inline(
                                            parent.clone(),
                                            name.clone(),
                                            task_set,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Key::CollapseAll => {
                self.tree.collapse_all(&self.proxies);
            }
            Key::ExpandAll => {
                self.tree.expand_all(&self.proxies);
            }
            Key::ToggleSort => {
                self.tree.sorted = !self.tree.sorted;
                self.tree.rebuild_from_proxies(&self.proxies);
            }
        }
    }
}

impl BasicTabContent for Proxies {
    type Key = Key;
    type State = ListState;

    const TITLE: &str = "Proxies";

    fn after_sync(&self, task_set: &mut FutureSet<Self>) {
        async {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let response = tri!(proxies::fetch_proxies(), or_set);
            wrapper(|content: &mut Self| {
                content.proxies = response.proxies;
                content.tree.rebuild_from_proxies(&content.proxies);
                content.error = None;
            })
        }
        .spawn_at(task_set);
    }
}

impl TabContent for Proxies {
    fn init(&mut self, task_set: &mut FutureSet<Self>, _state: &mut Self::State) {
        self.error = Some("Loading proxies...".to_owned());
        async {
            let response = tri!(proxies::fetch_proxies());
            wrapper(|content: &mut Self| {
                content.proxies = response.proxies.clone();
                content.tree = ProxyTree::build(response);
                content.error = None;
            })
        }
        .spawn_at(task_set);
    }

    fn handle_key_event(
        &mut self,
        key: Key,
        task_set: &mut FutureSet<Self>,
        state: &mut Self::State,
    ) {
        self.dispatch_key(key, task_set, state);
    }

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State) {
        let block = Block::bordered()
            .border_style(Theme::get().tab.tab_focused)
            .title(Self::TITLE);

        let block = if self.tree.sorted {
            block.title_bottom(Line::raw(" sorted ").right_aligned().reversed())
        } else {
            block
        };

        if self.tree.is_empty() {
            if let Some(ref err) = self.error {
                let widget = ratatui::widgets::Paragraph::new(err.as_str()).block(block);
                f.render_widget(widget, area);
            }
            return;
        }

        let items: Vec<ListItem> = self
            .tree
            .nodes
            .iter()
            .map(|node| {
                let indent = "  ".repeat(node.depth);
                let prefix = match node.node_type {
                    NodeType::Folder => {
                        if node.expanded { "▼" } else { "▶" }
                    }
                    NodeType::Link => {
                        if node.is_now { "*" } else { " " }
                    }
                    NodeType::File => {
                        if node.is_now { "*" } else { " " }
                    }
                };
                let type_str = match node.node_type {
                    NodeType::Link => String::new(),
                    _ => format!("[{}]", node.proxy_type),
                };
                let delay_str = node.delay.map(|d| format!("{}ms", d)).unwrap_or_default();

                let line = format!(
                    "{indent} {prefix} {}  {}  {}",
                    node.name, type_str, delay_str,
                );

                let style = match node.node_type {
                    NodeType::Folder => Theme::get().tab.tab_focused,
                    NodeType::Link => ratatui::style::Style::default().fg(Color::Rgb(100, 180, 150)),
                    _ => ratatui::style::Style::default(),
                };

                ListItem::new(Line::styled(line, style))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Theme::get().tab.item_highlighted);

        f.render_stateful_widget(list, area, state);
    }
}

impl Proxies {
    fn spawn_select_inline(group: String, node: String, task_set: &mut FutureSet<Self>) {
        async move {
            let _ = tri!(proxies::select_proxy(&group, &node), or_cancel);
            let response = tri!(proxies::fetch_proxies(), or_cancel);
            wrapper(move |content: &mut Self| {
                content.proxies = response.proxies;
                content.tree.rebuild_from_proxies(&content.proxies);
                content.error = None;
            })
        }
        .spawn_at(task_set);
    }
}
