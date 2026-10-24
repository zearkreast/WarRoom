/**
 * Galactic War Web content: training manuals on how the war works and the Federation's corporations.
 * Written for this app in its own words; facts follow the game and the Helldivers Wiki.
 */

export interface Manual {
  id: string;
  title: string;
  /** Address shown on the tile, in the Federation's net style. */
  address: string;
  summary: string;
  sections: { heading: string; text: string }[];
  tips: string[];
}

export const MANUALS: Manual[] = [
  {
    id: "liberation",
    title: "Liberation and defense",
    address: "se://hc/manuals/liberation",
    summary: "How a planet changes hands, and why the bar sometimes runs backwards.",
    sections: [
      {
        heading: "Liberation",
        text: "Every mission won on an enemy planet pushes its liberation bar towards 100%. The enemy pushes back every hour by a set share of the planet's strength, its regeneration. A planet only moves when Helldivers outpace that regeneration; at 100% it returns to Super Earth.",
      },
      {
        heading: "Defense",
        text: "When an enemy attacks a Super Earth planet, a defense campaign opens with a deadline. Missions fill the defense bar while the enemy fills theirs; reach 100% before the deadline and the planet holds, miss it and the planet falls.",
      },
      {
        heading: "Reading the rate",
        text: "This app shows each front's rate per hour after regeneration. A rate above zero means the planet is moving towards liberation; the projection assumes the current turnout holds.",
      },
    ],
    tips: [
      "A front with few Helldivers and high regeneration can stall at the same percentage for days. Joining a front that is already moving finishes it sooner.",
    ],
  },
  {
    id: "impact",
    title: "The galactic impact modifier",
    address: "se://hc/manuals/impact",
    summary: "Why one mission counts for more on a quiet night than on a busy one.",
    sections: [
      {
        heading: "What it is",
        text: "Each mission's contribution to a front is scaled by a galaxy-wide number, the impact modifier. It moves with the number of Helldivers in the war, so fronts advance at a similar pace whether many or few are fighting.",
      },
      {
        heading: "Where to see it",
        text: "The High Command overview charts the modifier over the last day under the Helldivers on each front. Peaks in turnout usually line up with dips in the modifier.",
      },
    ],
    tips: ["The modifier is shared by every front, so it never favours one planet over another."],
  },
  {
    id: "supply-lines",
    title: "Supply lines",
    address: "se://hc/manuals/supply-lines",
    summary: "Which planets can be attacked, and why cutting a line matters.",
    sections: [
      {
        heading: "The network",
        text: "Planets are linked by supply lines. Super Earth can only open a liberation campaign on an enemy planet linked to a planet it holds, and the enemy attacks along the same links.",
      },
      {
        heading: "Encirclement",
        text: "Taking the planets around an enemy holding cuts it off: it can no longer be reinforced from elsewhere and its neighbours stop being attacked through it.",
      },
    ],
    tips: ["The map's Galaxy overlay draws every supply line; the Sectors overlay colours the lines inside enemy territory."],
  },
  {
    id: "cities",
    title: "Cities and regions",
    address: "se://hc/manuals/cities",
    summary: "Planets with cities, and what freeing them does for the planet.",
    sections: [
      {
        heading: "Regions",
        text: "Some planets carry cities, each with its own health. Freeing a city adds to the planet's liberation and takes away part of the enemy's hold, so a planet with free cities is easier to finish.",
      },
      {
        heading: "Maximum Security Cities",
        text: "New Hope City, New Aspiration City and New Yearning City are the Federation's three Maximum Security Cities. They are marked on the map as points of interest.",
      },
    ],
    tips: ["A planet's cities are listed under its operational parameters."],
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
    tips: ["Sub-faction pawns beside a planet show which Gloom strains have been reported on it."],
  },
  {
    id: "dss",
    title: "The Democracy Space Station",
    address: "se://hc/manuals/dss",
    summary: "The Helldiver-funded station, its votes and its tactical actions.",
    sections: [
      {
        heading: "Votes",
        text: "Helldivers vote in game on the station's next destination. When the vote closes, the station jumps to the winning planet and supports the fight there.",
      },
      {
        heading: "Tactical actions",
        text: "Helldivers fund the station's tactical actions by donating resources. A funded action runs for a while, then cools down before it can be funded again.",
      },
    ],
    tips: ["The Hellpad's space station page shows which actions are running now and when the others are ready again."],
  },
];

export interface Company {
  name: string;
  address: string;
  trade: string;
  text: string;
}

export const COMPANIES: Company[] = [
  {
    name: "Strohmann Media",
    address: "se://strohmann.news",
    trade: "News and broadcasting",
    text: "The Federation's news network, widely believed to answer to the Ministry of Truth. Its dispatches are this app's Strohmann News.",
  },
  {
    name: "Morgunson",
    address: "se://morgunson.arsenal",
    trade: "Arms",
    text: "Also known as Morgunson Arsenal; the Ministry of Defense's main weapons supplier.",
  },
  {
    name: "Permacura",
    address: "se://permacura.health",
    trade: "Medicine",
    text: "Maker of the stims and other healing products that keep Helldivers on their feet.",
  },
  {
    name: "C.O.B. Genetics",
    address: "se://cob.genetics",
    trade: "Agriculture",
    text: "Engineers and grows enhanced crops to feed the citizens of Super Earth.",
  },
  {
    name: "AmpuDyne",
    address: "se://ampudyne.med",
    trade: "Prosthetics",
    text: "Builds cybernetic limbs for wounded citizens, of whom the war supplies many.",
  },
  {
    name: "Trustor",
    address: "se://trustor.bank",
    trade: "Banking",
    text: "The bank serving Super Earth and its worlds.",
  },
  {
    name: "Admiral Defense",
    address: "se://am.defense",
    trade: "Vehicles and defense",
    text: "AM Defense; the maker behind the Maelstrom tank production the Major Orders fought for in 2186.",
  },
  {
    name: "Ulterior",
    address: "se://ulterior.corp",
    trade: "Conglomerate",
    text: "A Mega-Corporation based on Super Earth whose business is not made public.",
  },
];
