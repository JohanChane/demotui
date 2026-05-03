# Profile Template

Templates let you define a parameterized clash configuration that expands into a full profile YAML. Multiple subscription URLs combine with one template to produce a complete set of proxy-providers, proxy-groups, and rules.

## Overview

A template is a standard Clash YAML file with three extra features:

1. **`tpl_param`** markers on `proxy-providers` and `proxy-groups` entries — these entries are templates that expand at generation time
2. **`<>`** angle-bracket placeholders in `use` and `proxies` lists — these reference template entries and expand to all generated names
3. **`clashtui.uses`** section — lists which profiles (by name) supply the subscription URLs for expansion

When you generate a profile from a template, demotui reads the template YAML, expands all `tpl_param` entries using URLs from the profiles listed in `clashtui.uses`, resolves `<>` placeholders, and writes the result to `profiles/<template_name>.clashtui_generated`.

**Output YAML preserves the section ordering of the input template.** Entries appear in the same relative order — template entries expand "in place" at their original position.

## Template YAML Format

```yaml
# Required sections:
proxy-providers:    # Mapping — at least one entry
proxy-groups:       # Sequence — at least one entry

# Optional:
clashtui:           # Template configuration
  uses:             # List of profile names to use as URL sources
    - profile1
    - profile2
rules:              # Passthrough — copied as-is
rule-providers:     # Passthrough — copied as-is
```

## Proxy-Provider Template Entries

A proxy-provider entry with an empty `tpl_param` key is a **template provider**. At generation time, it is removed and replaced by one provider per URL.

### Input template
```yaml
proxy-providers:
  pvd:                      # Template entry — has tpl_param
    tpl_param:              # Marker (empty value)
    type: http
    interval: 3600
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

### After generation (with 2 URLs)
```yaml
proxy-providers:
  static:                   # Passthrough preserved in place
    type: http
    interval: 3600
    url: https://static.example.com/proxy.yaml
    path: ./proxy-providers/static.yaml
  pvd0:                     # Expanded: URL #1
    type: http
    interval: 3600
    health-check:
      enable: true
      url: https://www.gstatic.com/generate_204
      interval: 300
    url: https://example.com/sub1.yaml
    path: proxy-providers/tpl/my-tpl/pvd0.yaml
  pvd1:                     # Expanded: URL #2
    type: http
    interval: 3600
    health-check:
      enable: true
      url: https://www.gstatic.com/generate_204
      interval: 300
    url: https://example.com/sub2.yaml
    path: proxy-providers/tpl/my-tpl/pvd1.yaml
```

Key behaviors:
- `tpl_param` is removed from generated entries
- `url` is injected from the profile's subscription URL
- `path` is auto-generated as `proxy-providers/tpl/<template>/<providerN>.yaml`
- Entries without `tpl_param` pass through unchanged
- The template entry's original position in the mapping determines where the expanded entries appear (**ordering guarantee**)

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

### After generation (with pvd expanded to [pvd0, pvd1])
```yaml
proxy-groups:
  - name: Select
    type: select
    proxies:
      - DIRECT
      - Auto-pvd0          # <Auto> expanded
      - Auto-pvd1
  - name: Auto-pvd0        # Generated from Auto template
    type: url-test
    url: https://www.gstatic.com/generate_204
    interval: 300
    use:
      - pvd0
  - name: Auto-pvd1        # Generated from Auto template
    type: url-test
    url: https://www.gstatic.com/generate_204
    interval: 300
    use:
      - pvd1
  - name: Direct           # Passthrough preserved
    type: select
    proxies:
      - DIRECT
```

Key behaviors:
- Generated group names follow the pattern `{group_name}-{provider_name}` (e.g., `Auto-pvd0`)
- The `use` field is set to the specific provider name
- The `tpl_param` key is removed from generated entries
- Non-template groups pass through unchanged
- Group ordering is preserved; template groups expand in place

### Edge case: no matching providers

If `tpl_param.providers` references a provider name that has no expanded instances (e.g., zero URLs), that group template generates **no entries** and is silently skipped. The group is removed from output.

## `<>` Placeholder Expansion

Angle-bracket placeholders in `use` and `proxies` lists expand to all matching generated names.

| Placeholder | Expands in | Expands to |
|-------------|-----------|------------|
| `<pvd>` | `use` | All generated proxy-provider names (e.g., `pvd0`, `pvd1`) |
| `<Auto>` | `proxies` | All generated proxy-group names with that prefix (e.g., `Auto-pvd0`, `Auto-pvd1`) |

```yaml
# Before:
proxy-groups:
  - name: Entry
    type: select
    use:
      - <pvd>         # Expands to all pvdN providers
    proxies:
      - DIRECT
      - <Auto>        # Expands to all Auto-* groups

