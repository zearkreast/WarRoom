/**
 * Galactic Wide Web content: training manuals on how the war works, Super Earth's ministries, the
 * Federation's corporations and the adverts between them. The manuals are written for this app in its
 * own words from the Helldivers Wiki (see each manual's `sources`); quotes marked as the game's are its
 * Training Manual Tips and slogans, word for word.
 */

export interface Source {
  title: string;
  url: string;
}

/** The licence of the Helldivers Wiki's text, which the manuals adapted from it share. */
export const WIKI_LICENCE: Source = {
  title: "CC BY-NC-SA 4.0",
  url: "https://creativecommons.org/licenses/by-nc-sa/4.0/",
};

export const WIKI = {
  home: { title: "Helldivers Wiki", url: "https://helldivers.wiki.gg/" },
  mechanics: { title: "Second Galactic War Mechanics", url: "https://helldivers.wiki.gg/wiki/Second_Galactic_War_Mechanics" },
  tips: { title: "Training Manual Tips", url: "https://helldivers.wiki.gg/wiki/Training_Manual_Tips" },
  corporations: { title: "Corporations", url: "https://helldivers.wiki.gg/wiki/Corporations" },
  ministries: { title: "Government of Super Earth", url: "https://helldivers.wiki.gg/wiki/Government_of_Super_Earth" },
} satisfies Record<string, Source>;

export interface Section {
  heading: string;
  text?: string;
  points?: string[];
  /** `icon` names a picture from the G.W.W. art, drawn at the start of its row. */
  table?: { head: string[]; rows: { icon?: string; cells: string[] }[]; note?: string };
  /** A diagram drawn by the app. */
  figure?: "supply-lines" | "siege" | "decay" | "impact-live";
  /** Named kinds of something, each with its picture. */
  kinds?: { icon: string; name: string; text: string }[];
  faq?: { q: string; a: string }[];
}

