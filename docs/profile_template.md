# Profile Template (Unified — mihomo + sing-box)

Templates let you define a parameterized configuration that expands into a full profile. The same template syntax works for both mihomo (YAML output) and sing-box (JSON output). Template proxy-provider subscriptions are stored per-profile in the database — each template profile records its template file name and subscription URLs.

## Overview

A template is a standard Clash-style YAML file with two extra features:

1. **`tpl_param`** markers on `proxy-providers` and `proxy-groups` entries — these entries are templates that expand at generation time
2. **`<>`** angle-bracket placeholders in `use` and `proxies` lists — these reference template entries and expand to all generated names

### Profile Storage

**Profile types** in the database (`clashtui.db`):

| Type | Description |
|------|-------------|
| `File` | Local file imported to `profile_yamls/` |
| `Url(url)` | Downloaded from a subscription URL |
| `Template { template, urls }` | Generated from a template file with per-profile subscription URLs |
| `Singbox` | sing-box JSON profile |

### Per-Core Output

| Backend | Template dir | Output dir | Format |
|---------|-------------|------------|--------|
| mihomo | `mihomo/templates/` | `profile_yamls/<name>.yaml` | YAML with proxy-providers |
| sing-box | `sing-box/templates/` | `profile_jsons/<name>.json` | JSON with embedded outbounds |

### Migration

Legacy template-generated profiles (previously stored as `!File`) are auto-migrated to `!Template` on startup. The migration detects the `clashtui` marker in profile YAML files and converts the database entry. Migrated profiles have an empty URL list — users should add subscription URLs via regeneration.

## Template YAML Format

```yaml
# Required sections:
proxy-providers:    # Mapping — at least one entry (with url for tpl_param entries)
proxy-groups:       # Sequence — at least one entry

# Passthrough sections (copied as-is):
rules:
rule-providers:
# ... any other clash config keys
```

Templates are **pure clash YAML**. No `clashtui.uses` section is needed — each proxy-provider provides its own URL directly.

## Proxy-Provider Template Entries

A proxy-provider entry with a `tpl_param` key is a **template provider**. At generation time, `tpl_param` is removed and the entry passes through with all other fields intact.

### Input template
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

### After generation
```yaml
proxy-providers:
  pvd:                      # tpl_param removed, url kept from template
    type: http
    interval: 3600
    url: https://example.com/sub1.yaml
    health-check:
      enable: true
      url: https://www.gstatic.com/generate_204
      interval: 300
  static:                   # Passthrough preserved in place
    type: http
    interval: 3600
    url: https://static.example.com/proxy.yaml
    path: ./proxy-providers/static.yaml
```

Key behaviors:
- `tpl_param` is removed from generated entries
- All other fields (url, path, type, interval, health-check, etc.) are kept as-is from the template
- Entries without `tpl_param` pass through unchanged
- The entry keeps its original name and position (**ordering guarantee**)

## Proxy-Group Template Entries

A proxy-group entry with `tpl_param.providers` is a **template group**. It generates one group per matching proxy-provider.

### Input template
```yaml
proxy-groups:
  - name: Select           # Passthrough — no tpl_param
    type: select
    proxies:
      - DIRECT
      - <Auto>             # Placeholder — expands to all Auto-* groups
  - name: Auto             # Template group — has tpl_param
    type: url-test
    tpl_param:
      providers:           # Generate one group per provider matched here
        - pvd
    url: https://www.gstatic.com/generate_204
    interval: 300
  - name: Direct           # Passthrough — no tpl_param
    type: select
    proxies:
      - DIRECT
```

### After generation
```yaml
proxy-groups:
  - name: Select
    type: select
    proxies:
      - DIRECT
      - Auto-pvd           # <Auto> expanded
  - name: Auto-pvd         # Generated from Auto template
    type: url-test
    url: https://www.gstatic.com/generate_204
    interval: 300
    use:
      - pvd
  - name: Direct           # Passthrough preserved
    type: select
    proxies:
      - DIRECT
```

Key behaviors:
- Generated group names follow the pattern `{group_name}-{provider_name}` (e.g., `Auto-pvd`)
- The `use` field is set to the specific provider name
- The `tpl_param` key is removed from generated entries
- Non-template groups pass through unchanged
- Group ordering is preserved; template groups expand in place

### Edge case: no matching providers

If `tpl_param.providers` references a provider name that has no expanded instances, that group template generates **no entries** and is silently skipped. The group is removed from output.

## `<>` Placeholder Expansion

Angle-bracket placeholders in `use` and `proxies` lists expand to all matching generated names.

| Placeholder | Expands in | Expands to |
|-------------|-----------|------------|
| `<pvd>` | `use` | All generated proxy-provider names with that key (e.g., `pvd`) |
| `<Auto>` | `proxies` | All generated proxy-group names with that prefix (e.g., `Auto-pvd`) |

```yaml
# Before:
proxy-groups:
  - name: Entry
    type: select
    use:
      - <pvd>         # Expands to pvd
    proxies:
      - DIRECT
      - <Auto>        # Expands to Auto-pvd

# After (pvd→[pvd], Auto→[Auto-pvd]):
proxy-groups:
  - name: Entry
    type: select
    use:
      - pvd
    proxies:
      - DIRECT
      - Auto-pvd
```

