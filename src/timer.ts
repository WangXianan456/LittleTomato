export type Snapshot = {
  phase: "focus" | "short_break" | "long_break";
  status: "ready" | "running" | "paused" | "completed";
  plannedSeconds: number;
  remainingSeconds: number;
  elapsedMs: number;
  rounds: number;
  recovery: boolean;
  recoveryReason?: string | null;
};
export function recoveryMessage(reason?: string | null) {
  switch (reason) {
    case "locked": return "锁屏前的进度已保存，继续吗？";
    case "sleep": return "欢迎回来，休眠前的计时还在。";
    case "disconnected": return "连接回来啦，要继续计时吗？";
    case "unresponsive": return "刚才暂停了一下，准备好再继续。";
    default: return "上次的计时还在，继续吗？";
  }
}
export function formatTime(seconds: number) {
  const value = Math.max(0, Math.ceil(seconds));
  return `${Math.floor(value / 60).toString().padStart(2, "0")}:${(value % 60).toString().padStart(2, "0")}`;
}
export function phaseLabel(phase: Snapshot["phase"]) {
  return { focus: "专注", short_break: "短休息", long_break: "长休息" }[phase];
}
export function primaryAction(timer: Snapshot) {
  switch (timer.status) {
    case "running": return { action: "pause", label: "暂停" };
    case "paused": return { action: "resume", label: "继续" };
    case "completed": return { action: "next", label: timer.phase === "focus" ? "休息" : "专注" };
    default: return { action: "start", label: "开始" };
  }
}
