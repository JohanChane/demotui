# clashtui Filetree

> **Last verified**: 2026-05-07

## 1. `/opt/clashtui/` — Install Directory

Created by the `install` script. Contains the TUI binary and per-core subdirectories.

```
/opt/clashtui/
├── clashtui                       # demotui TUI binary (compiled Rust executable)
├── mihomo/                        # Mihomo core directory
│   ├── mihomo                     # Mihomo binary (downloaded/copied)
│   ├── config/                    # Mihomo runtime config directory
│   │   └── config.yaml            #   Generated mihomo YAML config (merged from basic + profile + templates)
│   └── clashtui_mihomo.service    # Systemd unit file for mihomo service
└── sing-box/                      # Sing-box core directory
    ├── sing-box                   # Sing-box binary (downloaded/copied)
    ├── config/                    # Sing-box runtime config directory
    │   └── config.json            #   Generated sing-box JSON config (merged from basic + profile + templates)
    └── clashtui_singbox.service   # Systemd unit file for sing-box service
```

### File/Directory Definitions

| Path | Type | Description |
|------|------|-------------|
| `clashtui` | Bin | The TUI application binary. Launches the interactive dashboard that manages proxies, connections, settings, etc. |
| `mihomo/` | Dir | Mihomo core directory. Holds the mihomo binary, its generated runtime config, and its systemd service unit. |
| `mihomo/mihomo` | Bin | Mihomo (clash.meta) core binary. The actual proxy engine process — invoked with `systemctl start clashtui_mihomo`. |
| `mihomo/config/` | Dir | Runtime config directory for mihomo. The generated `config.yaml` is placed here and passed to mihomo via `-d <dir> -f <path>`. |
| `mihomo/config/config.yaml` | File | Final merged YAML config for mihomo. Assembled from `~/.config/clashtui/mihomo/basic_clashtui_config.yaml` (basic settings), the active profile, and `~/.config/clashtui/mihomo/templates/`. |
| `mihomo/clashtui_mihomo.service` | File | Systemd service unit file. Defines the mihomo daemon process, its args, environment, and restart policy. Installed to `/etc/systemd/system/`. |
| `sing-box/` | Dir | Sing-box core directory. Holds the sing-box binary, its generated runtime config, and its systemd service unit. |
| `sing-box/sing-box` | Bin | Sing-box core binary. The proxy engine — invoked with `systemctl start clashtui_singbox`. |
| `sing-box/config/` | Dir | Runtime config directory for sing-box. The generated `config.json` is placed here and passed to sing-box via `-c <path>`. |
| `sing-box/config/config.json` | File | Final merged JSON config for sing-box. Assembled from `~/.config/clashtui/sing-box/basic_singbox_config.json` (basic settings), the active profile, and `~/.config/clashtui/sing-box/templates/`. |
| `sing-box/clashtui_singbox.service` | File | Systemd service unit file. Defines the sing-box daemon process. Installed to `/etc/systemd/system/`. |

---

## 2. `~/.config/clashtui/` — User Config Directory

Resolved via `config::init()` (portable mode → `$XDG_CONFIG_HOME/clashtui` → `~/.config/clashtui`). Contains per-core user-editable config files, templates, the profile database, and logs.

```
~/.config/clashtui/
├── mihomo/                                # Mihomo user config
│   ├── basic_clashtui_config.yaml         #   Basic settings (port, mode, TUN, log-level, etc.)
│   ├── clashtui.db                        #   Profile manager database (SQLite)
│   ├── clashtui.log                       #   Application log (env_logger Pipe target)
│   ├── config.yaml                        #   demotui TUI config (theme, keymap, core settings)
│   ├── keymap.yaml                        #   Per-tab key remappings
│   ├── profile_yamls/                     #   Downloaded profile YAML files (subscriptions)
│   └── templates/                         #   Profile export templates
│       └── template_proxy_providers       #     Template for proxy-provider stanzas
└── sing-box/                              # Sing-box user config
    ├── basic_singbox_config.json          #   Basic settings (clash_api, inbounds, log, etc.)
    ├── clashtui.db                        #   Profile manager database (SQLite)
    ├── clashtui.log                       #   Application log
    ├── config.yaml                        #   demotui TUI config (theme, keymap, core settings)
    ├── keymap.yaml                        #   Per-tab key remappings
    ├── profile_jsons/                     #   Downloaded profile JSON files (subscriptions)
    └── templates/                         #   Profile export templates
        └── template_proxy_providers       #     (N/A for sing-box — proxy-provider not supported)
```

