//! Delaunay triangulation and clipped Voronoi cells, ported from delaunator 5 and d3-delaunay 6 so the
//! map's territory shapes come out exactly as they did in the JavaScript app.

const EPSILON: f64 = f64::EPSILON;

/// `orient2d` from robust-predicates, whose sign is the opposite of Shewchuk's.
fn orient2d(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> f64 {
    -robust::orient2d(robust::Coord { x: ax, y: ay }, robust::Coord { x: bx, y: by }, robust::Coord { x: cx, y: cy })
}

fn dist(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    let (dx, dy) = (ax - bx, ay - by);
    dx * dx + dy * dy
}

fn pseudo_angle(dx: f64, dy: f64) -> f64 {
    let p = dx / (dx.abs() + dy.abs());
    (if dy > 0.0 { 3.0 - p } else { 1.0 + p }) / 4.0
}

#[allow(clippy::too_many_arguments)]
fn in_circle(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64, px: f64, py: f64) -> bool {
    let (dx, dy) = (ax - px, ay - py);
    let (ex, ey) = (bx - px, by - py);
    let (fx, fy) = (cx - px, cy - py);
    let ap = dx * dx + dy * dy;
    let bp = ex * ex + ey * ey;
    let cp = fx * fx + fy * fy;
    dx * (ey * cp - bp * fy) - dy * (ex * cp - bp * fx) + ap * (ex * fy - ey * fx) < 0.0
}

fn circumradius(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> f64 {
    let (dx, dy) = (bx - ax, by - ay);
    let (ex, ey) = (cx - ax, cy - ay);
    let bl = dx * dx + dy * dy;
    let cl = ex * ex + ey * ey;
    let d = 0.5 / (dx * ey - dy * ex);
    let x = (ey * bl - dy * cl) * d;
    let y = (dx * cl - ex * bl) * d;
    x * x + y * y
}

fn circumcenter(ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64) -> (f64, f64) {
    let (dx, dy) = (bx - ax, by - ay);
    let (ex, ey) = (cx - ax, cy - ay);
    let bl = dx * dx + dy * dy;
    let cl = ex * ex + ey * ey;
    let d = 0.5 / (dx * ey - dy * ex);
    (ax + (ey * bl - dy * cl) * d, ay + (dx * cl - ex * bl) * d)
}

fn quicksort(ids: &mut [usize], dists: &[f64], left: isize, right: isize) {
    if right - left <= 20 {
        for i in (left + 1)..=right {
            let temp = ids[i as usize];
            let temp_dist = dists[temp];
            let mut j = i - 1;
            while j >= left && dists[ids[j as usize]] > temp_dist {
                ids[(j + 1) as usize] = ids[j as usize];
                j -= 1;
            }
            ids[(j + 1) as usize] = temp;
        }
    } else {
        let median = (left + right) >> 1;
        let mut i = left + 1;
        let mut j = right;
        ids.swap(median as usize, i as usize);
        if dists[ids[left as usize]] > dists[ids[right as usize]] {
            ids.swap(left as usize, right as usize);
        }
        if dists[ids[i as usize]] > dists[ids[right as usize]] {
            ids.swap(i as usize, right as usize);
        }
        if dists[ids[left as usize]] > dists[ids[i as usize]] {
            ids.swap(left as usize, i as usize);
        }
        let temp = ids[i as usize];
        let temp_dist = dists[temp];
        loop {
            loop {
                i += 1;
                if dists[ids[i as usize]] >= temp_dist {
                    break;
                }
            }
            loop {
                j -= 1;
                if dists[ids[j as usize]] <= temp_dist {
                    break;
                }
            }
            if j < i {
                break;
            }
            ids.swap(i as usize, j as usize);
        }
        ids[(left + 1) as usize] = ids[j as usize];
        ids[j as usize] = temp;
        if right - i + 1 >= j - left {
            quicksort(ids, dists, i, right);
            quicksort(ids, dists, left, j - 1);
        } else {
            quicksort(ids, dists, left, j - 1);
            quicksort(ids, dists, i, right);
        }
    }
}

struct Delaunator {
    triangles: Vec<i64>,
    halfedges: Vec<i64>,
    hull: Vec<i64>,
}

impl Delaunator {
    fn new(coords: &[f64]) -> Delaunator {
        let n = coords.len() >> 1;
        let max_triangles = (2 * n as isize - 5).max(0) as usize;
        let mut b = Builder {
            coords,
            triangles: vec![0; max_triangles * 3],
            halfedges: vec![0; max_triangles * 3],
            hash_size: (n as f64).sqrt().ceil() as usize,
            hull_prev: vec![0; n],
            hull_next: vec![0; n],
            hull_tri: vec![0; n],
            hull_hash: vec![-1; (n as f64).sqrt().ceil() as usize],
            triangles_len: 0,
            cx: 0.0,
            cy: 0.0,
            hull_start: 0,
        };
        b.run()
    }
}

struct Builder<'a> {
    coords: &'a [f64],
    triangles: Vec<i64>,
    halfedges: Vec<i64>,
    hash_size: usize,
    hull_prev: Vec<usize>,
    hull_next: Vec<usize>,
    hull_tri: Vec<i64>,
    hull_hash: Vec<i64>,
    triangles_len: usize,
    cx: f64,
    cy: f64,
    hull_start: usize,
}

impl Builder<'_> {
    fn x(&self, i: usize) -> f64 {
        self.coords[2 * i]
    }
    fn y(&self, i: usize) -> f64 {
        self.coords[2 * i + 1]
    }

    fn hash_key(&self, x: f64, y: f64) -> usize {
        ((pseudo_angle(x - self.cx, y - self.cy) * self.hash_size as f64).floor() as usize) % self.hash_size
    }

    fn link(&mut self, a: usize, b: i64) {
        self.halfedges[a] = b;
        if b != -1 {
            self.halfedges[b as usize] = a as i64;
        }
    }

    fn add_triangle(&mut self, i0: usize, i1: usize, i2: usize, a: i64, b: i64, c: i64) -> usize {
        let t = self.triangles_len;
        self.triangles[t] = i0 as i64;
        self.triangles[t + 1] = i1 as i64;
        self.triangles[t + 2] = i2 as i64;
        self.link(t, a);
        self.link(t + 1, b);
        self.link(t + 2, c);
        self.triangles_len += 3;
        t
    }

    fn legalize(&mut self, mut a: usize) -> usize {
        let mut stack: Vec<usize> = Vec::new();
        let mut ar;
        loop {
            let b = self.halfedges[a];
            let a0 = a - a % 3;
            ar = a0 + (a + 2) % 3;
            if b == -1 {
                match stack.pop() {
                    None => break,
                    Some(next) => {
                        a = next;
                        continue;
                    }
                }
            }
            let b = b as usize;
            let b0 = b - b % 3;
            let al = a0 + (a + 1) % 3;
            let bl = b0 + (b + 2) % 3;
            let p0 = self.triangles[ar] as usize;
            let pr = self.triangles[a] as usize;
            let pl = self.triangles[al] as usize;
            let p1 = self.triangles[bl] as usize;
            let illegal = in_circle(
                self.x(p0),
                self.y(p0),
                self.x(pr),
                self.y(pr),
                self.x(pl),
                self.y(pl),
                self.x(p1),
                self.y(p1),
            );
            if illegal {
                self.triangles[a] = p1 as i64;
                self.triangles[b] = p0 as i64;
                let hbl = self.halfedges[bl];
                // Edge swapped on the other side of the hull (rare); fix the half-edge reference.
                if hbl == -1 {
                    let mut e = self.hull_start;
                    loop {
                        if self.hull_tri[e] == bl as i64 {
                            self.hull_tri[e] = a as i64;
                            break;
                        }
                        e = self.hull_prev[e];
                        if e == self.hull_start {
                            break;
                        }
                    }
                }
                self.link(a, hbl);
                let har = self.halfedges[ar];
                self.link(b, har);
                self.link(ar, bl as i64);
                let br = b0 + (b + 1) % 3;
                if stack.len() < 512 {
                    stack.push(br);
                }
            } else {
                match stack.pop() {
                    None => break,
                    Some(next) => a = next,
                }
            }
        }
        ar
    }

    fn run(&mut self) -> Delaunator {
        let n = self.coords.len() >> 1;
        let mut ids: Vec<usize> = (0..n).collect();
        let mut dists = vec![0.0; n];
        let (mut min_x, mut min_y, mut max_x, mut max_y) =
            (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for i in 0..n {
            let (x, y) = (self.x(i), self.y(i));
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
        let cx = (min_x + max_x) / 2.0;
        let cy = (min_y + max_y) / 2.0;
        let (mut i0, mut i1, mut i2) = (0, 0, 0);
        let mut min_dist = f64::INFINITY;
        for i in 0..n {
            let d = dist(cx, cy, self.x(i), self.y(i));
            if d < min_dist {
                i0 = i;
                min_dist = d;
            }
        }
        let (i0x, i0y) = (self.x(i0), self.y(i0));
        min_dist = f64::INFINITY;
        for i in 0..n {
            if i == i0 {
                continue;
            }
            let d = dist(i0x, i0y, self.x(i), self.y(i));
            if d < min_dist && d > 0.0 {
                i1 = i;
                min_dist = d;
            }
        }
        let (mut i1x, mut i1y) = (self.x(i1), self.y(i1));
        let mut min_radius = f64::INFINITY;
        for i in 0..n {
            if i == i0 || i == i1 {
                continue;
            }
            let r = circumradius(i0x, i0y, i1x, i1y, self.x(i), self.y(i));
            if r < min_radius {
                i2 = i;
                min_radius = r;
            }
        }
        let (mut i2x, mut i2y) = (self.x(i2), self.y(i2));

        if min_radius == f64::INFINITY {
            // Collinear points: order them by dx (or dy if all x are identical) and return them as a hull.
            for (i, d) in dists.iter_mut().enumerate() {
                let dx = self.x(i) - self.coords[0];
                *d = if dx != 0.0 && !dx.is_nan() { dx } else { self.y(i) - self.coords[1] };
            }
            if n > 0 {
                quicksort(&mut ids, &dists, 0, n as isize - 1);
            }
            let mut hull = Vec::new();
            let mut d0 = f64::NEG_INFINITY;
            for &id in &ids {
                if dists[id] > d0 {
                    hull.push(id as i64);
                    d0 = dists[id];
                }
            }
            return Delaunator { triangles: Vec::new(), halfedges: Vec::new(), hull };
        }

        if orient2d(i0x, i0y, i1x, i1y, i2x, i2y) < 0.0 {
            std::mem::swap(&mut i1, &mut i2);
            std::mem::swap(&mut i1x, &mut i2x);
            std::mem::swap(&mut i1y, &mut i2y);
        }
        let (ccx, ccy) = circumcenter(i0x, i0y, i1x, i1y, i2x, i2y);
        self.cx = ccx;
        self.cy = ccy;
        for (i, d) in dists.iter_mut().enumerate() {
            *d = dist(self.x(i), self.y(i), ccx, ccy);
        }
        quicksort(&mut ids, &dists, 0, n as isize - 1);

        self.hull_start = i0;
        let mut hull_size = 3;
        self.hull_next[i0] = i1;
        self.hull_prev[i2] = i1;
        self.hull_next[i1] = i2;
        self.hull_prev[i0] = i2;
        self.hull_next[i2] = i0;
        self.hull_prev[i1] = i0;
        self.hull_tri[i0] = 0;
        self.hull_tri[i1] = 1;
        self.hull_tri[i2] = 2;
        self.hull_hash.fill(-1);
        let k0 = self.hash_key(i0x, i0y);
        self.hull_hash[k0] = i0 as i64;
        let k1 = self.hash_key(i1x, i1y);
        self.hull_hash[k1] = i1 as i64;
        let k2 = self.hash_key(i2x, i2y);
        self.hull_hash[k2] = i2 as i64;
        self.triangles_len = 0;
        self.add_triangle(i0, i1, i2, -1, -1, -1);

        let (mut xp, mut yp) = (0.0, 0.0);
        for (k, &i) in ids.iter().enumerate() {
            let (x, y) = (self.x(i), self.y(i));
            // Skip near-duplicate points.
            if k > 0 && (x - xp).abs() <= EPSILON && (y - yp).abs() <= EPSILON {
                continue;
            }
            xp = x;
            yp = y;
            if i == i0 || i == i1 || i == i2 {
                continue;
            }
            // Find a visible edge on the convex hull using the edge hash.
            let mut start = 0usize;
            let key = self.hash_key(x, y);
            for j in 0..self.hash_size {
                let s = self.hull_hash[(key + j) % self.hash_size];
                start = s.max(0) as usize;
                if s != -1 && s as usize != self.hull_next[s as usize] {
                    break;
                }
            }
            start = self.hull_prev[start];
            let mut e = start as i64;
            loop {
                let q = self.hull_next[e as usize];
                if orient2d(x, y, self.x(e as usize), self.y(e as usize), self.x(q), self.y(q)) < 0.0 {
                    break;
                }
                e = q as i64;
                if e as usize == start {
                    e = -1;
                    break;
                }
            }
            if e == -1 {
                continue;
            }
            let mut e = e as usize;
            let next = self.hull_next[e];
            let mut t = self.add_triangle(e, i, next, -1, -1, self.hull_tri[e]);
            self.hull_tri[i] = self.legalize(t + 2) as i64;
            self.hull_tri[e] = t as i64;
            hull_size += 1;

            let mut nn = self.hull_next[e];
            loop {
                let q = self.hull_next[nn];
                if orient2d(x, y, self.x(nn), self.y(nn), self.x(q), self.y(q)) >= 0.0 {
                    break;
                }
                t = self.add_triangle(nn, i, q, self.hull_tri[i], -1, self.hull_tri[nn]);
                self.hull_tri[i] = self.legalize(t + 2) as i64;
                self.hull_next[nn] = nn;
                hull_size -= 1;
                nn = q;
            }
            if e == start {
                loop {
                    let q = self.hull_prev[e];
                    if orient2d(x, y, self.x(q), self.y(q), self.x(e), self.y(e)) >= 0.0 {
                        break;
                    }
                    t = self.add_triangle(q, i, e, -1, self.hull_tri[e], self.hull_tri[q]);
                    self.legalize(t + 2);
                    self.hull_tri[q] = t as i64;
                    self.hull_next[e] = e;
                    hull_size -= 1;
                    e = q;
                }
            }
            self.hull_start = e;
            self.hull_prev[i] = e;
            self.hull_next[e] = i;
            self.hull_prev[nn] = i;
            self.hull_next[i] = nn;
            let ki = self.hash_key(x, y);
            self.hull_hash[ki] = i as i64;
            let ke = self.hash_key(self.x(e), self.y(e));
            self.hull_hash[ke] = e as i64;
        }

        let mut hull = Vec::with_capacity(hull_size);
        let mut e = self.hull_start;
        for _ in 0..hull_size {
            hull.push(e as i64);
            e = self.hull_next[e];
        }
        self.triangles.truncate(self.triangles_len);
        self.halfedges.truncate(self.triangles_len);
        Delaunator {
            triangles: std::mem::take(&mut self.triangles),
            halfedges: std::mem::take(&mut self.halfedges),
            hull,
        }
    }
}

/// A Delaunay triangulation of points, with the neighbour and nearest-point queries d3-delaunay offers.
pub struct Delaunay {
    pub points: Vec<f64>,
    halfedges: Vec<i64>,
    hull: Vec<i64>,
    triangles: Vec<i64>,
    inedges: Vec<i64>,
    hull_index: Vec<i64>,
    collinear: Option<Vec<usize>>,
}

fn next_halfedge(e: i64) -> i64 {
    if e % 3 == 2 { e - 2 } else { e + 1 }
}

impl Delaunay {
    pub fn from(points: &[[f64; 2]]) -> Delaunay {
        let mut coords: Vec<f64> = points.iter().flat_map(|p| [p[0], p[1]]).collect();
        let n = points.len();
        let mut d = Delaunator::new(&coords);
        let mut collinear = None;
        if d.hull.len() > 2 && is_collinear(&d, &coords) {
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by(|&i, &j| {
                (coords[2 * i] - coords[2 * j])
                    .partial_cmp(&0.0)
                    .unwrap()
                    .then((coords[2 * i + 1] - coords[2 * j + 1]).partial_cmp(&0.0).unwrap())
            });
            let (e, f) = (order[0], order[order.len() - 1]);
            let r = 1e-8 * (coords[2 * f + 1] - coords[2 * e + 1]).hypot(coords[2 * f] - coords[2 * e]);
            for i in 0..n {
                let (x, y) = (coords[2 * i], coords[2 * i + 1]);
                coords[2 * i] = x + (x + y).sin() * r;
                coords[2 * i + 1] = y + (x - y).cos() * r;
            }
            d = Delaunator::new(&coords);
            collinear = Some(order);
        }
        let mut inedges = vec![-1i64; n];
        let mut hull_index = vec![-1i64; n];
        for e in 0..d.halfedges.len() {
            let p = d.triangles[if e % 3 == 2 { e - 2 } else { e + 1 }] as usize;
            if d.halfedges[e] == -1 || inedges[p] == -1 {
                inedges[p] = e as i64;
            }
        }
        for (i, &h) in d.hull.iter().enumerate() {
            hull_index[h as usize] = i as i64;
        }
        let (mut triangles, mut halfedges) = (d.triangles, d.halfedges);
        // Degenerate case: one or two distinct points.
        if !d.hull.is_empty() && d.hull.len() <= 2 {
            triangles = vec![-1; 3];
            halfedges = vec![-1; 3];
            triangles[0] = d.hull[0];
            inedges[d.hull[0] as usize] = 1;
            if d.hull.len() == 2 {
                inedges[d.hull[1] as usize] = 0;
                triangles[1] = d.hull[1];
                triangles[2] = d.hull[1];
            }
        }
        Delaunay { points: coords, halfedges, hull: d.hull, triangles, inedges, hull_index, collinear }
    }

    /// Points sharing an edge with point `i`.
    pub fn neighbors(&self, i: usize) -> Vec<usize> {
        let mut out = Vec::new();
        if let Some(col) = &self.collinear {
            if let Some(l) = col.iter().position(|&c| c == i) {
                if l > 0 {
                    out.push(col[l - 1]);
                }
                if l + 1 < col.len() {
                    out.push(col[l + 1]);
                }
            }
            return out;
        }
        let e0 = self.inedges[i];
        if e0 == -1 {
            return out;
        }
        let mut e = e0;
        loop {
            let p0 = self.triangles[e as usize];
            out.push(p0 as usize);
            e = next_halfedge(e);
            if self.triangles[e as usize] != i as i64 {
                return out;
            }
            e = self.halfedges[e as usize];
            if e == -1 {
                let p = self.hull[((self.hull_index[i] + 1) as usize) % self.hull.len()];
                if p != p0 {
                    out.push(p as usize);
                }
                return out;
            }
            if e == e0 {
                return out;
            }
        }
    }

    /// The point nearest to (x, y), walking the triangulation from point `i`.
    pub fn find(&self, x: f64, y: f64, mut i: usize) -> Option<usize> {
        if x.is_nan() || y.is_nan() {
            return None;
        }
        let i0 = i;
        loop {
            let c = self.step(i, x, y);
            if c < 0 {
                return None;
            }
            let c = c as usize;
            if c == i || c == i0 {
                return Some(c);
            }
            i = c;
        }
    }

    fn step(&self, i: usize, x: f64, y: f64) -> i64 {
        let n = self.points.len() >> 1;
        if self.inedges[i] == -1 || self.points.is_empty() {
            return ((i + 1) % n) as i64;
        }
        let p = &self.points;
        let mut c = i as i64;
        let mut dc = (x - p[i * 2]).powi(2) + (y - p[i * 2 + 1]).powi(2);
        let e0 = self.inedges[i];
        let mut e = e0;
        loop {
            let t = self.triangles[e as usize];
            let dt = (x - p[t as usize * 2]).powi(2) + (y - p[t as usize * 2 + 1]).powi(2);
            if dt < dc {
                dc = dt;
                c = t;
            }
            e = next_halfedge(e);
            if self.triangles[e as usize] != i as i64 {
                break;
            }
            e = self.halfedges[e as usize];
            if e == -1 {
                let h = self.hull[((self.hull_index[i] + 1) as usize) % self.hull.len()];
                if h != t && (x - p[h as usize * 2]).powi(2) + (y - p[h as usize * 2 + 1]).powi(2) < dc {
                    return h;
                }
                break;
            }
            if e == e0 {
                break;
            }
        }
        c
    }

    pub fn voronoi(&self, bounds: [f64; 4]) -> Voronoi<'_> {
        Voronoi::new(self, bounds)
    }
}

fn is_collinear(d: &Delaunator, coords: &[f64]) -> bool {
    d.triangles.chunks(3).all(|t| {
        let (a, b, c) = (2 * t[0] as usize, 2 * t[1] as usize, 2 * t[2] as usize);
        let cross = (coords[c] - coords[a]) * (coords[b + 1] - coords[a + 1])
            - (coords[b] - coords[a]) * (coords[c + 1] - coords[a + 1]);
        cross <= 1e-10
    })
}

pub struct Voronoi<'a> {
    delaunay: &'a Delaunay,
    circumcenters: Vec<f64>,
    vectors: Vec<f64>,
    xmin: f64,
    ymin: f64,
    xmax: f64,
    ymax: f64,
}

