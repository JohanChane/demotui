use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq)]
pub enum NodeType {
    Folder,
    Link,
    File,
}

#[derive(Clone)]
pub struct NodeItem {
    pub name: String,
    pub depth: usize,
    pub node_type: NodeType,
    pub proxy_type: String,
    pub delay: Option<u64>,
    pub parent: Option<String>,
    pub expanded: bool,
    pub is_now: bool,
}

pub struct ProxyTree {
    pub nodes: Vec<NodeItem>,
    pub name_index: HashMap<String, usize>,
    pub sorted: bool,
    pub sort_by_delay: bool,
}

impl Default for ProxyTree {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            name_index: HashMap::new(),
            sorted: false,
            sort_by_delay: false,
        }
    }
}

impl ProxyTree {
    pub fn build(response: crate::functions::restful::proxies::ProxiesResponse) -> Self {
        let proxies = response.proxies;
        let mut tree = ProxyTree::default();
        tree.rebuild_from_proxies(&proxies);
        tree
    }

    pub fn rebuild_from_proxies(&mut self, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
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
            Self::push_entry(&mut nodes, name, None, None, 0, proxies, &expanded_map, self.sort_by_delay);
        }

        self.nodes = nodes;
        self.rebuild_index();
    }

    /// Push a top-level entry (Folder if it has children, otherwise File).
    /// If expanded, push its children as Link (for sub-groups) or File (for leaves).
    pub fn push_entry(
        nodes: &mut Vec<NodeItem>,
        name: &str,
        parent: Option<String>,
        parent_now: Option<&str>,
        depth: usize,
        proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>,
        expanded_map: &HashMap<String, bool>,
        sort_by_delay: bool,
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
            delay: None,
            parent,
            expanded,
            is_now: parent_now == Some(name),
        });

        if has_kids && expanded {
            if let Some(ref kids) = proxy.all {
                let my_now = proxy.now.as_deref();
                let ordered_kids: Vec<&String> = if sort_by_delay {
                    let mut v: Vec<&String> = kids.iter().collect();
                    v.sort_by_key(|kid| {
                        proxies.get(kid.as_str())
                            .and_then(|p| p.history.last())
                            .and_then(|r| if r.delay > 0 { Some(r.delay) } else { None })
                            .or_else(|| {
                                proxy.extra.get(kid.as_str())
                                    .and_then(|info| info.history.last())
                                    .and_then(|r| if r.delay > 0 { Some(r.delay) } else { None })
                            })
                            .unwrap_or(u64::MAX)
                    });
                    v
                } else {
                    kids.iter().collect()
                };
                for kid in &ordered_kids {
                    let is_group = proxies
                        .get(kid.as_str())
                        .map(|p| p.all.as_ref().map(|a| !a.is_empty()).unwrap_or(false))
                        .unwrap_or(false);
                    if is_group {
                        let kid_proxy = proxies.get(kid.as_str());
                        let has_data = kid_proxy.map(|p| !p.history.is_empty()).unwrap_or(false)
                            || proxy.extra.get(kid.as_str()).map(|info| !info.history.is_empty()).unwrap_or(false);
                        nodes.push(NodeItem {
                            name: (*kid).clone(),
                            depth: depth + 1,
                            node_type: NodeType::Link,
                            proxy_type: kid_proxy.map(|p| p.proxy_type.clone()).unwrap_or_default(),
                            delay: kid_proxy.and_then(|p| p.history.last()
                                .and_then(|r| if r.delay > 0 { Some(r.delay) } else { None })
                            ).or_else(|| {
                                proxy.extra.get(kid.as_str())
                                    .and_then(|info| info.history.last())
                                    .and_then(|r| if r.delay > 0 { Some(r.delay) } else { None })
                            }).or_else(|| {
                                resolve_now_delay(kid.as_str(), proxies)
                            }).or(if has_data { Some(0) } else { None }),
                            parent: Some(name.to_owned()),
                            expanded: false,
                            is_now: my_now == Some(kid.as_str()),
                        });
                    } else {
                        let kid_proxy = proxies.get(kid.as_str());
                        let has_data = kid_proxy.map(|p| !p.history.is_empty()).unwrap_or(false)
                            || proxy.extra.get(kid.as_str()).map(|info| !info.history.is_empty()).unwrap_or(false);
                        nodes.push(NodeItem {
                            name: (*kid).clone(),
                            depth: depth + 1,
                            node_type: NodeType::File,
                            proxy_type: kid_proxy.map(|p| p.proxy_type.clone()).unwrap_or_default(),
                            delay: kid_proxy.and_then(|p| p.history.last()
                                .and_then(|r| if r.delay > 0 { Some(r.delay) } else { None })
                            ).or_else(|| {
                                proxy.extra.get(kid.as_str())
                                    .and_then(|info| info.history.last())
                                    .and_then(|r| if r.delay > 0 { Some(r.delay) } else { None })
                            }).or(if has_data { Some(0) } else { None }),
                            parent: Some(name.to_owned()),
                            expanded: false,
                            is_now: my_now == Some(kid.as_str()),
                        });
                    }
                }
            }
        }
    }

    pub fn toggle_expand_at(&mut self, name: &str, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        if let Some(idx) = self.find_folder_index(name) {
            self.nodes[idx].expanded = !self.nodes[idx].expanded;
            self.rebuild_from_proxies(proxies);
        }
    }

    pub fn expand_at(&mut self, name: &str, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        if let Some(idx) = self.find_folder_index(name) {
            self.nodes[idx].expanded = true;
            self.rebuild_from_proxies(proxies);
        }
    }

    pub fn collapse_at(&mut self, name: &str, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        if let Some(idx) = self.find_folder_index(name) {
            self.nodes[idx].expanded = false;
            self.rebuild_from_proxies(proxies);
        }
    }

    pub fn collapse_all(&mut self, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        for n in &mut self.nodes {
            n.expanded = false;
        }
        self.rebuild_from_proxies(proxies);
    }

    pub fn expand_all(&mut self, proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>) {
        for n in &mut self.nodes {
            if n.node_type == NodeType::Folder {
                n.expanded = true;
            }
        }
        self.rebuild_from_proxies(proxies);
    }

    pub fn find_folder_index(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.node_type == NodeType::Folder && n.name == name)
    }

    pub fn rebuild_index(&mut self) {
        self.name_index.clear();
        for (i, node) in self.nodes.iter().enumerate() {
            self.name_index.insert(node.name.clone(), i);
        }
    }

    pub fn node_at(&self, idx: usize) -> Option<&NodeItem> {
        self.nodes.get(idx)
    }
}

