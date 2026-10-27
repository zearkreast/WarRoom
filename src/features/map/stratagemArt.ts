/** The game's stratagem icons, as traced on the Helldivers Wiki, by the stratagem's name without its designation. */
const ICONS = import.meta.glob<string>("../../assets/stratagems/*.svg", { eager: true, import: "default", query: "?url" });

/** The stratagem an arsenal augmentation adds, as the game names it ("E/MG-101 HMG EMPLACEMENT"). */
export const augmentedStratagem = (effectName: string) => effectName.replace(/^ARSENAL AUGMENTATION:\s*/i, "");

/** The icon for a stratagem by its full name; Eagles and orbitals carry no designation to drop. */
export function stratagemIcon(name: string): string | undefined {
  const words = name.trim().split(/\s+/);
  const plain = /[\d/]/.test(words[0]) ? words.slice(1) : words;
  const slug = plain
    .join(" ")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
  return ICONS[`../../assets/stratagems/${slug}.svg`];
}
