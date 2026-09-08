export type Snapshot = {
  phase: "focus" | "short_break" | "long_break";
  status: "ready" | "running" | "paused" | "completed";
  plannedSeconds: number;
  remainingSeconds: number;
  elapsedMs: number;
  rounds: number;
  recovery: boolean;
};
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