/// Follow `now` chain from a group/link proxy name to the ultimate leaf node,
/// returning its delay from `history`. Used when link nodes lack their own delay
/// data (Mihomo stores child delays in the parent's `extra`, which goes stale
/// after a selection without re-test).
fn resolve_now_delay(
    start: &str,
    proxies: &IndexMap<String, crate::functions::restful::proxies::Proxy>,
) -> Option<u64> {
    let mut current = start.to_string();
    let mut prev: Option<String> = None;
    let mut visited = HashSet::new();
    loop {
        if !visited.insert(current.clone()) {
            return None; // cycle
        }
        let proxy = proxies.get(current.as_str())?;
        let has_kids = proxy.all.as_ref().map(|a| !a.is_empty()).unwrap_or(false);
        if !has_kids {
            if let Some(d) = proxy.history.last().and_then(|r| if r.delay > 0 { Some(r.delay) } else { None }) {
                return Some(d);
            }
            if let Some(ref parent) = prev {
                if let Some(p) = proxies.get(parent.as_str()) {
                    if let Some(info) = p.extra.get(current.as_str()) {
                        return info.history.last().and_then(|r| if r.delay > 0 { Some(r.delay) } else { None });
                    }
                }
            }
            return None;
        }
        prev = Some(current.clone());
        match proxy.now.as_deref() {
            Some(now) if now != current => current = now.to_string(),
            _ => {
                return proxy.history.last().and_then(|r| if r.delay > 0 { Some(r.delay) } else { None });
            }
        }
    }
}

impl ProxyTree {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}