### File/Directory Definitions

| Path | Type | Description |
|------|------|-------------|
| `mihomo/` | Dir | Per-core directory for mihomo user configuration. |
| `mihomo/basic_clashtui_config.yaml` | File | Base YAML config for mihomo. Contains top-level settings like `mixed-port`, `mode`, `tun.enable`, `log-level`, `allow-lan`, `ipv6`, `external-controller`, `profile.store-selected`, etc. Merged with profile + template during config generation. |
| `mihomo/clashtui.db` | File | SQLite database for mihomo profile management. Stores saved profiles, template associations, and persistence flags. Saved on exit via `config::CONFIG.save()`. |
| `mihomo/clashtui.log` | File | Log output for the mihomo session. Written by `env_logger` using the `Pipe` target. |
| `mihomo/config.yaml` | File | Demotui TUI config for mihomo. Contains mihomo-specific TUI settings: binary path, config directory, service name, external controller address, secret, etc. Loaded by `ConfigFile::from_file()`. |
| `mihomo/keymap.yaml` | File | Per-tab key remapping config for mihomo. Users can override default key bindings. Loaded by `agent::init()` in `src/tui/agent.rs`. |
| `mihomo/profile_yamls/` | Dir | Downloaded profile YAML files (subscription data). Each profile is stored as a `.yaml` file containing proxy nodes and rules. |
| `mihomo/templates/` | Dir | Template directory for mihomo profile export. Templates define how proxy groups reference providers (or inline proxies in no-pp mode). |
| `mihomo/templates/template_proxy_providers` | File | Template file for proxy-provider stanzas in mihomo profiles. Contains `<TemplateParam>` placeholders expanded during profile processing. |
| `sing-box/` | Dir | Per-core directory for sing-box user configuration. |
| `sing-box/basic_singbox_config.json` | File | Base JSON config for sing-box. Contains `experimental.clash_api` (external_controller + secret), `inbounds[]` (mixed port, TUN), `log.level`, etc. Merged with profile + template during config generation. |
| `sing-box/clashtui.db` | File | SQLite database for sing-box profile management. Same structure as `mihomo/clashtui.db` but scoped to sing-box profiles. |
| `sing-box/clashtui.log` | File | Log output for the sing-box session. |
| `sing-box/config.yaml` | File | Demotui TUI config for sing-box. Contains sing-box-specific TUI settings: binary path, config directory, service name, external controller, etc. |
| `sing-box/keymap.yaml` | File | Per-tab key remapping config for sing-box. Users can override default key bindings. Loaded by `agent::init()` in `src/tui/agent.rs`. |
| `sing-box/profile_jsons/` | Dir | Downloaded profile JSON files (subscription data). Each profile is stored as a `.json` file containing outbounds and route rules. |
| `sing-box/templates/` | Dir | Template directory for sing-box profile export. |
| `sing-box/templates/template_proxy_providers` | File | Template file. Proxy-provider is not supported in sing-box. This template is functionally unused — displayed as "N/A" in TUI when `ProfileType::Singbox` is active. |

---

## 3. Notes

- **Portable mode**: If `data/` exists next to the `clashtui` binary, the config directory is resolved to `data/` instead of `~/.config/clashtui/`. The same per-core subdirectory layout applies.
- **Per-core isolation**: Each core (mihomo / sing-box) has its own `config.yaml`, `clashtui.db`, and `clashtui.log`. The active core determines which files are read/written.
- **Runtime vs user config**: `/opt/clashtui/{core}/config/` holds the **generated** (merged & expanded) config fed to the proxy engine. `~/.config/clashtui/{core}/` holds the **user-editable** base config, profiles, and templates that are merged into the runtime config.
- **`config_dir`**: In `src/config.rs`, `config_dir` refers to the per-core user config directory (e.g. `~/.config/clashtui/mihomo/`), not `/opt/clashtui/`.
