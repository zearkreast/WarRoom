/**
 * Training manuals in reading order, and the game's Training Manual Tips. Manual text is adapted from the
 * Helldivers Wiki in this app's words (credited in `sources`) and kept in-universe, like the sites in gwwSites.ts;
 * the game's quotes are word for word. Text links are written `[[kind:target|label]]` (domain/gwwText.ts).
 */

export interface Source {
  title: string;
  url: string;
}

/** Pages adapted from the wiki share its licence. */
export const WIKI_LICENCE: Source = {
  title: "CC BY-NC-SA 4.0",
  url: "https://creativecommons.org/licenses/by-nc-sa/4.0/",
};

const wiki = (title: string, page = title.replace(/ /g, "_")): Source => ({
  title,
  url: `https://helldivers.wiki.gg/wiki/${page}`,
});

export const WIKI = {
  home: { title: "Helldivers Wiki", url: "https://helldivers.wiki.gg/" },
  mechanics: wiki("Second Galactic War Mechanics"),
  tips: wiki("Training Manual Tips"),
  corporations: wiki("Corporations"),
  minorCorporations: wiki("Minor Corporations"),
  government: wiki("Government of Super Earth"),
  libertyDay: wiki("Liberty Day"),
  malevelonDay: wiki("Malevelon Creek Memorial Day"),
  wallOfMartyrs: wiki("Wall of Martyrs"),
  ministryOfScience: wiki("Ministry of Science"),
  admiralDefense: wiki("Admiral Defense"),
  dss: wiki("Democracy Space Station"),
  superDestroyer: wiki("Super Destroyer"),
  shipModules: wiki("Ship Modules"),
} satisfies Record<string, Source>;

export interface Section {
  heading: string;
  text?: string;
  /** "Term: text" points get their term set in bold. */
  points?: string[];
  /** `icon` names a picture from the G.W.W. art, drawn at the start of its row. */
  table?: { head: string[]; rows: { icon?: string; cells: string[] }[]; note?: string };
  /** A diagram drawn by the app. */
  figure?: "supply-lines" | "siege" | "decay" | "impact-live";
  /** Named kinds of something, each with its picture. */
  kinds?: { icon: string; name: string; text: string }[];
  faq?: { q: string; a: string }[];
  /** Groups shown folded, each a titled list of "Term: text" points. */
  folds?: { title: string; note: string; points: string[] }[];
}

export interface Chapter {
  id: string;
  title: string;
  summary: string;
}

/** In course order; MANUALS are listed chapter by chapter in the same order. */
export const CHAPTERS: Chapter[] = [
  { id: "war", title: "The war", summary: "What a campaign is, and where one can open." },
  { id: "fronts", title: "Fronts", summary: "Liberating planets, holding them, and losing them." },
  { id: "tide", title: "Turning the tide", summary: "What every mission is worth, and the station Helldivers fund." },
  { id: "story", title: "The bigger picture", summary: "Galactic Campaigns, Major Orders and the Gloom." },
  { id: "appendix", title: "Appendix", summary: "Questions from the barracks, answered." },
];

export interface Manual {
  id: string;
  chapter: Chapter["id"];
  title: string;
  /** Address shown on the tile and in the address bar, in the Federation's net style. */
  address: string;
  summary: string;
  /** A picture from the G.W.W. art; manuals without one get a drawn icon. */
  icon?: string;
  sections: Section[];
  /** The game's own Training Manual Tips that fit the manual, quoted. */
  quotes?: string[];
  /** Tips written for this app, mostly on where to see the mechanic in it. */
  tips: string[];
  /** Pages to read next, as link targets (`manual:defense`, `app:map`). */
  related: string[];
  sources?: Source[];
}

/** Influence per mission, for the mission that completes an operation, and for a whole operation. */
const DIFFICULTY_INFLUENCE: [string, string, number, number, number][] = [
  ["trivial", "Trivial", 0, 136, 136],
  ["easy", "Easy", 0, 144, 144],
  ["medium", "Medium", 40, 160, 200],
  ["challenging", "Challenging", 44, 176, 220],
  ["hard", "Hard", 50, 200, 300],
  ["extreme", "Extreme", 58, 232, 348],
  ["suicide", "Suicide Mission", 67, 268, 402],
  ["impossible", "Impossible", 77, 308, 462],
  ["helldive", "Helldive", 89, 356, 534],
  ["super-helldive", "Super Helldive", 102, 408, 612],
];

const n = (v: number) => v.toLocaleString("en-US");

