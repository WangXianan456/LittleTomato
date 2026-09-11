export const characters = [
  { id: "tomato", name: "小番茄", subtitle: "熟悉的暖心搭档", greeting: "今天也一起慢慢来吧。", reply: "需要专注时，我会陪着你。" },
  { id: "peach", name: "蜜桃桃", subtitle: "把日子过得软一点", greeting: "今天也留一点甜给自己。", reply: "慢慢来，努力会悄悄结果。" },
  { id: "sprout", name: "芽芽", subtitle: "陪你每天长大一点", greeting: "一点点进步，也在生长。", reply: "休息一下，给自己浇点水。" },
  { id: "cloud", name: "云朵", subtitle: "一小团自在与轻盈", greeting: "把烦恼放轻，我陪着你。", reply: "深呼吸，让思绪飘一会儿。" },
  { id: "cat", name: "奶油猫", subtitle: "你的安静同桌", greeting: "喵，今天坐在你身边。", reply: "你认真做事，我认真陪你。" },
  { id: "pikachu", name: "皮卡丘", subtitle: "电力满满的小搭档", greeting: "皮卡！今天也充满能量。", reply: "给你的努力充一点电！" },
  { id: "doraemon", name: "哆啦 A 梦", subtitle: "口袋里装着好心情", greeting: "今天的计划，一起完成吧。", reply: "一步一步来，总会有办法。" },
  { id: "totoro", name: "龙猫", subtitle: "森林里安静的大朋友", greeting: "听，风吹过树叶的声音。", reply: "在这里歇一会儿也很好。" },
  { id: "kirby", name: "卡比", subtitle: "粉色星星的元气陪伴", greeting: "向今天的小目标出发！", reply: "认真休息，再一起冒险。" },
  { id: "hello_kitty", name: "Hello Kitty", subtitle: "系着蝴蝶结的温柔日常", greeting: "今天也要照顾好自己。", reply: "一点小小的快乐，送给你。" },
  { id: "shinchan", name: "小新", subtitle: "爱动感超人的五岁小孩", greeting: "动感光波，哔哔哔！", reply: "认真玩，也要认真休息。" },
  { id: "spongebob", name: "海绵宝宝", subtitle: "比奇堡的快乐捕手", greeting: "我准备好了！一起加油。", reply: "抓水母之前，先抓住专注。" },
  { id: "cinnamoroll", name: "玉桂狗", subtitle: "扇着大耳朵飞的小狗", greeting: "今天的风，软软的哦。", reply: "慢慢来，云朵会等你的。" },
  { id: "kuromi", name: "库洛米", subtitle: "酷酷的粉色小恶魔", greeting: "哼，今天也由我陪着你。", reply: "嘴上不说，心里支持你。" },
  { id: "pooh", name: "维尼熊", subtitle: "爱蜂蜜的温柔大熊", greeting: "来一点蜂蜜一样的休息？", reply: "休息，就是心里的蜂蜜。" },
] as const;
export type CharacterId = typeof characters[number]["id"];
export const cartoonIds: readonly CharacterId[] = ["pikachu", "doraemon", "totoro", "kirby", "hello_kitty", "shinchan", "spongebob", "cinnamoroll", "kuromi", "pooh"];
export function getCharacter(id: CharacterId) { return characters.find(item => item.id === id) ?? characters[0]; }
