/** Pictures for the Galactic Wide Web, by the names `data/gww.ts` uses. */

const ICONS = import.meta.glob<string>("../../assets/gww/icons/*.svg", { eager: true, import: "default", query: "?url" });
const LOGOS = import.meta.glob<string>("../../assets/gww/logos/*.webp", { eager: true, import: "default", query: "?url" });
const ADS = import.meta.glob<string>("../../assets/gww/ads/*.webp", { eager: true, import: "default", query: "?url" });
const DSS = import.meta.glob<string>("../../assets/dss-actions/*.png", { eager: true, import: "default", query: "?url" });

/** A game icon: ministry, campaign type, difficulty, or a space station action. */
export const iconUrl = (name: string): string | undefined =>
  ICONS[`../../assets/gww/icons/${name}.svg`] ?? DSS[`../../assets/dss-actions/${name}.png`];

/** A corporation's logo; some are kept as icons. */
export const logoUrl = (name: string): string | undefined => LOGOS[`../../assets/gww/logos/${name}.webp`] ?? iconUrl(name);

export const adUrl = (name: string): string | undefined => ADS[`../../assets/gww/ads/${name}.webp`];
