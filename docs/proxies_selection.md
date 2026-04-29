# ProxiesTab — API 调研与实现

## 一、Mihomo API

| 端点 | 方法 | 用途 |
|------|------|------|
| `/proxies` | GET | 获取所有代理信息（含策略组与节点） |
| `/proxies/<name>` | GET | 获取单个代理详情 |
| `/proxies/<name>` | PUT | 为 Selector 组选择节点 (`{"name":"节点名"}`) |
| `/proxies/<name>/delay` | GET | 对指定代理测速 (`?url=xxx&timeout=5000`) |
| `/group` | GET | 获取策略组信息 |
| `/group/<name>/delay` | GET | 对策略组内所有节点批量测速 |

base URL = `config.external_controller`（如 `http://127.0.0.1:9090`），认证 `Authorization: Bearer ${secret}`。

### 响应结构

```jsonc
{
  "proxies": {                        // 扁平 map，key = name
    "Entry": {
      "type": "Selector",
      "all": ["vmess-xxx", "DIRECT"], // 子节点引用 (DAG)
      "now": "vmess-xxx",             // 当前选中
      "history": [{"time": "...", "delay": 191}]
    },
    "vmess-xxx": {
      "type": "Vmess",
      "history": [{"time": "...", "delay": 191}]
    }
  }
}
```

`all` 字段形成 DAG 引用图。`hidden` 字段在 DIRECT/REJECT 等内置代理中缺失，需 `#[serde(default)]`。

### REST 封装

```rust
// src/functions/restful/proxies.rs
pub fn fetch_proxies()          -> Result<ProxiesResponse>  // GET  /proxies
pub fn select_proxy(g, n)       -> Result<()>               // PUT  /proxies/<g>  {"name": n}
pub fn test_proxy_delay(n,u,t)  -> Result<u64>              // GET  /proxies/<n>/delay
pub fn test_group_delay(n,u,t)  -> Result<()>               // GET  /group/<n>/delay
```

---

## 二、设计理念

NERDTree 风格文件浏览器 —— 策略组是文件夹，节点是文件/链接。

**核心决策：**

1. **Direct KeyCode match** — 不用 `mod_agent!` HashMap，直接 match `KeyCode`
2. **扁平 Vec + name_index** — 渲染顺序即存储顺序，O(1) 名字查找
3. **三种节点类型** — Folder（真实目录）、Link（交叉引用）、File（叶子节点）
4. **多组可同时展开** — 不像 clashctl 只展开一组，NERDTree 风格允许多目录展开

---

## 三、架构

```
┌─────────────────────────────────────────────────────────────┐
│  ProxiesTab                                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Proxies { tree, proxies, error }                    │    │
│  │  ┌─────────────────────────────────────────────┐    │    │
│  │  │  ProxyTree                                   │    │    │
│  │  │  nodes: Vec<NodeItem>    // 展平渲染顺序     │    │    │
│  │  │  name_index: HashMap     // name → idx      │    │    │
│  │  └─────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  render 直接遍历 nodes:                                       │
│   ▶ GLOBAL     [Selector]                                    │
│       vmess-1              ← Link，颜色区分，l 跳转到 Folder │
│       ss-2                 ← Link                           │
│   ▶ Entry      [Selector]                                    │
│     * vmess-2  [Vmess]     ← now，Enter 选择                 │
│       DIRECT               ← Link（跳到 DIRECT Folder）      │
│   ▶ DIRECT     [Direct]                                      │
│                                                              │
│  光标 = ListState.selected() → flat index → node_at(idx)    │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、数据结构

### 4.1 NodeItem

```rust
struct NodeItem {
    name: String,               // proxy 名称
    depth: usize,               // 缩进层级 (0 = 顶层)
    node_type: NodeType,        // Folder | Link | File
    proxy_type: String,         // "Selector" | "Vmess" | ...
    delay: Option<u64>,         // 延迟 (ms)
    parent: Option<String>,     // parent name (h 键回上层)
    expanded: bool,             // Folder 是否展开
    is_now: bool,               // 是否当前选中 (显示 *)
}
```

### 4.2 NodeType

| 类型 | 含义 | 前缀 | l 键 | Enter 键 |
|------|------|------|------|----------|
| `Folder` | 真实目录位置 | `▶`/`▼` | 展开子节点 | toggle 展开/折叠 |
| `Link` | 指向 Folder 的引用 | ` ` /`*` | 跳到目标 Folder | 选择（Selector 组内）/ 跳转 |
| `File` | 叶子节点 | ` `/`*` | — | 选择（Selector 组内） |

颜色区分：Folder 用 `tab_focused`，Link 用浅绿色 `Rgb(100,180,150)`，File 用默认颜色。
Link 若父组是 Selector 则 Enter 执行 PUT 选择，否则跳转到 Folder 位置。

### 4.3 ProxyTree

```rust
struct ProxyTree {
    nodes: Vec<NodeItem>,                  // 展平，顺序即渲染顺序
    name_index: HashMap<String, usize>,    // name → nodes[idx]
    sorted: bool,                          // true = 按字母排序
}
```

关键方法：
- `build(response)` → 从 ProxiesResponse 构建
- `rebuild_from_proxies(proxies)` → 保留展开状态重建
- `toggle_expand_at(name)` → 切换 Folder 展开/折叠
- `expand_at(name)` / `collapse_at(name)` → 展开/折叠指定 Folder
- `collapse_all()` / `expand_all()` → 全部折叠/展开
- `find_folder_index(name)` → 查找 Folder 的索引（线性扫描，不受 Link 干扰）
- `node_at(idx)` → 按索引获取节点

### 4.4 构建逻辑

```
build():
  1. 从 proxies map 筛选顶层：非 hidden 且 all 非空的策略组
  2. 排序：
     默认（sorted=false），按 GLOBAL.all 顺序排列，GLOBAL 放最后
     按 s 后（sorted=true），按字母顺序排列
  3. 对每个顶层组调用 push_entry():
     a. 生成 Folder (深度 0)
     b. 若 expanded: 遍历 all，子项是策略组 → Link，否则 → File (深度+1)
  4. 重建 name_index

