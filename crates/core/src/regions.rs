//! Enemy territory outlines for the Regions overlay, from merged Voronoi cells.

use crate::api::{Faction, Planet};
use crate::format::to_fixed;
use crate::galaxy::{GALAXY_SCALE, Pt, RINGS, project, ring_radius};
use crate::sieges::{besieged, neighbours};
use crate::voronoi::Delaunay;
use std::collections::HashMap;

/// Region of one enemy faction's contiguous territory; Super Earth space is left open.
#[derive(Clone, Debug, PartialEq)]
pub struct RegionShape {
    pub name: String,
    pub owner: Faction,
    /// Closed, smoothed SVG path data (may contain several loops).
    pub path: String,
    /// Planet indices in this region.
    pub planets: Vec<i64>,
    /// Its planets under attack, hatched in the region's colour inside the outline.
    pub contested: Vec<Patch>,
    /// Its planets under a SEAF siege, hatched in Super Earth's colour inside the outline.
    pub besieged: Vec<Patch>,
    /// Label anchor: the centroid of its planets.
    pub center: Pt,
}

const VOID_RING_POINTS: usize = 72;
const VOID_RING_RADIUS: f64 = 1.07 * GALAXY_SCALE;
const BOUNDS: f64 = 1.25 * GALAXY_SCALE;
/// Sides of the polygon the cells are clipped to, standing in for the galaxy's rim.
const RIM_SIDES: usize = 180;
const PATCH_SMOOTHING: usize = 3;
/// How far a besieged planet's cell is drawn in towards the planet, so a gap shows it cut off from its faction.
const SIEGE_INSET: f64 = 0.74;
/// Spacing of the grid the outlines are traced on, map units.
const FIELD_STEP: f64 = 3.0;
/// Box blur radius in grid steps, run twice: it rounds the corners where cells meet, closes notches and drops
/// slivers narrower than about 15 map units, a third of the usual distance between planets.
const FIELD_BLUR: usize = 3;

/// One planet's cell inside a region, smoothed, for a hatch that fades out from the planet.
#[derive(Clone, Debug, PartialEq)]
pub struct Patch {
    pub planet: i64,
    pub path: String,
    /// The planet's position.
    pub center: Pt,
    /// Map units from the planet to the cell's farthest point.
    pub reach: f64,
}

/// Enemy territory outlines: same-owner cells merged into smooth fronts.
pub fn build_regions(planets: &[Planet]) -> Vec<RegionShape> {
    build_outlines(planets, |p| (p.current_owner != Faction::Humans).then_some(p.current_owner))
}

/// As `build_regions`, with each planet under attack drawn into its attacker's territory.
pub fn build_attack_regions(planets: &[Planet]) -> Vec<RegionShape> {
    build_outlines(planets, holder)
}

fn holder(p: &Planet) -> Option<Faction> {
    match &p.event {
        Some(e) => Some(e.faction),
        None => (p.current_owner != Faction::Humans).then_some(p.current_owner),
    }
}

fn key(p: Pt) -> String {
    format!("{},{}", to_fixed(p[0], 3), to_fixed(p[1], 3))
}