export interface Manual {
  id: string;
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
    title: "How the Galactic War works",
    address: "se://hc/manuals/basics",
    summary: "The war on one page: impact, planet health, supply lines and campaigns.",
    icon: "super-earth",
    sections: [
      {
        heading: "The short version",
        text: "Every mission won deals impact to the planet it was fought on. Each planet in a campaign has a health pool; empty it and the campaign ends in Super Earth's favour. Campaigns open only along supply lines, and the enemy wins back part of every planet it holds each hour. The war never resets and is steered by a human Game Master, with Major Orders telling its story.",
      },
      {
        heading: "Field glossary",
        points: [
          "Impact: what a won mission deals to a planet. It grows with difficulty and is scaled by the impact modifier.",
          "Impact modifier: a galaxy-wide multiplier that falls as more missions are completed, so the war moves at much the same pace however many Helldivers are online.",
          "Planet health: the pool a campaign has to empty. The liberation percentage the game shows is drawn from it.",
          "Supply lines: the warp links between planets. A campaign needs a link to a Super Earth planet to open and to keep running.",
          "Enemy resistance: the share of a planet's health the enemy wins back every hour, also called decay.",
          "Defense: a campaign with a deadline and no resistance, opened when the enemy attacks a Super Earth planet.",
          "Gambit: winning a defense by liberating the planet the attack came from.",
          "High-priority campaign: a story campaign with its own missions and rules.",
          "Regions: settlements, cities and megafactories with their own health and resistance; freeing one hits the whole planet hard.",
        ],
      },
    ],
    quotes: ["Managed Democracy is a cornerstone of advanced civilization."],
    tips: ["Every manual here goes deeper into one of these words; the glossary is the map."],
    sources: [WIKI.mechanics],
  },
  {
    id: "liberation",
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
        text: "A planet without regions starts with a million health, with a few exceptions. Regions add their own health on top; see Regions and cities.",
        table: {
          head: ["Planet", "Health"],
          rows: [
            { cells: ["Most planets", n(1_000_000)] },
            { cells: ["Magma, deciduous and autumn forest, desert oasis", `Usually ${n(2_000_000)}`] },
            { cells: ["Zagon Prime", n(2_000_000)] },
            { cells: ["Omicron", n(3_000_000)] },
            { cells: ["Cyberstan", `${n(4_250_000)}; ${n(15_000_000)} with its megafactories`] },
          ],
        },
      },
      {
        heading: "When the line is cut",
        text: "If Super Earth loses every supply line to the planet, no new operations can start there, and the progress already made keeps decaying until a line is opened again.",
      },
      {
        heading: "Operation modifiers",
        text: "Liberation operations often come with one or two operation modifiers, depending on difficulty. Defense missions never have them, though weather and other planet conditions still apply.",
      },
    ],
    quotes: ["The Galactic War is a united effort. Only by working together can we ensure planets don't fall to the enemy."],
    tips: [
      "The rate on each front in this app is per hour with resistance already taken off. Above zero, the planet is on its way.",
    ],
    sources: [WIKI.mechanics],
  },
  {
    id: "resistance",
    title: "Enemy resistance and sieges",
    address: "se://hc/manuals/resistance",
    summary: "Why some fronts stall for days, and how a surrounded planet falls on its own.",
    sections: [
      {
        heading: "Resistance is decay",
        text: "Every hour the enemy wins back a fixed share of a planet's health. The game calls it Enemy Resistance; Helldivers call it decay. A front only moves while missions deal more per hour than it wins back. The share is of the planet's own health pool, so the same percentage can mean very different amounts on two planets.",
        figure: "decay",
      },
      {
        heading: "Turnout counts twice",
        text: "Helldivers who only match the decay make no progress. On a front that is barely winning, every extra Helldiver also saves the time of everyone already there, so a few more can finish it much sooner.",
      },
      {
        heading: "It isn't difficulty",
        text: "Resistance has nothing to do with how hard missions on the planet are or how many enemies they spawn. Arrowhead's support confirmed it.",
      },
      {
        heading: "Sieges",
        text: "An enemy planet completely surrounded by Super Earth planets gets a negative decay and starts liberating itself, usually about 1% an hour. Only a planet entirely on its own counts: cutting off a group of two or more planets does not start a siege. Sieges have been automatic since around 16 June 2025; before that Arrowhead set them by hand.",
        figure: "siege",
      },
    ],
    tips: ["A front whose rate sits near zero for hours is one where turnout and resistance cancel out."],
    sources: [WIKI.mechanics],
  },
  {
    id: "defense",
    title: "Defense campaigns and gambits",
    address: "se://hc/manuals/defense",
    summary: "Deadlines, how strong an attack is, and winning a defense by taking the planet behind it.",
    icon: "campaign-defense",
    sections: [
      {
        heading: "Holding a planet",
        text: "When the enemy attacks a Super Earth planet along a supply line, the planet gets a defense campaign. It has its own health, set by the attack's strength and usually far below a liberation's million: the defense level shown when hovering the planet, times 50,000. There is no resistance while it runs.",
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
        text: "Regions weigh more in a defense, since missions there damage both the region and the defense. More regions come under threat as the deadline nears.",
      },
      {
        heading: "Gambits",
        text: "Every attack comes from an enemy planet. Liberate that planet before the deadline and the defense ends in Super Earth's favour: a gambit. It only works when the attacker's health and resistance are low enough to finish in time, and the game never points the attacker out, which makes gambits hard to rally behind. The Democracy Space Station's Orbital Blockade doesn't win one for you; it only stops new attacks from its planet.",
        figure: "supply-lines",
      },
    ],
    quotes: [
      "See a planet that's on the brink of falling under enemy control? Rally your fellow Helldivers and mount a defense. Every mission counts!",
    ],
    tips: ["On the galaxy map a dashed attack line runs from the attacker to its target: the attacker is the gambit."],
    sources: [WIKI.mechanics],
  },
  {
    id: "campaign-types",
    title: "Campaign types",
    address: "se://hc/manuals/campaign-types",
    summary: "Liberation, defense, and the rarer kinds the game brings out for its story.",
    icon: "campaign-high-priority",
    sections: [
      {
        heading: "Every planet you can fight on has a campaign",
        text: "A planet is only playable while a campaign is active on it, and each kind plays by its own rules.",
        kinds: [
          {
            icon: "campaign-liberation",
            name: "Liberation",
            text: "No time limit. Empty the planet's health faster than its resistance refills it.",
          },
          {
            icon: "campaign-defense",
            name: "Defense",
            text: "A deadline and no resistance. Empty the defense's health in time or lose the planet.",
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
            text: "Story campaigns with missions and rules of their own, cleared by following the Major Order. They usually open with a very high resistance that drops as milestones are reached, provided most Helldivers take part.",
          },
        ],
      },
    ],
    quotes: ["Each faction presents unique challenges. Mix up your gear and try new tactics!"],
    tips: ["Planet cards on Active planets say whether a front is a liberation or a defense."],
    sources: [WIKI.mechanics],
  },
  {
    id: "regions",
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
        text: "Some regions only open as the planet's liberation advances, or, during a defense, as time passes. Regions can carry modifiers of their own, such as the Illuminate's Leviathan Blockade from Impossible difficulty up.",
      },
      {
        heading: "Maximum Security Cities",
        text: "New Hope City, New Aspiration City and New Yearning City are the Federation's three Maximum Security Cities. They are marked on the map as points of interest.",
      },
    ],
    quotes: ["Missions are filled with unexplored points of interest. Check them out—you never know what you might find."],
    tips: ["A planet's regions are listed under its operational parameters."],
    sources: [WIKI.mechanics],
  },
  {
    id: "impact",
    title: "Impact and the impact modifier",
    address: "se://hc/manuals/impact",
    summary: "How a mission turns into liberation, and why one counts for more on a quiet night.",
    icon: "difficulty-helldive",
    sections: [
      {
        heading: "Mission influence",
        text: "A won mission produces influence, set by its difficulty. Finishing a whole operation multiplies the last mission's influence by four. Only the main objective counts, and only missions started by their host; kicking players halfway doesn't create extra damage.",
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
        text: "Influence is multiplied by the galactic impact modifier, and the result, impact, comes off the planet's or the defense's health. The exact formula isn't public. The modifier falls as more missions are completed across the galaxy, so what moves a planet is the share of active Helldivers fighting on it, not their raw number.",
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
    tips: ["The High Command overview charts the modifier under the Helldivers on each front over the last day."],
    sources: [WIKI.mechanics],
  },
  {
    id: "supply-lines",
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
        text: "When two linked planets belong to different sides, one can attack the other along the link. An enemy attack opens a defense; a Super Earth attack opens a liberation.",
      },
      {
        heading: "Cut off",
        text: "Lose every Super Earth planet linked to a campaign and it stops: no new operations, while decay eats the progress made. Surround an enemy planet completely and it falls under siege instead; see Enemy resistance and sieges.",
      },
    ],
    quotes: ["If the enemies just keep coming, find out where they're coming from and unleash hell. Big hive? Destroy! Bot factory? Destroy!"],
    tips: ["The map's Galaxy overlay draws every supply line; the Sectors overlay colours the lines inside enemy territory."],
    sources: [WIKI.mechanics],
  },
  {
    id: "dss",
    title: "The Democracy Space Station",
    address: "se://hc/manuals/dss",
    summary: "The Helldiver-funded station: its votes, its boost and its tactical actions.",
    sections: [
      {
        heading: "Votes",
        text: "Unless the story takes it out of service, Helldivers vote every four hours on the station's next planet, from the planets with the most Helldivers when the vote opened. Liberation runs 10% faster on the planet it orbits.",
      },
      {
        heading: "Tactical actions",
        text: "Helldivers fund the station's tactical actions by donating resources such as samples. Only one action can run at a time, and donations made while one runs are lost. The game's estimate of when an action will be ready looks at a very short window, so it swings a lot.",
        kinds: [
          {
            icon: "heavy_ordnance",
            name: "Heavy Ordnance Distribution",
            text: "Counts the liberation on its planet more than once, likely twice. That lowers the galaxy-wide modifier, so in effect it moves impact from other planets to this one. It works on every kind of campaign, defenses included. Planetary Bombardment had the same effect and is now left to the Game Master.",
          },
          {
            icon: "eagle_storm",
            name: "Eagle Storm",
            text: "Pauses the enemy's progress, which can stretch a defense by a day if the station stays for the whole action. The enemy's progress itself doesn't move back, so regions that open with it stay as they were.",
          },
          {
            icon: "orbital_blockade",
            name: "Orbital Blockade",
            text: "Stops new defenses from being launched from its planet. Defenses already running are untouched; missions there get Hellpod Space Optimization.",
          },
        ],
      },
    ],
    tips: ["The Hellpad's space station page shows which actions are running now and when the others are ready again."],
    sources: [WIKI.mechanics],
  },
  {
    id: "campaigns",
    title: "Campaigns and orders",
    address: "se://hc/manuals/campaigns",
    summary: "Galactic Campaigns, Major Orders and strategic directives, and how rewards are earned.",
    sections: [
      {
        heading: "Galactic Campaigns",
        text: "A Galactic Campaign is a named story told through a run of Major Orders, usually three over about two weeks. Winning most of its Major Orders earns the campaign's own reward, such as a stratagem, weapon or armour.",
      },
      {
        heading: "Major Orders",
        text: "Each Major Order sets objectives for the whole community with a deadline and pays medals when it is won. A won or lost order also moves the campaign's story on.",
      },
      {
        heading: "Strategic directives",
        text: "Shorter side orders appear between Major Orders, often to defend or take a single planet.",
      },
    ],
    quotes: ["Every citizen is equally important to the war effort, but Helldivers are the most important."],
    tips: ["Past campaigns, with their orders and outcomes, are kept in the Hellpad."],
  },
  {
    id: "gloom",
    title: "The Gloom",
    address: "se://hc/manuals/gloom",
    summary: "The spore cloud on the Terminid frontier.",
    sections: [
      {
        heading: "What it does",
        text: "The Gloom is a cloud of Terminid spores spread across part of the galaxy. Planets deep inside it cannot be reached or monitored; planets on its edge can be fought on, and Gloom-enhanced Terminid strains appear there.",
      },
      {
        heading: "On the map",
        text: "Planets the game marks as in the Gloom or on its border sit under an amber cloud in every overlay.",
      },
    ],
    quotes: ["Watch out for alien storms. Super Earth scientists have officially classified many of them as dangerous. And freaky."],
    tips: ["Sub-faction pawns beside a planet show which Gloom strains have been reported on it."],
  },
  {
    id: "misconceptions",
    title: "Common misconceptions",
    address: "se://hc/manuals/misconceptions",
    summary: "What Helldivers often get wrong about the war, answered.",
    sections: [
      {
        heading: "Questions from the barracks",
        faq: [
          {
            q: "Does a planet's enemy resistance make its missions harder?",
            a: "No. Resistance is the planet's decay in the Galactic War and has nothing to do with mission difficulty.",
          },
          {
            q: "Do defense campaigns decay?",
            a: "No. A decay figure a companion app shows for a defense is cosmetic.",
          },
          {
            q: "Does cutting a planet's last supply line win it?",
            a: "Not at once. A planet surrounded on every side gets a negative decay and slowly liberates itself, usually about 1% an hour.",
          },
          {
            q: "Do lots of Trivial missions help?",
            a: "No. Higher difficulties carry more influence, and a flood of easy missions drags the impact modifier down.",
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
            a: "Gambits often aren't viable: the attacker's health or resistance can be too high to finish in time, and rallying enough Helldivers behind one is hard. The defense is usually the better bet.",
          },
          {
            q: "What if we got everyone to…",
            a: "No plan reaches the whole playerbase. Plans that need a reasonable majority, or a few Helldivers giving a lot, as with DSS donations, work better.",
          },
          {
            q: "Isn't a loss the fault of Helldivers who only fight one front?",
            a: "The Game Master already accounts for Helldivers who only fight one faction. Blaming them changes nothing.",
          },
        ],
      },
    ],
    quotes: ["Pay attention to these tips. They've been carefully calibrated to ensure your success."],
    tips: ["When a front looks wrong, compare its rate here with its resistance on the planet page before calling it lost."],
    sources: [WIKI.mechanics],
  },
];