注意：`rebuild_index` 为所有节点建索引，当同名 Folder 和 Link（如 GLOBAL 的子项）同时存在时，Link 会覆盖 Folder 条目。因此 `expand_at`、`toggle_expand_at`、`collapse_at` 使用 `find_folder_index` 线性扫描而非 `name_index` 查找。
```

---

## 五、按键系统

| 键 | 动作 | 行为 |
|----|------|------|
| `j` / `↓` | MoveDown | 光标下移 |
| `k` / `↑` | MoveUp | 光标上移 |
| `h` | Parent | Folder: 折叠自身 / Link/File: 折叠父目录并跳转 |
| `l` | Expand | Folder: 展开 / Link: 跳到目标 Folder / File: 无操作 |
| `Enter` | Select | Folder: toggle 展开 / Link: 跳到目标 / File: PUT 选择 |
| `f` | CollapseAll | 折叠全部 Folder |
| `e` | ExpandAll | 展开全部 Folder |
| `s` | ToggleSort | 开关排序（字母顺序 ↔ GLOBAL.all 顺序） |

```rust
impl TryFrom<&KeyEvent> for Key {
    fn try_from(ev: &KeyEvent) -> Result<Self, ()> {
        if ev.kind != KeyEventKind::Press { return Err(()); }
        match ev.code {
            KeyCode::Up    | KeyCode::Char('k') => Ok(Key::MoveUp),
            KeyCode::Down  | KeyCode::Char('j') => Ok(Key::MoveDown),
            KeyCode::Char('h')                   => Ok(Key::Parent),
            KeyCode::Char('l')                   => Ok(Key::Expand),
            KeyCode::Enter                       => Ok(Key::Select),
            KeyCode::Char('f')                   => Ok(Key::CollapseAll),
            KeyCode::Char('e')                   => Ok(Key::ExpandAll),
            KeyCode::Char('s')                   => Ok(Key::ToggleSort),
            _ => Err(()),
        }
    }
}
```

---

## 六、文件结构

```
src/
├── functions/restful/proxies.rs     REST API 封装
│   ├── ProxiesResponse, Proxy, DelayRecord
│   ├── fetch_proxies()              GET  /proxies
│   ├── select_proxy(group, node)    PUT  /proxies/<group>
│   ├── test_proxy_delay(name)       GET  /proxies/<name>/delay
│   └── test_group_delay(name)       GET  /group/<name>/delay
│
└── tui/tab/proxies.rs              TUI 实现
    ├── Key enum + TryFrom<&KeyEvent>
    ├── NodeType (Folder/Link/File), NodeItem
    ├── ProxyTree { nodes: Vec<NodeItem>, name_index }
    │   ├── build() / rebuild_from_proxies()
    │   ├── toggle_expand_at() / expand_at() / collapse_at()
    │   ├── collapse_all() / expand_all()
    │   ├── find_folder_index() / node_at() / len()
    │   └── push_entry() / rebuild_index()
    ├── Proxies (TabContent impl)
    │   ├── dispatch_key()     按键分发
    │   ├── render()            展平渲染 (ListItem)
    │   └── spawn_select_inline()  PUT /proxies/<group>
    └── ProxiesTab (newtype_tab!)
```

---

## 八、TODO

- **测速** — 单节点/组测速，显示延迟 ms。测速进行中用 spinner 动画指示（`/` `-` `\` `|` 循环）。
- **侧边栏** — 大屏时显示节点详情面板。

---

## 九、参考

- [Mihomo API](https://wiki.metacubex.one/api/)
