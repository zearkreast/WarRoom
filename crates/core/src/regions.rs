use crate::api::{Faction, Planet};
use crate::format::to_fixed;
use crate::galaxy::{GALAXY_SCALE, Pt, majority, project};
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
    /// Label anchor: the centroid of its planets.
    pub center: Pt,
}

const VOID_RING_POINTS: usize = 72;
const VOID_RING_RADIUS: f64 = 1.07 * GALAXY_SCALE;
const BOUNDS: f64 = 1.25 * GALAXY_SCALE;

/// Enemy territory outlines: same-owner cells merged into smooth fronts.
pub fn build_regions(planets: &[Planet]) -> Vec<RegionShape> {
    build_outlines(planets, |p| (p.current_owner != Faction::Humans).then(|| p.current_owner.as_str().to_string()), 4)
}

fn key(p: Pt) -> String {
    format!("{},{}", to_fixed(p[0], 3), to_fixed(p[1], 3))
}

fn edge_key(a: Pt, b: Pt) -> String {
    let (ka, kb) = (key(a), key(b));
    if ka < kb { format!("{ka}|{kb}") } else { format!("{kb}|{ka}") }
}

/// Builds one outline per group (`key_of`) by merging the Voronoi cells of its planets: an
/// edge shared by two cells of the same group is interior and dropped; the remaining edges
/// are chained into loops and smoothed. Planets keyed None belong to no outline. A ring of
/// "void" sites around the galaxy keeps the outer cells from stretching to the bounds.
fn build_outlines(
    planets: &[Planet],
    key_of: impl Fn(&Planet) -> Option<String>,
    smoothing: usize,
) -> Vec<RegionShape> {
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

    let mut edge_owners: HashMap<String, Vec<Option<&str>>> = HashMap::new();
    let mut cells: Vec<Vec<Pt>> = Vec::with_capacity(planet_sites);
    for i in 0..planet_sites {
        let poly = voronoi.cell_polygon(i).unwrap_or_default();
        for k in 0..poly.len().saturating_sub(1) {
            edge_owners.entry(edge_key(poly[k], poly[k + 1])).or_default().push(site_sector[i].as_deref());
        }
        cells.push(poly);
    }

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
        // Boundary edges of this sector, as directed segments in cell winding order.
        let mut segments: Vec<(Pt, Pt)> = Vec::new();
        for &i in sector_sites {
            let poly = &cells[i];
            for k in 0..poly.len().saturating_sub(1) {
                let owners = &edge_owners[&edge_key(poly[k], poly[k + 1])];
                let interior = owners.iter().filter(|o| **o == Some(name.as_str())).count() > 1;
                if !interior {
                    segments.push((poly[k], poly[k + 1]));
                }
            }
        }
        let path = chain_loops(&segments).iter().map(|l| to_path(&chaikin(l, smoothing))).collect::<Vec<_>>().join(" ");
        shapes.push(RegionShape {
            name: name.clone(),
            owner: majority(members.iter().map(|&i| planets[i].current_owner)).unwrap_or_default(),
            path,
            planets: members.iter().map(|&i| planets[i].index).collect(),
            center: centroid(&sector_sites.iter().map(|&i| sites[i]).collect::<Vec<_>>()),
        });
    }
    shapes
}

fn chain_loops(segments: &[(Pt, Pt)]) -> Vec<Vec<Pt>> {
    let mut by_start: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, (a, _)) in segments.iter().enumerate() {
        by_start.entry(key(*a)).or_default().push(i);
    }
    let mut used = vec![false; segments.len()];
    let mut loops = Vec::new();
    for s in 0..segments.len() {
        if used[s] {
            continue;
        }
        let mut lp = Vec::new();
        let mut cur = Some(s);
        while let Some(c) = cur.filter(|c| !used[*c]) {
            used[c] = true;
            let (a, b) = segments[c];
            lp.push(a);
            cur = by_start.get(&key(b)).and_then(|list| list.iter().copied().find(|i| !used[*i]));
        }
        if lp.len() >= 3 {
            loops.push(lp);
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
    [pts.iter().map(|p| p[0]).sum::<f64>() / n, pts.iter().map(|p| p[1]).sum::<f64>() / n]
}
