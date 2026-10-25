import { HoloGlobe } from "./HoloGlobe";

/** How long the hold lasts before what it stands in front of opens. */
export const PLEASE_WAIT_MS = 1600;

/** The game's "Please wait democratically" hold before something opens. */
export function PleaseWait() {
  return (
    <div className="bp-wait" role="status">
      <HoloGlobe size={84} />
      <span className="caps">Please wait democratically</span>
    </div>
  );
}
