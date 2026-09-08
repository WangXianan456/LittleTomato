# 小番茄 / Little Tomato

一个温柔提醒你专注与休息的 Windows 桌面小宠物。

## 当前状态

项目正在进行首个技术原型，已经包含：

- Tauri 2 + React + TypeScript 桌面应用骨架
- 透明、无边框、始终置顶的桌宠窗口
- 可拖动的 CSS 动画占位角色与聊天气泡
- Rust 驱动的专注、短休息和每四轮长休息，阶段由用户手动开始
- 开始、暂停、继续、提前结束，退出后恢复确认
- 透明区域穿透探针、右下工作区定位、拖动吸附与位置恢复
- 系统托盘显示、隐藏、计时快捷操作、剩余时间与退出确认
- SQLite 编号迁移、迁移前一致备份、状态与完成记录事务写入

正式角色美术、完整提醒流程、性格系统与设置面板将在后续迭代中加入。

本轮实现与实机验收见 [开发记录](docs/2026-09-08-progress.md)。当前仍为原型：锁屏事件、单实例、全屏隐藏、延后提醒及多屏/DPI 实测尚未完成。

## 开发环境

- Node.js 24+
- Rust stable（MSVC 工具链）
- Visual Studio Build Tools（Desktop development with C++）
- WebView2 Runtime

```powershell
. ./scripts/dev-env.ps1
npm install
npm run tauri dev
```

本机依赖实体位于 `D:\Coding\Packages\LittleTomato\node_modules`，仓库中的 `node_modules` 是目录联接，请保留此约束。

验证：`npm test`、`npm run check`、`npm run build`、`cargo test --manifest-path src-tauri/Cargo.toml`、`cargo fmt --check --manifest-path src-tauri/Cargo.toml` 和 `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`。原生构建前需加载 Visual Studio 开发环境。

## 数据与隐私

用户数据仅保存在本机的应用数据目录中，不上传、不遥测。开发中的数据库结构目前包含设置、计时记录与宠物状态。

开发机器可以设置 `LITTLE_TOMATO_RUNTIME_DIR`，将 SQLite 与 WebView2 运行数据重定向到指定磁盘；未设置时使用 Windows 标准应用数据目录。

## License

程序代码计划采用 MIT License。小番茄角色名称与美术资产保留版权。
