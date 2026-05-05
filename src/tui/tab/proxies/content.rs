use super::super::dev::*;
use crate::functions::restful::proxies::{self};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::tree::{NodeType, ProxyTree};

#[derive(Default)]
pub struct Proxies {
    pub tree: ProxyTree,
    pub proxies: IndexMap<String, crate::functions::restful::proxies::Proxy>,
    pub error: Option<String>,
    pub testing_since: Option<Instant>,
}

impl Proxies {
    pub fn dispatch_key(
        &mut self,
        key: super::Key,
        task_set: &mut FutureSet<Self>,
        state: &mut ListState,
    ) {
        let current = state.selected().unwrap_or(0);

        match key {
            super::Key::MoveUp => {
                if current > 0 {
                    state.select(Some(current - 1));
                }
            }
            super::Key::MoveDown => {
                if current + 1 < self.tree.len() {
                    state.select(Some(current + 1));
                }
            }
            super::Key::Parent => {
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
            super::Key::Expand => {
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
            super::Key::Select => {
                let info = self.tree.node_at(current)
                    .map(|n| (n.name.clone(), n.node_type.clone(), n.parent.clone()));
                if let Some((name, ntype, parent)) = info {
                    match ntype {
                        NodeType::Folder => {
                            self.tree.toggle_expand_at(&name, &self.proxies);
                        }
                        NodeType::Link | NodeType::File => {
                            if let Some(ref parent) = parent {
                                let timeout_ms = crate::config::CONFIG.cfg_file.timeout.unwrap_or(5) * 1000;
                                let test_url = self.proxies.get(&name)
                                    .and_then(|p| p.test_url.clone())
                                    .or_else(|| crate::config::CONFIG.cfg_file.test_url.clone());
                                self.error = Some(format!("Switching to {name}..."));
                                self.testing_since = Some(Instant::now());
                                Self::spawn_select_inline(
                                    parent.clone(),
                                    name.clone(),
                                    test_url,
                                    timeout_ms,
                                    task_set,
                                );
                            }
                        }
                    }
                }
            }
            super::Key::CollapseAll => {
                self.tree.collapse_all(&self.proxies);
            }
            super::Key::ExpandAll => {
                self.tree.expand_all(&self.proxies);
            }
            super::Key::Refresh => {
                async {
                    let response = tri!(proxies::fetch_proxies(), or_set);
                    wrapper(move |content: &mut Self| {
                        content.proxies = response.proxies;
                        content.tree.rebuild_from_proxies(&content.proxies);
                        content.error = None;
                    })
                }
                .spawn_at(task_set);
            }
            super::Key::SortByName => {
                self.tree.sorted = !self.tree.sorted;
                self.tree.sort_by_delay = false;
                self.tree.rebuild_from_proxies(&self.proxies);
            }
            super::Key::SortByDelay => {
                self.tree.sort_by_delay = !self.tree.sort_by_delay;
                self.tree.sorted = false;
                self.tree.rebuild_from_proxies(&self.proxies);
            }
            super::Key::ResetSort => {
                self.tree.sorted = false;
                self.tree.sort_by_delay = false;
                self.tree.rebuild_from_proxies(&self.proxies);
            }
            super::Key::TestDelay => {
                let info = self.tree.node_at(current)
                    .map(|n| (n.name.clone(), n.node_type.clone()));
                if let Some((name, ntype)) = info {
                    let timeout = crate::config::CONFIG.cfg_file.timeout.unwrap_or(5) * 1000;
                    let test_url = self.proxies.get(&name)
                        .and_then(|p| p.test_url.clone())
                        .or_else(|| crate::config::CONFIG.cfg_file.test_url.clone());
                    match ntype {
                        NodeType::Folder => {
                            self.error = Some(format!("Testing group {name}..."));
                            self.testing_since = Some(Instant::now());
                            let t_secs = crate::config::CONFIG.cfg_file.timeout.unwrap_or(5).max(1) + 3;
                            let (n, tu) = (name.clone(), test_url.clone());
                            async move {
                                let delays = match tokio::time::timeout(
                                    Duration::from_secs(t_secs),
                                    tokio::task::spawn_blocking(move || proxies::test_group_delay(&n, tu.as_deref(), timeout))
                                ).await {
                                    Ok(Ok(Ok(v))) => v,
                                    Ok(Ok(Err(e))) => {
                                        crate::tui::widget::popmsg::Confirm::err(e);
                                        return wrapper(move |content: &mut Self| {
                                            content.testing_since = None;
                                        });
                                    }
                                    _ => {
                                        return wrapper(move |content: &mut Self| {
                                            content.error = Some("Speed test timed out".to_string());
                                            content.testing_since = None;
                                        });
                                    }
                                };
                                let mut response = match tokio::time::timeout(
                                    Duration::from_secs(t_secs),
                                    tokio::task::spawn_blocking(|| proxies::fetch_proxies())
                                ).await {
                                    Ok(Ok(Ok(r))) => r,
                                    _ => {
                                        return wrapper(move |content: &mut Self| {
                                            content.error = Some("Failed to refresh proxies after test".to_string());
                                            content.testing_since = None;
                                        });
                                    }
                                };
                                for (child_name, d) in &delays {
                                    if *d > 0 {
                                        if let Some(proxy) = response.proxies.get_mut(child_name) {
                                            proxy.history.push(proxies::DelayRecord { delay: *d });
                                        }
                                    }
                                }
                                wrapper(move |content: &mut Self| {
                                    content.proxies = response.proxies;
                                    content.tree.rebuild_from_proxies(&content.proxies);
                                    content.error = None;
                                    content.testing_since = None;
                                })
                            }
                            .spawn_at(task_set);
                        }
                        _ => {
                            self.error = Some(format!("Testing {name}..."));
                            self.testing_since = Some(Instant::now());
                            let t_secs = crate::config::CONFIG.cfg_file.timeout.unwrap_or(5).max(1) + 3;
                            let (n, tu) = (name.clone(), test_url.clone());
                            async move {
                                let delay = match tokio::time::timeout(
                                    Duration::from_secs(t_secs),
                                    tokio::task::spawn_blocking(move || proxies::test_proxy_delay(&n, tu.as_deref(), timeout))
                                ).await {
                                    Ok(Ok(Ok(v))) => v,
                                    Ok(Ok(Err(e))) => {
                                        return wrapper(move |content: &mut Self| {
                                            content.error = Some(e.to_string());
                                            content.testing_since = None;
                                        });
                                    }
                                    _ => {
                                        return wrapper(move |content: &mut Self| {
                                            content.error = Some("Speed test timed out".to_string());
                                            content.testing_since = None;
                                        });
                                    }
                                };
                                let mut response = match tokio::time::timeout(
                                    Duration::from_secs(t_secs),
                                    tokio::task::spawn_blocking(|| proxies::fetch_proxies())
                                ).await {
                                    Ok(Ok(Ok(r))) => r,
                                    _ => {
                                        return wrapper(move |content: &mut Self| {
                                            content.error = Some("Failed to refresh proxies after test".to_string());
                                            content.testing_since = None;
                                        });
                                    }
                                };
                                if let (Some(d), Some(proxy)) = (delay, response.proxies.get_mut(&name)) {
                                    if d > 0 {
                                        proxy.history.push(proxies::DelayRecord { delay: d });
                                    }
                                }
                                wrapper(move |content: &mut Self| {
                                    content.proxies = response.proxies;
                                    content.tree.rebuild_from_proxies(&content.proxies);
                                    content.error = None;
                                    content.testing_since = None;
                                })
                            }
                            .spawn_at(task_set);
                        }
                    }
                }
            }
            super::Key::TestAllDelay => {
                let folders: Vec<String> = self.tree.nodes.iter()
                    .filter(|n| n.node_type == NodeType::Folder)
                    .map(|n| n.name.clone())
                    .collect();
                let files: Vec<String> = self.tree.nodes.iter()
                    .filter(|n| n.node_type == NodeType::File && n.depth == 0)
                    .map(|n| n.name.clone())
                    .collect();
                let total = folders.len() + files.len();
                if total == 0 {
                    return;
                }
                let proxies_map = self.proxies.clone();
                let timeout = crate::config::CONFIG.cfg_file.timeout.unwrap_or(5) * 1000;
                self.error = Some(format!("Testing all ({total} groups/nodes)..."));
                self.testing_since = Some(Instant::now());
                async move {
                    let t_secs = crate::config::CONFIG.cfg_file.timeout.unwrap_or(5).max(1) + 3;
                    let mut all_delays: HashMap<String, u64> = HashMap::new();
                    for name in &folders {
                        let url = proxies_map.get(name.as_str())
                            .and_then(|p| p.test_url.clone())
                            .or_else(|| crate::config::CONFIG.cfg_file.test_url.clone());
                        let n = name.clone();
                        match tokio::time::timeout(
                            Duration::from_secs(t_secs),
                            tokio::task::spawn_blocking(move || proxies::test_group_delay(&n, url.as_deref(), timeout))
                        ).await {
                            Ok(Ok(Ok(delays))) => all_delays.extend(delays),
                            _ => {}
                        }
                    }
                    for name in &files {
                        let url = proxies_map.get(name.as_str())
                            .and_then(|p| p.test_url.clone())
                            .or_else(|| crate::config::CONFIG.cfg_file.test_url.clone());
                        let n = name.clone();
                        match tokio::time::timeout(
                            Duration::from_secs(t_secs),
                            tokio::task::spawn_blocking(move || proxies::test_proxy_delay(&n, url.as_deref(), timeout))
                        ).await {
                            Ok(Ok(Ok(Some(d)))) if d > 0 => {
                                all_delays.insert(name.clone(), d);
                            }
                            _ => {}
                        }
                    }
                    let mut response = match tokio::time::timeout(
                        Duration::from_secs(t_secs),
                        tokio::task::spawn_blocking(|| proxies::fetch_proxies())
                    ).await {
                        Ok(Ok(Ok(r))) => r,
                        _ => {
                            return wrapper(move |content: &mut Self| {
                                content.error = Some("Failed to refresh proxies after test".to_string());
                                content.testing_since = None;
                            });
                        }
                    };
                    for (name, d) in &all_delays {
                        if *d > 0 {
                            if let Some(proxy) = response.proxies.get_mut(name) {
                                proxy.history.push(proxies::DelayRecord { delay: *d });
                            }
                        }
                    }
                    wrapper(move |content: &mut Self| {
                        content.proxies = response.proxies;
                        content.tree.rebuild_from_proxies(&content.proxies);
                        content.error = None;
                        content.testing_since = None;
                    })
                }
                .spawn_at(task_set);
            }
        }
    }
}

impl BasicTabContent for Proxies {
    type Key = super::Key;
    type State = ListState;

    const TITLE: &str = "Proxies";

    fn all_shortcuts() -> &'static [(KeyCombo, Self::Key, &'static str)] {
        super::agent::all_shortcuts()
    }

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
        key: super::Key,
        task_set: &mut FutureSet<Self>,
        state: &mut Self::State,
    ) {
        self.dispatch_key(key, task_set, state);
    }

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State) {
        super::render::render(self, f, area, state);
    }
}

impl Proxies {
    fn spawn_select_inline(
        group: String,
        node: String,
        _test_url: Option<String>,
        _timeout_ms: u64,
        task_set: &mut FutureSet<Self>,
    ) {
        let t_secs = crate::config::CONFIG.cfg_file.timeout.unwrap_or(5).max(1) + 3;
        async move {
            let _ = tri!(proxies::select_proxy(&group, &node), or_cancel);
            let response = match tokio::time::timeout(
                Duration::from_secs(t_secs),
                tokio::task::spawn_blocking(|| proxies::fetch_proxies()),
            )
            .await
            {
                Ok(Ok(Ok(r))) => r,
                _ => {
                    return wrapper(move |content: &mut Self| {
                        content.error = None;
                        content.testing_since = None;
                    });
                }
            };
            wrapper(move |content: &mut Self| {
                content.proxies = response.proxies;
                content.tree.rebuild_from_proxies(&content.proxies);
                content.error = None;
                content.testing_since = None;
            })
        }
        .spawn_at(task_set);
    }
}