export const MANUALS: Manual[] = [
  {
    id: "basics",
    chapter: "war",
    title: "How the Galactic War works",
    address: "se://hc/manuals/basics",
    summary: "The war on one page: impact, planet health, supply lines and campaigns.",
    sections: [
      {
        heading: "The short version",
        text: "Every mission won deals [[manual:impact|impact]] to the planet it was fought on. Each planet in a campaign has a health pool; empty it and the campaign ends in Super Earth's favour. Campaigns open only along [[manual:supply-lines|supply lines]], and every hour the enemy wins back part of each planet it holds: its [[manual:resistance|resistance]]. The war never resets, and High Command steers it with [[manual:campaigns|Major Orders]].",
      },
      {
        heading: "Field glossary",
        points: [
          "Impact: what a won mission deals to a planet. It grows with difficulty and is scaled by the impact modifier; see [[manual:impact|Impact and the impact modifier]].",
          "Impact modifier: a galaxy-wide multiplier that falls as more missions are completed, so the war moves at much the same pace however many Helldivers are online.",
          "Planet health: the pool a campaign has to empty. The liberation percentage on the galactic map is drawn from it; see [[manual:liberation|Liberation campaigns]].",
          "Supply lines: the warp links between planets. A campaign needs a link to a Super Earth planet to open and to keep running.",
          "Enemy resistance: the share of a planet's health the enemy wins back every hour, also called decay.",
          "Defense: a campaign with a deadline and no resistance, opened when the enemy attacks a Super Earth planet; see [[manual:defense|Defense campaigns and gambits]].",
          "Gambit: winning a defense by liberating the planet the attack came from.",
          "High-priority campaign: a campaign with its own missions and rules; see [[manual:campaign-types|Campaign types]].",
          "Regions: settlements, cities and megafactories with their own health and resistance; freeing one hits the whole planet hard. See [[manual:regions|Regions and cities]].",
        ],
      },
      {
        heading: "Following the war",
        points: [
          "[[app:campaigns|Active planets]]: every front, its rate per hour, and when it should fall or hold.",
          "[[app:map|Galaxy map]]: supply lines, attacks, points of interest and the Gloom.",
          "[[app:overview|High Command overview]]: Helldivers on each front and the impact modifier over the last day, with the running Galactic Campaign.",
          "[[app:stations|Space stations]]: where the Democracy Space Station is and what it is doing.",
          "[[app:past|Past campaigns]] and the [[site:log|war log]]: what has already been won and lost.",
        ],
      },
    ],
    quotes: ["Managed Democracy is a cornerstone of advanced civilization."],
    tips: ["Read the manuals in order and each one builds on the last; the G.W.W. front page remembers how far you got."],
    related: ["manual:supply-lines", "manual:campaign-types", "site:gov"],
    sources: [WIKI.mechanics],
  },
  {
    id: "supply-lines",
    chapter: "war",
    title: "Supply lines",
    address: "se://hc/manuals/supply-lines",
    summary: "Which planets can be fought for, and why cutting a line matters.",
    sections: [
      {
        heading: "Warp links",
        text: "Planets are joined by warp links, usually called supply lines. A campaign can only start, and only keeps running, on a planet linked to at least one Super Earth planet; a defense needs the planet itself to be Super Earth's.",
        figure: "supply-lines",
      },
      {
        heading: "Planet attacks",
        text: "When two linked planets belong to different sides, one can attack the other along the link. An enemy attack opens a [[manual:defense|defense]]; a Super Earth attack opens a [[manual:liberation|liberation]]. Liberating a planet opens new campaigns from it.",
      },
      {
        heading: "Cut off",
        text: "Lose every Super Earth planet linked to a campaign and it stops: no new operations, while decay eats the progress made. Surround an enemy planet completely and the SEAF lays siege to it instead; see [[manual:resistance|Enemy resistance and sieges]].",
      },
      {
        heading: "Reading the map",
        text: "On the [[app:map|galaxy map]] the Galaxy overlay draws every supply line and a dashed line marks each attack, from the attacker to its target. Every planet's own page lists the planets it is linked to.",
      },
    ],
    quotes: ["If the enemies just keep coming, find out where they're coming from and unleash hell. Big hive? Destroy! Bot factory? Destroy!"],
    tips: ["Before joining a front, look at what it links to: taking it can open new campaigns, or leave an enemy planet cut off."],
    related: ["manual:campaign-types", "manual:defense", "manual:resistance", "app:map"],
    sources: [WIKI.mechanics],
  },
  {
    id: "campaign-types",
    chapter: "war",
    title: "Campaign types",
    address: "se://hc/manuals/campaign-types",
    summary: "Liberation, defense, and the rarer kinds High Command calls for.",
    icon: "campaign-high-priority",
    sections: [
      {
        heading: "Every planet you can fight on has a campaign",
        text: "Helldivers can only deploy to a planet while a campaign is active on it, and each kind has its own rules.",
        kinds: [
          {
            icon: "campaign-liberation",
            name: "Liberation",
            text: "No time limit. Empty the planet's health faster than its resistance refills it. See [[manual:liberation|Liberation campaigns]].",
          },
          {
            icon: "campaign-defense",
            name: "Defense",
            text: "A deadline and no resistance. Empty the defense's health in time or lose the planet. See [[manual:defense|Defense campaigns and gambits]].",
          },
          {
            icon: "campaign-urgent-liberation",
            name: "Urgent liberation",
            text: "A defense in reverse: Super Earth attacks an enemy planet and has to take it before the timer runs out.",
          },
          {
            icon: "campaign-invasion",
            name: "Invasion",
            text: "Brought by the Illuminate in their first months: like a defense, but with operation modifiers and no attacking planet, mostly in colony cities. Losing one left the planet in Super Earth's hands.",
          },
          {
            icon: "campaign-recon",
            name: "Recon",
            text: "Planets Super Earth will not liberate, kept open while their sector is active.",
          },
          {
            icon: "campaign-high-priority",
            name: "High-priority",
            text: "Campaigns with missions and rules of their own, cleared by following the [[manual:campaigns|Major Order]]. They usually open with a very high resistance that drops as milestones are reached, provided most Helldivers take part.",
          },
        ],
      },
      {
        heading: "Where to see it",
        text: "Cards on [[app:campaigns|Active planets]] say whether a front is a liberation or a defense; defenses carry their deadline, and Major Order targets their badge.",
      },
    ],
    quotes: ["Each faction presents unique challenges. Mix up your gear and try new tactics!"],
    tips: ["The Major Order strip at the top of Active planets says, for each target, whether it makes the deadline at the current pace."],
    related: ["manual:liberation", "manual:defense", "manual:campaigns"],
    sources: [WIKI.mechanics],
  },
  {
    id: "liberation",
    chapter: "fronts",
    title: "Liberation campaigns",
    address: "se://hc/manuals/liberation",
    summary: "Planet health, what the liberation bar really measures, and what a cut supply line does.",
    icon: "campaign-liberation",
    sections: [
      {
        heading: "The objective",
        text: "A liberation campaign has no time limit. Behind the liberation percentage each planet has a health pool: every won mission takes a chunk off it, completing an operation adds a bonus, and higher difficulties take more. At zero the planet returns to Super Earth and new campaigns open from it.",
      },
      {
        heading: "Planet health",
        text: "A planet without regions starts with a million health, with a few exceptions. Regions add their own health on top; see [[manual:regions|Regions and cities]].",
        table: {
          head: ["Planet", "Health"],
          rows: [
            { cells: ["Most planets", n(1_000_000)] },
            { cells: ["Magma, deciduous and autumn forest, desert oasis", `Usually ${n(2_000_000)}`] },
            { cells: ["[[planet:Zagon Prime|Zagon Prime]]", n(2_000_000)] },
            { cells: ["[[planet:Omicron|Omicron]]", n(3_000_000)] },
            { cells: ["[[planet:Cyberstan|Cyberstan]]", `${n(4_250_000)}; ${n(15_000_000)} with its megafactories`] },
          ],
        },
      },
      {
        heading: "When the line is cut",
        text: "If Super Earth loses every [[manual:supply-lines|supply line]] to the planet, no new operations can start there, and the progress already made keeps decaying until a line is opened again.",
      },
      {
        heading: "Operation modifiers",
        text: "Liberation operations often come with one or two operation modifiers, depending on difficulty. Defense missions never have them, though weather and other planet conditions still apply.",
      },
      {
        heading: "Reading the figures",
        text: "Liberation is worked out from the planet's health as the galactic map does it: the share of its maximum already taken. The rate per hour is the trend across the last hour of readings, with [[manual:resistance|resistance]] already taken off, and each card's projection assumes that rate holds.",
      },
    ],
    quotes: ["The Galactic War is a united effort. Only by working together can we ensure planets don't fall to the enemy."],
    tips: ["A rate above zero on [[app:campaigns|Active planets]] means the planet is on its way; the card says when it should fall at that pace."],
    related: ["manual:resistance", "manual:regions", "manual:impact", "app:campaigns"],
    sources: [WIKI.mechanics],
  },
  {
    id: "resistance",
    chapter: "fronts",
    title: "Enemy resistance and sieges",
    address: "se://hc/manuals/resistance",
    summary: "Why some fronts stall for days, and how a surrounded planet falls on its own.",
    sections: [
      {
        heading: "Resistance is decay",
        text: "Every hour the enemy wins back a fixed share of a planet's health. The galactic map calls it Enemy Resistance; Helldivers call it decay. A front only moves while missions deal more per hour than it wins back. The share is of the planet's own health pool, so the same percentage can mean very different amounts on two planets.",
        figure: "decay",
      },
      {
        heading: "Turnout counts twice",
        text: "Helldivers who only match the decay make no progress. On a front that is barely winning, every extra Helldiver also saves the time of everyone already there, so a few more can finish it much sooner.",
      },
      {
        heading: "It isn't difficulty",
        text: "Resistance has nothing to do with how hard missions on the planet are or how many enemies they field.",
      },
      {
        heading: "Sieges",
        text: "An enemy planet completely surrounded by Super Earth planets is put under siege by the SEAF: its decay turns negative and it starts liberating itself, at between 0.5% and 1% an hour depending on the planet's health. Only a planet entirely on its own counts: cutting off a group of two or more planets does not start a siege.",
        figure: "siege",
      },
      {
        heading: "Where to see it",
        text: "Each planet's page sets the enemy's regeneration per hour beside the Helldivers' impact per hour, and the [[app:stations|space station page]] does the same for the planet the station orbits.",
      },
    ],
    tips: ["A front whose rate sits near zero for hours is one where turnout and resistance cancel out; a few more Helldivers tip it."],
    related: ["manual:liberation", "manual:supply-lines", "manual:misconceptions"],
    sources: [WIKI.mechanics],
  },
  {
    id: "regions",
    chapter: "fronts",
    title: "Regions and cities",
    address: "se://hc/manuals/regions",
    summary: "Settlements, cities and megafactories: health of their own, and a bonus for freeing them.",
    sections: [
      {
        heading: "A fight of their own",
        text: "Many planets carry regions: settlements, towns, cities and mega cities, and on Automaton worlds, megafactories. Each has its own health and its own resistance, generally about 1% of its health an hour, which keeps working during defenses too.",
      },
      {
        heading: "Health and the bonus for freeing one",
        table: {
          head: ["Region", "Rank", "Health", "Dealt to the planet when freed"],
          rows: [
            { cells: ["Settlement", "★", n(100_000), n(150_000)] },
            { cells: ["Town", "★★", n(200_000), n(300_000)] },
            { cells: ["City", "★★★", n(400_000), n(600_000)] },
            { cells: ["Mega city", "★★★★", n(600_000), n(900_000)] },
            { cells: ["Megafactory, class 1", "★", n(1_000_000), n(1_500_000)] },
            { cells: ["Megafactory, class 2", "★★", n(1_250_000), n(1_875_000)] },
            { cells: ["Megafactory, class 3", "★★★", n(1_500_000), n(2_250_000)] },
            { cells: ["Megafactory, class 4", "★★★★", n(2_000_000), n(3_000_000)] },
          ],
          note: "Each region adds its health to the planet's maximum; freeing it deals one and a half times that to the planet.",
        },
      },
      {
        heading: "Locked regions",
        text: "Some regions only open as the planet's liberation advances, or, during a [[manual:defense|defense]], as time passes. Regions can carry modifiers of their own, such as the Illuminate's Leviathan Blockade from Impossible difficulty up.",
      },
      {
        heading: "Maximum Security Cities",
        text: "New Hope City, New Aspiration City and New Yearning City are the Federation's three Maximum Security Cities. They are marked on the [[app:map|galaxy map]] as points of interest.",
      },
    ],
    quotes: ["Missions are filled with unexplored points of interest. Check them out—you never know what you might find."],
    tips: ["A planet's cities are listed under its operational parameters, on its own page."],
    related: ["manual:liberation", "manual:defense", "manual:impact"],
    sources: [WIKI.mechanics],
  },
  {
    id: "defense",
    chapter: "fronts",
    title: "Defense campaigns and gambits",
    address: "se://hc/manuals/defense",
    summary: "Deadlines, how strong an attack is, and winning a defense by taking the planet behind it.",
    icon: "campaign-defense",
    sections: [
      {
        heading: "Holding a planet",
        text: "When the enemy attacks a Super Earth planet along a [[manual:supply-lines|supply line]], the planet gets a defense campaign. It has its own health, set by the attack's strength and usually far below a liberation's million: the defense level the galactic map shows for the planet, times 50,000. There is no resistance while it runs.",
      },
      {
        heading: "Deadlines",
        table: {
          head: ["Planet", "Deadline"],
          rows: [
            { cells: ["Most planets", "24 hours"] },
            { cells: ["With a mega city or megafactory", "48 hours"] },
            { cells: ["Attack led by the Jet Brigade", "Usually 24 hours"] },
            { cells: ["Magma, deciduous and autumn forest, oasis", "48 hours, Jet Brigade or not"] },
          ],
        },
      },
      {
        heading: "If it falls",
        text: "Miss the deadline and the planet goes to the enemy at 0% liberation, however close the defense came. A liberation campaign opens for it if a linked planet is still held.",
      },
      {
        heading: "Regions under attack",
        text: "[[manual:regions|Regions]] weigh more in a defense, since missions there damage both the region and the defense. More regions come under threat as the deadline nears.",
      },
      {
        heading: "Gambits",
        text: "Every attack comes from an enemy planet. Liberate that planet before the deadline and the defense ends in Super Earth's favour: a gambit. It only works when the attacker's health and resistance are low enough to finish in time, and the galactic map never points the attacker out, which makes gambits hard to rally behind. The [[manual:dss|Democracy Space Station]]'s Orbital Blockade doesn't win one for you; it only stops new attacks from its planet.",
        figure: "supply-lines",
      },
      {
        heading: "Where to see it",
        text: "Defense cards on [[app:campaigns|Active planets]] count down to the deadline and project where the defense will stand when time runs out. The attack line on the [[app:map|galaxy map]] points from the gambit to its target.",
      },
    ],
    quotes: [
      "See a planet that's on the brink of falling under enemy control? Rally your fellow Helldivers and mount a defense. Every mission counts!",
    ],
    tips: ["The live panel on this page names the planet behind each attack: that attacker is the gambit."],
    related: ["manual:supply-lines", "manual:regions", "manual:dss", "ministry:defense"],
    sources: [WIKI.mechanics],
  },
  {
    id: "impact",
    chapter: "tide",
    title: "Impact and the impact modifier",
    address: "se://hc/manuals/impact",
    summary: "How a mission turns into liberation, and why one counts for more on a quiet night.",
    icon: "difficulty-helldive",
    sections: [
      {
        heading: "Mission influence",
        text: "A won mission produces influence, set by its difficulty. Finishing a whole operation multiplies the last mission's influence by four. Only the main objective counts, and each mission counts once for its whole squad: sending Helldivers away halfway through adds nothing.",
        table: {
          head: ["Difficulty", "Mission", "Last of an operation", "Whole operation"],
          rows: DIFFICULTY_INFLUENCE.map(([icon, name, mission, last, whole]) => ({
            icon: `difficulty-${icon}`,
            cells: [name, String(mission), String(last), String(whole)],
          })),
          note: "Operations get longer at higher difficulties, so the whole operation adds up more missions.",
        },
      },
      {
        heading: "The modifier",
        text: "Influence is multiplied by the galactic impact modifier, and the result, impact, comes off the planet's or the defense's health. High Command hasn't published the exact formula. The modifier falls as more missions are completed across the galaxy, so what moves a planet is the share of active Helldivers fighting on it, not their raw number.",
        figure: "impact-live",
      },
      {
        heading: "What it means in the field",
        points: [
          "A flood of Trivial missions doesn't help; it pushes the modifier down for every front.",
          "Higher difficulties carry much more influence.",
          "Rushing only the main objectives doesn't help either; the modifier drops to make up for it.",
          "Liberation comes only from finished missions, so a planet takes a few hours after Helldivers arrive before its rate shows their work. That is the ramp-up time.",
        ],
      },
    ],
    quotes: ["More challenging missions yield greater rewards!"],
    tips: ["[[app:overview|High Command overview]] charts the modifier under the Helldivers on each front over the last day."],
    related: ["manual:liberation", "manual:dss", "manual:misconceptions", "app:overview"],
    sources: [WIKI.mechanics],
  },
  {
    id: "dss",
    chapter: "tide",
    title: "The Democracy Space Station",
    address: "se://hc/manuals/dss",
    summary: "The Helldiver-funded station: its votes, its boost and its tactical actions.",
    sections: [
      {
        heading: "Votes",
        text: "Unless High Command takes it out of service, Helldivers vote every four hours on the station's next planet, from the planets with the most Helldivers when the vote opened. Liberation runs 10% faster on the planet it orbits. The [[app:stations|space station page]] counts down to the next jump.",
      },
      {
        heading: "Tactical actions",
        text: "Helldivers fund the station's tactical actions by donating resources such as samples. Only one action can run at a time, and donations made while one runs are lost. The station's estimate of when an action will be ready looks at a very short window, so it swings a lot.",
        kinds: [
          {
            icon: "heavy_ordnance",
            name: "Heavy Ordnance Distribution",
            text: "Counts the liberation on its planet more than once, likely twice. That lowers the galaxy-wide [[manual:impact|modifier]], so in effect it moves impact from other planets to this one. It works on every kind of campaign, defenses included. Planetary Bombardment had the same effect and is now left to High Command.",
          },
          {
            icon: "eagle_storm",
            name: "Eagle Storm",
            text: "Pauses the enemy's progress, which can stretch a [[manual:defense|defense]] by a day if the station stays for the whole action. The enemy's progress itself doesn't move back, so regions that open with it stay as they were. In October 2186 [[company:cob-genetics|C.O.B. Genetics]] helped refit it to spread aerosol deterrents.",
          },
          {
            icon: "orbital_blockade",
            name: "Orbital Blockade",
            text: "Stops new defenses from being launched from its planet. Defenses already running are untouched; missions there get Hellpod Space Optimization.",
          },
        ],
      },
      {
        heading: "The Star of Peace",
        text: "In January 2186 the station was fitted with the Star of Peace, a beam of Dark Fluid that collapsed [[planet:Penta|Penta]] into a black hole on 18 January. The shot spent all the Dark Fluid Super Earth had, and the weapon has been unarmed since. The [[app:stations|space station page]] tells its story, and that of the Automaton superweapon destroyed over [[planet:Mort|Mort]] in July.",
      },
    ],
    tips: ["The [[app:stations|space station page]] shows which actions are running now and when the others are ready again."],
    related: ["app:stations", "manual:defense", "company:cob-genetics"],
    sources: [WIKI.mechanics, WIKI.dss],
  },
  {
    id: "super-destroyers",
    chapter: "tide",
    title: "Super Destroyers",
    address: "se://hc/manuals/super-destroyers",
    summary: "The Helldivers' own warships: what they do in a mission, what is on board, and the modules you can upgrade.",
    sections: [
      {
        heading: "The ship",
        text: "A Super Destroyer is the Helldivers' warship and their base between missions. It keeps its Helldivers in cryonic storage, and each one thawed is given full command of the ship. An Alcubierre drive fuelled by Element-710 takes it between star systems in seconds, and about fifty crew are aboard at any time.",
      },
      {
        heading: "In a mission",
        points: [
          "Stratagems: strikes from its orbital cannons, Eagles from its hangar, and the supplies and weapons it drops.",
          "The mission timer: when it runs out the ship leaves low orbit and sends only the extraction shuttle, with no more stratagems or reinforcements. Blitz missions get 12 minutes, eradication 15, defense 25 and the rest 40.",
          "Commando missions: enemy anti-air allows it two minutes in low orbit in all, one at the drop and one more if a Helldiver calls it back with the Call In Super Destroyer stratagem.",
          "Traitors: a Helldiver branded a traitor, for leaving the mission area or killing too many citizens, is shelled with 380mm rounds until dead.",
        ],
      },
      {
        heading: "On board",
        points: [
          "Bridge: the Galactic War Table, where missions are chosen, with the Democracy Officer beside it; the hellpods wait past a short stair.",
          "Armory hallway: the Armory for loadouts, and the Control Center terminal with past and current campaigns and Major Orders.",
          "Main hall: the ship module terminal, the Ship Master and the Service Technician.",
          "Hangar: the Eagle-1 and the Pelican-1 extraction shuttle.",
        ],
      },
      {
        heading: "Ship modules",
        text: "Six parts of the ship can be upgraded, five tiers each, paid for in samples and, from the fourth tier, Requisition Slips. An upgrade only helps the Helldiver who buys it, and it changes how the ship looks inside and out.",
        folds: [
          {
            title: "Patriotic Administration Center",
            note: "Support weapons and backpacks",
            points: [
              "Donation Access License: support weapons arrive with as many magazines as can be carried.",
              "Streamlined Request Process: support weapon cooldowns 10% shorter.",
              "Hand Carts: backpack cooldowns 10% shorter.",
              "Superior Packing Methodology: resupply boxes refill support weapons to every magazine they can carry.",
              "Payroll Management System: support weapons reload 10% faster.",
            ],
          },
          {
            title: "Orbital Cannons",
            note: "Orbital strikes",
            points: [
              "Exploding Shrapnel: less damage falloff away from the centre of orbital explosions.",
              "More Guns: barrages fire one more salvo.",
              "Zero-G Breech Loading: orbital cooldowns 10% shorter.",
              "Atmospheric Monitoring: the orbital HE barrage spreads 15% less.",
              "High-Density Explosives: orbital explosions reach 10% wider.",
            ],
          },
          {
            title: "Hangar",
            note: "Eagles",
            points: [
              "Liquid-Ventilated Cockpit: Eagle cooldowns 50% shorter.",
              "Pit Crew Hazard Pay: Eagle Rearm 20% faster.",
              "Expanded Weapons Bay: one more use of each Eagle between rearms.",
              "XXL Weapons Bay: Eagles that drop several bombs drop one more.",
              "Advanced Crew Training: Eagle Rearm a further 10% faster when called with uses left.",
            ],
          },
          {
            title: "Bridge",
            note: "Every stratagem, the hellpod and the radar",
            points: [
              "Targeting Software Upgrade: orbital call-ins one second faster.",
              "Nuclear Radar: the minimap picks up enemies 50% farther out.",
              "Power Steering: better hellpod steering on the way down.",
              "Enhanced Combustion: fire damage from stratagems 25% higher.",
              "Morale Augmentation: every stratagem cooldown 5% shorter.",
            ],
          },
          {
            title: "Engineering Bay",
            note: "Sentries, emplacements and support weapons",
            points: [
              "Synthetic Supplementation: sentry, emplacement and resupply cooldowns 10% shorter.",
              "Advanced Construction: sentries have 50% more health.",
              "Rapid Launch System: emplacements launch the moment they are called in.",
              "Circuit Expansion: lightning arcs from weapons and turrets jump to one more enemy.",
              "Streamlined Launch Process: support weapons launch the moment they are called in.",
            ],
          },
          {
            title: "Robotics Workshop",
            note: "Sentries",
            points: [
              "Dynamic Tracking: sentries launch the moment they are called in.",
              "Shock Absorption Gel: sentries carry 50% more ammunition.",
              "High-Quality Lubricant: sentries turn to new targets faster.",
              "Blast Absorption: sentries take 50% less damage from explosions.",
              "Cross-Platform Compatibility: mortar sentries fire at marked targets first.",
            ],
          },
        ],
      },
    ],
    tips: [
      "Each Helldiver's stratagems follow their own ship's modules, so in a squad the same orbital can come down faster for one Helldiver than another.",
      "Key the Call In Super Destroyer code, ↑ ↑ ↓ ↓ ← → ← →, into the Hellpad's masthead and it opens this page.",
    ],
    related: ["manual:dss", "manual:campaigns", "app:stations"],
    sources: [WIKI.superDestroyer, WIKI.shipModules],
  },
  {
    id: "campaigns",
    chapter: "story",
    title: "Campaigns and orders",
    address: "se://hc/manuals/campaigns",
    summary: "Galactic Campaigns, Major Orders and strategic directives, and how rewards are earned.",
    sections: [
      {
        heading: "Galactic Campaigns",
        text: "A Galactic Campaign is a named operation fought through a run of Major Orders, usually three over about two weeks. Winning most of its Major Orders earns the campaign's own reward: so far a stratagem, a weapon, armour, a cape or a tactical action.",
      },
      {
        heading: "Major Orders",
        text: "Each Major Order sets objectives for every Helldiver with a deadline and pays medals when it is won. A won or lost order also decides what comes next in the campaign, and the targets it names are where the fighting gathers.",
      },
      {
        heading: "Strategic directives",
        text: "Shorter side orders appear between Major Orders, often to defend or take a single planet.",
      },
      {
        heading: "Where to see it",
        text: "[[app:overview|High Command overview]] shows the running campaign and the Major Order; [[app:past|Past campaigns]] keeps every ended one with its orders, briefings, result and reward.",
      },
    ],
    quotes: ["Every citizen is equally important to the war effort, but Helldivers are the most important."],
    tips: ["On [[app:campaigns|Active planets]], the Major Order strip shows each target's progress against the deadline."],
    related: ["app:past", "app:overview", "manual:campaign-types"],
  },
  {
    id: "gloom",
    chapter: "story",
    title: "The Gloom",
    address: "se://hc/manuals/gloom",
    summary: "The spore cloud on the Terminid frontier, and the work to see inside it.",
    sections: [
      {
        heading: "What it does",
        text: "The Gloom is a cloud of Terminid spores spread across part of the galaxy. Planets deep inside it cannot be reached or monitored; planets on its edge can be fought on, and Gloom-enhanced Terminid strains appear there.",
      },
      {
        heading: "Reaching inside",
        text: "The [[ministry:science|Ministry of Science]] developed a non-permeable armour while investigating the Gloom. In July 2185 [[company:admiral-defense|AM Defense]] and the Xenoentomology Center on [[planet:Turing|Turing]] built a new shielding for Alcubierre drives that could open an era of Gloom exploration, with security and Terminid tissue supplied by the Helldivers.",
      },
      {
        heading: "On the map",
        text: "Planets High Command marks as in the Gloom or on its border sit under an amber cloud in every overlay of the [[app:map|galaxy map]].",
      },
    ],
    quotes: ["Watch out for alien storms. Super Earth scientists have officially classified many of them as dangerous. And freaky."],
    tips: ["Sub-faction pawns beside a planet show which Gloom strains have been reported on it."],
    related: ["ministry:science", "company:admiral-defense", "app:map"],
    sources: [WIKI.ministryOfScience, WIKI.admiralDefense],
  },
  {
    id: "misconceptions",
    chapter: "appendix",
    title: "Common misconceptions",
    address: "se://hc/manuals/misconceptions",
    summary: "What Helldivers often get wrong about the war, answered.",
    sections: [
      {
        heading: "Questions from the barracks",
        faq: [
          {
            q: "Does a planet's enemy resistance make its missions harder?",
            a: "No. Resistance is the planet's decay in the Galactic War and has nothing to do with mission difficulty. See [[manual:resistance|Enemy resistance and sieges]].",
          },
          {
            q: "Do defense campaigns decay?",
            a: "No. Any decay figure shown for a defense is cosmetic.",
          },
          {
            q: "Does cutting a planet's last supply line win it?",
            a: "Not at once. A planet surrounded on every side gets a negative decay and slowly liberates itself, usually about 1% an hour.",
          },
          {
            q: "Do lots of Trivial missions help?",
            a: "No. Higher difficulties carry more influence, and a flood of easy missions drags the impact modifier down. See [[manual:impact|Impact and the impact modifier]].",
          },
          {
            q: "Does rushing the main objectives speed liberation up?",
            a: "No. The modifier counts missions over time, so if everyone rushed, it would fall to cancel the gain.",
          },
          {
            q: "A planet has a big share of Helldivers and is still losing. Why?",
            a: "Its resistance may be too high; the Helldivers may still be ramping up, so give it a few hours; most of them may be fighting in a city, where the progress lands first; or a low resistance percentage may hide a very large health pool.",
          },
          {
            q: "Why not go for the gambit instead of the defense?",
            a: "Gambits often aren't viable: the attacker's health or resistance can be too high to finish in time, and rallying enough Helldivers behind one is hard. The defense is usually the better bet. See [[manual:defense|Defense campaigns and gambits]].",
          },
          {
            q: "What if we got everyone to…",
            a: "No plan reaches every Helldiver. Plans that need a reasonable majority, or a few Helldivers giving a lot, as with [[manual:dss|DSS]] donations, work better.",
          },
          {
            q: "Isn't a loss the fault of Helldivers who only fight one front?",
            a: "High Command already accounts for Helldivers who only fight one faction. Blaming them changes nothing.",
          },
        ],
      },
    ],
    quotes: ["Pay attention to these tips. They've been carefully calibrated to ensure your success."],
    tips: ["When a front looks wrong, compare its rate with its enemy regeneration on the planet's page before calling it lost."],
    related: ["manual:resistance", "manual:impact", "manual:defense"],
    sources: [WIKI.mechanics],
  },
];

