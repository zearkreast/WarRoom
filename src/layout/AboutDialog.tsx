import { useEffect, useRef } from "react";
import { AboutModule } from "../features/hellpad/GwwModule";
import { useUi } from "../state";
import { KeyButton } from "../ui/primitives";

/** The About text in a modal over whatever page is open; ESC, the button or a click outside closes it. */
export function AboutDialog() {
  const { aboutOpen, showAbout } = useUi();
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (aboutOpen && !d.open) d.showModal();
    if (!aboutOpen && d.open) d.close();
  }, [aboutOpen]);
  return (
    <dialog
      ref={ref}
      className="about-dialog"
      aria-label="About"
      onClose={() => showAbout(false)}
      onClick={(e) => e.target === e.currentTarget && showAbout(false)}
    >
      {aboutOpen && (
        <>
          <AboutModule />
          <div className="about-foot">
            <KeyButton k="ESC" onClick={() => showAbout(false)}>
              Close
            </KeyButton>
          </div>
        </>
      )}
    </dialog>
  );
}
