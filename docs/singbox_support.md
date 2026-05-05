# sing-box Support Analysis

> **Last verified**: 2026-05-06
> **References**:
> - [sing-box docs](https://sing-box.sagernet.org/) — authoritative sing-box reference
> - [mihomo docs](https://wiki.metacubex.one/) — authoritative mihomo reference
> - [v2rayn core_api.md](https://github.com/2dust/v2rayN) — API comparison & compat layer
> - [v2rayn design_zh.md](https://github.com/2dust/v2rayN) — multi-core architecture

This document maps the differences between **sing-box** and **mihomo** across three domains — REST API, configuration format, and CLI/bin interface — with the goal of informing a phased plan to add sing-box support to demotui.

---

## 1. REST API Differences

### 1.1 Overview: sing-box's `clash_api`

sing-box implements a **partial mihomo (Clash) REST API compatibility layer** under `experimental.clash_api`. It is NOT a complete clone. The compatibility is sufficient for proxy management, connection viewing, and config updates — but there are key differences, particularly around traffic statistics and mihomo-specific endpoints.

```
sing-box config:
{
  "experimental": {
    "clash_api": {
      "external_controller": "127.0.0.1:9090",
      "secret": "your-secret"
    }
  }
}

mihomo config:
external-controller: '127.0.0.1:9090'
secret: 'your-secret'
```

### 1.2 Endpoint Comparison Table

| Endpoint | Method | mihomo | sing-box (`clash_api`) | Status | Notes |
|---|---|---|---|---|---|
| `/proxies` | `GET` | Full proxy tree with groups | Full proxy tree with groups | **Compatible** | Response structure nearly identical. sing-box uses `type` field for proxy group type (Selector, URLTest, etc.) |
| `/proxies/{name}` | `GET` | Single proxy/group info | Single proxy/group info | **Compatible** | |
| `/proxies/{name}` | `PUT` | Switch selector node | Switch selector node | **Compatible** | Header-based (`{name: nodeName}`) |
| `/proxies/{name}/delay` | `GET` | Delay test result `{delay: ms}` | Delay test result `{delay: ms}` | **Compatible** | `?url=xxx&timeout=5000` query params |
| `/connections` | `GET` | Active connections + downloadTotal/uploadTotal | Active connections (no totals) | **Field differences** | sing-box connections lack `type`, `nsMode` fields in metadata. No `downloadTotal`/`uploadTotal` in sing-box — use `/traffic` WebSocket instead |
| `/connections/{id}` | `DELETE` | Close single connection | Close single connection | **Compatible** | |
| `/connections` | `DELETE` | Close all connections | Close all connections | **Compatible** | |
| `/configs` | `GET` | Basic running config | Basic running config | **Different** | sing-box returns different config structure (JSON-native, not YAML). `mode` field is the only guaranteed overlap |
| `/configs` | `PUT` | Reload config from file | Not available | **Missing** | sing-box uses signals (SIGHUP) for config reload, not REST API |
| `/configs` | `PATCH` | Update running settings (`mode`, `log-level`, etc.) | Partially available | **Limited** | sing-box only supports `mode` patch. Other mihomo config fields (port, TUN, etc.) have no API equivalent |
| `/restart` | `POST` | Soft restart core | Not available | **Missing** | sing-box has no in-process restart; must stop/start externally |
| `/version` | `GET` | Core version string | Core version string | **Compatible** | |
| `/traffic` | `GET` / `WS` | Traffic in kbps (HTTP poll or WebSocket) | Traffic in bytes (WebSocket push) | **Different model** | See §1.3 |
| `/group` | `GET` | Policy group info | Not available separately | **Missing** | sing-box embeds groups in `/proxies` response |
| `/providers/proxies` | `GET` | Proxy provider info | Limited | **Partial** | sing-box proxy providers accessible via `/proxies` hierarchy |
| `/rules` | `GET` | Rule info | Not available | **Missing** | |
| `/logs` | `GET` / `WS` | Real-time logs | Not available in `clash_api` | **Missing** | |
| `/dns/query` | `GET` | DNS query | Not available | **Missing** | |

### 1.3 Traffic Statistics: The Critical Difference

This is the most significant API discrepancy.

**mihomo**:  
Provides `downloadTotal` and `uploadTotal` as fields in the `/connections` response. demotui's `ConnInfo` struct (`src/functions/restful.rs:200`) parses these cumulative byte counters to derive speed. Can also use WebSocket `/traffic` for real-time push.

**sing-box**:  
Provides traffic via a **WebSocket-only** `/traffic` endpoint. Data is pushed as a JSON stream:
```json
{ "up": 123456789, "down": 987654321 }
```
- Units: **bytes** (cumulative)
- Mode: **push** (server pushes updates; client must maintain WebSocket connection)
- No proxy/direct split — only total up/down
- No equivalent in REST `/connections` response

**v2rayn's approach**: `StatisticsSingboxService` wraps the WebSocket into a polling-like interface, computes speed deltas between messages, and feeds the same `ServerSpeedItem` model as mihomo. sing-box traffic lacks proxy/direct split, so only `ProxyUp`/`ProxyDown` are populated.

**Implication for demotui**: The current `ConnInfo`-based traffic model (poll REST `/connections`) must be extended with a WebSocket-based alternative for sing-box. The two sources must normalize to the same internal representation.

### 1.4 Connection Metadata Field Differences

sing-box's `/connections` response has **fewer metadata fields** than mihomo's:

| Field | mihomo | sing-box | Impact on demotui |
|---|---|---|---|
| `metadata.type` | Present (e.g., "http", "socks") | **Missing** | demotui's `ConnMetaData.ctype` will be empty/null for sing-box |
| `metadata.nsMode` | Present | **Missing** | Not used by demotui |
| `metadata.process` | Present | Present (Linux) | Same |
| `metadata.processPath` | Present | Present (Linux) | Same |
| `metadata.host` | Present | Present | Same |
| `metadata.network` | Present | Present | Same |
| `metadata.destinationIP` | Present | Present | Same |
| `metadata.destinationPort` | Present | Present | Same |
| `metadata.sourceIP` | Present | Present | Same |
| `metadata.sourcePort` | Present | Present | Same |
| `uid` | Present | Present | Same |
| `download` | Present | Present | Same |
| `upload` | Present | Present | Same |
| `start` | Present | Present | Same |
| `chains` | Present | Present | Same |
| `rule` | Present | Present | Same |
| `rulePayload` | Present | Present | Same |

### 1.5 API Compatibility Verdict

| Category | Verdict | Action |
|---|---|---|
| Proxy management (`/proxies`) | Drop-in compatible | Reuse existing REST client |
| Connection display (`/connections`) | Compatible with minor field nulls | Make `ctype` field `Option<String>` or default to empty string |
| Connection close | Drop-in compatible | Reuse existing REST client |
| Config read (`GET /configs`) | Structurally different | Needs sing-box-specific config deser |
| Config update (`PATCH /configs`) | Limited to `mode` only | Accept reduced functionality for sing-box |
| Config reload (`PUT /configs`) | Not available | Use external process restart |
| Restart | Not available | Use systemd/process restart |
| Traffic stats | Different model (WS vs poll) | **New implementation needed**: WebSocket client + speed delta calculation |
| Logs, rules, DNS query | Not available | Unavailable in sing-box (inform user) |

---

## 2. Configuration Format Differences

### 2.1 Top-Level Structure

| Aspect | mihomo (YAML) | sing-box (JSON) |
|---|---|---|
| Format | YAML | JSON |
| Config file | Single file (can use `proxy-providers` for external files) | Single JSON file |
| Proxy nodes | `proxies:` list | `outbounds[]` array |
| Proxy groups | `proxy-groups:` list | `outbounds[]` with `type: "selector"` / `type: "urltest"` |
| Routing rules | `rules:` inline string matchers | `route.rules[]` object array + `route.rule_set[]` external refs |
| DNS | `dns.nameserver:` / `dns.nameserver-policy:` | `dns.servers[]` with `address` / `tag` / `detour` |
| TUN | `tun.enable: true` / `tun.stack:` | `inbounds[]` with `type: "tun"` |
| Inbound listeners | `listeners:` / `mixed-port:` / `port:` / `socks-port:` | `inbounds[]` array (each `type` is a listener) |
| API binding | `external-controller:` + `secret:` (top-level) | `experimental.clash_api.external_controller` + `.secret` |
| Cache | `profile.store-selected:` etc. | `experimental.cache_file` or deprecated `experimental.clash_api.cache_file` |
| Log | `log-level:` | `log.level:` / `log.output:` |
| Geo data | `geodata-mode:` / `geo-auto-update:` | N/A (rule_set replaces role) |
| External UI | `external-ui:` / `external-ui-url:` | `experimental.clash_api.external_ui` / `.external_ui_download_url` |

### 2.2 Proxy Node Representation: VLESS+Reality+WebSocket Example

**mihomo YAML**:
```yaml
proxies:
  - name: "example-vless"
    type: vless
    server: example.com
    port: 443
    uuid: bf000d23-0752-40b4-affe-68f7707a9661
    flow: xtls-rprx-vision
    tls: true
    servername: example.com
    fingerprint: chrome
    reality-opts:
      public-key: xxxxx
      short-id: xxxx
    network: ws
    ws-opts:
      path: /ws
      headers:
        Host: example.com
    smux:
      enabled: true
```

**sing-box JSON**:
```json
{
  "type": "vless",
  "tag": "example-vless",
  "server": "example.com",
  "server_port": 443,
  "uuid": "bf000d23-0752-40b4-affe-68f7707a9661",
  "flow": "xtls-rprx-vision",
  "tls": {
    "enabled": true,
    "server_name": "example.com",
    "utls": {
      "enabled": true,
      "fingerprint": "chrome"
    },
    "reality": {
      "enabled": true,
      "public_key": "xxxxx",
      "short_id": "xxxxx"
    }
  },
  "transport": {
    "type": "ws",
    "path": "/ws",
    "headers": {
      "Host": "example.com"
    }
  },
  "multiplex": {
    "enabled": true,
    "protocol": "smux"
  }
}
```

**Key structural differences**:
1. Names: `name`/`port`/`server` vs `tag`/`server_port`/`server`
2. TLS: flat booleans + nested options vs `tls: { enabled: true, ... }` object
3. Transport: inline type-specific options (`ws-opts:`) vs `transport: { type: "ws", ... }`
4. Multiplex: `smux: { enabled: true }` vs `multiplex: { enabled: true, protocol: "smux" }`
5. Reality: `reality-opts: { ... }` vs `tls.reality: { enabled: true, ... }`

### 2.3 Routing Rule Representation

**mihomo** — Inline string matchers in a single list:
```yaml
rules:
  - DOMAIN-SUFFIX,google.com,Proxy
  - DOMAIN-KEYWORD,chat,Proxy
  - GEOSITE,google,Proxy
  - GEOIP,CN,DIRECT
  - MATCH,Proxy
```

**sing-box** — Object array with `rule_set` external references:
```json
{
  "route": {
    "rules": [
      { "domain_suffix": ["google.com"], "outbound": "proxy" },
      { "domain_keyword": ["chat"], "outbound": "proxy" },
      { "rule_set": "geosite-google", "outbound": "proxy" },
      { "rule_set": "geoip-cn", "outbound": "direct" },
      { "outbound": "proxy" }
    ],
    "rule_set": [
      { "tag": "geosite-google", "type": "remote", "format": "binary",
        "url": "https://...geosite-google.srs" },
      { "tag": "geoip-cn", "type": "remote", "format": "binary",
        "url": "https://...geoip-cn.srs" }
    ]
  }
}
```

**Key differences**:
- mihomo uses string-based inline rules; sing-box uses JSON objects
- sing-box `rule_set` references external binary files (`.srs` format), replacing mihomo's implicit geo data loading
- Rule types differ: `DOMAIN-SUFFIX` → `domain_suffix`, `DOMAIN-KEYWORD` → `domain_keyword`, `GEOIP`/`GEOSITE` → `rule_set` references
- sing-box adds `process_name`, `process_path`, `wifi_ssid` matchers not available in mihomo's inline rules

### 2.4 DNS Configuration

**mihomo**:
```yaml
dns:
  enable: true
  listen: 0.0.0.0:53
  ipv6: true
  enhanced-mode: fake-ip
  fake-ip-range: 198.18.0.1/16
  nameserver:
    - 223.5.5.5
    - 119.29.29.29
  nameserver-policy:
    "geosite:cn,private":
      - 223.5.5.5
  fallback:
    - 8.8.8.8
    - 1.1.1.1
```

**sing-box**:
```json
{
  "dns": {
    "servers": [
      { "tag": "dns-direct", "address": "223.5.5.5", "detour": "direct" },
      { "tag": "dns-proxy", "address": "8.8.8.8", "detour": "proxy" },
      { "tag": "dns-fakeip", "address": "fakeip" }
    ],
    "rules": [
      { "rule_set": "geosite-cn", "server": "dns-direct" },
      { "rule_set": "geosite-geolocation-!cn", "server": "dns-proxy" }
    ]
  }
}
```

**Key differences**: mihomo has dedicated `nameserver`, `fallback`, `nameserver-policy` keys. sing-box unifies all DNS under `servers[]` with explicit routing via `rules[]`. FakeIP is a server type (`"fakeip"`) rather than an enhancement mode.

### 2.5 TUN Configuration

**mihomo**:
```yaml
tun:
  enable: true
  stack: system  # or gvisor, mixed
  dns-hijack:
    - any:53
  auto-route: true
  auto-detect-interface: true
```

**sing-box** — TUN is an inbound, not a top-level config section:
```json
{
  "inbounds": [
    {
      "type": "tun",
      "tag": "tun-in",
      "interface_name": "tun0",
      "inet4_address": "172.19.0.1/30",
      "auto_route": true,
      "strict_route": true,
      "stack": "system",   // or gvisor, mixed
      "sniff": true
    }
  ]
}
```

**Key difference**: In mihomo, TUN is a top-level feature toggled `on/off`. In sing-box, TUN is one of many inbound types, enabling more flexible multi-listener configurations.

### 2.6 API/`clash_api` Binding

| Key | mihomo | sing-box |
|---|---|---|
| Enable API | `external-controller: '127.0.0.1:9090'` | `experimental.clash_api.external_controller` |
| API Secret | `secret: 'xxx'` | `experimental.clash_api.secret` |
| CORS | `external-controller-cors: { allow-origins: ... }` | `experimental.clash_api.access_control_allow_origin` |
| Unix socket | `external-controller-unix: mihomo.sock` | Not available |
| HTTPS API | `external-controller-tls:` | Not available |
| Pipename (Win) | `external-controller-pipe:` | Not available |

---

## 3. CLI/Bin Command Differences

### 3.1 General Pattern

| Aspect | mihomo | sing-box |
|---|---|---|
| Binary name | `mihomo` (or `clash-meta`) | `sing-box` |
| Command style | Flags directly on binary | Subcommand-based (`sing-box <cmd>`) |
| Config flag | `-f <file>` | `-c <file>` (on subcommand) |
| Working dir flag | `-d <dir>` | `-D <dir>` (note: uppercase) |
| Test config | `-t` flag (combined with `-d`/`-f`) | `check` subcommand |
| Run | Default mode (no subcommand) | `run` subcommand |
| Version | `-v` | `version` subcommand |

### 3.2 Command Comparison

#### Config Validation
```
mihomo:   mihomo -t -d /etc/mihomo -f config.yaml
sing-box: sing-box check -c /etc/sing-box/config.json
```

**Key differences**:
- mihomo uses flag-style (`-t`); sing-box uses a dedicated `check` subcommand
- mihomo requires explicit working directory (`-d`); sing-box resolves paths relative to config file or uses `-D`
- sing-box `check` validates JSON schema, outbound connectivity is not tested

#### Run/Daemon
```
mihomo:   mihomo -d /etc/mihomo -f config.yaml
sing-box: sing-box run -c /etc/sing-box/config.json -D /etc/sing-box
```

**Key differences**:
- sing-box `run` subcommand is mandatory
- Working directory flag: `-d` (mihomo) vs `-D` (sing-box, uppercase)
- mihomo auto-daemonizes on Linux; sing-box runs in foreground by default, can use `--disable-color`

#### Version
```
mihomo:   mihomo -v
sing-box: sing-box version
```

#### Other sing-box Subcommands
| Subcommand | Purpose |
|---|---|
| `sing-box run` | Run the proxy server |
| `sing-box check` | Validate configuration file |
| `sing-box version` | Print version |
| `sing-box generate` | Generate crypto keys/config templates |
| `sing-box format` | Format/prettify JSON config |
| `sing-box merge` | Merge multiple JSON configs |
| `sing-box tools` | Run network diagnostic tools |

### 3.3 Service Management

Both cores use the same patterns on Linux:

| Action | mihomo service | sing-box service |
|---|---|---|
| Systemd unit | `mihomo.service` (user-defined name) | `sing-box.service` (from package install) |
| Start | `systemctl start mihomo` | `systemctl start sing-box` |
| Stop | `systemctl stop mihomo` | `systemctl stop sing-box` |
| Restart | `systemctl restart mihomo` | `systemctl restart sing-box` |
| Status | `systemctl is-active mihomo` | `systemctl is-active sing-box` |
| Logs | `journalctl -u mihomo -f` | `journalctl -u sing-box -f` |

**Implication for demotui**: The current service control (`src/tui/tab/srvctl.rs`, `src/functions/command.rs`) is hardcoded to mihomo with a configurable service name. To support sing-box, the service name and binary path must be per-core, and config validation must dispatch to the correct command.

### 3.4 Binary Naming & Paths

| Platform | mihomo typical path | sing-box typical path |
|---|---|---|
| Linux (package) | `/usr/bin/mihomo` | `/usr/bin/sing-box` |
| Linux (manual) | Varies | `~/.local/bin/sing-box` |
| macOS (brew) | `/usr/local/bin/mihomo` | `/usr/local/bin/sing-box` |

---

## 4. v2rayn's Compatibility Approach

v2rayn (Windows/macOS/Linux) already supports both mihomo and sing-box with full feature parity. Its architecture offers practical patterns for demotui.

### 4.1 Unified Clash API Client

v2rayn treats sing-box and mihomo as **identical REST API clients** via `ClashApiManager`. The decision logic:

```
if (IsRunningCore(ECoreType.sing_box) || IsRunningCore(ECoreType.mihomo)) {
    // Same ClashApiManager for both
    // proxies, connections, config endpoints — all shared
}
```

**Why this works**: sing-box's `clash_api` was designed as a drop-in replacement for mihomo's API. The proxy tree structure, selector switching, delay testing, and connection display all follow the same JSON contract.

**The exception — traffic stats**: v2rayn has two separate stat services:
- `StatisticsSingboxService` — wraps sing-box's WebSocket `/traffic` (push model, bytes per message)
- Mihomo stats — pulled from `GET /connections` `downloadTotal`/`uploadTotal`

Both normalize to the same `ServerSpeedItem` model and call the same `UpdateServerStatHandler` callback.

### 4.2 Per-Core Config Generation (No Shared Abstraction)

v2rayn **intentionally keeps config generation separate** for each core. There is no shared interface or abstract config class:

```
CoreConfigHandler.GenerateClientConfig():
  if (RunCoreType == sing_box):
    → CoreConfigSingboxService    (generates SingboxConfig JSON)
  elif (RunCoreType == mihomo):
    → CoreConfigClashService      (generates YAML)
  else:
    → CoreConfigV2rayService      (generates Xray JSON)
```

**Design rationale** (from v2rayn docs):
> Xray and sing-box have fundamentally different config schemas — a shared interface would be a leaky abstraction. The number of cores is small and stable (3 needing config generation) — the cost of abstraction exceeds the benefit.

Each core's config generator is organized as parallel partial classes covering the same concerns:
| Concern | sing-box | mihomo |
|---|---|---|
| Outbound generation | `SingboxOutboundService` | Built into YAML serialization |
| Inbound generation | `SingboxInboundService` | Built into YAML serialization |
| DNS | `SingboxDnsService` | Built into YAML serialization |
| Routing | `SingboxRoutingService` | Built into YAML serialization |
| Stats config | `SingboxStatisticService` | (native, no config needed) |

### 4.3 Shared Intermediate Model

Despite separate config generation, v2rayn has a **shared intermediate model** (`ProfileItem`, `CoreConfigContext`):
- `ProfileItem`: unified proxy node representation (address, port, protocol, transport, TLS params)
- `CoreConfigContext`: resolved proxy graph, DNS settings, routing rules, validated core type

Both cores receive the same `ProfileItem`, but each config generator converts it to its native format.

### 4.4 Protocol Support Matrix

v2rayn validates protocol compatibility per-core before config generation:

| Protocol | sing-box | mihomo |
|---|---|---|
| VMess | ✓ | ✓ |
| VLESS | ✓ | ✓ |
| Shadowsocks | ✓ | ✓ |
| Trojan | ✓ | ✓ |
| Hysteria2 | ✓ | ✓ |
| WireGuard | ✓ | ✓ |
| SOCKS | ✓ | ✓ |
| HTTP | ✓ | ✓ |
| TUIC | ✓ | ✓ |
| AnyTLS | ✓ | — |
| Naive | ✓ | — |
| SSR | — | ✓ |
| Snell | — | ✓ |
| MASQUE | — | ✓ |

### 4.5 What v2rayn Decided: Summary

| Decision | Rationale | Applicable to demotui? |
|---|---|---|
| Unified REST API client for both cores | `clash_api` is a near-clone | **Yes** — demotui's `src/functions/restful/` can serve both |
| Separate config generators per core | Config schemas fundamentally different | **Yes** — demotui should create a `src/functions/config_gen_singbox/` module |
| Shared intermediate model | Single source of truth for proxy data | **Yes** — demotui's existing proxy data structures can be shared |
| Per-core traffic stat services with unified output | Different protocols (WS vs poll) → different services, same callback | **Yes** — add `src/functions/restful/traffic_ws.rs` |
| No shared process management abstraction | Small number of cores, same OS patterns | **Yes** — extend `command.rs` with core-type dispatch |

---

## 5. Phased Implementation Plan for demotui

### Phase 0: Prerequisites & Config Schema

**Goal**: demotui can discover and configure a sing-box installation alongside mihomo.

| Task | File(s) | Description |
|---|---|---|
| Add core type enum | `src/config/core.rs` | Add `CoreType { Mihomo, Singbox }` to `ConfigFile`, defaulting to `Mihomo` for backward compatibility |
| Add sing-box config paths | `src/config/core.rs` | `singbox_bin_path` (default `/usr/bin/sing-box`), `singbox_config_dir`, `singbox_config_path` |
| Add sing-box service name | `src/config/core.rs` | `singbox_service_name` (default `"sing-box"`) |
| Add sing-box controller info | `src/config.rs` | `singbox_external_controller`, `singbox_secret` — or generalize `external_controller` to be per-core |

**Verification checkpoints**:
- [ ] Configuration can specify `core_type: "singbox"` and sing-box paths
- [ ] Config loading doesn't break for existing mihomo users
- [ ] `cargo check` passes

### Phase 1: Launch & Basic API

**Goal**: Start sing-box process and use demotui's existing REST API client to display proxies, connections, and perform node switching.

**Decision**: Use a shared REST API client (like v2rayn's `ClashApiManager`). The tested sing-box endpoints (`/proxies`, `/connections`) are drop-in compatible.

| Task | File(s) | Description |
|---|---|---|
| Select controller by core type | `src/functions/restful/utils.rs` | `request()` reads `CONFIG.external_controller` — generalize to select mihomo or sing-box controller based on active core |
| Add sing-box service control | `src/functions/command.rs` | Add `singbox_restart_service()`, `singbox_stop_service()` — dispatch same systemd/openrc/nssm calls with different service names |
| Add sing-box config test | `src/functions/command.rs` | `test_singbox_config(path)` → `{singbox_bin_path} check -c {path}` |
| Update SrvCtl tab | `src/tui/tab/srvctl.rs` | Add sing-box service operations, show active core type, allow switching |
| Make `ConnMetaData.ctype` optional | `src/functions/restful.rs` | `ctype` field → `Option<String>` to handle sing-box's missing `type` field |

**Verification checkpoints**:
- [ ] sing-box starts via systemd (or direct process) and serves on the configured `external_controller` port
- [ ] Proxies tab displays proxy groups and nodes from sing-box API
- [ ] Connections tab displays active connections (with empty `ctype` for sing-box)
- [ ] User can switch proxy selectors
- [ ] Delay tests work on sing-box proxies

### Phase 2: Config Generation & Traffic Stats

**Goal**: demotui can generate sing-box-native JSON config from its internal proxy data model, and display real-time traffic stats for sing-box.

**Decision**: Create separate sing-box config generator (`src/functions/config_gen_singbox/`). Config schemas are too different for a shared abstraction — follow v2rayn's pattern. Use WebSocket for sing-box traffic stats, with a normalization layer that maps to the same internal data model.

| Task | File(s) | Description |
|---|---|---|
| Create sing-box config generator module | `src/functions/config_gen_singbox/` | `mod.rs`, `outbound.rs`, `route.rs`, `dns.rs`, `inbound.rs` — converts demotui's proxy/profile data to sing-box JSON |
| Generate `singbox_config.json` from profile | `src/functions/config_gen_singbox/mod.rs` | Walk proxy tree, generate `outbounds[]` + `route.rules[]` + `dns.servers[]` + `inbounds[]` |
| Add WebSocket traffic client | `src/functions/restful/traffic_ws.rs` | `SingboxTrafficClient` — connects to `ws://{controller}/traffic`, parses `{up, down}` JSON messages, computes speed delta |
| Normalize traffic to shared model | `src/functions/restful.rs` | Add `TrafficStats { up: u64, down: u64, proxy_up: u64, proxy_down: u64 }` — filled by both mihomo poll and sing-box WS |
| Wire traffic into TUI | `src/tui/tab/status.rs` or per-tab `sync()` | Feed normalized `TrafficStats` into the same display pipeline |
| Profile/template for sing-box | `src/tui/tab/files.rs` | Allow creating/editing sing-box profile type alongside mihomo profiles |

**Verification checkpoints**:
- [ ] `singbox_config.json` is generated and passes `sing-box check`
- [ ] Traffic speed displays correctly in status bar (or relevant UI) for sing-box
- [ ] Generated config contains correct outbounds, route rules, DNS, TUN
- [ ] Profile switching works: selecting a profile regenerates + reloads sing-box config

### Phase 3: Feature Parity

**Goal**: Close remaining gaps so sing-box users have the same experience as mihomo users.

| Task | Description |
|---|---|
| Config hot-reload via SIGHUP | sing-box uses SIGHUP for config reload; implement `kill -HUP $(pidof sing-box)` as reload mechanism |
| TUN mode parity | The demotui `Config` already has `tun` settings — ensure they map correctly to sing-box `inbounds[type=tun]` |
| Speed test nuance handling | sing-box delay test returns same format as mihomo, but `url-test` group behavior may differ — verify and document |
| Error message mapping | Map sing-box-specific errors (config parse failures, outbound connection errors) to demotui's user-facing messages |
| Signal handling | Ensure `SIGINT`/`SIGTERM` handling works correctly for sing-box via service control |

**Verification checkpoints**:
- [ ] All Proxies tab operations work identically for sing-box and mihomo
- [ ] All Connections tab operations work identically
- [ ] Settings changes (mode, log level) applied correctly for sing-box
- [ ] Service control (start/stop/restart) works for both cores
- [ ] Config reload works without full process restart

### Phase 3b: Advanced Features (Optional)

| Task | Description |
|---|---|
| Proxy provider support | sing-box proxy providers act differently from mihomo — verify GET/PUT `/providers/proxies` compatibility |
| Geo data / Rule set management | mihomo `geodata-mode` vs sing-box `rule_set` — different update mechanisms |
| External UI integration | sing-box can serve yacd/metacubexd web panels — similar to mihomo, config path differs |
| Concurrent core switching | Allow user to switch between mihomo and sing-box without restarting demotui |

---

## 6. Compatibility Layer Decision Summary

For each integration point, the recommended approach:

| Integration Point | Use Shared Layer? | Rationale |
|---|---|---|
| REST API — proxies | **Yes** | `clash_api` is a near-perfect clone; same routes, same JSON format |
| REST API — connections | **Yes** | Same endpoint; make optional fields nullable for sing-box |
| REST API — config read/write | **Partial** | `mode` is shared; all other config keys differ → use core-specific code |
| REST API — restart | **No** | sing-box has no `/restart`; use external process restart |
| Traffic stats | **No (separate transport, same model)** | WebSocket vs poll; different transport, same output model |
| Config generation | **No** | JSON vs YAML, fundamentally different schemas |
| Config validation | **No** | Different CLI flags and subcommands |
| Service control | **Yes (dispatch by core type)** | Same systemd/openrc interface, different service names |
| Process management | **Yes (dispatch by core type)** | Same lifecycle, different binary paths and flags |
