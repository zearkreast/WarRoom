import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, RotateCcw } from "lucide-react";
import { useCallback, useEffect, useRef, useState, type CSSProperties, type KeyboardEvent } from "react";
import { STRATAGEMS, UPLINK } from "../../data/stratagems";
import { arrowOf, dial, type Arrow } from "../../domain/stratagems";
import { useUi } from "../../state";
import { LiveDot } from "../../ui/primitives";

const SLOTS = Math.max(...STRATAGEMS.map((s) => s.code.length));
const PAD: { arrow: Arrow; icon: typeof ArrowUp; label: string }[] = [
  { arrow: "up", icon: ArrowUp, label: "Up" },
  { arrow: "left", icon: ArrowLeft, label: "Left" },
  { arrow: "down", icon: ArrowDown, label: "Down" },
  { arrow: "right", icon: ArrowRight, label: "Right" },
];

type Status = { tone: "idle" | "dialling" | "match" | "miss"; text: string };
const IDLE: Status = { tone: "idle", text: "Dial a code with the pad, or select the terminal and use W A S D or the arrow keys." };

function Keys({ code, lit = code.length }: { code: Arrow[]; lit?: number }) {
  return (
    <span className="hp-keys" aria-hidden>
      {code.map((a, i) => (
        <i key={i} className={`hp-key ${a} ${i < lit ? "on" : ""}`} style={{ "--i": i } as CSSProperties} />
      ))}
    </span>
  );
}

/** Opens the G.W.W. on its Super Destroyer manual, in one step so Back returns here. */
function openUplink() {
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

/** Dial stratagem codes as on the ground; the Super Destroyer Uplink opens its manual in the G.W.W. */
export function StratagemTerminal() {
  const [entered, setEntered] = useState<Arrow[]>([]);
  const [status, setStatus] = useState<Status>(IDLE);
  // Read synchronously, so fast presses each see the one before.
  const code = useRef<Arrow[]>([]);
  const busy = useRef(false);
  const timer = useRef<ReturnType<typeof setTimeout>>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);

  const reset = useCallback(() => {
    clearTimeout(timer.current);
    busy.current = false;
    code.current = [];
    setEntered([]);
    setStatus(IDLE);
  }, []);

  const press = useCallback(
    (a: Arrow) => {
      if (busy.current) return;
      const next = [...code.current, a];
      code.current = next;
      setEntered(next);
      const r = dial(next, STRATAGEMS);
      if (r.kind === "partial") return setStatus({ tone: "dialling", text: `Dialling: ${next.join(", ")}…` });
      busy.current = true;
      let then = reset;
      if (r.kind === "miss") setStatus({ tone: "miss", text: "No stratagem on file with that code." });
      else if (r.stratagem === UPLINK) {
        setStatus({ tone: "match", text: "Super Destroyer uplink established. Opening the G.W.W.…" });
        then = () => {
          reset();
          openUplink();
        };
      } else setStatus({ tone: "match", text: `${r.stratagem.name}: call-in confirmed.` });
      timer.current = setTimeout(then, r.kind === "match" && r.stratagem !== UPLINK ? 1600 : 900);
    },
    [reset],
  );

  // The terminal or anything in it takes every key. Elsewhere W A S D count only while nothing has focus,
  // so keyboard users moving through the page don't dial by accident.
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey || e.altKey || e.key.startsWith("Arrow")) return;
      if (document.activeElement && document.activeElement !== document.body) return;
      const a = arrowOf(e.key);
      if (a) press(a);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [press]);
  const onKey = (e: KeyboardEvent) => {
    const a = !e.ctrlKey && !e.metaKey && !e.altKey ? arrowOf(e.key) : undefined;
    if (!a) return;
    e.preventDefault();
    e.stopPropagation();
    press(a);
  };

  return (
    <section className={`hp-term ${status.tone}`} aria-label="Stratagem terminal" tabIndex={0} onKeyDown={onKey}>
      <header className="hp-term-head caps">
        <LiveDot ok={status.tone !== "miss"} /> Stratagem terminal
        <span className="hp-term-uplink">
          Uplink on file <Keys code={UPLINK.code} lit={0} />
          <span className="sr-only">: {UPLINK.code.join(", ")}</span>
        </span>
      </header>
      <div className="hp-term-body">
        <ol className="hp-term-slots" aria-label={`Dialled: ${entered.join(", ") || "nothing"}`}>
          {Array.from({ length: SLOTS }, (_, i) => (
            <li key={i} className={i < entered.length ? "on" : ""}>
              {entered[i] && <i className={`hp-key ${entered[i]} on`} />}
            </li>
          ))}
        </ol>
        <p className="hp-term-read" role="status">
          {status.text}
        </p>
        <div className="hp-term-pad">
          {PAD.map((k) => (
            <button key={k.arrow} className={`hp-pad ${k.arrow}`} aria-label={k.label} onClick={() => press(k.arrow)}>
              <k.icon size={18} strokeWidth={2.5} />
            </button>
          ))}
          <button className="hp-pad reset" aria-label="Clear the code" title="Clear" onClick={reset}>
            <RotateCcw size={15} />
          </button>
        </div>
      </div>
    </section>
  );
}