# After (2 URLs, pvd→[pvd0,pvd1], Auto→[Auto-pvd0,Auto-pvd1]):
proxy-groups:
  - name: Entry
    type: select
    use:
      - pvd0
      - pvd1
    proxies:
      - DIRECT
      - Auto-pvd0
      - Auto-pvd1
```

Non-bracket values pass through unchanged.

If a placeholder references a non-existent target, generation fails with an error.

## URL Sourcing (`clashtui.uses`)

The URLs used for template expansion come from the **profile database**. Profiles of type `Url` have a subscription URL. The `clashtui.uses` list in the template specifies which profile names to use:

```yaml
clashtui:
  uses:
    - my-subscription
    - another-sub
```

At generation time, demotui:
1. Reads `clashtui.uses` from the template
2. Looks up each name in the profile database
3. Collects URLs from matching `ProfileType::Url` entries
4. Passes the `(name, url)` pairs to `gen_template()`

If `clashtui.uses` is empty or missing, no URLs are available and template entries produce no expansions. If a referenced profile doesn't exist or isn't a `Url` type, it's silently skipped.

**To add subscription URLs before using a template:**
1. Switch to the **Profile** tab (key `p`)
2. Press `i` to import a new profile
3. Enter a name and the subscription URL
4. Edit the template's `clashtui.uses` to reference the profile name
5. Generate the template

## Full Workflow

```
┌──────────────────────────────────────────────────────────────────┐
│  1. CREATE TEMPLATE                                              │
│     Write a YAML file with proxy-providers, proxy-groups,        │
│     tpl_param markers, and clashtui.uses.                        │
│     Place in config directory: templates/my-template.yaml        │
├──────────────────────────────────────────────────────────────────┤
│  2. ADD SUBSCRIPTION URLS                                        │
│     Import profiles with URLs (Profile tab → 'i' key).           │
│     Edit template uses to reference them.                        │
├──────────────────────────────────────────────────────────────────┤
│  3. GENERATE PROFILE                                             │
│     In Template pane, select template → press Enter.             │
│     Output: profiles/my-template.yaml.clashtui_generated         │
├──────────────────────────────────────────────────────────────────┤
│  4. SELECT PROFILE                                               │
│     In Profile pane, select the generated profile → press Enter. │
│     demotui merges it with basic_clash_config.yaml and deploys   │
│     to the clash config path.                                    │
├──────────────────────────────────────────────────────────────────┤
│  5. UPDATE PROFILE                                               │
│     Press 'u' to re-download all proxy-provider URLs.            │
│     Press 'a' to update all profiles.                            │
└──────────────────────────────────────────────────────────────────┘
```

## Generated Profile Database Entry

When a template is applied, demotui creates a `ProfileType::Generated(template_name)` entry in the profile database (`clashtui.db`). This:

- Identifies the profile as generated from a specific template
- Allows re-generation (overwrites file and updates database)
- Enables the "no proxy-providers" update mode (`m` key) which downloads and inlines proxy content, stripping the `proxy-providers` section
- Ensures the profile appears in the Profile pane for selection, update, and management

The generated profile file always contains a `clashtui: null` top-level key as a type marker, distinguishing it from manually imported profiles.

## Complete Example

### Template: `templates/my-config.yaml`
```yaml
proxy-providers:
  pvd:
    tpl_param:
    type: http
    interval: 3600
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
clashtui:
  uses:
    - sub1
    - sub2
```

### Profile database (`clashtui.db`)
```yaml
sub1: !Url https://cdn.example.com/sub1.yaml
sub2: !Url https://cdn.example.com/sub2.yaml
```

### Generated output: `profiles/my-config.yaml.clashtui_generated`
```yaml
proxy-providers:
  pvd0:
    type: http
    interval: 3600
    health-check:
      enable: true
      url: https://www.gstatic.com/generate_204
      interval: 300
    url: https://cdn.example.com/sub1.yaml
    path: proxy-providers/tpl/my-config/pvd0.yaml
  pvd1:
    type: http
    interval: 3600
    health-check:
      enable: true
      url: https://www.gstatic.com/generate_204
      interval: 300
    url: https://cdn.example.com/sub2.yaml
    path: proxy-providers/tpl/my-config/pvd1.yaml
proxy-groups:
  - name: Entry
    type: select
    proxies:
      - DIRECT
      - Auto-pvd0
      - Auto-pvd1
      - REJECT
  - name: Auto-pvd0
    type: url-test
    url: https://www.gstatic.com/generate_204
    interval: 300
    use:
      - pvd0
  - name: Auto-pvd1
    type: url-test
    url: https://www.gstatic.com/generate_204
    interval: 300
    use:
      - pvd1
  - name: Direct
    type: select
    proxies:
      - DIRECT
rules:
  - DOMAIN-SUFFIX,google.com,Entry
  - MATCH,DIRECT
clashtui: null
```
