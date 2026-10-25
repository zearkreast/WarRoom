/**
 * Site text is adapted from the Helldivers Wiki in this app's words; slogans, poster lines and broadcast
 * lines are the game's, word for word. Text links are written `[[kind:target|label]]` (domain/gwwText.ts).
 */
import { WIKI, type Section, type Source } from "./gww";

const wiki = (title: string, page = title.replace(/ /g, "_")): Source => ({
  title,
  url: `https://helldivers.wiki.gg/wiki/${encodeURI(page)}`,
});

export interface Dated {
  when: string;
  text: string;
}

export interface SiteSection extends Section {
  timeline?: Dated[];
  /** Named as the game names them. */
  products?: string[];
}

/** A link to another site, with what ties the two. */
export interface Tie {
  to: string;
  how: string;
}

export interface Ministry {
  id: string;
  name: string;
  address: string;
  icon: string;
  color: string;
  /** One line for its tile. */
  summary: string;
  /** The game's Training Manual Tip about the ministry, quoted. */
  motto: string;
  /** A line from its posters, quoted. */
  slogan?: string;
  sections: SiteSection[];
  ties?: Tie[];
  related?: string[];
  ads?: string[];
  source: Source;
}

export const MINISTRIES: Ministry[] = [
  {
    id: "defense",
    name: "Ministry of Defense",
    address: "se://gov/mindef",
    icon: "ministry-defense",
    color: "#f84450",
    summary: "Commands the armed forces and the Helldivers, and buys the arms they carry.",
    motto:
      "The Ministry of Defense is responsible for the defense of Liberty against all enemies, human and non-human, realized and potential.",
    slogan: "We defend Democracy's Future.",
    sections: [
      {
        heading: "Command",
        text: "MINDEF commands the Super Earth Armed Forces, and through them the Helldivers; it also designs and deploys most Warbonds, now and then with another ministry's help. The Minister of Defense is first in the presidential line of succession: when the President was killed during the Illuminate assault on [[planet:Super Earth|Super Earth]] on 29 May 2185, presidential authority passed to the Minister until a new President was elected.",
      },
      {
        heading: "Recruitment",
        text: "Its Directorate of Helldiver Readiness, on Mars, counts every intake. One daily report logged 48,736 trainees with an average age of 18.7, a patriotism rating of 97.4% and an expected survival rate of 21.3%: within quota.",
      },
      {
        heading: "Orders",
        text: "Helldivers take their orders from MINDEF; the Democracy Officer aboard each Super Destroyer takes theirs from the [[ministry:truth|Ministry of Truth]]. The two ministries write the Hellpod deployment tips together. MINDEF's own orders reach the fleet as [[manual:campaigns|Major Orders]].",
      },
    ],
    ties: [
      { to: "company:morgunson", how: "Its main weapons supplier" },
      { to: "company:admiral-defense", how: "Tier-1 contractor for vehicles" },
      { to: "company:stahl-arms", how: "Signed when the Illuminate returned" },
    ],
    related: ["manual:defense", "manual:campaigns", "site:gov"],
    ads: ["helldivers-take-a-life"],
    source: wiki("Ministry of Defense"),
  },
  {
    id: "truth",
    name: "Ministry of Truth",
    address: "se://gov/truth",
    icon: "ministry-truth",
    color: "#f074d8",
    summary: "The Federation's only source of news, and the eyes that watch for dissent.",
    motto:
      "The Ministry of Truth is responsible for keeping all citizens well-informed and mentally equipped to withstand our enemies' vile propaganda.",
    slogan: "Truth: the backbone of Democracy.",
    sections: [
      {
        heading: "The only source",
        text: "The Ministry of Truth is the sole provider of information to the public: every poster about the other ministries is its work. It supervises Strohmann News, anchored by Coretta Kelly, through [[company:strohmann-media|Strohmann Media]]; the broadcasts reach this tracker as [[app:news|Strohmann News]].",
      },
      {
        heading: "Watching",
        text: "It runs the Center for Civilian Surveillance and Safety on [[planet:Mastia|Mastia]], which finds dissidents for the [[ministry:unity|Ministry of Unity]]'s Unity Corps and looks for signs of Illuminate influence among citizens. Truth Enforcers protect citizens from treasonous lies; the Informant of the Month receives the Pride of the Whistleblower cape.",
      },
      {
        heading: "In the field",
        text: "Its field offices stand in the colony cities Helldivers fight through, beside the [[ministry:expansion|Ministry of Expansion]]'s. In October 2186 it ordered enough combat footage from the front for its newest film, Democratic Sniper.",
      },
    ],
    ties: [{ to: "company:strohmann-media", how: "Believed to own it; they share a logo" }],
    related: ["app:news", "ministry:unity", "site:gov"],
    ads: ["truth-see-treason"],
    source: wiki("Ministry of Truth"),
  },
  {
    id: "unity",
    name: "Ministry of Unity",
    address: "se://gov/unity",
    icon: "ministry-unity",
    color: "#3fcbe0",
    summary: "Protects mankind from internal threats: deviant behaviour and dangerous ideas.",
    motto:
      "The Ministry of Unity ensures that all of mankind is united as one by protecting us from internal threats, such as deviant behavior and expression of dangerous ideas.",
    slogan: "It only takes ONE TRAITOR to destroy Utopia.",
    sections: [
      {
        heading: "The Unity Corps",
        text: "Created by the FREEDOM Act, the Unity Corps arrest the loyalty-challenged citizens the [[ministry:truth|Ministry of Truth]]'s surveillance finds. A week before the War on Dissidence began, the ministry's Loyalty Intervention & Enforcement Squads went after dissidents hiding on [[planet:Gaellivare|Gaellivare]], [[planet:Alta V|Alta V]] and [[planet:Diluvia|Diluvia]].",
      },
      {
        heading: "Programmes",
        text: "Its Federal Pre-Dissidence Program rewards extra-vigilant citizens with citizenship score boosts for each report of subversive activity. Its Board of Psychiatrists approved a guide to the five stages of grief after the destruction of [[planet:Moradesh|Moradesh]], and its Greener Galaxy campaign supports Superstone National Park.",
      },
      {
        heading: "Where it sits",
        text: "The ministry meets in the Chamber of Unanimous Decision, reached through its Golden Gate.",
      },
    ],
    ties: [{ to: "company:permacura", how: "Logos all but identical; ties unknown" }],
    related: ["ministry:truth", "site:gov"],
    source: wiki("Ministry of Unity"),
  },
  {
    id: "expansion",
    name: "Ministry of Expansion",
    address: "se://gov/expansion",
    icon: "ministry-expansion",
    color: "#29c4a9",
    summary: "Pushes the frontier out and secures the resources the Federation runs on.",
    motto:
      "The Ministry of Expansion tirelessly works to expand the reach of Freedom and secures vital resources for our society to thrive.",
    slogan: "Billions of stars. One Federation.",
    sections: [
      {
        heading: "The frontier",
        text: "The Colonial Overseers, part of the ministry, run Super Earth's colonies by the Colonial Overseer's Handbook: planetary terraforming, resource extraction and colonist management. When colonists go missing, it conscripts more volunteers.",
      },
      {
        heading: "Still early",
        text: "The ministry holds that galactic colonisation is still in its early stages, however many planets have been settled since the first Galactic War. It keeps field offices in the colony cities, as the [[ministry:truth|Ministry of Truth]] does. Every planet it adds is one more node on the [[manual:supply-lines|supply lines]].",
      },
    ],
    ties: [{ to: "company:ulterior", how: "Believed to own it; they share a logo" }],
    related: ["manual:supply-lines", "manual:liberation"],
    ads: ["expansion-work-songs"],
    source: wiki("Ministry of Expansion"),
  },
  {
    id: "science",
    name: "Ministry of Science",
    address: "se://gov/science",
    icon: "ministry-science",
    color: "#ff9a2e",
    summary: "Research and development, and the reports on what the war does to the galaxy.",
    motto:
      "The Ministry of Science is responsible for continuous scientific progress, augmenting our moral superiority over our enemies with technological superiority.",
    sections: [
      {
        heading: "Research",
        text: "It runs the Centers of Science found as points of interest on some planets of the [[app:map|galaxy map]], and reminds citizens of the citizenship benefits of enrolling their families in medical studies.",
      },
      {
        heading: "The Meridia report",
        text: "Its inquiry into the Termicide found that the toxin worked as intended, killing 99.6% of the Terminids on the Barrier Planets, but that survivors on [[planet:Meridia|Meridia]] began breeding up to twenty times faster: the Supercolony. The scientists behind the Termicide were taken into custody.",
      },
      {
        heading: "The Gloom",
        text: "It developed a non-permeable armour while investigating [[manual:gloom|the Gloom]].",
      },
    ],
    related: ["manual:gloom", "app:map"],
    source: wiki("Ministry of Science"),
  },
  {
    id: "humanity",
    name: "Ministry of Humanity",
    address: "se://gov/humanity",
    icon: "ministry-humanity",
    color: "#9fc43a",
    summary: "Keeps voters healthy and productive: education, healthcare, social grading and genes.",
    motto:
      "The Ministry of Humanity maintains healthy, productive voters by managing education, healthcare, social grading and gene management.",
    sections: [
      {
        heading: "Cradle to grave",
        text: "The ministry approves marriages, with instructions for consolidating domiciles; closes missing-person reports (one, after review: “Mine mishap.”); and sends grieving families the forms for proper patriot disposal.",
      },
      {
        heading: "Social grading",
        text: "Grading is the Citizenship Classification Structure, which sorts citizens from Super-Class down to Class F; see [[site:gov|Managed Democracy]].",
      },
    ],
    related: ["site:gov", "company:permacura", "company:ampudyne"],
    ads: ["humanity-franchisement"],
    source: wiki("Ministry of Humanity"),
  },
  {
    id: "prosperity",
    name: "Ministry of Prosperity",
    address: "se://gov/prosperity",
    icon: "ministry-prosperity",
    color: "#a185e0",
    summary: "Runs the economy and keeps resources flowing to the most deserving.",
    motto:
      "The Ministry of Prosperity manages Super Earth's economy and ensure a steady flow of resources to the most deserving.",
    sections: [
      {
        heading: "The economy",
        text: "It manages Super Earth's economy and sees that every colony gets the latest beverages. Super Credits are issued in its name, down to the refunds Arrowhead's support hands out, and it closed the Midasburg Mint after 75 years.",
      },
      {
        heading: "A shared logo",
        text: "Its logo is the same as [[company:trustor|Trustor]]'s; how far the two are tied isn't known.",
      },
    ],
    ties: [{ to: "company:trustor", how: "Same logo; ties unknown" }],
    related: ["company:trustor", "dir:corps"],
    source: wiki("Ministry of Prosperity"),
  },
];

