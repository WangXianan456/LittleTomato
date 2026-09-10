# 小番茄 / Little Tomato

一个温柔提醒你专注与休息的 Windows 桌面小宠物。

## 当前状态

默认仅显示小宠物。双击桌宠展开对话气泡、计时卡与底栏；再次双击、点击“收起”或按 Esc 可隐藏这些信息。收起后继续计时，透明空白区域不会拦截鼠标。键盘聚焦宠物后可用 Enter/空格切换显示。

宠物的瞳孔会跟随桌面鼠标，身体轻微转向；按住拖动时会悬起并随移动摆动，松开后平滑落回。采用轻量 CSS 透视效果，减少动画模式下保持静态。

桌宠支持点击摸摸、右键或底部“互动”打开动作面板，可打招呼、伸懒腰；配有短语、表情、鼠标视线跟随及短暂特效。计时卡显示本轮进度，正在运行或暂停时点重置会先确认。设置页支持 Ctrl+S 保存。

现有五位可切换伙伴：小番茄、蜜桃桃、芽芽、云朵和奶油猫。在“设置 → 伙伴小屋”选择角色并保存即可应用，各有独立造型和点击短语，保留现有专注记录。详见 [角色美化记录](docs/2026-09-09-characters.md)。

项目正在进行首个技术原型，已经包含：

- Tauri 2 + React + TypeScript 桌面应用骨架
- 透明、无边框、始终置顶的桌宠窗口
- 可拖动的 CSS 动画占位角色与聊天气泡
- Rust 驱动的专注、短休息和定期长休息，时长及轮次可配置，阶段由用户手动开始
- 开始、暂停、继续、提前结束，退出后恢复确认
- 透明区域穿透探针、右下工作区定位、拖动吸附与位置恢复
- 系统托盘显示、隐藏、计时快捷操作、剩余时间与退出确认
- SQLite 编号迁移、迁移前一致备份、状态与完成记录事务写入

点击桌宠底部“设置”或托盘“设置…”可打开独立设置窗口，配置专注与休息时长、长休息间隔、桌宠大小、置顶、减少动画、主题及开机启动。设置保存在本地；修改计时计划不改变正在运行或暂停的阶段。支持取消、恢复推荐设置以及 Esc 关闭。

正式角色美术、完整提醒流程与性格系统将在后续迭代中加入。设置实现与验证见 [设置面板记录](docs/2026-09-09-settings.md)。

窗口与计时基础见 [首轮记录](docs/2026-09-08-progress.md)，单实例、原生锁屏/睡眠暂停与托盘菜单更新见 [第二轮记录](docs/2026-09-09-progress.md)。当前仍为原型：真实锁屏/硬件睡眠、全屏隐藏、延后提醒及多屏/DPI 实测尚未全部完成。

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

关闭已有小番茄后，可运行 `./scripts/smoke-windows.ps1`。脚本启动独立测试实例，通过鼠标点击和向其窗口发送消息验证实际 DPI、鼠标穿透、单实例、暂停恢复及隐藏唤回；不会锁定或休眠 Windows，测试数据库、日志与截图保留在 D 盘独立目录。增加 `-ColdStartRounds 10` 可检查两个进程同时冷启动。100%/125%/150% 系统缩放及冷启动结果见 [Windows 验收记录](docs/2026-09-09-windows-validation.md)。

## 数据与隐私

设置实机回归：`./scripts/smoke-windows.ps1 -SettingsChecks`，覆盖打开、校验、保存、外观更新、关闭重开、重启持久化和恢复推荐设置。

用户数据仅保存在本机的应用数据目录中，不上传、不遥测。开发中的数据库结构目前包含设置、计时记录与宠物状态。

开发机器可以设置 `LITTLE_TOMATO_RUNTIME_DIR`，将 SQLite 与 WebView2 运行数据重定向到指定磁盘；未设置时使用 Windows 标准应用数据目录。

## License

程序代码计划采用 MIT License。小番茄角色名称与美术资产保留版权。
