# DemoTUI

## 创建该项目的原因

在学习 [yazi](https://github.com/sxyazi/yazi) 的框架设计, 发现不错。所以准备给 ClashTUI 用上该框架设计 (不照搬只取适合的)。该项目不包含业务代码。

## 进度

-   [x] 前端/后端统一消息源, 并使用 channel 通信。
-   [x] 使用 Context 进入依赖注入, 防止过多的混乱指针。同时使得执行 Action 时, 内部对象能访问外部对象。如果使用指针的方式则比较麻烦。
-   [x] 使用 Data 统一 Action 的返回值。
-   [ ] 使用 Layer、tab_idx 和 fouce 路由 keyevent。
-   [ ] 如何复用 widget 的代码? 不使用 EventState (Consumed) 的方式, 不直观。
-   [ ] 在 list 的选项的前面加 `*` 表示已经选择该项。如果执行该项的操作时, 在该项的前面使用 (`/`, `-`, `|`, `-`, `\`) 表示正在执行该项的操作, 这样能减少弹窗次数。
-   [ ] 消息弹窗 (info, warning, error) 有不同的颜色。
-   [ ] 添加 switch 面板
-   [ ] 输入框支持输入中文