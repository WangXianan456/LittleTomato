export const characters = [
  { id: "tomato", name: "小番茄", subtitle: "熟悉的暖心搭档", greeting: "今天也一起慢慢来吧。", reply: "需要专注时，我会陪着你。" },
  { id: "peach", name: "蜜桃桃", subtitle: "把日子过得软一点", greeting: "今天也留一点甜给自己。", reply: "慢慢来，努力会悄悄结果。" },
  { id: "sprout", name: "芽芽", subtitle: "陪你每天长大一点", greeting: "一点点进步，也在生长。", reply: "休息一下，给自己浇点水。" },
  { id: "cloud", name: "云朵", subtitle: "一小团自在与轻盈", greeting: "把烦恼放轻，我陪着你。", reply: "深呼吸，让思绪飘一会儿。" },
  { id: "cat", name: "奶油猫", subtitle: "你的安静同桌", greeting: "喵，今天坐在你身边。", reply: "你认真做事，我认真陪你。" },
] as const;
export type CharacterId = typeof characters[number]["id"];
export function getCharacter(id: CharacterId) { return characters.find(item => item.id === id) ?? characters[0]; }