export interface Company {
  id: string;
  name: string;
  address: string;
  trade: string;
  /** One line for its tile. */
  summary: string;
  color: string;
  logo?: string;
  /** The company's slogan as the game gives it. */
  slogan?: string;
  /** A line from its Super Destroyer broadcast, quoted. */
  broadcast?: string;
  /** A Training Manual Tip about the company, quoted. */
  quote?: string;
  showcase?: { image: string; alt: string };
  sections: SiteSection[];
  ties?: Tie[];
  related?: string[];
  ads?: string[];
  source: Source;
}

export const COMPANIES: Company[] = [
  {
    id: "strohmann-media",
    name: "Strohmann Media",
    address: "se://strohmann.news",
    trade: "News and broadcasting",
    summary: "The Federation's news network, with the Ministry of Truth behind it.",
    color: "#f074d8",
    logo: "strohmann-news",
    slogan: "Strohmann: truth, tailored to you.",
    sections: [
      {
        heading: "Strohmann News",
        text: "Strohmann Media runs Strohmann News, the broadcast on every Super Destroyer, anchored by Coretta Kelly and approved by the [[ministry:truth|Ministry of Truth]], which is believed to own the company and shares its logo. This tracker carries its dispatches as [[app:news|Strohmann News]].",
      },
    ],
    ties: [{ to: "ministry:truth", how: "Believed to own it; they share a logo" }],
    related: ["app:news", "ministry:truth"],
    source: wiki("Strohmann Media"),
  },
  {
    id: "morgunson",
    name: "Morgunson",
    address: "se://morgunson.arsenal",
    trade: "Arms",
    summary: "The Ministry of Defense's main weapons supplier.",
    color: "#e0323c",
    logo: "morgunson",
    slogan: "Morgunson: Securing the future of freedom.",
    broadcast: "Guns. They keep us safe. They put meat on our plates. They're fun.",
    sections: [
      {
        heading: "The arsenal",
        text: "Also known as Morgunson Arsenal, the [[ministry:defense|Ministry of Defense]]'s main weapons supplier.",
        products: ["SG-225 Breaker", "EAT-17 Expendable Anti-Tank", "EXO-45 Patriot Exosuit"],
      },
      {
        heading: "The Tien Kwan exosuits",
        timeline: [
          { when: "7 Feb 2184", text: "The new exosuits are slated to join the Helldiver Division." },
          {
            when: "7 Mar 2184",
            text: "The Automatons capture [[planet:Tien Kwan|Tien Kwan]], home of the only arsenal of the new technology, before a single exosuit is deployed.",
          },
          {
            when: "8 Mar 2184",
            text: "The Helldivers retake Tien Kwan in a day. The stockpile is recovered intact and the EXO-45 Patriot becomes available for requisition.",
          },
        ],
      },
    ],
    ties: [{ to: "ministry:defense", how: "Main weapons supplier" }],
    related: ["company:admiral-defense", "company:stahl-arms"],
    source: wiki("Morgunson"),
  },
  {
    id: "permacura",
    name: "Permacura",
    address: "se://permacura.health",
    trade: "Medicine",
    summary: "The stims and treatments that keep Helldivers on their feet.",
    color: "#39a2ce",
    logo: "permacura",
    slogan: "Permacura: put your life in our hands.",
    broadcast: "Injured on the job but still alive? See if you can afford Permacura's latest life-saving treatments.",
    quote:
      "Don't worry—stims have zero addictive properties!* *Study paid for by Permacura. Permacura: put your life in our hands.",
    sections: [
      {
        heading: "Stims",
        text: "The de facto maker of stims and other healing products, for injuries major or minor. Its Galaxy Plan promises the most universal healthcare there is, to those who qualify.",
        products: ["Stims"],
      },
      {
        heading: "The fine print",
        text: "Its broadcasts claim nine out of nine doctors endorse combat stims as a source of Vitamin D, for Democracy, then concede there is no evidence they provide any.",
      },
    ],
    ties: [{ to: "ministry:unity", how: "Logos all but identical; ties unknown" }],
    related: ["ministry:humanity", "company:ampudyne"],
    source: wiki("Permacura"),
  },
  {
    id: "cob-genetics",
    name: "C.O.B. Genetics",
    address: "se://cob.genetics",
    trade: "Agriculture",
    summary: "Genetically enhanced food for Super Earth, and lately chemistry for the DSS.",
    color: "#8fbf3a",
    logo: "cob-genetics",
    slogan: "C.O.B. Genetics: Nature 2.0.",
    broadcast: "Genes. You have them. I have them. Food has them. And C.O.B. Genetics makes genes better.",
    sections: [
      {
        heading: "Nature 2.0",
        text: "Engineers and grows enhanced crops and food for Super Earth's citizens, and insists it works well within standardized safety protocols.",
      },
      {
        heading: "The Eagle Gas Storm",
        text: "On 6 October 2186 it worked with engineers to refit the [[manual:dss|Democracy Space Station]]'s Eagle Storm to spread aerosol deterrents. The Eagle Gas Storm became the reward of the Epic Purge campaign, kept in [[app:past|Past campaigns]].",
      },
    ],
    related: ["manual:dss", "app:past"],
    source: wiki("C.O.B. Genetics"),
  },
  {
    id: "ampudyne",
    name: "AmpuDyne",
    address: "se://ampudyne.med",
    trade: "Prosthetics",
    summary: "Replacement limbs for the citizens the war leaves short of one.",
    color: "#c9ced6",
    slogan: "AmpuDyne: Let us give you a hand.",
    broadcast: "Sometimes life deals you a bad hand. But AmpuDyne can deal you a better one.",
    sections: [
      {
        heading: "A helping hand",
        text: "Builds cybernetic prosthetics for wounded and dismembered citizens. Its broadcasts suggest signing your life over to its indentured labour force, where you'll get every replacement limb the job requires.",
      },
    ],
    related: ["company:permacura", "ministry:humanity"],
    source: wiki("AmpuDyne"),
  },
  {
    id: "trustor",
    name: "Trustor",
    address: "se://trustor.bank",
    trade: "Banking",
    summary: "The bank of Super Earth, with insurance for every threat at the door.",
    color: "#8a5cf0",
    logo: "trustor",
    slogan: "Trustor: we trust you to trust us.",
    broadcast: "With Trustor, Super Earth will decide what you need, when you need it.",
    showcase: { image: "eagle-sweat", alt: "A bottle of Eagle Sweat, Eau de Liberté." },
    sections: [
      {
        heading: "Banking",
        text: "The bank serving Super Earth and its planets, for citizens who know a good citizen spends their earnings to keep the economy running.",
      },
      {
        heading: "Insurance",
        text: "Its billboards in the colony cities sell bug insurance and invasion insurance.",
      },
      {
        heading: "Eagle Sweat",
        text: "Trustor also makes Eagle Sweat, Eau de Liberté: an all-new formula with 83% more sweat, and nano-fragrance technology said to make its wearer smell like a hero of the Federation.",
      },
    ],
    ties: [{ to: "ministry:prosperity", how: "Same logo; ties unknown" }],
    related: ["ministry:prosperity", "dir:corps"],
    ads: ["trustor-bug-insurance", "trustor-invasion-insurance"],
    source: wiki("Trustor"),
  },
  {
    id: "admiral-defense",
    name: "Admiral Defense",
    address: "se://am.defense",
    trade: "Vehicles and defense",
    summary: "The Tier-1 contractor behind the FRV and the Maelstrom tank.",
    color: "#b9a37a",
    logo: "admiral-defense",
    sections: [
      {
        heading: "The contractor",
        text: "AM Defense had long built armaments for the SEAF before the [[ministry:defense|Ministry of Defense]] awarded it a multitrillion-Supercredit contract to equip the Helldivers. Its subsidiary X-UDE develops the TD-110 Maelstrom.",
        products: ["M-102 Gunner FRV", "TD-110 Maelstrom"],
      },
      {
        heading: "The Factory Hub",
        timeline: [
          { when: "11 Dec 2184", text: "Contract to build the Factory Hub on [[planet:Achernar Secundus|Achernar Secundus]]." },
          {
            when: "16 Dec 2184",
            text: "The Hub opens and starts on the M-102 Gunner FRV; the Helldivers turn back an Illuminate attack the same day.",
          },
          { when: "14 Mar 2185", text: "Achernar Secundus falls to the Terminids. The Hub is evacuated and FRV fuel cut by 90%." },
          { when: "17 Mar 2185", text: "The planet is retaken." },
          { when: "26 Apr 2185", text: "A second Illuminate attack is repelled after two days." },
          {
            when: "30 Jul 2185",
            text: "With the Xenoentomology Center on [[planet:Turing|Turing]], new Alcubierre drive shielding for [[manual:gloom|Gloom]] exploration.",
          },
          { when: "19 Aug 2185", text: "Achernar Secundus falls to the Terminids again." },
          { when: "9 Mar 2186", text: "Retaken; in thanks, the FRV is made available fleetwide for a day." },
        ],
      },
      {
        heading: "The Maelstrom",
        timeline: [
          {
            when: "22 Sep 2186",
            text: "A new agreement to build the TD-110 Maelstrom. The planets allotted to it, [[planet:Marfark|Marfark]] and [[planet:Phact Bay|Phact Bay]], are in enemy hands, so prototypes go to the Helldivers to retake them.",
          },
          { when: "26 Sep 2186", text: "Ground is broken for the Maelstrom factories." },
          {
            when: "After",
            text: "Debt and short supplies put the tank at risk, until Maelstrom defenses of [[planet:Gatria|Gatria]] and [[planet:Wasat|Wasat]] lift its sales. Mass production follows and the CEO receives the Distinguished Civilian Patriot Award. The Maelstrom was also the reward of the Armored Eagle campaign, in [[app:past|Past campaigns]].",
          },
        ],
      },
    ],
    ties: [{ to: "ministry:defense", how: "Tier-1 contractor" }],
    related: ["company:morgunson", "manual:gloom"],
    source: wiki("Admiral Defense"),
  },
  {
    id: "stahl-arms",
    name: "Ståhl Arms",
    address: "se://stahl.arms",
    trade: "Arms",
    summary: "The arms maker signed when the Illuminate strained the defense budget.",
    color: "#e5372e",
    logo: "stahl-arms",
    sections: [
      {
        heading: "Under contract",
        text: "The Illuminate's return put the defense budget under unexpected strain, and the [[ministry:defense|Ministry of Defense]] signed Ståhl Arms to supply the Helldivers with more weapons and gear.",
        products: [
          "StA-52 Assault Rifle",
          "StA-11 SMG",
          "PLAS-39 Accelerator Rifle",
          "StA-X3 W.A.S.P. Launcher",
          "AC-1 Dutiful armour",
          "AC-2 Obedient armour",
        ],
      },
      {
        heading: "From another war",
        text: "A crossover from the Killzone series, where Ståhl Arms is a Helghast arms maker. The weapons' own descriptions misspell the name as Stål Arms.",
      },
    ],
    ties: [{ to: "ministry:defense", how: "Contract for weapons and gear" }],
    related: ["company:morgunson", "company:admiral-defense"],
    source: wiki("Ståhl Arms"),
  },
  {
    id: "ulterior",
    name: "Ulterior",
    address: "se://ulterior.corp",
    trade: "Mining",
    summary: "The miner Super Earth trusts with its Element 710.",
    color: "#7f93a0",
    slogan: "Ulterior: operating under the mantle.",
    broadcast: "Super Earth runs on Element 710. And Element 710 runs on Ulterior Industries.",
    sections: [
      {
        heading: "Under the mantle",
        text: "The miner Super Earth trusts with all its mining needs. Concept art suggests the [[ministry:expansion|Ministry of Expansion]] owns it; the two share a logo.",
      },
    ],
    ties: [{ to: "ministry:expansion", how: "Believed to own it; they share a logo" }],
    related: ["ministry:expansion"],
    source: wiki("Ulterior"),
  },
];

