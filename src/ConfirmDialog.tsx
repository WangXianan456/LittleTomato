import { useEffect, useRef } from "react";

export default function ConfirmDialog({ title, children, confirmLabel, cancelLabel = "留下来", onCancel, onConfirm, busy = false }: {
  title: string; children: React.ReactNode; confirmLabel: string; cancelLabel?: string; onCancel: () => void; onConfirm: () => void; busy?: boolean;
}) {
  const dialog = useRef<HTMLElement>(null);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    dialog.current?.querySelector<HTMLButtonElement>("button")?.focus();
    return () => previous?.focus();
  }, []);
  return <section ref={dialog} data-hit="20" className="quit-dialog" role="dialog" aria-modal="true" aria-label={title} onKeyDown={event => {
    if (event.key === "Escape" && !busy) { event.preventDefault(); onCancel(); }
    if (event.key === "Tab") {
      const buttons = Array.from(dialog.current?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? []);
      event.preventDefault();
      const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
      buttons[(index + (event.shiftKey ? buttons.length - 1 : 1)) % buttons.length]?.focus();
    }
  }}><strong>{title}</strong><p>{children}</p><div className="confirm-actions"><button disabled={busy} onClick={onCancel}>{cancelLabel}</button><button disabled={busy} onClick={onConfirm}>{confirmLabel}</button></div></section>;
}