export interface Ministry {
  id: string;
  name: string;
  address: string;
  icon: string;
  color: string;
  /** The game's Training Manual Tip about the ministry, quoted. */
  motto: string;
  text: string;
  ads?: string[];
  /** Corporations tied to the ministry, by name. */
  partners?: string[];
}

export const MINISTRIES: Ministry[] = [
  {
    id: "defense",
    name: "Ministry of Defense",
    address: "se://gov/mindef",
    icon: "ministry-defense",
    color: "#f84450",
    motto:
      "The Ministry of Defense is responsible for the defense of Liberty against all enemies, human and non-human, realized and potential.",
    text: "MINDEF defends Liberty and buys the arms to do it: Morgunson, AM Defense and Ståhl Arms all build to its contracts.",
    partners: ["Morgunson", "Admiral Defense", "Ståhl Arms"],
  },
  {
    id: "expansion",
    name: "Ministry of Expansion",
    address: "se://gov/expansion",
    icon: "ministry-expansion",
    color: "#29c4a9",
    motto:
      "The Ministry of Expansion tirelessly works to expand the reach of Freedom and secures vital resources for our society to thrive.",
    text: "Pushes the frontier outwards and brings new worlds, and what lies under them, into the Federation.",
    ads: ["expansion-work-songs"],
  },
  {
    id: "humanity",
    name: "Ministry of Humanity",
    address: "se://gov/humanity",
    icon: "ministry-humanity",
    color: "#9fc43a",
    motto:
      "The Ministry of Humanity maintains healthy, productive voters by managing education, healthcare, social grading and gene management.",
    text: "Looks after the citizen from school to clinic, and keeps the social grades in order.",
    ads: ["humanity-franchisement"],
  },
  {
    id: "prosperity",
    name: "Ministry of Prosperity",
    address: "se://gov/prosperity",
    icon: "ministry-prosperity",
    color: "#a185e0",
    motto:
      "The Ministry of Prosperity manages Super Earth's economy and ensure a steady flow of resources to the most deserving.",
    text: "Keeps the Federation's economy running and its goods flowing to where they are deserved.",
  },
  {
    id: "science",
    name: "Ministry of Science",
    address: "se://gov/science",
    icon: "ministry-science",
    color: "#ff9a2e",
    motto:
      "The Ministry of Science is responsible for continuous scientific progress, augmenting our moral superiority over our enemies with technological superiority.",
    text: "Research and development, fed in part by the samples Helldivers bring home from the front.",
  },
  {
    id: "truth",
    name: "Ministry of Truth",
    address: "se://gov/truth",
    icon: "ministry-truth",
    color: "#f074d8",
    motto:
      "The Ministry of Truth is responsible for keeping all citizens well-informed and mentally equipped to withstand our enemies' vile propaganda.",
    text: "Owns Strohmann Media, whose news network wears the ministry's own logo.",
    ads: ["truth-see-treason"],
    partners: ["Strohmann Media"],
  },
  {
    id: "unity",
    name: "Ministry of Unity",
    address: "se://gov/unity",
    icon: "ministry-unity",
    color: "#3fcbe0",
    motto:
      "The Ministry of Unity ensures that all of mankind is united as one by protecting us from internal threats, such as deviant behavior and expression of dangerous ideas.",
    text: "Watches the home front. Report deviant behaviour to your nearest Democracy Officer.",
  },
];

