# 九个卡通角色的官方图片抠像与纸偶骨架 — 2026-09-10

用户先要求直接从网络获取官方图片抠出角色形象以保证还原，随后要求这些角色"能动起来、四肢做动作、不穿模"，并把同样的处理应用到皮卡丘、哆啦 A 梦、卡比和 Hello Kitty。本轮将九位卡通角色做成"官方抠像 + 纸偶骨架"：手臂、脚、耳朵切成独立图层，身体缺口修补，复用 SVG 角色已有的挥手/伸懒腰/摸摸/拖起/眼睛跟随动画。龙猫仍为 SVG 重绘（见下文边界）。

## 素材来源

检索与下载日期：2026-09-10。原始页面与图片保存在 `D:\Coding\References\LittleTomato\2026-09-10`（同日目录内含上一轮前五位角色的参考图）。

| 角色 | 来源页面 | 使用图片 | 处理 |
| --- | --- | --- | --- |
| 小新 | [朝日电视台《蜡笔小新》角色介绍](https://www.tv-asahi.co.jp/shinchan/character/) | `character/img/01.png`（しんのすけ 官方立绘） | 已带透明通道，按 alpha 裁剪、加边距、等比缩放 |
| 海绵宝宝 | Nickelodeon 官方 stock art（经 [Encyclopedia SpongeBobia](https://spongebob.fandom.com/wiki/SpongeBob_SquarePants_(character)) og:image 定位，fandom 图床下载） | `Spongebob happy stock art 1`（官方宣传立绘） | WebP 转 PNG 后同样裁剪缩放 |
| 玉桂狗 | [三丽鸥官方角色页](https://www.sanrio.co.jp/characters/cinnamon/) | `mv-cinnamon.png`（官网主视觉立绘） | 同上 |
| 库洛米 | [三丽鸥官方角色页](https://www.sanrio.co.jp/characters/kuromi/) | `list-kuromi.png`（官网角色一览立绘） | 同上 |
| 维尼熊 | [pngimg.com 维尼熊合集](http://pngimg.com/images/cartoon/winnie_poo)（经典 2D 动画造型抠像图，原始美术版权属 Disney） | `uploads/winnie_pooh/winnie_pooh_PNG37580.png`（竖版站立害羞姿势） | 同上 |
| 皮卡丘 | [宝可梦日本官方图鉴 0025](https://zukan.pokemon.co.jp/detail/0025) | [图鉴立绘](https://zukan.pokemon.co.jp/zukan-api/up/images/index/5bb0cfd44302cd4df0c0c88d37457931.png) | 同上 |
| 哆啦 A 梦 | [哆啦 A 梦频道官方角色资料](https://dora-world.com/character/doraemon) | `d_001_card_detail.png`（官方角色卡，带卡片底） | 颜色分类洪泛抠出角色，擦除卡片边框、道具与迷你角色 |
| 卡比 | [任天堂 Kirby 官方 About 页](https://kirby.nintendo.com/about/) | `characters-kirby_2x.png`（官方 3D 渲染） | 半透明背景星形按 alpha 阈值剔除 |
| Hello Kitty | [三丽鸥官方角色页](https://www.sanrio.co.jp/characters/hellokitty/) | `mv-hellokitty.png`（官网主视觉坐姿版） | 擦除两侧苹果与牛奶瓶道具 |

维尼熊曾先后尝试 Disney 日本官网（Akamai 403）、Wikipedia（本机网络不可达）、Disney/KH fandom（infobox 为剧照或占位图失败）、必应图片检索等路径；最终采用 pngimg 的透明版本。该图与电影剧照不同，为经典 2D 造型，与其余角色的平面风格一致。

龙猫未能转换为抠像素材：吉卜力官方只发布场景剧照，逐一核验本地已下载的 018–050 号剧照，均为躺姿（小梅压在身上）、咆哮特写、飞行（小智小梅扒在身上）或与猫巴士同框，没有可用的干净全身立绘；fandom 词条图同样为剧照，pngimg 无龙猫分类且站内搜索无结果。龙猫本轮保持依据官方剧照重绘的 SVG 形象；如用户提供透明立绘，可走同一管线接入。

检索中 khwiki 返回的图片路径为占位链接、fandom 部分图片无透明通道，均已弃用，未当作有效素材。

## 实现

- 素材分层保存于 `src/assets/characters/{id}/`（body、arm-left/right、feet、ear-left/right），由 `D:\Coding\References\LittleTomato\2026-09-10\extract_rigs.py` 从原始抠像切分，`gen_stock_rigs.py` 汇总为 `src/stockRigs.ts`；两份脚本可重跑，配置勿手改。
- 切分方式：贴着角色轮廓的部件（手臂、前置的脚）用"深色轮廓线包围的洪泛填充"分割，或沿关节的半平面切线；耳朵等在身体后方的部件用包围盒。部件切走后，身体缺口用"非轮廓最近邻填充"修补，保证手臂摆动时不露出破洞（不穿模）。
- `src/StockCharacter.tsx` 按图层顺序渲染纸偶：手臂等前置部件的 `transform-origin` 设在肩/髋关节，直接复用现有 `rig-arm`、`rig-foot`、`rig-ear` 类的挥手、伸懒腰、拖起、摸摸动画；`--gaze-k` 按角色尺寸归一化视线幅度。
- 眼睛为叠加 SVG：原图瞳孔按检测窗口聚类定位后平填为眼白，叠加可移动的瞳孔/虹膜（视线跟随）、每 5.3 秒一次的盖片眨眼，以及摸摸/庆祝/休息时的开心眼弧线。
- `src/PetFeatures.tsx` 优先走纸偶骨架；龙猫继续走 `ReferenceCharacter` SVG；原创五位不变。设置窗口"伙伴小屋"的"卡通伙伴 · 10"筛选、卡片预览、桌面 hit area 自动沿用，无新增迁移或设置项。

## 行为边界

- 九位纸偶角色的眨眼由叠加盖片实现（原 SVG 为整体缩放眼睛）；卡比、库洛米等无独立可见手臂的角色，挥手动作幅度较小或由脚部/耳朵动作承担；Hello Kitty 蝴蝶结保留在身体层（试切后头部修补出现放射纹，放弃切分）。
- 龙猫保留 SVG 重绘，行为与前版一致。其余行为（拖动悬起、摸摸语音、计时）全部不变。

## 验证

- `npx tsc --noEmit`、`npm run build`（分层 PNG 进入 `dist/assets/`）、9 项 Node 测试、19 项 Rust 测试通过（Rust 仅保留角色枚举变更，序列化覆盖全部十五个角色 ID）。
- 预览页 `preview.html`（`npm run dev` 后访问 `/preview.html`）为九位纸偶角色并排展示常态/挥手/摸摸/拖起四状态，已目视核对：挥手时手臂绕肩摆动、拖起时脚下移、摸摸时盖片闭合并出现开心眼弧线，静止时部件完全复位、无穿模。
- Windows 实机验收 `scripts/smoke-windows.ps1 -SettingsChecks` 全部断言通过，含：通过设置菜单选择 hello_kitty 并同步到桌面、猫/角色切换保留暂停计时、重启后设置与角色持久化、五位卡通伙伴真实鼠标点击逐个响应（pikachu/doraemon/totoro/kirby/hello_kitty）、卡比/哆啦A梦/皮卡丘等纸偶角色在实机渲染正常。首次运行曾在"提前结束并重置"一步失败，重跑全程通过，确认为 ESC 时序抖动，与角色改动无关。
- 日常实例已用新构建重启（沿用 `D:\Coding\Runtime\LittleTomato` 运行目录，设置与专注记录保留）。

## 版权边界

九位角色内嵌的图片文件（电视朝日、Nickelodeon、三丽鸥、宝可梦、哆啦 A 梦频道、任天堂、Disney 美术）权利归各自权利人，不属于项目 MIT 许可范围，仅限本机个人使用，不得单独提取、再分发或商用；README License 一节已同步说明。
