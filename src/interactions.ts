import type { CharacterId } from "./characters";
import type { Snapshot } from "./timer";

export type Interaction = "pet" | "wave" | "stretch";
const responses: Record<CharacterId, Record<Interaction, string>> = {
  tomato: { pet: "收到你的摸摸，今天也元气满满！", wave: "嗨！很高兴又和你一起。", stretch: "伸伸小手，我们一起放松一下。" },
  peach: { pet: "软乎乎的心情，分你一半。", wave: "桃桃在这里，陪你慢慢来。", stretch: "松松肩膀，给自己一点甜。" },
  sprout: { pet: "被照顾到啦，又长大了一点。", wave: "芽芽报到，今天也一起生长。", stretch: "向上伸展，像一棵小树一样。" },
  cloud: { pet: "送你一团软软的好心情。", wave: "飘过来，和你打个招呼。", stretch: "呼——慢慢呼气，放松一下。" },
  cat: { pet: "呼噜呼噜，喜欢这样陪着你。", wave: "喵！你的小同桌来啦。", stretch: "伸个懒腰，再舒舒服服地坐好。" },
  pikachu: { pet: "皮卡皮卡！收到你的好心情。", wave: "皮卡！今天的能量已就位。", stretch: "放松一下，给自己充充电。" },
  doraemon: { pet: "这份温暖，收进口袋里啦。", wave: "我来啦，一起想想好办法。", stretch: "先活动活动，再继续努力。" },
  totoro: { pet: "软软的肚子，借你靠一会儿。", wave: "森林里的朋友向你问好。", stretch: "像大树一样，慢慢伸展开。" },
  kirby: { pet: "啵哟！快乐像星星一样冒出来。", wave: "啵哟！一起开始今天的冒险。", stretch: "伸伸小手，准备轻盈地出发。" },
  hello_kitty: { pet: "把这份温柔系成一个蝴蝶结。", wave: "你好呀，今天也一起加油。", stretch: "松松肩膀，给自己一个拥抱。" },
  shinchan: { pet: "嘿嘿，被摸得美滋滋的。", wave: "嘿！动感小孩前来报到。", stretch: "伸展一下，像动感超人一样。" },
  spongebob: { pet: "咯咯咯，痒痒的，别停呀。", wave: "我准备好了！一起开始吧。", stretch: "去水母田之前，先拉伸一下。" },
  cinnamoroll: { pet: "耳朵扇一扇，好运飞过来。", wave: "汪！小云朵飞来陪你了。", stretch: "张开耳朵，像云朵一样轻盈。" },
  kuromi: { pet: "哼，才、才没有很开心呢。", wave: "来啦，酷酷的伙伴登场。", stretch: "活动一下，计划才不会乱。" },
  pooh: { pet: "软软的，像刚刚好的蜂蜜。", wave: "你好呀，我是小熊维尼。", stretch: "伸个懒腰，想想蜂蜜罐。" },
};
export function interactionMessage(character: CharacterId, action: Interaction) { return responses[character][action]; }
export function petMood(timer: Snapshot | null) {
  if (timer?.status === "completed") return "celebrate";
  if (timer?.status === "running") return timer.phase === "focus" ? "focus" : "rest";
  return "idle";
}
export function progressPercent(timer: Snapshot | null) {
  if (!timer || timer.plannedSeconds <= 0) return 0;
  return Math.max(0, Math.min(100, (1 - timer.remainingSeconds / timer.plannedSeconds) * 100));
}
