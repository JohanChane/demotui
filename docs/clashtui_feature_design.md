# ClashTui 的功能设计

## 功能分类

-   与 core api 相关的功能:
    -   Status、Proxies、Connections 和 Settings tab
-   非 api 相关的功能 (也有可能用到 api):
    -   Files tab
        -   Profile panal
        -   template panel
    -   CoreSrvCtl tab

## ClashTui 的文件结构设计

ClashTui 配置的文件结构:

```
.
├── clashtui.db                     # 存放 ClashTui 的持久化数据
├── clashtui.log                    # ClashTui 的日志
├── config.yaml                     # ClashTui 的配置
├── mihomo
│   ├── basic_core_config.yaml      # Core Config 的基础字段配置
│   ├── profiles                    # Profile 对应的 yaml 文件 (mihomo 的配置格式是 yaml)
│   ├── template_proxy_providers    # 存放生成 template type profile 时, 需要的 urls
│   └── templates                   # template 存放的目录
└── sing-box
    ├── proxy-providers             # proxy-providers 文件的根目录
    ├── basic_core_config.json
    ├── profiles                    # Profile 对应的 json 文件 (sing-box 的配置是 json 格式)
    ├── template_proxy_providers
    └── templates
```

ClashTui Core 的文件结构设计:

```
.
├── mihomo
│   ├── clashtui_mihomo.service       # Mihomo Core 的 systemd unit file
│   ├── config                        # Core Config Dir
│   │   ├── config.yaml               # Core Config Path
│   └── mihomo -> /usr/bin/mihomo
└── sing-box
    ├── clashtui_singbox.service
    ├── config                        # Core Config Dir
    │   ├── config.json               # Core Config Path
    └── sing-box -> /usr/bin/sing-box
```

## ClashTui 的 clashtui.db 格式设计

```yaml
core_type: mihomo
mihomo:
  cur_profile:
  profiles:
sing-box:
  cur_profile:
  profiles:
```

设计原则: Mihomo 和 sing-box 不能共同使用的, 分别放在 mihomo 和 sing-box section。

## ClashTui 的配置设计

```
mihomo:
  core:
    config_dir: /opt/clashtui/mihomo/config
    bin_path: /opt/clashtui/mihomo/mihomo
    config_path: /opt/clashtui/mihomo/config/config.yaml
  core_service:
    service_name: clashtui_mihomo
    is_user: false
singbox:
  core:
    bin_path: /opt/clashtui/sing-box/sing-box
    config_dir: /opt/clashtui/sing-box/config
    config_path: /opt/clashtui/sing-box/config/config.json
  core_service:
    service_name: clashtui_singbox
    is_user: false
timeout: null
extra:
  edit_cmd: kitty -e nvim "%s"
  open_dir_cmd: kitty -e yazi "%s"
```

设计原则: Mihomo 和 sing-box 不能共同使用的, 分别放在 mihomo 和 sing-box section。

## ClashTui 管理 Core 文件的设计

ClashTui 使用 Linux 组文件权限管理 Core 的文件: User 加入每个 Core 的文件权限的组即可。

文件权限的检测与修复:
-   ClashTui 启动时, 取得 Core 目录 e.g. `/opt/clashtui/mihomo` 的 Group name
-   然后递归判断 Core 目录下的文件的 Group name 是否一致。
-   如果不一致则统一修复。否则不做什么。
-   同时确保 Core 目录设置 Group sticky bit。

为了使用户知道修改了什么, ClashTui 会转到 CLI 模式, 让用户输入密码。修复文件权限之后, ClashTui 重新启动。

## Mihomo 和 sing-box 配置的基础字段与非基础字段

为了合并 profile 和 basic_core_config。这里将 Core 的配置划分为基础字段与非基础字段 (顶层 key):
-   非基础字段: proxies, proxy-providers, rule, rule-providers, proxy-groups
-   基础字段: 不是非基础字段的, 就是基础字段

Mihomo 的非基础字段:
-   `proxies`: 代理节点定义
-   `proxy-providers`: 远程代理节点源
-   `proxy-groups`: 选择器 / url-test 等分组
-   `rules`: 内联路由规则
-   `rule-providers`: 远程规则集源
-   `sub-rules`: 嵌套/导入的规则预设

Mihomo 的基础字段:
-   `external-controller`, `mixed-port`, `mode`, `tun`, `log-level`, `allow-lan`, `ipv6`, `dns`, `sniffer`, `hosts`, `secret`, `profile`, `geodata-mode`, `find-process-mode`, `tcp-concurrent`, `unified-delay`, `keep-alive-interval` 等 (所有非非基础字段的顶层 YAML key)

sing-box 的非基础字段:
-   `outbounds`: 代理节点 + 代理分组 (selector/urltest)，全部在一个扁平列表中，通过 `tag` 标识
-   `route.rules`: 内联路由规则
-   `route.rule_set`: 远程 `.srs` 规则集引用

