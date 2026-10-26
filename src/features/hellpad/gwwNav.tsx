import { ArrowUpRight } from "lucide-react";
import { createContext, Fragment, useContext, useMemo, type MouseEvent, type ReactNode } from "react";
import { useFreshPlanets } from "../../api/queries";
import type { Planet } from "../../api/types";
import { MANUALS } from "../../data/gww";
import { CITIZENS, COMPANIES, GOV, HOLIDAYS, HOLIDAYS_SITE, MINISTRIES } from "../../data/gwwSites";
import { factionColor } from "../../domain/factions";
import { parseRich } from "../../domain/gwwText";
import { useUi, type GwwPage } from "../../state";

const HOME = "se://gww";
const SITE_ADDRESS = { gov: GOV.address, citizens: CITIZENS.address, holidays: HOLIDAYS_SITE.address, log: "se://hc/war-log" };
const DIR_ADDRESS = { manuals: "se://hc/manuals", gov: "se://gov/directory", corps: "se://corp" };

export function addressOf(p: GwwPage): string {
  if (p.kind === "manual") return MANUALS.find((m) => m.id === p.id)?.address ?? HOME;
  if (p.kind === "ministry") return MINISTRIES.find((m) => m.id === p.id)?.address ?? HOME;
  if (p.kind === "company") return COMPANIES.find((c) => c.id === p.id)?.address ?? HOME;
  if (p.kind === "holiday") return HOLIDAYS.find((h) => h.id === p.id)?.address ?? HOME;
  if (p.kind === "site") return SITE_ADDRESS[p.id];
  if (p.kind === "dir") return DIR_ADDRESS[p.id];
  if (p.kind === "missing") return p.address;
  return HOME;
}

/** Every address on the G.W.W., to the link target it opens. */
const ADDRESSES = new Map<string, string>([
  [HOME, "home"],
  ...Object.entries(SITE_ADDRESS).map(([id, a]): [string, string] => [a, `site:${id}`]),
  ...Object.entries(DIR_ADDRESS).map(([id, a]): [string, string] => [a, `dir:${id}`]),
  ...MANUALS.map((m): [string, string] => [m.address, `manual:${m.id}`]),
  ...MINISTRIES.map((m): [string, string] => [m.address, `ministry:${m.id}`]),
  ...COMPANIES.map((c): [string, string] => [c.address, `company:${c.id}`]),
  ...HOLIDAYS.map((h): [string, string] => [h.address, `holiday:${h.id}`]),
  ["se://strohmann.news", "app:news"],
]);

/** Typed addresses may skip the scheme or end in a slash. */
export function normalize(typed: string): string {
  const a = typed.trim().toLowerCase().replace(/\/+$/, "");
  return a.startsWith("se://") ? a : `se://${a.replace(/^\/+/, "")}`;
}

export const ADDRESS_LIST = [...ADDRESSES.keys()];

/** Pages reached only through Managed Democracy: they open when typed, but aren't suggested. */
export const isGovSubpage = (target: string) =>
  target === "site:citizens" || target === "site:holidays" || target.startsWith("holiday:");

export const SUGGESTED_ADDRESSES = ADDRESS_LIST.filter((a) => !isGovSubpage(ADDRESSES.get(a)!));

/** The link target a typed address opens, if any site answers at it. */
export const targetAt = (typed: string): string | undefined => ADDRESSES.get(normalize(typed));

/** `app:` link targets and their titles. */
export const APP_PAGES: Record<string, string> = {
  news: "Strohmann News",
  overview: "High Command overview",
  map: "Galaxy map",
  campaigns: "Active planets",
  stations: "Space stations",
  past: "Past campaigns",
  planets: "Planet index",
};

export const DIRS: Record<Extract<GwwPage, { kind: "dir" }>["id"], string> = {
  manuals: "Training manuals",
  gov: "Super Earth",
  corps: "Corporations",
};

/** The G.W.W. page a link target opens, or null for targets outside it. */
export function pageOf(target: string): GwwPage | null {
  if (target === "home") return { kind: "home" };
  const [kind, id] = [target.slice(0, target.indexOf(":")), target.slice(target.indexOf(":") + 1)];
  if (kind === "manual" && MANUALS.some((m) => m.id === id)) return { kind, id };
  if (kind === "ministry" && MINISTRIES.some((m) => m.id === id)) return { kind, id };
  if (kind === "company" && COMPANIES.some((c) => c.id === id)) return { kind, id };
  if (kind === "holiday" && HOLIDAYS.some((h) => h.id === id)) return { kind, id };
  if (kind === "site" && (id === "log" || id === "gov" || id === "citizens" || id === "holidays")) return { kind, id };
  if (kind === "dir" && id in DIRS) return { kind, id: id as keyof typeof DIRS };
  return null;
}

