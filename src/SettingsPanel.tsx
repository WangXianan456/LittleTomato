import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { defaultSettings, numberFields, validateSettings, type Settings, type SettingsDraft, type NumberKey } from "./settings";
import { useColorTheme } from "./appearance";
import "./SettingsPanel.css";
import PetFeatures from "./PetFeatures";
import { characters } from "./characters";

const tabs = [{ id: "characters", label: "伙伴小屋", icon: "♡" }, { id: "timer", label: "专注与休息", icon: "◷" }, { id: "appearance", label: "外观与陪伴", icon: "☀" }, { id: "general", label: "通用", icon: "⚙" }] as const;
type Tab = typeof tabs[number]["id"];
export default function SettingsPanel() {
  const [draft, setDraft] = useState<SettingsDraft>(defaultSettings);
  const [saved, setSaved] = useState<Settings | null>(null);
  const [tab, setTab] = useState<Tab>("timer");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const [failed, setFailed] = useState(false);
  const saving = useRef(false);
  const form = useRef<HTMLFormElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const theme = useColorTheme(draft.theme);
  const dirty = saved !== null && JSON.stringify(draft) !== JSON.stringify(saved);
  const validation = validateSettings(draft);

  async function load() {
    try { const next = await invoke<Settings>("get_settings"); setSaved(next); setDraft(next); setNotice(""); setFailed(false); }
    catch { setFailed(true); setNotice("暂时无法读取设置，请重试。"); }
  }
  useEffect(() => { void load(); }, []);
  useEffect(() => { content.current?.scrollTo({ top: 0 }); }, [tab]);
  useEffect(() => {
    const close = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !saving.current) void getCurrentWindow().hide();
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") { event.preventDefault(); form.current?.requestSubmit(); }
    };
    window.addEventListener("keydown", close);
    return () => window.removeEventListener("keydown", close);
  }, []);
  function update<K extends keyof SettingsDraft>(key: K, value: SettingsDraft[K]) {
    setDraft(current => ({ ...current, [key]: value })); setNotice(""); setFailed(false);
  }
  async function save(event: React.FormEvent) {
    event.preventDefault();
    if (saving.current || !saved || !dirty) return;
    if (validation) { setFailed(true); setNotice(validation); return; }
    saving.current = true; setBusy(true);
    try {
      const result = await invoke<{ settings: Settings; warning: string | null }>("save_settings", { settings: draft });
      setSaved(result.settings); setDraft(result.settings); setFailed(Boolean(result.warning));
      setNotice(result.warning ?? "已保存，小番茄准备好啦。");
    } catch (error) { setFailed(true); setNotice(typeof error === "string" ? error : "没有保存成功，请稍后重试。"); }
    finally { saving.current = false; setBusy(false); }
  }
  function numberField(key: NumberKey) {
    const field = numberFields[key];
    return <label className="duration-field" key={key}>
      <span>{field.label}</span>
      <span className="number-control"><input name={key} type="number" inputMode="numeric" min={field.min} max={field.max} step={1} required value={draft[key]} onFocus={e => e.target.select()} onChange={e => update(key, e.target.value === "" ? "" : Number(e.target.value))} /><span>{field.unit}</span></span>
      <small>{field.min}–{field.max} {field.unit}</small>
    </label>;
  }
  function toggle(key: "alwaysOnTop" | "reducedMotion" | "launchOnStartup", title: string, detail: string) {
    return <label className="setting-row"><span><strong>{title}</strong><small>{detail}</small></span><input className="switch" type="checkbox" name={key} checked={draft[key]} onChange={e => update(key, e.target.checked)} /></label>;
  }
  return <main className="settings-app" data-theme={theme}>
    <header className="settings-header"><span className="brand-tomato" aria-hidden="true">●</span><div><h1>小番茄设置</h1><p>找到适合你的陪伴节奏。</p></div><span className="local-badge">本地保存</span></header>
    <form ref={form} className="settings-form" onSubmit={save}>
      <div className="settings-body">
        <nav className="settings-nav" aria-label="设置分类">{tabs.map(item => <button key={item.id} type="button" aria-current={tab === item.id ? "page" : undefined} onClick={() => setTab(item.id)}><span aria-hidden="true">{item.icon}</span>{item.label}</button>)}</nav>
        <div ref={content} className="settings-content">
          {!saved && <div role="status" className="settings-loading">{failed ? <><p>{notice}</p><button type="button" onClick={() => void load()}>重新读取</button></> : "正在读取你的设置…"}</div>}
          <fieldset disabled={!saved || busy}>
            {tab === "characters" && <section aria-labelledby="characters-heading"><div className="section-heading"><span className="collection-label">MEET YOUR LITTLE FRIENDS</span><h2 id="characters-heading">今天，想和谁一起？</h2><p>五位小伙伴，同一份陪伴。选好后点击保存。</p></div><div className="character-grid" role="group" aria-label="选择桌宠角色">{characters.map(item => <button key={item.id} type="button" className="character-card" aria-pressed={draft.character === item.id} aria-label={item.name} onClick={() => update("character", item.id)}><span className="character-art reduce-motion" data-palette={item.id}><span className="tomato" data-character={item.id}><PetFeatures /></span></span><span className="character-name">{item.name}<span className="character-check" aria-hidden="true">{draft.character === item.id ? "✓" : "+"}</span></span><span className="character-subtitle">{item.subtitle}</span></button>)}</div><p className="character-hint">切换伙伴会保留你的计时与专注记录。</p></section>}
            {tab === "timer" && <section aria-labelledby="timer-heading"><div className="section-heading"><h2 id="timer-heading">专注与休息</h2><p>认真做一件事，也留一点时间休息。</p></div><div className="settings-card duration-grid">{(["focusMinutes", "shortBreakMinutes", "longBreakMinutes", "roundsBeforeLongBreak"] as NumberKey[]).map(numberField)}</div><div className="setting-note"><span aria-hidden="true">↳</span><p>当前进行中或暂停的计时保持不变。<br />新的时长将在下一轮开始时使用。</p></div></section>}
            {tab === "appearance" && <section aria-labelledby="appearance-heading"><div className="section-heading"><h2 id="appearance-heading">外观与陪伴</h2><p>安静一点，或靠近一点，都由你决定。</p></div>
              <div className="settings-card"><div className="size-setting"><label htmlFor="pet-size"><strong>桌宠大小</strong><span>{draft.petSize} px</span></label><input id="pet-size" name="petSize" type="range" min="80" max="240" step="10" value={draft.petSize} onChange={e => update("petSize", Number(e.target.value))} /><div className="range-labels"><span>小巧 · 80</span><span>默认 · 160</span><span>醒目 · 240</span></div><div className={`pet-preview ${draft.reducedMotion ? "reduce-motion" : ""}`} aria-label="桌宠大小预览"><div className="tomato" data-character={draft.character} style={{ zoom: Number(draft.petSize) / 240 }}><PetFeatures /></div><small>大小预览 · 保存后应用到桌面</small></div></div>{toggle("alwaysOnTop", "始终置顶", "让小番茄留在其他普通窗口前面")}{toggle("reducedMotion", "减少动画", "停止呼吸、眨眼和叶片摆动；也遵循系统偏好")}</div>
              <div className="settings-card theme-setting"><label htmlFor="theme"><strong>界面主题</strong><small>预览会立即更新</small></label><select id="theme" name="theme" value={draft.theme} onChange={e => update("theme", e.target.value as Settings["theme"])}><option value="system">跟随系统</option><option value="light">浅色</option><option value="dark">深色</option></select></div>
            </section>}
            {tab === "general" && <section aria-labelledby="general-heading"><div className="section-heading"><h2 id="general-heading">通用</h2><p>让小番茄融入你的日常。</p></div><div className="settings-card">{toggle("launchOnStartup", "开机启动", "登录 Windows 后自动打开小番茄")}</div><div className="privacy-card"><span aria-hidden="true">⌂</span><h3>只属于你的本地空间</h3><p>设置和专注记录保存在这台电脑。<br />无需登录，也不会上传你的记录。</p></div></section>}
          </fieldset>
        </div>
      </div>
      <footer className="settings-footer"><div className={`save-status ${failed ? "is-error" : ""}`} role={failed ? "alert" : "status"}>{notice || (dirty ? validation || "有尚未保存的修改" : "按自己的节奏来就好。")}</div><div className="settings-footer-actions"><button className="restore-button" type="button" disabled={!saved || busy} onClick={() => { setDraft(defaultSettings); setNotice("已填入推荐设置，点击保存后生效。"); setFailed(false); }}>恢复推荐设置</button><span /><button className="cancel-button" type="button" disabled={busy} onClick={() => { if (saved) setDraft(saved); setNotice(""); void getCurrentWindow().hide(); }}>取消</button><button className="save-button" type="submit" disabled={!saved || busy || !dirty || Boolean(validation)}>{busy ? "正在保存…" : "保存设置"}</button></div></footer>
    </form>
  </main>;
}
