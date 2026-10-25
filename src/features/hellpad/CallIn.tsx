import { useCallback, useEffect, useRef, useState } from "react";
import { CALL_IN } from "../../data/stratagems";
import { arrowOf, step, type Arrow } from "../../domain/stratagems";
import { useUi } from "../../state";
import { PLEASE_WAIT_MS, PleaseWait } from "../../ui/PleaseWait";

/** Opens the G.W.W. on its Super Destroyer manual, in one step so Back returns to the Hellpad. */
function openManual() {
  const { gww } = useUi.getState();
  const top = gww[gww.length - 1];
  const there = top?.kind === "manual" && top.id === "super-destroyers";
  useUi.setState({
    view: "hellpad",
    module: "gww",
    gww: there ? gww : [...gww, { kind: "manual" as const, id: "super-destroyers" }].slice(-50),
    options: null,
    detail: null,
  });
}

/** Keying in Call In Super Destroyer: how far it has got, a miss, and the hold once it is complete. */
export function useCallIn() {
  const [at, setAt] = useState(0);
  const [miss, setMiss] = useState(false);
  const [waiting, setWaiting] = useState(false);
  // Read synchronously, so fast presses each see the one before.
  const done = useRef(0);
  const flash = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(flash.current), []);

  /** Whether the press counted towards the code; idle, only its first arrow does. */
  const press = useCallback((a: Arrow) => {
    if (done.current === CALL_IN.code.length) return true;
    const next = step(done.current, a, CALL_IN.code);
    if (next === 0 && done.current === 0) return false;
    if (next <= done.current && done.current > 0) {
      setMiss(true);
      clearTimeout(flash.current);
      flash.current = setTimeout(() => setMiss(false), 400);
    }
    done.current = next;
    setAt(next);
    if (next === CALL_IN.code.length) {
      clearTimeout(flash.current);
      setMiss(false);
      setWaiting(true);
    }
    return true;
  }, []);
  const reset = useCallback(() => {
    done.current = 0;
    setAt(0);
    setWaiting(false);
  }, []);
  return { at, miss, waiting, press, reset };
}

const USES_ARROWS = "input, textarea, select, [contenteditable='true'], [role='slider'], [role='listbox'], [role='menu']";

/** The code shown steady; keying it in greys it, and each correct arrow lights up again. */
export function CallInCode({ at, miss, press }: { at: number; miss: boolean; press(a: Arrow): boolean }) {
  const box = useRef<HTMLDivElement>(null);
  // W A S D, being letters, only count with the code itself focused.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      const a = arrowOf(e.key);
      if (!a || document.querySelector("dialog[open]")) return;
      const onCode = document.activeElement === box.current;
      if (!onCode && (!e.key.startsWith("Arrow") || document.activeElement?.closest(USES_ARROWS))) return;
      if (press(a)) e.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [press]);
  const keying = at > 0 && at < CALL_IN.code.length;
  return (
    <div
      ref={box}
      className={`hp-code ${keying ? "keying" : ""} ${miss ? "miss" : ""}`}
      tabIndex={0}
      role="group"
      aria-label={`${CALL_IN.name}: ${CALL_IN.code.join(", ")}`}
    >
      <span className="hp-keys" aria-hidden>
        {CALL_IN.code.map((a, i) => (
          <i key={i} className={`hp-key ${a} ${!keying || i < at ? "on" : ""}`} />
        ))}
      </span>
      <span className="sr-only" aria-live="polite">
        {keying ? `${at} of ${CALL_IN.code.length}` : ""}
      </span>
    </div>
  );
}

const PAD: { arrow: Arrow; label: string }[] = [
  { arrow: "up", label: "Up" },
  { arrow: "right", label: "Right" },
  { arrow: "down", label: "Down" },
  { arrow: "left", label: "Left" },
];

/** The Hellpad mark's four arrows as keys, for touch screens. */
export function MarkPad({ press }: { press(a: Arrow): unknown }) {
  return (
    <span className="hp-mark-pad">
      {PAD.map((k) => (
        <button key={k.arrow} className={`hp-mark-key ${k.arrow}`} aria-label={`Stratagem key: ${k.label}`} onClick={() => press(k.arrow)} />
      ))}
    </span>
  );
}

/** "Please wait democratically", then the Super Destroyer manual; Esc calls it off. */
export function CallInWait({ open, onCancel }: { open: boolean; onCancel(): void }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const d = ref.current;
    if (!open || !d) return;
    d.showModal();
    const go = setTimeout(() => {
      d.close();
      onCancel();
      openManual();
    }, PLEASE_WAIT_MS);
    return () => {
      clearTimeout(go);
      if (d.open) d.close();
    };
  }, [open, onCancel]);
  return (
    <dialog ref={ref} className="briefing waiting" aria-label={CALL_IN.name} onClose={onCancel}>
      {open && <PleaseWait />}
    </dialog>
  );
}