/** Training Manual Tips the game shows while diving, quoted for the G.W.W.'s front page. */
export const TIPS: string[] = [
  "Managed Democracy is a cornerstone of advanced civilization.",
  "Super Earth has many holidays, but the most important is October 26th-Liberty Day!",
  "If an enemy ever attempts to engage in diplomacy, SHOOT THEM. We mustn't believe their lies.",
  "Super Earth recommends spending at least 2.4 seconds per mission enjoying the scenery. A happy Helldiver is a deadly Helldiver.",
  "Don't worry, if you fail an objective, you will NOT be sent to a freedom camp. This is merely a dissident rumor.",
  "If you notice a squadmate sympathizing with an enemy, report them to your Democracy Officer. Thoughtcrimes kill!",
  "Helldivers traveling the galaxy might come across ancient ruins and other curiosities. Just remember: Only a traitor is curious about alien artifacts!",
  "The Galactic War is a united effort. Only by working together can we ensure planets don't fall to the enemy.",
  "More challenging missions yield greater rewards!",
  "Want to show off how firmly you support capitalism? Spend medals on a Warbond!",
  "Friendly Fire isn't.",
  "When you're up against the bots, remember the 3 Cs: Cover, Courage and more Cover.",
  "Don't panic.",
  "Dive responsibly.",
  "Pay attention to these tips. They've been carefully calibrated to ensure your success.",
  "Did you know tips are shown during these transition sequences?",
];