/** `by` is the advertiser's site, as a link target; `brand` names an advertiser without one. */
export type Ad = {
  file: string;
  /** What the advert shows and says, for screen readers. */
  alt: string;
  shape: "wide" | "tall";
} & ({ by: string } | { brand: string });

export const ADS: Ad[] = [
  { file: "trustor-bug-insurance", alt: "Trustor Bank: put your foot down with bug insurance.", shape: "wide", by: "company:trustor" },
  { file: "trustor-invasion-insurance", alt: "Trustor Bank: hold your ground with invasion insurance.", shape: "tall", by: "company:trustor" },
  { file: "truth-see-treason", alt: "Ministry of Truth: see treason? Shout treason!", shape: "wide", by: "ministry:truth" },
  { file: "humanity-franchisement", alt: "Ministry of Humanity: franchisement is freedom.", shape: "tall", by: "ministry:humanity" },
  { file: "expansion-work-songs", alt: "Ministry of Expansion: democratic work songs encouraged.", shape: "tall", by: "ministry:expansion" },
  { file: "helldivers-take-a-life", alt: "Join the Helldivers today: take a life, save a life.", shape: "wide", by: "ministry:defense" },
  { file: "patriot-hut-helldawgs", alt: "Patriot Hut: flame-grilled Helldawgs, comin' in hot.", shape: "wide", brand: "Patriot Hut" },
  { file: "patriot-hut-burger", alt: "Patriot Hut: free your mouth.", shape: "tall", brand: "Patriot Hut" },
  { file: "libertea", alt: "LiberTea: a voter's paradise, a herbal tea for calming, patriotic thoughts.", shape: "tall", brand: "LiberTea" },
  { file: "libertea-tea", alt: "LiberTea: tea flavoured tea.", shape: "wide", brand: "LiberTea" },
  { file: "cornstitution", alt: "Cornstitution cereal: it's got corn!", shape: "wide", brand: "Cornstitution" },
  { file: "freegum", alt: "Freegum: you can't get enough!", shape: "tall", brand: "Freegum" },
  { file: "freegum-strawberry", alt: "Freegum strawberry flavour: you need it right now!", shape: "wide", brand: "Freegum" },
  { file: "democroffee-coffee-time", alt: "Democroffee: it's coffee time.", shape: "tall", brand: "Democroffee" },
  { file: "democroffee-drink-now", alt: "Democroffee: drink coffee now, wake up and smell the freedom.", shape: "tall", brand: "Democroffee" },
  { file: "voting-voyager", alt: "Voting Voyager: believe the unbelievable.", shape: "wide", brand: "Voting Voyager" },
  { file: "super-clean", alt: "Super Clean nuclear detergent.", shape: "tall", brand: "Super Clean" },
  { file: "super-clean-stain-remover", alt: "Super Clean stain remover, for oil stains, spores, bile and blood.", shape: "wide", brand: "Super Clean" },
  { file: "truth-cram-school", alt: "T. Ruth's Cram School: dare to dream.", shape: "tall", brand: "T. Ruth's Cram School" },
  { file: "juicetice-lemon-lime", alt: "Juicetice Lemonizer and Liberty Lime: hurts so good.", shape: "wide", brand: "Juicetice" },
  { file: "juicetice-berry-barrage", alt: "Juicetice Berry Barrage: coming in thirst place.", shape: "tall", brand: "Juicetice" },
  { file: "acar-drive-anywhere", alt: "ACAR: drive anywhere.", shape: "wide", brand: "ACAR" },
  { file: "seafood-caught-fresh", alt: "SEAFood: caught fresh.", shape: "tall", brand: "SEAFood" },
  { file: "seafood-whale", alt: "SEAFood: where there's a will, there's a whale.", shape: "wide", brand: "SEAFood" },
  { file: "inka-rebuilding", alt: "INKA Interior: rebuilding your world one piece at a time.", shape: "wide", brand: "INKA Interior" },
  { file: "spearhead-real-estate", alt: "Spearhead Real Estate: views to die for. Putting you first.", shape: "tall", brand: "Spearhead Real Estate" },
  { file: "justos-rectangle-8", alt: "JustOS: introducing the Rectangle 8.", shape: "wide", brand: "JustOS" },
];

