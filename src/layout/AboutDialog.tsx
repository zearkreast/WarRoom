import { useEffect, useRef } from "react";
import { AboutModule, LegendModule } from "../features/hellpad/GwwModule";
import { useUi } from "../state";
import { KeyButton } from "../ui/primitives";

/** About or the icons legend in a modal over whatever page is open; ESC, the button or a click outside closes it. */
export function AboutDialog() {
  const { dialog, showDialog } = useUi();
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (dialog && !d.open) d.showModal();
    if (!dialog && d.open) d.close();
  }, [dialog]);
  return (
    <dialog
      ref={ref}
      className={`about-dialog ${dialog === "legend" ? "wide" : ""}`}
      aria-label={dialog === "legend" ? "Icons legend" : "About"}
      onClose={() => showDialog(null)}
      onClick={(e) => e.target === e.currentTarget && showDialog(null)}
    >
      {dialog && (
        <>
          {dialog === "about" ? <AboutModule /> : <LegendModule />}
          <div className="about-foot">
            <KeyButton k="ESC" onClick={() => showDialog(null)}>
              Close
            </KeyButton>
          </div>
        </>
      )}
    </dialog>
  );
}
