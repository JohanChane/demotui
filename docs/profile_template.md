# Profile Template (Unified — mihomo + sing-box)

Templates let you define a parameterized configuration that expands into a full profile. Mihomo templates are **YAML** files; sing-box templates are **JSON** files. Both use the same `${}` placeholder syntax and `expand_this_group_with` / `expand_this_outbounds_with` markers. Subscription proxy-provider groups are stored per-profile in the database as name+URL pairs.

## Overview

A template has two extra features on top of the native config format:

1. **`tpl_param`** markers on proxy-provider entries (unchanged) — marks entries for expansion
2. **`expand_this_group_with`** (mihomo) / **`expand_this_outbounds_with`** (sing-box) on proxy-group entries — marks groups for expansion with `${group_name}` references
3. **`${name}`** placeholders in `use`/`proxies`/`outbounds` lists — expand to all generated names matching that prefix

### Template Formats

| Backend | Template format | File extension | Output format |
|---------|----------------|----------------|---------------|
| mihomo | YAML | `.yaml` | YAML with proxy-providers |
| sing-box | JSON | `.json` | JSON with embedded outbounds |

### Per-Core Directories

| Backend | Template dir | Output dir |
|---------|-------------|------------|
| mihomo | `mihomo/templates/` | `profile_yamls/<name>.yaml` |
| sing-box | `sing-box/templates/` | `profile_jsons/<name>.json` |

### Profile Types

| Type | Description |
|------|-------------|
| `File` | Local file imported |
| `Url(url)` | Downloaded subscription |
| `Template { template, proxy_provider_groups }` | Generated from template with per-profile provider groups |
| `Singbox` | sing-box JSON profile imported directly |

Legacy `!Generated` entries auto-migrate to `!Template` on load. Legacy `!File` entries with `clashtui` marker auto-migrate to `!Template`.

---

## Mihomo Template Format (YAML)

### Proxy-Provider Template Entries

```yaml
proxy-providers:
  pvd:                      # Template entry — has tpl_param
    tpl_param:              # Marker (empty value)
    type: http
    interval: 3600
    url: https://example.com/sub1.yaml
    health-check:
      enable: true
      url: https://www.gstatic.com/generate_204
      interval: 300
  static:                   # Passthrough entry — no tpl_param
    type: http
    interval: 3600
    url: https://static.example.com/proxy.yaml
    path: ./proxy-providers/static.yaml
```

After generation, `tpl_param` is removed and the entry expands with per-profile URLs.

### Proxy-Group Template Entries

```yaml
proxy-groups:
  - name: Select           # Passthrough
    type: select
    proxies:
      - DIRECT
      - ${Auto}            # Placeholder — expands to all Auto-* groups
  - name: Auto             # Template group
    type: url-test
    expand_this_group_with:
      - ${pvd}
    url: https://www.gstatic.com/generate_204
    interval: 300
  - name: Direct           # Passthrough
    type: select
    proxies:
      - DIRECT
```

After generation:
```yaml
proxy-groups:
  - name: Select
    type: select
    proxies: [DIRECT, Auto-foo_pvd]
  - name: Auto-foo_pvd
    type: url-test
    url: https://www.gstatic.com/generate_204
    interval: 300
    use: [foo_pvd]
  - name: Direct
    type: select
    proxies: [DIRECT]
```

### `${}` Placeholder Expansion

| Placeholder | In | Expands to |
|-------------|-----|------------|
| `${pvd}` | `use` | All generated provider names in the `pvd` group (`foo_pvd`, `bar_pvd`, ...) |
| `${Auto}` | `proxies` | All generated group names matching `Auto-*` prefix (`Auto-foo_pvd`, `Auto-bar_pvd`, ...) |

### Mihomo Complete Example

**Template** `mihomo/templates/my-config.yaml`:
```yaml
proxy-providers:
  pvd:
    tpl_param:
    type: http
    interval: 3600
    url: https://example.com/sub1.yaml
    health-check:
      enable: true
      url: https://www.gstatic.com/generate_204
      interval: 300
proxy-groups:
  - name: Entry
    type: select
    proxies: [DIRECT, ${Auto}, REJECT]
  - name: Auto
    type: url-test
    expand_this_group_with: [${pvd}]
    url: https://www.gstatic.com/generate_204
    interval: 300
  - name: Direct
    type: select
    proxies: [DIRECT]
rules:
  - DOMAIN-SUFFIX,google.com,Entry
  - MATCH,DIRECT
```