Non-bracket values pass through unchanged.

If a placeholder references a non-existent target, generation fails with an error.

## Profile Storage

demotui uses per-core directories under the config root:

| Directory | Purpose |
|-----------|---------|
| `mihomo/templates/` | Mihomo template YAML files with `tpl_param` markers |
| `mihomo/profile_yamls/` | All mihomo profile YAML (generated, imported, and downloaded) |
| `sing-box/templates/` | sing-box template YAML files with `tpl_param` markers |
| `sing-box/profile_jsons/` | All sing-box profile JSON (generated, imported, and downloaded) |

**Profile types** in the database (`clashtui.db`):

| Type | Description |
|------|-------------|
| `File` | Local file imported to `profile_yamls/` |
| `Url` | Downloaded from a subscription URL |
| `Template { template, urls }` | Generated from a template with per-profile subscription URLs |
| `Singbox` | sing-box JSON profile imported directly |

Legacy `!Generated` entries auto-migrate to `!Template` with empty URLs on load. Legacy `!File` entries with `clashtui` marker in their YAML are auto-migrated to `!Template` with inferred template name and empty URLs.

## File Path Import

You can import a local clash YAML configuration by filesystem path:

1. Switch to the **Profile** tab
2. Press `I` (shift-i) to import from file
3. Enter a profile name
4. Enter the source file path

The file is copied to `profile_yamls/<name>.yaml` and registered as `ProfileType::File`.

## Full Workflow

### Mihomo Template Workflow

```
┌──────────────────────────────────────────────────────────────────┐
│  1. CREATE TEMPLATE                                              │
│     Write a clash YAML file with tpl_param markers.              │
│     Place in config directory: mihomo/templates/my-template.yaml │
├──────────────────────────────────────────────────────────────────┤
│  2. GENERATE PROFILE                                             │
│     In Template tab, select template → press Enter.               │
│     Enter profile name and subscription URLs (comma-separated).   │
│     Creates profile_yamls/<name>.yaml                             │
│     Registered as ProfileType::Template in the database.         │
├──────────────────────────────────────────────────────────────────┤
│  3. SELECT PROFILE                                               │
│     In Profile pane, select the profile → press Enter.           │
│     demotui merges it with basic_clash_config.yaml and deploys   │
│     to the clash config path.                                    │
├──────────────────────────────────────────────────────────────────┤
│  4. UPDATE PROFILE                                               │
│     Press 'u' to re-generate from template using recorded URLs.   │
│     Press 'a' 'u' to update all profiles.                        │
└──────────────────────────────────────────────────────────────────┘
```

### sing-box Template Workflow

sing-box has no proxy-provider concept, so templates generate self-contained JSON with all proxies embedded as `outbounds[]` entries.

1. **Create template** — same template syntax as mihomo. Place in `sing-box/templates/`.
2. **Generate profile** — prompts for profile name and subscription URLs. Downloads subscriptions, parses proxy nodes, and generates `profile_jsons/<name>.json`.
3. **Update** — re-downloads subscriptions and re-generates the JSON. No proxy-providers in the output — all proxies are inlined.

### sing-box Template Output Format

Template concepts map to sing-box as follows:

| Template | sing-box Output |
|----------|----------------|
| `proxy-providers.pvd` with `tpl_param` | Downloaded proxies → `outbounds[{type: vmess/shadowsocks, tag: ..., ...}]` |
| `proxy-groups[].type: select` | `outbounds[{type: selector, tag: <name>, outbounds: [...]}]` |
| `proxy-groups[].type: url-test` | `outbounds[{type: urltest, tag: <name>, outbounds: [...], url: ..., interval: "5m"}]` |
| `use: [pvd0]` | `outbounds` references by `tag` |
| `rules` inline strings | `route.rules[{domain_suffix: [...], outbound: ...}]` |
| `rule-providers` | `route.rule_set[{tag: ..., type: remote, url: ...}]` |
| `MATCH,Target` | `route.final: "Target"` |

Proxy-provider identities use hardcoded prefix `pvd`: `pvd0`, `pvd1`, etc. Proxy-group name is always `pvd`.
```

## Complete Example

### Template: `templates/my-config.yaml`
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
    proxies:
      - DIRECT
      - <Auto>
      - REJECT
  - name: Auto
    type: url-test
    tpl_param:
      providers:
        - pvd
    url: https://www.gstatic.com/generate_204
    interval: 300
  - name: Direct
    type: select
    proxies:
      - DIRECT
rules:
  - DOMAIN-SUFFIX,google.com,Entry
  - MATCH,DIRECT
```

### Generated output: `profile_yamls/my-config.yaml`
```yaml
proxy-providers:
  pvd:
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
    proxies:
      - DIRECT
      - Auto-pvd
      - REJECT
  - name: Auto-pvd
    type: url-test
    url: https://www.gstatic.com/generate_204
    interval: 300
    use:
      - pvd
  - name: Direct
    type: select
    proxies:
      - DIRECT
rules:
  - DOMAIN-SUFFIX,google.com,Entry
  - MATCH,DIRECT
clashtui: null
```
