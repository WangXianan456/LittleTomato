import type { CharacterId } from "./characters";
import type { Snapshot } from "./timer";

export type Interaction = "pet" | "wave" | "stretch";
const responses: Record<CharacterId, Record<Interaction, string>> = {
  tomato: { pet: "收到你的摸摸，今天也元气满满！", wave: "嗨！很高兴又和你一起。", stretch: "伸伸小手，我们一起放松一下。" },
  peach: { pet: "软乎乎的心情，分你一半。", wave: "桃桃在这里，陪你慢慢来。", stretch: "松松肩膀，给自己一点甜。" },
  sprout: { pet: "被照顾到啦，又长大了一点。", wave: "芽芽报到，今天也一起生长。", stretch: "向上伸展，像一棵小树一样。" },
  cloud: { pet: "送你一团软软的好心情。", wave: "飘过来，和你打个招呼。", stretch: "呼——慢慢呼气，放松一下。" },
  cat: { pet: "呼噜呼噜，喜欢这样陪着你。", wave: "喵！你的小同桌来啦。", stretch: "伸个懒腰，再舒舒服服地坐好。" },
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