**Output** `profile_yamls/my-config.yaml`:
```yaml
proxy-providers:
  foo_pvd:
    type: http
    interval: 3600
    url: https://example.com/sub1.yaml
    health-check:
      enable: true
      url: https://www.gstatic.com/generate_204
      interval: 300
    path: proxy-providers/tpl/my-config/foo_pvd.yaml
proxy-groups:
  - name: Entry
    type: select
    proxies: [DIRECT, Auto-foo_pvd, REJECT]
  - name: Auto-foo_pvd
    type: url-test
    url: https://www.gstatic.com/generate_204
    interval: 300
    use: [foo_pvd]
  - name: Direct
    type: select
    proxies: [DIRECT]
rules:
  - DOMAIN-SUFFIX,google.com,Entry
  - MATCH,DIRECT
clashtui: null
```

---

## sing-box Template Format (JSON)

sing-box templates use the same `${}` syntax but in JSON format. The engine downloads subscriptions, extracts proxy nodes, and embeds them directly into `outbounds[]` (sing-box has no proxy-provider concept).

### Template Markers (JSON)

```json
{
  "proxy-providers": {
    "pvd": {
      "tpl_param": {},
      "url": "https://example.com/sub.yaml"
    }
  },
  "proxy-groups": [
    {
      "name": "Auto",
      "type": "url-test",
      "expand_this_outbounds_with": ["${pvd}"],
      "url": "https://www.gstatic.com/generate_204",
      "interval": 300
    },
    {
      "name": "Proxy",
      "type": "select",
      "proxies": ["DIRECT", "${Auto}", "REJECT"]
    }
  ]
}
```

- `"tpl_param": {}` — marks a proxy-provider for expansion (unchanged)
- `"expand_this_outbounds_with": ["${pvd}"]` — marks an outbound for expansion with `${group_name}` reference
- `"${Auto}"` — placeholder, expands to all generated group tags

### Proxy-provider naming

Proxy-provider names come from the `ProviderUrl.name` field in the proxy-provider group configured in `template_proxy_providers.yaml`. Each subscription URL from the group becomes one provider entry with the given name.

### Mapping: Template → sing-box Output

| Template | sing-box Output |
|----------|----------------|
| `proxy-providers.pvd.tpl_param` | Downloaded proxies → `outbounds[{type: vmess/shadowsocks, tag: "pvd0-<server>", ...}]` |
| `proxy-groups[].type: select` | `outbounds[{type: selector, tag: <name>, outbounds: [...]}]` |
| `proxy-groups[].type: url-test` | `outbounds[{type: urltest, tag: <name>, outbounds: [...], url: ..., interval: "5m"}]` |
| `use: [pvd0]` | `outbounds` references by `tag` |
| `rules` (inline string array) | `route.rules[{domain_suffix: [...], outbound: ...}]` |
| `rule-providers` | `route.rule_set[{tag: ..., type: remote, url: ...}]` |
| `MATCH,Target` | `route.final: "Target"` |

Rule matchers:
- `DOMAIN-SUFFIX` → `domain_suffix`
- `DOMAIN-KEYWORD` → `domain_keyword`
- `DOMAIN` → `domain`
- `IP-CIDR` → `ip_cidr`
- `GEOSITE` → `rule_set` (requires matching `rule-providers` entry with `.srs` URL)
- `GEOIP` → `rule_set`
- `PROCESS-NAME` → `process_name`
- `MATCH` → `route.final`

### sing-box Complete Example