export interface Holiday {
  name: string;
  /** 1-12. */
  month: number;
  day: number;
}

export const HOLIDAYS: Holiday[] = [
  { name: "Malevelon Creek Memorial Day", month: 4, day: 3 },
  { name: "Avengement Day", month: 10, day: 14 },
  { name: "Liberty Day", month: 10, day: 26 },
  { name: "Festival of Reckoning", month: 12, day: 25 },
];

export const GOV = {
  title: "Managed Democracy",
  address: "se://gov",
  icon: "super-earth",
  summary: "How Super Earth governs: the vote, the Council, citizen classes and the holidays.",
  sections: [
    {
      heading: "How the vote works",
      text: "Citizens answer a handful of questions at a voting terminal, and the voting algorithm matches their answers to candidates and reports the result to Super Earth. The Federation calls it freedom from the burden of choice; anyone who doubts it is a dissident. A higher citizen score weighs more in an election.",
    },
    {
      heading: "The Galactic Wide Web",
      text: "The algorithm runs over the Galactic Wide Web, the network of linked communication devices that carries data by quantum communication: the net this site is on.",
    },
    {
      heading: "Who governs",
      text: "The Democratic Council presented the first and final draft of Managed Democracy, on the day now kept as Liberty Day, and sees that elected leaders act on the values all citizens share. The President leads the government. On 29 May 2185, during the Illuminate assault on [[planet:Super Earth|Super Earth]], the President was killed in the Presidential Security Bunker; authority passed to the [[ministry:defense|Minister of Defense]] until a managed election chose a successor.",
    },
    {
      heading: "The ministries",
      points: MINISTRIES.map((m) => `[[ministry:${m.id}|${m.name}]]: ${m.summary.charAt(0).toLowerCase()}${m.summary.slice(1)}`),
    },
    {
      heading: "Citizen classes",
      text: "The Citizenship Classification Structure, kept by the [[ministry:humanity|Ministry of Humanity]], decides which citizens are the most equal: from the Super Citizens down through Classes A to F. What each class is owed, and how a citizen climbs, is on the [[site:citizens|Citizenship Classification Structure]] site.",
    },
    {
      heading: "Holidays",
      table: {
        head: ["Day", "Holiday"],
        rows: [
          {
            cells: [
              "3 April",
              "Malevelon Creek Memorial Day: three minutes of the lunch break in remembrance of those who died freeing [[planet:Malevelon Creek|Malevelon Creek]].",
            ],
          },
          {
            cells: [
              "14 October",
              "Avengement Day, on [[planet:Seyshel Beach|Seyshel Beach]] only: its citizens made the day of its liberation in 2185, after six weeks of fighting by a small cohort of Helldivers, a planetary holiday to remember the planets lost to humanity's foes.",
            ],
          },
          { cells: ["26 October", "Liberty Day: the Federation's most important holiday, for the day Managed Democracy was presented."] },
          {
            cells: [
              "25 to 28 December",
              "Festival of Reckoning: the winter holiday of “love, family, and the pursuit of peace”, with deep discounts, televised Helldiver victories, a Reckoning Fund for extra ordnance and a dissident execution parade. Its last day is the Feast of the Electorate, for the first free and managed election.",
            ],
          },
        ],
        note: "In 2186 the Liberty Day preparations began on 6 October with the Liberty Day Party Planning Protocol, under which “not being a party person” is no exemption.",
      },
    },
    {
      heading: "The Wall of Martyrs",
      text: "Every SEAF soldier and Helldiver who falls in combat has their name inscribed on the Wall of Martyrs; millions are added every day.",
    },
  ] satisfies SiteSection[],
  quotes: [
    "Managed Democracy is a cornerstone of advanced civilization.",
    "Super Earth has many holidays, but the most important is October 26th-Liberty Day!",
  ],
  related: ["dir:gov", "ministry:defense", "ministry:truth", "site:log"],
  sources: [WIKI.government, WIKI.libertyDay, WIKI.malevelonDay, wiki("Festival of Reckoning"), wiki("Seyshel Beach"), WIKI.wallOfMartyrs],
};