export interface Company {
  name: string;
  address: string;
  trade: string;
  text: string;
  /** Brand colour, for the site's banner. */
  color: string;
  logo?: string;
  /** The company's slogan as the game gives it. */
  slogan?: string;
  /** A Training Manual Tip about the company, quoted. */
  quote?: string;
  ads?: string[];
}

export const COMPANIES: Company[] = [
  {
    name: "Strohmann Media",
    address: "se://strohmann.news",
    trade: "News and broadcasting",
    text: "The Federation's news network, owned by the Ministry of Truth and sharing its logo. Its dispatches are this app's Strohmann News.",
    color: "#f074d8",
    logo: "strohmann-news",
    slogan: "Strohmann: truth, tailored to you.",
  },
  {
    name: "Morgunson",
    address: "se://morgunson.arsenal",
    trade: "Arms",
    text: "Also known as Morgunson Arsenal; the Ministry of Defense's main weapons supplier, behind the SG-225 Breaker, the EAT-17 Expendable Anti-Tank and the EXO-45 Patriot Exosuit.",
    color: "#e0323c",
    logo: "morgunson",
    slogan: "Morgunson: Securing the future of freedom.",
  },
  {
    name: "Permacura",
    address: "se://permacura.health",
    trade: "Medicine",
    text: "Maker of the stims and other healing products that keep Helldivers on their feet.",
    color: "#39a2ce",
    logo: "permacura",
    slogan: "Permacura: put your life in our hands.",
    quote:
      "Don't worry—stims have zero addictive properties!* *Study paid for by Permacura. Permacura: put your life in our hands.",
  },
  {
    name: "C.O.B. Genetics",
    address: "se://cob.genetics",
    trade: "Agriculture",
    text: "Engineers and grows enhanced crops to feed the citizens of Super Earth.",
    color: "#8fbf3a",
    logo: "cob-genetics",
    slogan: "C.O.B. Genetics: Nature 2.0.",
  },
  {
    name: "AmpuDyne",
    address: "se://ampudyne.med",
    trade: "Prosthetics",
    text: "Builds cybernetic limbs for wounded citizens, of whom the war supplies many.",
    color: "#c9ced6",
    slogan: "AmpuDyne: Let us give you a hand.",
  },
  {
    name: "Trustor",
    address: "se://trustor.bank",
    trade: "Banking",
    text: "The bank serving Super Earth and its worlds, with insurance for every threat the war brings to the door.",
    color: "#8a5cf0",
    logo: "trustor",
    slogan: "Trustor: we trust you to trust us.",
    ads: ["trustor-bug-insurance", "trustor-invasion-insurance"],
  },
  {
    name: "Admiral Defense",
    address: "se://am.defense",
    trade: "Vehicles and defense",
    text: "AM Defense had long built armaments for the SEAF before the Ministry of Defense awarded it a multitrillion-Supercredit contract to equip the Helldivers; the maker behind the Maelstrom tank production the Major Orders fought for in 2186.",
    color: "#b9a37a",
    logo: "admiral-defense",
  },
  {
    name: "Ståhl Arms",
    address: "se://stahl.arms",
    trade: "Arms",
    text: "An arms maker the Ministry of Defense signed when the fight against the Illuminate strained the defense budget; it arrived with a crossover promotion.",
    color: "#e5372e",
    logo: "stahl-arms",
  },
  {
    name: "Ulterior",
    address: "se://ulterior.corp",
    trade: "Conglomerate",
    text: "A Mega-Corporation based on Super Earth whose business is not made public.",
    color: "#7f93a0",
    slogan: "Ulterior: operating under the mantle.",
  },
];