/// One outline per faction (`owner_of`) from its planets' Voronoi cells cut to the galaxy's rim: painted onto a
/// grid, blurred and traced at half strength, so where cells meet the outline comes out rounded. A ring of "void"
/// sites keeps the outer cells from stretching to the bounds. Holes are loops of their own: fill and clip region
/// paths even-odd.
fn build_outlines(planets: &[Planet], owner_of: impl Fn(&Planet) -> Option<Faction>) -> Vec<RegionShape> {
    // A besieged planet is cut off from its faction's ground, so it is a region of its own.
    let sieged = besieged(planets, &neighbours(planets));
    let key_of = |p: &Planet| {
        owner_of(p).map(|f| {
            if sieged.contains(&p.index) { format!("{} siege {}", f.as_str(), p.index) } else { f.as_str().to_string() }
        })
    };
    // Several planets can share a position (e.g. at the origin). Delaunay gives duplicates
    // no cell, so only the first planet at a position becomes a site.
    let mut sites: Vec<Pt> = Vec::new();
    let mut site_sector: Vec<Option<String>> = Vec::new();
    let mut site_of: HashMap<(u64, u64), usize> = HashMap::new();
    let mut planet_site: Vec<usize> = Vec::new();
    for p in planets {
        let k = (p.position.x.to_bits(), p.position.y.to_bits());
        let s = *site_of.entry(k).or_insert_with(|| {
            sites.push(project(p.position));
            site_sector.push(key_of(p));
            sites.len() - 1
        });
        planet_site.push(s);
    }
    let planet_sites = sites.len();
    for i in 0..VOID_RING_POINTS {
        let a = (i as f64 / VOID_RING_POINTS as f64) * std::f64::consts::PI * 2.0;
        sites.push([a.cos() * VOID_RING_RADIUS, a.sin() * VOID_RING_RADIUS]);
        site_sector.push(None);
    }

    let delaunay = Delaunay::from(&sites);
    let voronoi = delaunay.voronoi([-BOUNDS, -BOUNDS, BOUNDS, BOUNDS]);

    let cells: Vec<Vec<Pt>> =
        (0..planet_sites).map(|i| clip_to_rim(voronoi.cell_polygon(i).unwrap_or_default())).collect();

    // Group → its planets (by array position) and the sites it owns, in the order groups first appear.
    let mut by_sector: Vec<(String, Vec<usize>, Vec<usize>)> = Vec::new();
    for (i, p) in planets.iter().enumerate() {
        let Some(k) = key_of(p) else { continue };
        let at = match by_sector.iter().position(|(name, _, _)| *name == k) {
            Some(at) => at,
            None => {
                by_sector.push((k.clone(), Vec::new(), Vec::new()));
                by_sector.len() - 1
            }
        };
        let entry = &mut by_sector[at];
        entry.1.push(i);
        let site = planet_site[i];
        if site_sector[site].as_deref() == Some(k.as_str()) && !entry.2.contains(&site) {
            entry.2.push(site);
        }
    }

    let mut shapes = Vec::new();
    for (name, members, sector_sites) in &by_sector {
        if sector_sites.is_empty() {
            continue;
        }
        let cut_off = members.iter().any(|&i| sieged.contains(&planets[i].index));
        let painted: Vec<Vec<Pt>> = sector_sites
            .iter()
            .map(|&i| if cut_off { shrink(&cells[i], sites[i], SIEGE_INSET) } else { cells[i].clone() })
            .collect();
        let painted: Vec<&[Pt]> = painted.iter().map(Vec::as_slice).collect();
        let path = trace(&painted).iter().map(|l| to_path(&chaikin(l, 1))).collect::<Vec<_>>().join(" ");
        let patches = |pick: &dyn Fn(&Planet) -> bool| {
            members
                .iter()
                .filter(|&&i| pick(&planets[i]) && sector_sites.contains(&planet_site[i]))
                .filter_map(|&i| patch(planets[i].index, sites[planet_site[i]], &cells[planet_site[i]]))
                .collect::<Vec<_>>()
        };
        let contested = patches(&|p| p.event.is_some());
        let besieged = patches(&|p| sieged.contains(&p.index));
        shapes.push(RegionShape {
            name: name.clone(),
            owner: owner_of(&planets[members[0]]).unwrap_or_default(),
            path,
            planets: members.iter().map(|&i| planets[i].index).collect(),
            contested,
            besieged,
            center: centroid(&sector_sites.iter().map(|&i| sites[i]).collect::<Vec<_>>()),
        });
    }
    shapes
}

/// A cell (closed ring, first point repeated last) cut to the galaxy's rim, a polygon of tangent half-planes.
fn clip_to_rim(cell: Vec<Pt>) -> Vec<Pt> {
    let rim = ring_radius(RINGS - 1) * GALAXY_SCALE;
    let inner = rim * (std::f64::consts::PI / RIM_SIDES as f64).cos();
    if cell.iter().all(|p| p[0].hypot(p[1]) <= inner) {
        return cell;
    }
    let mut ring: Vec<Pt> = cell[..cell.len().saturating_sub(1)].to_vec();
    for k in 0..RIM_SIDES {
        let a = (k as f64 + 0.5) / RIM_SIDES as f64 * std::f64::consts::TAU;
        let n = [a.cos(), a.sin()];
        let side = |p: Pt| p[0] * n[0] + p[1] * n[1] - inner;
        let mut out = Vec::with_capacity(ring.len() + 1);
        for i in 0..ring.len() {
            let (p, q) = (ring[i], ring[(i + 1) % ring.len()]);
            let (sp, sq) = (side(p), side(q));
            if sp <= 0.0 {
                out.push(p);
            }
            if (sp <= 0.0) != (sq <= 0.0) {
                let (u, v) = if key(p) < key(q) { (p, q) } else { (q, p) };
                let (su, sv) = (side(u), side(v));
                let t = su / (su - sv);
                out.push([u[0] + (v[0] - u[0]) * t, u[1] + (v[1] - u[1]) * t]);
            }
        }
        ring = out;
        if ring.len() < 3 {
            return Vec::new();
        }
    }
    ring.push(ring[0]);
    ring
}

