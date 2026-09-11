import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { formatTime, primaryAction, phaseLabel, recoveryMessage, type Snapshot } from "./timer";
import "./App.css";
import PetFeatures from "./PetFeatures";
import { getCharacter, cartoonIds } from "./characters";
import { defaultSettings, type Settings } from "./settings";
import { useColorTheme } from "./appearance";
import ConfirmDialog from "./ConfirmDialog";
import { interactionMessage, petMood, progressPercent, type Interaction } from "./interactions";
import "./interactions.css";
import { usePetPhysics } from "./usePetPhysics";
import "./petPhysics.css";

function App() {
  const [timer, setTimer] = useState<Snapshot | null>(null);
  const [count, setCount] = useState(0);
  const [message, setMessage] = useState("今天也一起慢慢来吧。");
  const [error, setError] = useState(false);
  const [busy, setBusy] = useState(false);
  const [quit, setQuit] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const lastDrag = useRef(0);
  const [resetConfirm, setResetConfirm] = useState(false);
  const [interactionOpen, setInteractionOpen] = useState(false);
  const [reaction, setReaction] = useState<Interaction | null>(null);
  const reactionTimer = useRef<number | undefined>(undefined);
  const reactionCooldown = useRef(0);
  const interactionEntry = useRef<HTMLButtonElement>(null);
  const interactionPanel = useRef<HTMLElement>(null);
  const [settings, setSettings] = useState(defaultSettings);
  const theme = useColorTheme(settings.theme);
  const physics = usePetPhysics(settings.reducedMotion, settings.petSize);
  const character = getCharacter(settings.character);
  useEffect(() => { setMessage(character.greeting); setReaction(null); }, [character]);
  useEffect(() => () => window.clearTimeout(reactionTimer.current), []);
  useEffect(() => {
    if (interactionOpen) interactionPanel.current?.querySelector<HTMLButtonElement>(".interaction-options button")?.focus();
  }, [interactionOpen]);
  const acting = useRef(false);
  const revision = useRef(0);
  const root = useRef<HTMLElement>(null);
  const drag = useRef<{ x: number; y: number; moved: boolean } | null>(null);
  useEffect(() => {
    let disposed=false;
    const listener=listen<Settings>("settings-changed", event => setSettings(event.payload));
    void listener.then(() => invoke<Settings>("get_settings")).then(value => { if (!disposed) setSettings(value); }).catch(() => setError(true));
    return () => { disposed=true; void listener.then(unlisten => unlisten()); };
  }, []);

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
      listen("quit-requested", () => { setInteractionOpen(false); setResetConfirm(false); setQuit(true); }),
      listen("app-error", () => setError(true)),
      listen("timer-changed", () => void refresh()),
    ];
    return () => { disposed = true; window.clearInterval(interval); void Promise.all(listeners).then(items => items.forEach(unlisten => unlisten())); };
  }, []);

  useEffect(() => {
    const update = () => {
      const areas = Array.from(root.current?.querySelectorAll<HTMLElement>("[data-hit]") ?? []).filter(el => getComputedStyle(el).visibility !== "hidden").map(el => {
        const r = el.getBoundingClientRect();
        return { x: r.x, y: r.y, width: r.width, height: r.height, radius: (Number(el.dataset.hit) || 0) * (el.classList.contains("pet-stage") || el.classList.contains("reference-hit") ? settings.petSize / 160 : 1) };
      });
      void invoke("set_hit_areas", { areas }).catch(() => setError(true));
    };
    update();
    const observer = new ResizeObserver(update);
    if (root.current) observer.observe(root.current);
    // Also follows the small breathing motion of the pet's hit target.
    const interval = window.setInterval(update, 200);
    return () => { observer.disconnect(); window.clearInterval(interval); };
  }, [quit, resetConfirm, interactionOpen, expanded, settings.petSize]);

  function collapse() {
    setExpanded(false); setInteractionOpen(false);
    physics.stage.current?.focus();
  }

  function interact(kind: Interaction) {
    if (performance.now() < reactionCooldown.current) return;
    reactionCooldown.current = performance.now() + 500;
    window.clearTimeout(reactionTimer.current);
    setReaction(kind);
    setMessage(interactionMessage(character.id, kind));
    reactionTimer.current = window.setTimeout(() => setReaction(null), 2200);
  }
  function closeInteractions() { setInteractionOpen(false); (expanded ? interactionEntry.current : physics.stage.current)?.focus(); }

  async function act(action: string) {
    if (acting.current) return;
    acting.current = true; setBusy(true);
    revision.current += 1;
    try { setTimer(await invoke<Snapshot>("timer_action", { action })); setError(false); return true; }
    catch { setError(true); return false; }
    finally { acting.current = false; setBusy(false); }
  }

  const running = timer?.status === "running";
  const completed = timer?.status === "completed";
  const speech = error ? "暂时没能保存，请稍后重试。" : timer?.recovery ? recoveryMessage(timer.recoveryReason) : completed ? (timer.phase === "focus" ? "这一轮完成啦，休息一下吧！" : "休息好啦，准备开始下一轮？") : reaction ? message : running ? (timer.phase === "focus" ? "我会安静陪你，专心做眼前的事。" : "放松肩膀，也让眼睛休息一会儿。") : message;
  const action = timer ? primaryAction(timer) : { action: "start", label: "准备中" };

  return (
    <main ref={root} className={`pet-window ${expanded ? "is-expanded" : "is-compact"} ${settings.reducedMotion ? "reduce-motion" : ""}`} data-theme={theme} style={{ "--pet-scale": settings.petSize / 160 } as React.CSSProperties} onKeyDown={event => { if (event.key === "Escape" && !quit && !resetConfirm && !interactionOpen) { event.preventDefault(); collapse(); } }}>
      <div style={{ display: "contents" }} inert={quit || resetConfirm}>
      <section data-hit="22" className="speech-bubble" aria-live={expanded ? "polite" : "off"} aria-hidden={!expanded} inert={!expanded}
        onPointerDown={event => { if (event.button === 0) void getCurrentWindow().startDragging().catch(() => setError(true)); }} title="拖动气泡也可以移动小番茄">
        <p>{speech}</p><span className="speech-tail" aria-hidden="true" />
      </section>
      <button ref={physics.stage} data-hit="62" className="pet-stage" type="button" aria-label={`和${character.name}说话；按住拖动可移动`} aria-description="单击摸摸，双击展开或收起信息，按住拖动，右键打开互动；键盘 Enter 或空格切换信息" aria-expanded={expanded}
        onPointerDown={e => { if (e.button === 0) { drag.current = { x: e.screenX, y: e.screenY, moved: false }; e.currentTarget.setPointerCapture(e.pointerId); } }}
        onPointerMove={e => {
          const d = drag.current;
          if (d && !d.moved && Math.hypot(e.screenX-d.x, e.screenY-d.y) > 5) {
            d.moved = true; e.currentTarget.releasePointerCapture(e.pointerId);
            lastDrag.current = performance.now();
            setReaction(null);
            physics.lift();
            void getCurrentWindow().startDragging().catch(() => setError(true));
          }
        }}
        onPointerCancel={() => { if (drag.current) drag.current.moved = true; }}
        onContextMenu={e => { e.preventDefault(); setInteractionOpen(true); }}
        onDoubleClick={() => { if (performance.now() - lastDrag.current > 400) { setExpanded(value => !value); setInteractionOpen(false); } }}
        onClick={e => { if (e.detail === 0) { setExpanded(value => !value); setInteractionOpen(false); } else if (e.detail === 1 && !drag.current?.moved) interact("pet"); drag.current = null; }}>
        {cartoonIds.includes(character.id) && <span className="reference-hit" data-hit="18" aria-hidden="true" />}
        <span className="pet-depth"><span className={`tomato ${petMood(timer) === "focus" ? "is-focusing" : ""}`} data-character={character.id} data-mood={petMood(timer)} data-reaction={reaction ?? undefined}><PetFeatures character={character.id} />
        {reaction && <span className="pet-sparkles" aria-hidden="true"><i>♡</i><i>✦</i><i>♡</i></span>}
        </span></span>
      </button>
      <section data-hit="20" className="timer-card" aria-label="计时器" aria-hidden={!expanded} inert={!expanded}>
        <div className="timer-copy"><span className="timer-state">{timer ? `${phaseLabel(timer.phase)} · ${running ? "进行中" : timer.status === "paused" ? "已暂停" : completed ? "已完成" : "准备好了"}` : "正在准备"}</span><strong>{formatTime(timer?.remainingSeconds ?? 1500)}</strong></div>
        <div className="timer-actions">
          <button className="primary-action" disabled={!timer || busy} onClick={() => void act(action.action)}>{action.label}</button>
          <button className="icon-action" disabled={!timer || busy} onClick={() => { if (timer?.status === "running" || timer?.status === "paused") setResetConfirm(true); else void act(completed && timer?.phase === "focus" ? "skip" : "reset"); }} aria-label={completed ? "跳过，回到专注" : "提前结束并重置"} title={completed ? "跳过，回到专注" : "提前结束并重置"}>↻</button>
        </div>
        <div className="timer-progress" data-rest={timer?.phase !== "focus"} role="progressbar" aria-label="本轮进度" aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(progressPercent(timer))}><span style={{ width: `${progressPercent(timer)}%` }} /></div>
      </section>
      <footer data-hit="8" className="runtime-status" aria-hidden={!expanded} inert={!expanded}><span>已完成 {count} 次</span><button ref={interactionEntry} className="settings-entry" type="button" aria-label="打开互动" aria-expanded={interactionOpen} onClick={() => setInteractionOpen(value => !value)}>♡ 互动</button><button className="settings-entry" type="button" aria-label="打开设置" title="设置" onClick={() => void invoke("open_settings").catch(() => setError(true))}>⚙ 设置</button><button className="settings-entry" type="button" aria-label="收起信息栏" onClick={collapse}>收起</button></footer>
      {interactionOpen && <section ref={interactionPanel} data-hit="18" className="pet-interactions" aria-label="伙伴互动" onKeyDown={e => { if (e.key === "Escape") { e.preventDefault(); closeInteractions(); } }}>
        <div className="interaction-heading"><strong>和{character.name}待一会儿</strong><button type="button" aria-label="关闭互动" onClick={closeInteractions}>×</button></div>
        <div className="interaction-options">{([{ id: "pet", label: "摸摸", icon: "♡" }, { id: "wave", label: "打招呼", icon: "☀" }, { id: "stretch", label: "伸懒腰", icon: "↟" }] as const).map(item => <button type="button" key={item.id} onClick={() => { interact(item.id); closeInteractions(); }}><span aria-hidden="true">{item.icon}</span>{item.label}</button>)}</div>
        <p className="interaction-hint">点击角色也能摸摸它，按住可以拖动。<br />互动不会暂停或重置你的计时。</p>
      </section>}
      </div>
      {quit && <ConfirmDialog title="退出小番茄" confirmLabel="保存并退出" onCancel={() => setQuit(false)} onConfirm={() => void invoke("quit_app").catch(() => setError(true))}>先休息一下？计时进度会保存，下次回来可以继续。</ConfirmDialog>}
      {resetConfirm && <ConfirmDialog title="结束这一轮？" cancelLabel="继续这一轮" confirmLabel="结束并重置" busy={busy} onCancel={() => setResetConfirm(false)} onConfirm={() => { void act("reset").then(ok => { if (ok) setResetConfirm(false); }); }}>这一轮不会计入已完成次数。想稍后继续，可以先取消再暂停。</ConfirmDialog>}
    </main>
  );
}
export default App;