export interface Ad {
  file: string;
  /** What the advert shows and says, for screen readers. */
  alt: string;
  shape: "wide" | "tall";
}

/** Adverts seen on the game's city billboards. */
export const ADS: Ad[] = [
  { file: "trustor-bug-insurance", alt: "Trustor Bank: put your foot down with bug insurance.", shape: "wide" },
  { file: "trustor-invasion-insurance", alt: "Trustor Bank: hold your ground with invasion insurance.", shape: "tall" },
  { file: "truth-see-treason", alt: "Ministry of Truth: see treason? Shout treason!", shape: "wide" },
  { file: "humanity-franchisement", alt: "Ministry of Humanity: franchisement is freedom.", shape: "tall" },
  { file: "expansion-work-songs", alt: "Ministry of Expansion: democratic work songs encouraged.", shape: "tall" },
  { file: "patriot-hut-helldawgs", alt: "Patriot Hut: flame-grilled Helldawgs.", shape: "wide" },
  { file: "libertea", alt: "Libertea herbal tea: a voter's paradise.", shape: "tall" },
  { file: "helldivers-take-a-life", alt: "Join the Helldivers: take a life, save a life.", shape: "wide" },
  { file: "cornstitution", alt: "Cornstitution cereal: it has corn.", shape: "wide" },
  { file: "freegum", alt: "Freegum: you can't get enough.", shape: "tall" },
  { file: "democoffee", alt: "Democoffee: it's coffee time.", shape: "tall" },
  { file: "voting-voyager", alt: "Voting Voyager: believe the unbelievable.", shape: "wide" },
  { file: "super-clean", alt: "Super Clean nuclear detergent.", shape: "tall" },
  { file: "truth-cram-school", alt: "TRuth Cram School: dare to dream.", shape: "tall" },
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