fn patch(planet: i64, center: Pt, cell: &[Pt]) -> Option<Patch> {
    if cell.len() < 4 {
        return None;
    }
    let ring = chaikin(&cell[..cell.len() - 1], PATCH_SMOOTHING);
    let reach = ring.iter().map(|p| (p[0] - center[0]).hypot(p[1] - center[1])).fold(0.0, f64::max);
    Some(Patch { planet, path: to_path(&ring), center, reach })
}

/// A cell drawn in towards `towards` by `k` (1 leaves it as it is).
fn shrink(cell: &[Pt], towards: Pt, k: f64) -> Vec<Pt> {
    cell.iter().map(|p| [towards[0] + (p[0] - towards[0]) * k, towards[1] + (p[1] - towards[1]) * k]).collect()
}

/// The grid the outlines are traced on: square, centred on the galaxy, a step past the rim on every side.
struct Grid {
    n: usize,
    origin: f64,
    rim: f64,
}

impl Grid {
    fn new() -> Grid {
        let rim = ring_radius(RINGS - 1) * GALAXY_SCALE;
        let pad = (FIELD_BLUR * 2 + 2) as f64 * FIELD_STEP;
        let n = ((rim + pad) * 2.0 / FIELD_STEP).ceil() as usize + 1;
        Grid { n, origin: -(rim + pad), rim }
    }

    fn at(&self, i: usize) -> f64 {
        self.origin + i as f64 * FIELD_STEP
    }

    /// The node range covering `lo..=hi` map units.
    fn span(&self, lo: f64, hi: f64) -> std::ops::RangeInclusive<usize> {
        let a = ((lo - self.origin) / FIELD_STEP).floor().max(0.0) as usize;
        let b = (((hi - self.origin) / FIELD_STEP).ceil() as usize).min(self.n - 1);
        a..=b
    }
}

