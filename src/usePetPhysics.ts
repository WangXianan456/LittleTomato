import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { gazeTarget, springStep } from "./petPhysics";

type PointerPose = { x: number; y: number; windowX: number; windowY: number; pressed: boolean };
export function usePetPhysics(reducedMotion: boolean, petSize: number) {
  const stage = useRef<HTMLButtonElement>(null);
  const lift = useRef(() => {});
  useEffect(() => {
    const element = stage.current;
    if (!element) return;
    const preference = window.matchMedia("(prefers-reduced-motion: reduce)");
    let disposed = false, lifted = false, frame = 0, lastTime = 0;
    let lastPose: PointerPose | null = null;
    let lastSample = 0;
    const target = [0, 0, 0, 0]; // gaze x/y, lift, drag sway
    const position = [0, 0, 0, 0], velocity = [0, 0, 0, 0];
    const disabled = () => reducedMotion || preference.matches || document.hidden;
    function render(time: number) {
      frame = 0;
      if (disposed) return;
      const dt = lastTime ? (time - lastTime) / 1000 : 1 / 60;
      lastTime = time;
      if (time - lastSample > 90) target[3] = 0;
      let moving = false;
      for (let i = 0; i < position.length; i++) {
        const next = springStep(position[i], velocity[i], disabled() ? 0 : target[i], dt);
        position[i] = next.position; velocity[i] = next.velocity;
        moving ||= Math.abs(position[i] - (disabled() ? 0 : target[i])) > .001 || Math.abs(velocity[i]) > .001;
      }
      element!.style.setProperty("--gaze-x", `${position[0] * 3.5}px`);
      element!.style.setProperty("--gaze-y", `${position[1] * 3}px`);
      element!.style.setProperty("--tilt-x", `${-position[1] * 8}deg`);
      element!.style.setProperty("--tilt-y", `${position[0] * 12}deg`);
      element!.style.setProperty("--sway", `${position[3]}deg`);
      element!.style.setProperty("--lift", `${position[2] * -9}px`);
      element!.style.setProperty("--lift-scale", `${1 + position[2] * .04}`);
      element!.dataset.lifted = String(lifted && !disabled());
      if (moving) frame = requestAnimationFrame(render);
    }
    function wake() { if (!frame && !disposed) { lastTime = 0; frame = requestAnimationFrame(render); } }
    function reset() { lifted = false; target.fill(0); wake(); }
    lift.current = () => { lifted = true; target[2] = 1; wake(); };
    const subscription = listen<PointerPose>("pet-pointer", ({ payload }) => {
      if (disposed || disabled()) return;
      const now = performance.now();
      const rect = element.getBoundingClientRect();
      const gaze = gazeTarget(payload.x, payload.y, rect.x + rect.width / 2, rect.y + rect.height / 2);
      target[0] = gaze.x; target[1] = gaze.y;
      if (lifted && !payload.pressed) { lifted = false; target[2] = 0; }
      if (lifted && lastPose) {
        const elapsed = Math.max(16, now - lastSample);
        target[3] = Math.max(-12, Math.min(12, -(payload.windowX - lastPose.windowX) / elapsed * 7));
      } else target[3] = 0;
      lastPose = payload; lastSample = now;
      wake();
    });
    preference.addEventListener("change", reset);
    document.addEventListener("visibilitychange", reset);
    wake();
    return () => {
      disposed = true; cancelAnimationFrame(frame); lift.current = () => {};
      preference.removeEventListener("change", reset);
      document.removeEventListener("visibilitychange", reset);
      void subscription.then(unlisten => unlisten());
    };
  }, [reducedMotion, petSize]);
  return { stage, lift: () => lift.current() };
}
