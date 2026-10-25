// The game API's cache sometimes answers an English request with a copy made for a player in another language.

const ENGLISH = new Set(
  "the of and to is for on with our your are will be by that this from has have it as at their was".split(" "),
);
// Common words of the game's Latin-script languages that English never uses. No one-letter words: timestamps
// split into "t" and "z".
const OTHER = new Set(
  [
    "de la el los las que del por para con una su sus se al",
    "der das und ist den dem mit zu von des für ein eine nicht sich auf",
    "le les et du est une pour dans sur au aux avec ce qui",
    "il di che della dei delle nel gli",
    "os da das dos em um uma não ao pela pelo nas nos",
    "na się nie jest oraz że aby",
  ]
    .join(" ")
    .split(" "),
);

function collect(value: unknown, out: string[]) {
  if (typeof value === "string") out.push(value);
  else if (Array.isArray(value)) for (const v of value) collect(v, out);
  else if (value && typeof value === "object") for (const v of Object.values(value)) collect(v, out);
}

/** Whether an API response holds text in a language other than English. */
export function notEnglish(body: unknown): boolean {
  const strings: string[] = [];
  collect(body, strings);
  const text = strings.join(" ").replace(/<\/?i(=\d+)?>/g, " ");

  let letters = 0;
  let other = 0;
  for (const ch of text) {
    if (!/\p{L}/u.test(ch)) continue;
    letters++;
    if (!/[a-z]/i.test(ch)) other++;
  }
  // Letters beyond A-Z mean another script or accents. A share, not any one: English text names planets like Pöpli IX.
  if (other > letters * 0.02) return true;

  let en = 0;
  let foreign = 0;
  for (const word of text.toLowerCase().match(/\p{L}+/gu) ?? []) {
    if (ENGLISH.has(word)) en++;
    else if (OTHER.has(word)) foreign++;
  }
  return foreign >= 3 && foreign > en;
}