export const CITIZENS = {
  title: "Citizenship Classification Structure",
  address: "se://gov/citizens",
  icon: "ministry-humanity",
  summary: "Every citizen is equal; the CCS says exactly which are the most equal. The classes, what each is owed, and how a citizen climbs.",
  broadcasts: [
    "It doesn't matter who you are or where you were born- all citizens are equal in the eyes of freedom. Which is why Super Earth's most patriotic minds instituted the Citizenship Classification Structure. Thanks to the CCS, we can determine exactly which citizens are the most equal. Continue your war efforts, and one day you too could reach Class A citizenship. The CCS: Liberty Optimized.",
    "Keep working to get your Citizen Score up. A higher Citizen Score means a better you!",
  ],
  sections: [
    {
      heading: "The classes",
      text: "Seven classes are known, each from a briefing, a mission or a contract.",
      table: {
        head: ["Class", "What is known"],
        rows: [
          {
            cells: [
              "Super",
              "The highest known class, the Super Citizens. Super Citizen Anne, whose name is on a hospital for very sick children, may hold it, and the Democracy Officer may have meant it by “A+” during the Battle for Super Earth.",
            ],
          },
          {
            cells: [
              "A",
              "The top 1% of patriots, a 2186 briefing says. Helldivers rescue them in Emergency Evacuation missions; they retire to orbital stations, buy luxury curios and get the invitations.",
            ],
          },
          { cells: ["B", "A place in the Priority Emergency Bunkers when colonists are evacuated."] },
          {
            cells: [
              "C",
              "Granted to SEAF soldiers posted in exceptional hazard, and their approved families, when the mission ends or they die. Evacuated first when a planet is about to be destroyed.",
            ],
          },
          { cells: ["D", "No priority when a planet under immediate threat of destruction is evacuated."] },
          { cells: ["E", "The lowest grade that may enlist: the Helldiver contract asks for Citizenship Grade E or above."] },
          { cells: ["F", "The lowest known class, known only because it falls below the Helldivers' minimum."] },
        ],
      },
    },
    {
      heading: "Climbing the structure",
      points: [
        "Citizen Score: presumably tied to the class, and it counts in the vote: a higher score weighs more in an election.",
        "Service: when a Helldiver enlists, their immediate family receive 4.5 citizenship points, theirs to allocate and not to be exchanged for cash.",
        "Sacrifice: a SEAF posting of exceptional hazard earns the soldier and their family Class C.",
      ],
    },
    {
      heading: "Class in the war",
      timeline: [
        {
          when: "Apr 2185",
          text: "A party at the Free Springs Retreat on [[planet:Widow's Harbor|Widow's Harbor]], once the Meridian Singularity is halted, is by invitation to select Class A citizens.",
        },
        {
          when: "Feb 2186",
          text: "During Operation Valid Pretext, disaster relief funds for Class C citizens and below are redirected to give every Helldiver the Orbital Railcannon Strike for a time.",
        },
        {
          when: "Feb 2186",
          text: "Titan-class Terminid eggs, sold as patriotic curios to Class A citizens, hatch in their homes. Helldivers clear Class A zones on [[planet:Polaris Prime|Polaris Prime]] while some of their own cryo-pods carry displaced Class A citizens and their luxury goods to safety.",
        },
        {
          when: "Apr 2186",
          text: "Helldivers secure Terminid-held natural parks so homes can be built there for high-classification citizens.",
        },
        {
          when: "Sep 2186",
          text: "To hunt the network that sabotaged the TCS+, citizens of Class B and above are fully deputized: trained to spot traitors, issued printable badges and allowed necessary force.",
        },
        {
          when: "Oct 2186",
          text: "Gas residue drifting from [[planet:Senge 23|Senge 23]] clouds the Class A retirement stations in its orbit; the Helldivers' kills fund their new respiratory protection.",
        },
      ],
    },
  ] satisfies SiteSection[],
  related: ["site:gov", "ministry:humanity", "ministry:defense"],
  sources: [WIKI.government, wiki("Helldiver Contract of Employment"), wiki("SEAF Communications Device"), wiki("Major Orders")],
};