sing-box 的基础字段:
-   `experimental.clash_api` (含 `external_controller`, `secret`), `inbounds` (mixed / tun / tun gateway), `log` (含 `log.level`), `dns`, `route` (减去 `rules` 和 `rule_set`, 如 `route.auto_detect_interface`, `route.final`), `domain_strategy` 等 (所有非非基础字段的顶层 JSON key)

## Template 的管理设计

## Profile 的管理设计

将 Profile 的信息存放到 clashtui.db, 格式如下:

```yaml
mihomo_cur_profile: my
singbox_cur_profile: johan
mihomo_profiles:
  my:
    dtype: !Url https://example.com
    no_pp: false
  file:
    dtype: !File
    no_pp: false
  template:
    dtype: !Template
    no_pp: false
  common_tpl.yaml.tpl:    # Template type profile name 会以 `.tpl` 作为后缀
    dtype: !Template
      template: common_tpl.yaml
      proxy_provider_urls:
      - https://example.com
    no_pp: false
singbox_profiles:
  my:
    dtype: !Url https://example.com
    no_pp: false
  file:
    dtype: !File
    no_pp: false
  template:
    dtype: !Template
    no_pp: false
  common_tpl.json.tpl:    # Template type profile name 会以 `.tpl` 作为后缀
    dtype: !Template
      template: common_tpl.json
      proxy_provider_urls:
      - https://example.com
    no_pp: false
```

根据 profile name 取得 profile_yamls/profile_jsons 内相应的 yaml/json profile:
-   `profiles/<profile_name>.{yaml | json}`

Profile 不能 rename, 用户想要 rename 只能 delete + import, 所以这样管理是可行的。

profiles 目录下的文件是 profile 的原始文件, 不受其他因素影响。比如: `no_pp` option

File/Url Profile 的导入:
-   如果用户输入是一个文件路径, 则 profile type 是 `File`
-   如果是一个 url, 则是 `Url`。

File/Url Profile 的更新:
-   如果是 Url Profile, 则先更新 profile 的内容。
-   确保 profile 的文件存放到了 profiles 目录
-   取得 profiles 的网络资源 (proxy-providers 和 rule-providers), 然后更新到 Core Config Dir 的相应目录

File/Url Profile 的选择:
-   取出 Profile 的非基础字段和 basic_core_config 的基础字段, 将它们组成 Core Config, 写到 Core Config Path (保证基础字段在前面)

为什么不使用 api 来更新 Profile:
-   因为通过 api 更新 Profile 并没有返回值 (不知道是否更新成功), 则不知道有哪些东西要更新。
-   所以自己实现更新 Profile 会有比较好的体验。

因为我比较喜欢将每个 proxy-providers 分组, 而不是混合在一起。所以设计了 Template 的功能。

Template 文件主要有下面几个信息:
-   生成 proxy-provider groups。比如: pvd {pvd0, pvd1, ...}
-   为每个 proxy-provider 生成一个 proxy-group:

    比如:

    ```yaml
    - name: "At"
      tpl_param:
        providers: ["pvd"]
      type: url-test
      <<: *pa_dt
    ```

    会展开为 `At-pvd0, At-pvd1, ...`

-   在 proxy-groups 中使用 proxy-provider groups:
    -   比如: 用 `<pvd>`, 表示使用 proxy-provider group。它会被展开为 `pvd0, pvd1, ...`

综上, 只要提供 proxy-provider urls, 则可以生成一个 Profile 文件。

因为 sing-box 不支持 proxy-providers, 但是可以用 Template 的功能来替代它:
-   生成 Tempate type profile 时, 将 urls 存放到 clashtui.db 的 profile 字段中:
    
    比如:
    ```
    mihomo_profiles:
      common_tpl.json.tpl:
        dtype: !Template
          template: singbox_common_tpl.json
          proxy_provider_urls:
          - https://hajimi.nvimy.com/file/e7f20e25-058a-4c87-84db-8dc87e183b41/mojie_johan.yaml
    ```

-   proxy-providers 还有 url 的文件的路径信息, 为了方便将它固定为 `<clash_config_root>/sing-box/<profile_name>/{pvd0,pvd1,...}`。 
-   有了上面的信息就可以替代 proxy-providers 的功能了。

Template type profile 的生成:
-   前提 proxy-providers 的内容已经更新了, 如果没有内容则更新, 否则不更新。
-   上面的描述可以知道 Profile 的内容是如何生成的, 将它存放到 profiles 目录 (同理 sing-box 亦如此)
-   生成 clashtui.db 的 profile 信息

Template type profile 的更新:
-   更新 proxy_provider_urls
-   为了 mihomo 和 sing-box 的统一, 重新生成 profile

Mihomo template type pofile 的选择:
-   和 File/Url profile 的选择是一样的

sing-box template type pofile 的选择:
-   和 File/Url profile 的选择是一样的
