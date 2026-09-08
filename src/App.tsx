import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { formatTime, primaryAction, phaseLabel, type Snapshot } from "./timer";
import "./App.css";

function App() {
  const [timer, setTimer] = useState<Snapshot | null>(null);
  const [count, setCount] = useState(0);
  const [message, setMessage] = useState("今天也一起慢慢来吧。");
  const [error, setError] = useState(false);
  const [busy, setBusy] = useState(false);
  const [quit, setQuit] = useState(false);
  const acting = useRef(false);
  const revision = useRef(0);
  const root = useRef<HTMLElement>(null);
  const drag = useRef<{ x: number; y: number; moved: boolean } | null>(null);

  useEffect(() => {
    let disposed = false;
    let polling = false;
    const refresh = async () => {
      if (polling || acting.current) return;
      polling = true;
      const startedAtRevision = revision.current;
      try {
        const [snapshot, runtime] = await Promise.all([
          invoke<Snapshot>("timer_snapshot"),
          invoke<{ completedSessions: number }>("runtime_status"),
        ]);
        if (!disposed && !acting.current && startedAtRevision === revision.current) { setTimer(snapshot); setCount(runtime.completedSessions); setError(false); }
      } catch { if (!disposed) setError(true); }
      finally { polling = false; }
    };
    void refresh();
    const interval = window.setInterval(() => void refresh(), 1000);
    const listeners = [
      listen("quit-requested", () => setQuit(true)),
      listen("app-error", () => setError(true)),
    ];
    return () => { disposed = true; window.clearInterval(interval); void Promise.all(listeners).then(items => items.forEach(unlisten => unlisten())); };
  }, []);

  useEffect(() => {
    const update = () => {
      const areas = Array.from(root.current?.querySelectorAll<HTMLElement>("[data-hit]") ?? []).map(el => {
        const r = el.getBoundingClientRect();
        return { x: r.x, y: r.y, width: r.width, height: r.height, radius: Number(el.dataset.hit) || 0 };
      });
      void invoke("set_hit_areas", { areas }).catch(() => setError(true));
    };
    update();
    const observer = new ResizeObserver(update);
    if (root.current) observer.observe(root.current);
    // Also follows the small breathing motion of the pet's hit target.
    const interval = window.setInterval(update, 200);
    return () => { observer.disconnect(); window.clearInterval(interval); };
  }, [quit]);

  async function act(action: string) {
    if (acting.current) return;
    acting.current = true; setBusy(true);
    revision.current += 1;
    try { setTimer(await invoke<Snapshot>("timer_action", { action })); setError(false); }
    catch { setError(true); }
    finally { acting.current = false; setBusy(false); }
  }

  const running = timer?.status === "running";
  const completed = timer?.status === "completed";
  const speech = error ? "暂时没能保存，请稍后重试。" : timer?.recovery ? "上次的计时还在，继续吗？" : completed ? (timer.phase === "focus" ? "这一轮完成啦，休息一下吧！" : "休息好啦，准备开始下一轮？") : message;
  const action = timer ? primaryAction(timer) : { action: "start", label: "准备中" };

  return (
    <main ref={root} className="pet-window">
      <section data-hit="22" className="speech-bubble" aria-live="polite"
        onPointerDown={event => { if (event.button === 0) void getCurrentWindow().startDragging().catch(() => setError(true)); }} title="拖动气泡也可以移动小番茄">
        <p>{speech}</p><span className="speech-tail" aria-hidden="true" />
      </section>
      <button data-hit="62" className={`tomato ${running ? "is-focusing" : ""}`} type="button" aria-label="和小番茄说话；按住拖动可移动"
        onPointerDown={e => { if (e.button === 0) { drag.current = { x: e.screenX, y: e.screenY, moved: false }; e.currentTarget.setPointerCapture(e.pointerId); } }}
        onPointerMove={e => {
          const d = drag.current;
          if (d && !d.moved && Math.hypot(e.screenX-d.x, e.screenY-d.y) > 5) {
            d.moved = true; e.currentTarget.releasePointerCapture(e.pointerId);
            void getCurrentWindow().startDragging().catch(() => setError(true));
          }
        }}
        onClick={() => { if (!drag.current?.moved) setMessage(message === "今天也一起慢慢来吧。" ? "需要专注时，我会陪着你。" : "今天也一起慢慢来吧。"); drag.current = null; }}>
        <span className="leaf leaf-left" /><span className="leaf leaf-center" /><span className="leaf leaf-right" />
        <span className="face"><span className="eye eye-left" /><span className="eye eye-right" /><span className="mouth" /></span>
        <span className="arm arm-left" /><span className="arm arm-right" /><span className="foot foot-left" /><span className="foot foot-right" />
      </button>
      <section data-hit="20" className="timer-card" aria-label="计时器">
        <div className="timer-copy"><span className="timer-state">{timer ? `${phaseLabel(timer.phase)} · ${running ? "进行中" : timer.status === "paused" ? "已暂停" : completed ? "已完成" : "准备好了"}` : "正在准备"}</span><strong>{formatTime(timer?.remainingSeconds ?? 1500)}</strong></div>
        <div className="timer-actions">
          <button className="primary-action" disabled={!timer || busy} onClick={() => void act(action.action)}>{action.label}</button>
          <button className="icon-action" disabled={!timer || busy} onClick={() => void act(completed && timer?.phase === "focus" ? "skip" : "reset")} aria-label={completed ? "跳过，回到专注" : "提前结束并重置"} title={completed ? "跳过，回到专注" : "提前结束并重置"}>↻</button>
        </div>
      </section>
      <footer data-hit="8" className="runtime-status">已陪你完成 {count} 次专注</footer>
      {quit && <section data-hit="20" className="quit-dialog" role="dialog" aria-modal="true" aria-label="退出小番茄">
        <p>先休息一下？计时进度会保存，下次回来可以继续。</p>
        <button autoFocus onClick={() => setQuit(false)}>留下来</button>
        <button onClick={() => void invoke("quit_app").catch(() => setError(true))}>保存并退出</button>
      </section>}
    </main>
  );
}
export default App;