/// Paints convex cells (closed rings) onto the grid, 1 inside and 0 outside.
fn paint(grid: &Grid, cells: &[&[Pt]]) -> Vec<f32> {
    let mut field = vec![0f32; grid.n * grid.n];
    for cell in cells {
        if cell.len() < 4 {
            continue;
        }
        let ring = &cell[..cell.len() - 1];
        let turn = |a: Pt, b: Pt, p: Pt| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        let winding = turn(ring[0], ring[1], ring[2]).signum();
        let (mut x0, mut x1, mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
        for p in ring {
            (x0, x1, y0, y1) = (x0.min(p[0]), x1.max(p[0]), y0.min(p[1]), y1.max(p[1]));
        }
        for j in grid.span(y0, y1) {
            for i in grid.span(x0, x1) {
                let p = [grid.at(i), grid.at(j)];
                let inside = (0..ring.len()).all(|k| turn(ring[k], ring[(k + 1) % ring.len()], p) * winding >= 0.0);
                if inside {
                    field[j * grid.n + i] = 1.0;
                }
            }
        }
    }
    field
}

/// Two passes of a box blur along rows, then columns.
fn blur(field: &mut [f32], n: usize) {
    let r = FIELD_BLUR;
    let width = (2 * r + 1) as f32;
    let mut line = vec![0f32; n];
    for _ in 0..2 {
        for along_rows in [true, false] {
            for a in 0..n {
                let idx = |b: usize| if along_rows { a * n + b } else { b * n + a };
                let mut sum: f32 = (0..=r.min(n - 1)).map(|b| field[idx(b)]).sum();
                for (b, out) in line.iter_mut().enumerate() {
                    *out = sum / width;
                    if b + r + 1 < n {
                        sum += field[idx(b + r + 1)];
                    }
                    if b >= r {
                        sum -= field[idx(b - r)];
                    }
                }
                for (b, v) in line.iter().enumerate() {
                    field[idx(b)] = *v;
                }
            }
        }
    }
}

/// The loops where the painted, blurred cells fall to half strength (marching squares). Crossings are keyed by
/// the grid edge they sit on, so neighbouring squares meet exactly; where two diagonal corners are inside, the
/// square's centre decides whether they join.
fn trace(cells: &[&[Pt]]) -> Vec<Vec<Pt>> {
    let grid = Grid::new();
    let n = grid.n;
    let mut field = paint(&grid, cells);
    blur(&mut field, n);
    let v = |i: usize, j: usize| field[j * n + i];
    let inside = |i: usize, j: usize| v(i, j) >= 0.5;
    // Edge ids: 2 * node for the edge to the right of a node, 2 * node + 1 for the one below it.
    let right = |i: usize, j: usize| 2 * (j * n + i);
    let down = |i: usize, j: usize| 2 * (j * n + i) + 1;
    let mut links: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut join = |a: usize, b: usize| {
        links.entry(a).or_default().push(b);
        links.entry(b).or_default().push(a);
    };
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let corners = [inside(i, j), inside(i + 1, j), inside(i + 1, j + 1), inside(i, j + 1)];
            // Edges in order round the square: top, right, bottom, left; edge k runs from corner k to k + 1.
            let edges = [right(i, j), down(i + 1, j), right(i, j + 1), down(i, j)];
            let crossed: Vec<usize> = (0..4).filter(|&k| corners[k] != corners[(k + 1) % 4]).collect();
            match crossed.len() {
                2 => join(edges[crossed[0]], edges[crossed[1]]),
                4 => {
                    let centre = (v(i, j) + v(i + 1, j) + v(i + 1, j + 1) + v(i, j + 1)) / 4.0 >= 0.5;
                    // Cut off each corner on the losing side: its two edges are the one before it and its own.
                    for c in (0..4).filter(|&c| corners[c] != centre) {
                        join(edges[(c + 3) % 4], edges[c]);
                    }
                }
                _ => {}
            }
        }
    }
    let point = |e: usize| {
        let node = e / 2;
        let (i, j) = (node % n, node / n);
        let (i2, j2) = if e.is_multiple_of(2) { (i + 1, j) } else { (i, j + 1) };
        let (a, b) = (v(i, j), v(i2, j2));
        let t = f64::from((0.5 - a) / (b - a));
        let p = [grid.at(i) + (grid.at(i2) - grid.at(i)) * t, grid.at(j) + (grid.at(j2) - grid.at(j)) * t];
        // The grid can put a crossing up to half a step past the rim the cells were cut to.
        let r = p[0].hypot(p[1]);
        if r > grid.rim { [p[0] * grid.rim / r, p[1] * grid.rim / r] } else { p }
    };
    let mut ids: Vec<usize> = links.keys().copied().collect();
    ids.sort_unstable();
    let mut used: HashMap<usize, bool> = HashMap::new();
    let mut loops = Vec::new();
    for start in ids {
        if used.contains_key(&start) {
            continue;
        }
        let mut ring = Vec::new();
        let (mut prev, mut cur) = (usize::MAX, start);
        loop {
            used.insert(cur, true);
            ring.push(point(cur));
            let next = links[&cur].iter().copied().find(|&e| e != prev && !used.contains_key(&e));
            match next {
                Some(e) => (prev, cur) = (cur, e),
                None => break,
            }
        }
        if ring.len() >= 3 {
            loops.push(ring);
        }
    }
    loops
}

/// Chaikin corner cutting on a closed ring; each pass rounds the corners further.
pub fn chaikin(ring: &[Pt], passes: usize) -> Vec<Pt> {
    let mut pts = ring.to_vec();
    for _ in 0..passes {
        let mut out = Vec::with_capacity(pts.len() * 2);
        for i in 0..pts.len() {
            let a = pts[i];
            let b = pts[(i + 1) % pts.len()];
            out.push([a[0] * 0.75 + b[0] * 0.25, a[1] * 0.75 + b[1] * 0.25]);
            out.push([a[0] * 0.25 + b[0] * 0.75, a[1] * 0.25 + b[1] * 0.75]);
        }
        pts = out;
    }
    pts
}

fn to_path(ring: &[Pt]) -> String {
    let pts: Vec<String> = ring.iter().map(|p| format!("{},{}", to_fixed(p[0], 1), to_fixed(p[1], 1))).collect();
    format!("M{}Z", pts.join("L"))
}

fn centroid(pts: &[Pt]) -> Pt {
    let n = if pts.is_empty() { 1.0 } else { pts.len() as f64 };
    [crate::format::js_sum(pts.iter().map(|p| p[0])) / n, crate::format::js_sum(pts.iter().map(|p| p[1])) / n]
}