impl<'a> Voronoi<'a> {
    fn new(delaunay: &'a Delaunay, [xmin, ymin, xmax, ymax]: [f64; 4]) -> Voronoi<'a> {
        let points = &delaunay.points;
        let triangles = &delaunay.triangles;
        let hull = &delaunay.hull;
        let mut circumcenters = vec![0.0; triangles.len() / 3 * 2];
        let mut barycenter: Option<(f64, f64)> = None;
        for (j, t) in triangles.chunks(3).enumerate() {
            if t.len() < 3 || t.iter().any(|v| *v < 0) {
                continue;
            }
            let (t1, t2, t3) = (t[0] as usize * 2, t[1] as usize * 2, t[2] as usize * 2);
            let (x1, y1) = (points[t1], points[t1 + 1]);
            let (x2, y2) = (points[t2], points[t2 + 1]);
            let (x3, y3) = (points[t3], points[t3 + 1]);
            let (dx, dy) = (x2 - x1, y2 - y1);
            let (ex, ey) = (x3 - x1, y3 - y1);
            let ab = (dx * ey - dy * ex) * 2.0;
            let (x, y) = if ab.abs() < 1e-9 {
                // A degenerate triangle's circumcenter is at infinity, orthogonal to the half-edge and away
                // from the hull's barycenter.
                let (bx, by) = *barycenter.get_or_insert_with(|| {
                    let (sx, sy) = hull.iter().fold((0.0, 0.0), |(sx, sy), &i| {
                        (sx + points[i as usize * 2], sy + points[i as usize * 2 + 1])
                    });
                    (sx / hull.len() as f64, sy / hull.len() as f64)
                });
                let a = 1e9 * js_sign((bx - x1) * ey - (by - y1) * ex);
                ((x1 + x3) / 2.0 - a * ey, (y1 + y3) / 2.0 + a * ex)
            } else {
                let d = 1.0 / ab;
                let bl = dx * dx + dy * dy;
                let cl = ex * ex + ey * ey;
                (x1 + (ey * bl - dy * cl) * d, y1 + (dx * cl - ex * bl) * d)
            };
            circumcenters[j * 2] = x;
            circumcenters[j * 2 + 1] = y;
        }
        let mut vectors = vec![0.0; points.len() * 2];
        if let Some(&last) = hull.last() {
            let mut h = last as usize;
            let mut p1 = h * 4;
            let (mut x1, mut y1) = (points[2 * h], points[2 * h + 1]);
            for &hi in hull {
                h = hi as usize;
                let (p0, x0, y0) = (p1, x1, y1);
                p1 = h * 4;
                x1 = points[2 * h];
                y1 = points[2 * h + 1];
                vectors[p0 + 2] = y0 - y1;
                vectors[p1] = y0 - y1;
                vectors[p0 + 3] = x1 - x0;
                vectors[p1 + 1] = x1 - x0;
            }
        }
        Voronoi { delaunay, circumcenters, vectors, xmin, ymin, xmax, ymax }
    }

    /// The clipped cell of point `i` as a closed ring (first point repeated last), or None.
    pub fn cell_polygon(&self, i: usize) -> Option<Vec<[f64; 2]>> {
        let points = self.clip(i)?;
        if points.is_empty() {
            return None;
        }
        let mut out = vec![[points[0], points[1]]];
        let mut n = points.len();
        while n > 1 && points[0] == points[n - 2] && points[1] == points[n - 1] {
            n -= 2;
        }
        let mut k = 2;
        while k < n {
            if points[k] != points[k - 2] || points[k + 1] != points[k - 1] {
                out.push([points[k], points[k + 1]]);
            }
            k += 2;
        }
        out.push(out[0]);
        Some(out)
    }

    fn cell(&self, i: usize) -> Option<Vec<f64>> {
        let d = self.delaunay;
        let e0 = d.inedges[i];
        if e0 == -1 {
            return None;
        }
        let mut points = Vec::new();
        let mut e = e0;
        loop {
            let t = (e / 3) as usize;
            points.push(self.circumcenters[t * 2]);
            points.push(self.circumcenters[t * 2 + 1]);
            e = next_halfedge(e);
            if d.triangles[e as usize] != i as i64 {
                break;
            }
            e = d.halfedges[e as usize];
            if e == e0 || e == -1 {
                break;
            }
        }
        Some(points)
    }

    fn box_ring(&self) -> Vec<f64> {
        vec![self.xmax, self.ymin, self.xmax, self.ymax, self.xmin, self.ymax, self.xmin, self.ymin]
    }

    fn clip(&self, i: usize) -> Option<Vec<f64>> {
        if i == 0 && self.delaunay.hull.len() == 1 {
            return Some(self.box_ring());
        }
        let points = self.cell(i)?;
        let v = i * 4;
        let vv = &self.vectors;
        let clipped = if vv[v] != 0.0 || vv[v + 1] != 0.0 {
            self.clip_infinite(i, points, vv[v], vv[v + 1], vv[v + 2], vv[v + 3])
        } else {
            self.clip_finite(i, &points)
        };
        simplify(clipped)
    }

    fn contains(&self, i: usize, x: f64, y: f64) -> bool {
        if x.is_nan() || y.is_nan() {
            return false;
        }
        self.delaunay.step(i, x, y) == i as i64
    }

    fn clip_finite(&self, i: usize, points: &[f64]) -> Option<Vec<f64>> {
        let n = points.len();
        let mut p: Option<Vec<f64>> = None;
        let (mut x1, mut y1) = (points[n - 2], points[n - 1]);
        let mut c1 = self.regioncode(x1, y1);
        let mut e1 = 0u8;
        let mut j = 0;
        while j < n {
            let (x0, y0) = (x1, y1);
            x1 = points[j];
            y1 = points[j + 1];
            let c0 = c1;
            c1 = self.regioncode(x1, y1);
            if c0 == 0 && c1 == 0 {
                e1 = 0;
                p.get_or_insert_with(Vec::new).extend([x1, y1]);
            } else {
                let (sx0, sy0, sx1, sy1);
                if c0 == 0 {
                    let Some(s) = self.clip_segment(x0, y0, x1, y1, c0, c1) else {
                        j += 2;
                        continue;
                    };
                    (sx0, sy0, sx1, sy1) = (s[0], s[1], s[2], s[3]);
                    let _ = (sx0, sy0);
                } else {
                    let Some(s) = self.clip_segment(x1, y1, x0, y0, c1, c0) else {
                        j += 2;
                        continue;
                    };
                    (sx1, sy1, sx0, sy0) = (s[0], s[1], s[2], s[3]);
                    let e0 = e1;
                    e1 = self.edgecode(sx0, sy0);
                    if e0 != 0 && e1 != 0 {
                        let len = p.as_ref().map_or(0, Vec::len);
                        self.edge(i, e0, e1, p.get_or_insert_with(Vec::new), len);
                    }
                    p.get_or_insert_with(Vec::new).extend([sx0, sy0]);
                }
                let e0 = e1;
                e1 = self.edgecode(sx1, sy1);
                if e0 != 0 && e1 != 0 {
                    let len = p.as_ref().map_or(0, Vec::len);
                    self.edge(i, e0, e1, p.get_or_insert_with(Vec::new), len);
                }
                p.get_or_insert_with(Vec::new).extend([sx1, sy1]);
            }
            j += 2;
        }
        match p {
            Some(mut pp) => {
                let e0 = e1;
                e1 = self.edgecode(pp[0], pp[1]);
                if e0 != 0 && e1 != 0 {
                    let len = pp.len();
                    self.edge(i, e0, e1, &mut pp, len);
                }
                Some(pp)
            }
            None if self.contains(i, (self.xmin + self.xmax) / 2.0, (self.ymin + self.ymax) / 2.0) => {
                Some(self.box_ring())
            }
            None => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn clip_segment(
        &self,
        mut x0: f64,
        mut y0: f64,
        mut x1: f64,
        mut y1: f64,
        mut c0: u8,
        mut c1: u8,
    ) -> Option<[f64; 4]> {
        let flip = c0 < c1;
        if flip {
            (x0, y0, x1, y1, c0, c1) = (x1, y1, x0, y0, c1, c0);
        }
        loop {
            if c0 == 0 && c1 == 0 {
                return Some(if flip { [x1, y1, x0, y0] } else { [x0, y0, x1, y1] });
            }
            if c0 & c1 != 0 {
                return None;
            }
            let c = if c0 != 0 { c0 } else { c1 };
            let (x, y);
            if c & 0b1000 != 0 {
                x = x0 + (x1 - x0) * (self.ymax - y0) / (y1 - y0);
                y = self.ymax;
            } else if c & 0b0100 != 0 {
                x = x0 + (x1 - x0) * (self.ymin - y0) / (y1 - y0);
                y = self.ymin;
            } else if c & 0b0010 != 0 {
                y = y0 + (y1 - y0) * (self.xmax - x0) / (x1 - x0);
                x = self.xmax;
            } else {
                y = y0 + (y1 - y0) * (self.xmin - x0) / (x1 - x0);
                x = self.xmin;
            }
            if c0 != 0 {
                x0 = x;
                y0 = y;
                c0 = self.regioncode(x0, y0);
            } else {
                x1 = x;
                y1 = y;
                c1 = self.regioncode(x1, y1);
            }
        }
    }

    fn clip_infinite(&self, i: usize, points: Vec<f64>, vx0: f64, vy0: f64, vxn: f64, vyn: f64) -> Option<Vec<f64>> {
        let mut pts = points;
        if let Some(p) = self.project(pts[0], pts[1], vx0, vy0) {
            pts.splice(0..0, p);
        }
        let n = pts.len();
        if let Some(p) = self.project(pts[n - 2], pts[n - 1], vxn, vyn) {
            pts.extend(p);
        }
        match self.clip_finite(i, &pts) {
            Some(mut p) => {
                let mut n = p.len();
                let mut c1 = self.edgecode(p[n - 2], p[n - 1]);
                let mut j = 0;
                while j < n {
                    let c0 = c1;
                    c1 = self.edgecode(p[j], p[j + 1]);
                    if c0 != 0 && c1 != 0 {
                        j = self.edge(i, c0, c1, &mut p, j);
                        n = p.len();
                    }
                    j += 2;
                }
                Some(p)
            }
            None if self.contains(i, (self.xmin + self.xmax) / 2.0, (self.ymin + self.ymax) / 2.0) => {
                Some(vec![self.xmin, self.ymin, self.xmax, self.ymin, self.xmax, self.ymax, self.xmin, self.ymax])
            }
            None => None,
        }
    }

    fn edge(&self, i: usize, mut e0: u8, e1: u8, p: &mut Vec<f64>, mut j: usize) -> usize {
        while e0 != e1 {
            let (x, y);
            match e0 {
                0b0101 => {
                    e0 = 0b0100;
                    continue;
                }
                0b0100 => {
                    e0 = 0b0110;
                    x = self.xmax;
                    y = self.ymin;
                }
                0b0110 => {
                    e0 = 0b0010;
                    continue;
                }
                0b0010 => {
                    e0 = 0b1010;
                    x = self.xmax;
                    y = self.ymax;
                }
                0b1010 => {
                    e0 = 0b1000;
                    continue;
                }
                0b1000 => {
                    e0 = 0b1001;
                    x = self.xmin;
                    y = self.ymax;
                }
                0b1001 => {
                    e0 = 0b0001;
                    continue;
                }
                0b0001 => {
                    e0 = 0b0101;
                    x = self.xmin;
                    y = self.ymin;
                }
                _ => return j,
            }
            let differs = p.get(j) != Some(&x) || p.get(j + 1) != Some(&y);
            if differs && self.contains(i, x, y) {
                p.splice(j..j, [x, y]);
                j += 2;
            }
        }
        j
    }

    fn project(&self, x0: f64, y0: f64, vx: f64, vy: f64) -> Option<[f64; 2]> {
        let mut t = f64::INFINITY;
        let (mut x, mut y) = (f64::NAN, f64::NAN);
        if vy < 0.0 {
            if y0 <= self.ymin {
                return None;
            }
            let c = (self.ymin - y0) / vy;
            if c < t {
                y = self.ymin;
                t = c;
                x = x0 + t * vx;
            }
        } else if vy > 0.0 {
            if y0 >= self.ymax {
                return None;
            }
            let c = (self.ymax - y0) / vy;
            if c < t {
                y = self.ymax;
                t = c;
                x = x0 + t * vx;
            }
        }
        if vx > 0.0 {
            if x0 >= self.xmax {
                return None;
            }
            let c = (self.xmax - x0) / vx;
            if c < t {
                x = self.xmax;
                t = c;
                y = y0 + t * vy;
            }
        } else if vx < 0.0 {
            if x0 <= self.xmin {
                return None;
            }
            let c = (self.xmin - x0) / vx;
            if c < t {
                x = self.xmin;
                t = c;
                y = y0 + t * vy;
            }
        }
        let _ = t;
        Some([x, y])
    }

    fn edgecode(&self, x: f64, y: f64) -> u8 {
        (if x == self.xmin {
            0b0001
        } else if x == self.xmax {
            0b0010
        } else {
            0
        }) | (if y == self.ymin {
            0b0100
        } else if y == self.ymax {
            0b1000
        } else {
            0
        })
    }

    fn regioncode(&self, x: f64, y: f64) -> u8 {
        (if x < self.xmin {
            0b0001
        } else if x > self.xmax {
            0b0010
        } else {
            0
        }) | (if y < self.ymin {
            0b0100
        } else if y > self.ymax {
            0b1000
        } else {
            0
        })
    }
}

fn js_sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        x
    }
}

fn simplify(p: Option<Vec<f64>>) -> Option<Vec<f64>> {
    let mut p = p?;
    if p.len() > 4 {
        let mut i: isize = 0;
        while (i as usize) < p.len() {
            let len = p.len();
            let iu = i as usize;
            let j = (iu + 2) % len;
            let k = (iu + 4) % len;
            if (p[iu] == p[j] && p[j] == p[k]) || (p[iu + 1] == p[j + 1] && p[j + 1] == p[k + 1]) {
                p.drain(j..j + 2);
                i -= 2;
            }
            i += 2;
        }
        if p.is_empty() {
            return None;
        }
    }
    Some(p)
}