export function titleOf(target: string): string {
  if (target === "home") return "G.W.W. home";
  const id = target.slice(target.indexOf(":") + 1);
  const page = pageOf(target);
  if (page?.kind === "manual") return MANUALS.find((m) => m.id === id)!.title;
  if (page?.kind === "ministry") return MINISTRIES.find((m) => m.id === id)!.name;
  if (page?.kind === "company") return COMPANIES.find((c) => c.id === id)!.name;
  if (page?.kind === "holiday") return HOLIDAYS.find((h) => h.id === id)!.name;
  if (page?.kind === "site")
    return { gov: GOV.title, citizens: CITIZENS.title, holidays: HOLIDAYS_SITE.title, log: "Galactic war log" }[page.id];
  if (page?.kind === "dir") return DIRS[page.id];
  if (target.startsWith("app:")) return APP_PAGES[id] ?? id;
  return id;
}

interface Nav {
  go(page: GwwPage): void;
  /** Opens any link target: a G.W.W. page, a page of the app, or a planet (its options placed at `e`). */
  follow(target: string, e?: MouseEvent): void;
  planet(name: string): Planet | undefined;
}

const NavContext = createContext<Nav | null>(null);

export function useGwwNav(): Nav {
  const nav = useContext(NavContext);
  if (!nav) throw new Error("useGwwNav outside the G.W.W.");
  return nav;
}

export function GwwNavProvider({ go, children }: { go(page: GwwPage): void; children: ReactNode }) {
  const planets = useFreshPlanets().data;
  const openOptions = useUi((s) => s.openOptions);
  const openModule = useUi((s) => s.openModule);
  const setView = useUi((s) => s.setView);
  const nav = useMemo<Nav>(() => {
    const byName = new Map((planets ?? []).map((p) => [p.name.toLowerCase(), p]));
    const planet = (name: string) => byName.get(name.toLowerCase());
    return {
      go,
      planet,
      follow(target, e) {
        const page = pageOf(target);
        if (page) return go(page);
        const id = target.slice(target.indexOf(":") + 1);
        if (target.startsWith("planet:")) {
          const p = planet(id);
          if (!p) return;
          const box = (e?.currentTarget as HTMLElement | undefined)?.getBoundingClientRect();
          openOptions({ planet: p.index, x: e?.clientX || box?.left || 0, y: e?.clientY || box?.bottom || 0 });
          return;
        }
        if (id === "stations" || id === "planets") return openModule(id);
        if (id === "past") return openModule("campaigns");
        if (id === "news" || id === "overview" || id === "map" || id === "campaigns") return setView(id);
      },
    };
  }, [planets, go, openOptions, openModule, setView]);
  return <NavContext.Provider value={nav}>{children}</NavContext.Provider>;
}

/** Never followed (clicks are handled); it makes the anchor a real link for keyboards and screen readers. */
const hrefOf = (target: string) => `#gww/${target}`;

/** Text with its `[[kind:target|label]]` links made live. */
export function Rich({ text }: { text: string }) {
  const nav = useGwwNav();
  const open = (target: string) => (e: MouseEvent) => {
    e.preventDefault();
    nav.follow(target, e);
  };
  return (
    <>
      {parseRich(text).map((s, i) => {
        if (!("kind" in s)) return <Fragment key={i}>{s.text}</Fragment>;
        if (s.kind === "planet") {
          const p = nav.planet(s.target);
          if (!p) return <Fragment key={i}>{s.label}</Fragment>;
          return (
            <a
              key={i}
              className="gww-link planet"
              href={hrefOf(`planet:${s.target}`)}
              style={{ color: factionColor(p.currentOwner) }}
              title={`${p.name}: open its options`}
              onClick={open(`planet:${s.target}`)}
            >
              {s.label}
            </a>
          );
        }
        return (
          <a key={i} className={`gww-link ${s.kind}`} href={hrefOf(`${s.kind}:${s.target}`)} onClick={open(`${s.kind}:${s.target}`)}>
            {s.label}
            {s.kind === "app" && (
              <>
                <ArrowUpRight size={11} aria-hidden />
                <span className="sr-only"> (leaves the G.W.W.)</span>
              </>
            )}
          </a>
        );
      })}
    </>
  );
}
