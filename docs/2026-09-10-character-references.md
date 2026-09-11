# 官方参考核验与全身结构重绘

检索日期：2026-09-10。用户要求先上网找原始形象依据，再调整角色；本次实际下载并目视检查以下官方角色图和电影剧照。未找到完整官方三视图/骨骼设计稿，不将宣传图或剧照称为设计稿。

| 角色 | 官方页面 | 使用的图像与观察 |
| --- | --- | --- |
| 皮卡丘 | [宝可梦日本官方图鉴 0025](https://zukan.pokemon.co.jp/detail/0025) | [图鉴立绘](https://zukan.pokemon.co.jp/zukan-api/up/images/index/5bb0cfd44302cd4df0c0c88d37457931.png)：头身连贯、下腹宽、两耳朝向不对称、细小脚趾、独立折线尾巴、左臂伸出右掌抬起。不是球形。 |
| 哆啦 A 梦 | [哆啦 A 梦频道官方角色资料](https://dora-world.com/character/doraemon) | [角色卡全身图](https://dora-world.com/assets/images/characters/doraemon/doraemon/d_001_card_detail.png)：大头、独立蓝色短躯干、白腹半圆口袋、红项圈、铃铛、圆手与白脚、红尾球；眼睛在面部上方相邻。[官方漫画摘图](https://dora-world.com/assets/images/characters/doraemon/doraemon/d_001_ph001.png)辅助观察手臂和表情。 |
| 龙猫 | [吉卜力《龙猫》作品页](https://www.ghibli.jp/works/totoro/) | [官方剧照 044](https://www.ghibli.jp/gallery/totoro044.jpg)：高而宽的梨形身躯、腹部占身躯大半、小眼睛、宽鼻子、大笑露齿、下垂长臂、尖耳；[030](https://www.ghibli.jp/gallery/totoro030.jpg)、[032](https://www.ghibli.jp/gallery/totoro032.jpg)、[036](https://www.ghibli.jp/gallery/totoro036.jpg)辅助面部、侧面和飞行姿态观察。 |
| 卡比 | [任天堂 Kirby 官方 About 页](https://kirby.nintendo.com/about/) | [角色主图](https://kirby.nintendo.com/assets/img/about/characters-kirby_2x.png)：本来就是圆身体，但红脚大、双腿抬起角度不同，蓝黑高椭圆眼睛，手臂有独立轮廓；[动作插图](https://kirby.nintendo.com/assets/img/about/char-kirby_2x.png)辅助观察抬臂。 |
| Hello Kitty | [三丽鸥官方角色页](https://www.sanrio.co.jp/characters/hellokitty/) | [官方主图](https://www.sanrio.co.jp/wp-content/uploads/2022/06/mv-hellokitty.png)：选取该页蓝色服装、红白条纹上衣的坐姿版本，宽头、独立小身体、白色脚掌、三根胡须、无嘴，蝴蝶结在画面右上方。官网还有其他服装，不能混成唯一标准。 |

## 本地证据

下载页面、原始图片与联系表保存于 `D:\Coding\References\LittleTomato\2026-09-10`。`reference-contact.jpg` 与 `totoro-contact.jpg` 用于挑选图片；原始参考图片未打包进入应用，来源 URL 保留在此处。

检索中 `pokemon.com/us/pokedex/pikachu` 未返回可用图鉴，改用日本官方图鉴；`dora-world.com/characters` 是失效路径，从官方首页找到正确的 `/character/doraemon`。没有把这些失效页当成有效证据。

## 对应实现与边界

`src/ReferenceCharacter.tsx` 按上述图像重绘五套独立 SVG 结构，包含头、身体、手脚、耳朵/尾巴、眼睛、服装等分层。`src/cartoonCharacters.css` 改为控制这些结构，废弃上一版圆身体换装样式。原创五位伙伴继续使用原有绘制。

造型依据是上述可核验图片；耳朵轻摆、挥手、拖动悬起等动作是本应用的交互动画，不能称为逐帧还原官方动画。全身图中的外部道具/场景（例如指挥棒、星星背景、苹果）不作为角色身体的一部分。当前没有完整背面、转身序列和正式三维模型。

卡通角色仍为非官方参考重绘，角色名称和形象权利归各自权利人。该文档不构成授权证明。未提交或发布。

## 范围边界（2026-09-10 23:37 夜间值班补记）

上表目前只覆盖第一批五个卡通角色：皮卡丘、哆啦 A 梦、龙猫、卡比、Hello Kitty。第二批加入的小新、海绵宝宝、玉桂狗、库洛米、维尼熊尚未执行同样的官方检索、图片下载与逐项比对，形象依据常识性特征绘制；核验完成前不应把他们的造型描述为「依据官方参考」。后续核验完成后在此表补充对应行。

## 验证结果

- TypeScript 检查、9 项前端测试、Rust 格式检查与 Clippy `-D warnings` 通过。
- Tauri debug `--no-bundle` 构建通过，正常运行目录的应用已重新启动。
- Windows 设置及桌宠实机回归共 78 项断言通过，包含五个卡通角色的真实点击、选择持久化及重启后仅宠物模式。
- 已目视检查五个角色的实机截图。证据目录：`D:\Coding\Runtime\LittleTomato-smoke-691c3ab4223e4f55be6e80b950084f4c`。
- 官方参考与本轮实机截图对照：`D:\Coding\References\LittleTomato\2026-09-10\reference-vs-redraw.png`。截图保留实际桌面背景；插图和剧照并非完整模型设计稿。