**Template** `sing-box/templates/my-config.json`:
```json
{
  "proxy-providers": {
    "pvd": {
      "tpl_param": {},
      "type": "http",
      "interval": 3600,
      "url": "https://example.com/sub.yaml",
      "health-check": {
        "enable": true,
        "url": "https://www.gstatic.com/generate_204",
        "interval": 300
      }
    }
  },
  "proxy-groups": [
    {
      "name": "Proxy",
      "type": "select",
      "proxies": ["DIRECT", "${Auto}", "REJECT"]
    },
    {
      "name": "Auto",
      "type": "url-test",
      "expand_this_outbounds_with": ["${pvd}"],
      "url": "https://www.gstatic.com/generate_204",
      "interval": 300
    },
    {
      "name": "Direct",
      "type": "select",
      "proxies": ["DIRECT"]
    }
  ],
  "rules": [
    "DOMAIN-SUFFIX,google.com,Proxy",
    "GEOSITE,cn,Direct",
    "GEOIP,CN,Direct",
    "MATCH,Proxy"
  ],
  "rule-providers": {
    "geosite-cn": {
      "type": "http",
      "behavior": "domain",
      "url": "https://github.com/SagerNet/sing-geosite/raw/refs/heads/rule-set/geosite-cn.srs",
      "path": "./rule-providers/geosite-cn.srs",
      "interval": 86400
    },
    "geoip-cn": {
      "type": "http",
      "behavior": "ipcidr",
      "url": "https://github.com/SagerNet/sing-geoip/raw/refs/heads/rule-set/geoip-cn.srs",
      "path": "./rule-providers/geoip-cn.srs",
      "interval": 86400
    }
  }
}
```

**Output** `profile_jsons/my-config.json` (with proxy-provider group `pvd: [{name: "foo_pvd", url: "https://sub.example.com"}]` containing 2 VMess nodes):
```json
{
  "outbounds": [
    {
      "type": "vmess",
      "tag": "foo_pvd-1.2.3.4",
      "server": "1.2.3.4",
      "server_port": 443,
      "uuid": "...",
      "tls": { "enabled": true, "server_name": "example.com" }
    },
    {
      "type": "vmess",
      "tag": "foo_pvd-5.6.7.8",
      "server": "5.6.7.8",
      "server_port": 443,
      "uuid": "...",
      "transport": { "type": "ws", "path": "/ws" }
    },
    {
      "type": "selector",
      "tag": "Proxy",
      "outbounds": ["DIRECT", "Auto-foo_pvd", "REJECT"]
    },
    {
      "type": "urltest",
      "tag": "Auto-foo_pvd",
      "outbounds": ["foo_pvd-1.2.3.4", "foo_pvd-5.6.7.8"],
      "url": "https://www.gstatic.com/generate_204",
      "interval": "5m"
    },
    {
      "type": "selector",
      "tag": "Direct",
      "outbounds": ["DIRECT"]
    }
  ],
  "route": {
    "rules": [
      { "domain_suffix": ["google.com"], "outbound": "Proxy" },
      { "rule_set": "geosite-cn", "outbound": "Direct" },
      { "rule_set": "geoip-cn", "outbound": "Direct" }
    ],
    "rule_set": [
      {
        "tag": "geosite-cn",
        "type": "remote",
        "format": "binary",
        "url": "https://github.com/SagerNet/sing-geosite/raw/refs/heads/rule-set/geosite-cn.srs"
      },
      {
        "tag": "geoip-cn",
        "type": "remote",
        "format": "binary",
        "url": "https://github.com/SagerNet/sing-geoip/raw/refs/heads/rule-set/geoip-cn.srs"
      }
    ],
    "final": "Proxy"
  },
  "clashtui_template_name": "my-config"
}
```

---

## File Path Import

Import a local config by filesystem path:

1. Switch to the **Profile** tab
2. Press `I` (shift-i) to import from file
3. Enter a profile name and source file path

Mihomo: YAML → `profile_yamls/<name>.yaml`, registered as `File`.
sing-box: JSON → `profile_jsons/<name>.json`, registered as `Singbox`.

## Update Flow

### Non-template profiles (File, Url, Singbox)
- `u` re-reads the profile file, downloads net resources, reports status.

### Template profiles (Template)
- `u` re-downloads all subscription URLs, re-expands the template, overwrites the profile file.
- For mihomo: generates fresh YAML from template + URLs.
- For sing-box: downloads subscriptions, parses proxy nodes, generates fresh JSON.

## TUI Key Bindings (Template Tab)

| Key | Action |
|-----|--------|
| `Enter` | Generate — prompts for profile name + subscription URLs (comma-separated) |
| `d d` | Delete template |
| `e` | Edit template in `$EDITOR` |
| `p` | Preview template content |
| `f` | Fuzzy find template |
| `/` | Search/filter |
| `g g` / `G` | Go to top / bottom |
