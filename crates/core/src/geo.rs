//! An orthographic globe as d3-geo 3 draws it: TopoJSON decoding, rotation, clipping at the horizon
//! (cutting and rejoining polygons along the limb), adaptive resampling and SVG path output.

use crate::format::{js_number, js_round};
use serde::Deserialize;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

const EPSILON: f64 = 1e-6;
const EPSILON2: f64 = 1e-12;
const RADIANS: f64 = PI / 180.0;

/// Polygons of rings of [longitude, latitude] in degrees, rings closed (last point repeats the first).
pub type MultiPolygon = Vec<Vec<Vec<[f64; 2]>>>;

#[derive(Deserialize)]
struct Topology {
    transform: Option<TopoTransform>,
    arcs: Vec<Vec<Vec<f64>>>,
    objects: std::collections::HashMap<String, TopoObject>,
}

#[derive(Deserialize)]
struct TopoTransform {
    scale: [f64; 2],
    translate: [f64; 2],
}

#[derive(Deserialize)]
struct TopoObject {
    #[serde(default)]
    geometries: Vec<TopoGeometry>,
}

#[derive(Deserialize)]
struct TopoGeometry {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    arcs: serde_json::Value,
}

/// The polygons of a TopoJSON object, as topojson-client's `feature` decodes them.
pub fn topo_polygons(json: &str, object: &str) -> Result<MultiPolygon, String> {
    let topo: Topology = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let decode = |arc: &[Vec<f64>]| -> Vec<[f64; 2]> {
        let (mut x, mut y) = (0.0, 0.0);
        arc.iter()
            .map(|p| match &topo.transform {
                Some(t) => {
                    x += p[0];
                    y += p[1];
                    [x * t.scale[0] + t.translate[0], y * t.scale[1] + t.translate[1]]
                }
                None => [p[0], p[1]],
            })
            .collect()
    };
    let arcs: Vec<Vec<[f64; 2]>> = topo.arcs.iter().map(|a| decode(a)).collect();
    let ring = |ids: &[i64]| -> Vec<[f64; 2]> {
        let mut points: Vec<[f64; 2]> = Vec::new();
        for &i in ids {
            points.pop();
            let a = &arcs[if i < 0 { !i } else { i } as usize];
            if i < 0 {
                points.extend(a.iter().rev());
            } else {
                points.extend(a.iter());
            }
        }
        if points.len() < 2 {
            points.push(points[0]);
        }
        while points.len() < 4 {
            points.push(points[0]);
        }
        points
    };
    let parse_ids = |v: &serde_json::Value| -> Vec<i64> {
        v.as_array().map(|a| a.iter().filter_map(|x| x.as_i64()).collect()).unwrap_or_default()
    };
    let obj = topo.objects.get(object).ok_or("no such object")?;
    let mut out = Vec::new();
    for g in &obj.geometries {
        let polys: Vec<&serde_json::Value> = match g.kind.as_str() {
            "Polygon" => vec![&g.arcs],
            "MultiPolygon" => g.arcs.as_array().map(|a| a.iter().collect()).unwrap_or_default(),
            _ => continue,
        };
        for p in polys {
            out.push(p.as_array().map(|rings| rings.iter().map(|r| ring(&parse_ids(r))).collect()).unwrap_or_default());
        }
    }
    Ok(out)
}

/// d3's geoGraticule10: meridians and parallels every 10°, as lines of [longitude, latitude].
pub fn graticule10() -> Vec<Vec<[f64; 2]>> {
    fn range(start: f64, stop: f64, step: f64) -> Vec<f64> {
        let n = ((stop - start) / step).ceil().max(0.0) as usize;
        (0..n).map(|i| start + i as f64 * step).collect()
    }
    let (x0, x1, y0, y1) = (-180.0, 180.0, -80.0 - EPSILON, 80.0 + EPSILON);
    let (big_x0, big_x1, big_y0, big_y1) = (-180.0, 180.0, -90.0 + EPSILON, 90.0 - EPSILON);
    let (dx, dy, big_dx, big_dy, precision) = (10.0, 10.0, 90.0, 360.0, 2.5);
    let meridian = |ya: f64, yb: f64| {
        let mut ys = range(ya, yb - EPSILON, 90.0);
        ys.push(yb);
        move |x: f64| ys.iter().map(|y| [x, *y]).collect::<Vec<_>>()
    };
    let parallel = |xa: f64, xb: f64| {
        let mut xs = range(xa, xb - EPSILON, precision);
        xs.push(xb);
        move |y: f64| xs.iter().map(|x| [*x, y]).collect::<Vec<_>>()
    };
    let (minor_x, minor_y, major_x, major_y) =
        (meridian(y0, y1), parallel(x0, x1), meridian(big_y0, big_y1), parallel(big_x0, big_x1));
    let mut lines: Vec<Vec<[f64; 2]>> = Vec::new();
    lines.extend(range((big_x0 / big_dx).ceil() * big_dx, big_x1, big_dx).into_iter().map(&major_x));
    lines.extend(range((big_y0 / big_dy).ceil() * big_dy, big_y1, big_dy).into_iter().map(&major_y));
    lines.extend(
        range((x0 / dx).ceil() * dx, x1, dx).into_iter().filter(|x| (x % big_dx).abs() > EPSILON).map(&minor_x),
    );
    lines.extend(
        range((y0 / dy).ceil() * dy, y1, dy).into_iter().filter(|y| (y % big_dy).abs() > EPSILON).map(&minor_y),
    );
    lines
}

/// Great-circle distance in radians between two [longitude, latitude] points in degrees.
pub fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (l0, p0) = (a[0] * RADIANS, a[1] * RADIANS);
    let (l1, p1) = (b[0] * RADIANS, b[1] * RADIANS);
    let delta = (l1 - l0).abs();
    let (cd, sd) = (delta.cos(), delta.sin());
    let x = p1.cos() * sd;
    let y = p0.cos() * p1.sin() - p0.sin() * p1.cos() * cd;
    let z = p0.sin() * p1.sin() + p0.cos() * p1.cos() * cd;
    (x * x + y * y).sqrt().atan2(z)
}

fn asin(x: f64) -> f64 {
    if x > 1.0 {
        FRAC_PI_2
    } else if x < -1.0 {
        -FRAC_PI_2
    } else {
        x.asin()
    }
}

fn acos(x: f64) -> f64 {
    if x > 1.0 {
        0.0
    } else if x < -1.0 {
        PI
    } else {
        x.acos()
    }
}

type V3 = [f64; 3];

fn cartesian(lambda: f64, phi: f64) -> V3 {
    let c = phi.cos();
    [c * lambda.cos(), c * lambda.sin(), phi.sin()]
}

fn spherical(c: V3) -> [f64; 2] {
    [c[1].atan2(c[0]), asin(c[2])]
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}

fn scale(v: V3, k: f64) -> V3 {
    [v[0] * k, v[1] * k, v[2] * k]
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn normalize(d: V3) -> V3 {
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    [d[0] / l, d[1] / l, d[2] / l]
}

fn point_equal(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < EPSILON && (a[1] - b[1]).abs() < EPSILON
}

/// A sink for d3's geometry stream protocol.
trait Stream {
    fn point(&mut self, x: f64, y: f64, m: u8);
    fn line_start(&mut self);
    fn line_end(&mut self);
    fn polygon_start(&mut self);
    fn polygon_end(&mut self);
}

/// d3-path's string output, coordinates rounded to 3 decimals.
#[derive(Default)]
struct PathString {
    out: String,
    in_polygon: bool,
    point: Option<bool>,
}

impl Stream for PathString {
    fn point(&mut self, x: f64, y: f64, _: u8) {
        let n = |v: f64| js_number(js_round(v * 1000.0) / 1000.0);
        match self.point {
            Some(false) => {
                self.out.push_str(&format!("M{},{}", n(x), n(y)));
                self.point = Some(true);
            }
            Some(true) => self.out.push_str(&format!("L{},{}", n(x), n(y))),
            None => self.out.push_str(&format!("M{},{}m0,4.5a4.5,4.5 0 1,1 0,-9a4.5,4.5 0 1,1 0,9z", n(x), n(y))),
        }
    }
    fn line_start(&mut self) {
        self.point = Some(false);
    }
    fn line_end(&mut self) {
        if self.in_polygon {
            self.out.push('Z');
        }
        self.point = None;
    }
    fn polygon_start(&mut self) {
        self.in_polygon = true;
    }
    fn polygon_end(&mut self) {
        self.in_polygon = false;
    }
}

/// An orthographic projection centred at the origin: `scale` pixels to the globe's radius, rotated by
/// [λ, φ] degrees, clipped to the visible hemisphere.
#[derive(Clone, Copy, Debug)]
pub struct Orthographic {
    pub scale: f64,
    pub rotate: [f64; 2],
}

impl Orthographic {
    fn rotate_radians(&self, lambda: f64, phi: f64) -> [f64; 2] {
        let dl = (self.rotate[0] % 360.0 * RADIANS) % TAU;
        let dp = self.rotate[1] % 360.0 * RADIANS;
        let mut lambda = lambda;
        if dl != 0.0 {
            lambda += dl;
            if lambda.abs() > PI {
                lambda -= js_round(lambda / TAU) * TAU;
            }
        }
        if dp == 0.0 {
            if dl == 0.0 && lambda.abs() > PI {
                lambda -= js_round(lambda / TAU) * TAU;
            }
            return [lambda, phi];
        }
        let (cdp, sdp) = (dp.cos(), dp.sin());
        let cp = phi.cos();
        let (x, y, z) = (lambda.cos() * cp, lambda.sin() * cp, phi.sin());
        let k = z * cdp + x * sdp;
        [y.atan2(x * cdp - z * sdp), asin(k)]
    }

    fn project_raw(&self, lambda: f64, phi: f64) -> [f64; 2] {
        let (x, y) = (phi.cos() * lambda.sin(), phi.sin());
        [self.scale * x, -self.scale * y]
    }

    /// Where a [longitude, latitude] point lands, hidden side included.
    pub fn project(&self, p: [f64; 2]) -> [f64; 2] {
        let r = self.rotate_radians(p[0] * RADIANS, p[1] * RADIANS);
        self.project_raw(r[0], r[1])
    }

    /// The SVG path of the polygons' visible parts.
    pub fn polygons_path(&self, polygons: &MultiPolygon) -> String {
        let mut path = PathString::default();
        {
            let mut resample = Resample::new(*self, &mut path);
            let mut clip = Clip::new(&mut resample);
            for poly in polygons {
                clip.polygon_start();
                for ring in poly {
                    clip.line_start();
                    for p in &ring[..ring.len().saturating_sub(1)] {
                        let r = self.rotate_radians(p[0] * RADIANS, p[1] * RADIANS);
                        clip.point(r[0], r[1]);
                    }
                    clip.line_end();
                }
                clip.polygon_end();
            }
        }
        path.out
    }

    /// The SVG path of the lines' visible parts.
    pub fn lines_path(&self, lines: &[Vec<[f64; 2]>]) -> String {
        let mut path = PathString::default();
        {
            let mut resample = Resample::new(*self, &mut path);
            let mut clip = Clip::new(&mut resample);
            for line in lines {
                clip.line_start();
                for p in line {
                    let r = self.rotate_radians(p[0] * RADIANS, p[1] * RADIANS);
                    clip.point(r[0], r[1]);
                }
                clip.line_end();
            }
        }
        path.out
    }
}

const MAX_DEPTH: u32 = 16;
const DELTA2: f64 = 0.5;

/// d3's adaptive resampling: subdivides projected edges until they follow the curve within √0.5 px.
struct Resample<'a> {
    proj: Orthographic,
    out: &'a mut dyn Stream,
    in_line: bool,
    ring: bool,
    first: bool,
    prev: (f64, f64, f64, f64, f64, f64),
    start: (f64, f64, f64, f64, f64, f64),
}

impl<'a> Resample<'a> {
    fn new(proj: Orthographic, out: &'a mut dyn Stream) -> Self {
        let nan = (f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN, f64::NAN);
        Resample { proj, out, in_line: false, ring: false, first: false, prev: nan, start: nan }
    }

    #[allow(clippy::too_many_arguments)]
    fn line_to(&mut self, a: (f64, f64, f64, f64, f64, f64), b: (f64, f64, f64, f64, f64, f64), depth: u32) {
        let (x0, y0, lambda0, a0, b0, c0) = a;
        let (x1, y1, lambda1, a1, b1, c1) = b;
        let (dx, dy) = (x1 - x0, y1 - y0);
        let d2 = dx * dx + dy * dy;
        if d2 > 4.0 * DELTA2 && depth > 0 {
            let depth = depth - 1;
            let (sa, sb, sc) = (a0 + a1, b0 + b1, c0 + c1);
            let m = (sa * sa + sb * sb + sc * sc).sqrt();
            let c = sc / m;
            let phi2 = asin(c);
            let lambda2 = if ((c.abs() - 1.0).abs() < EPSILON) || (lambda0 - lambda1).abs() < EPSILON {
                (lambda0 + lambda1) / 2.0
            } else {
                sb.atan2(sa)
            };
            let p = self.proj.project_raw(lambda2, phi2);
            let (x2, y2) = (p[0], p[1]);
            let (dx2, dy2) = (x2 - x0, y2 - y0);
            let dz = dy * dx2 - dx * dy2;
            if dz * dz / d2 > DELTA2
                || ((dx * dx2 + dy * dy2) / d2 - 0.5).abs() > 0.3
                || a0 * a1 + b0 * b1 + c0 * c1 < COS_MIN_DISTANCE
            {
                let mid = (x2, y2, lambda2, sa / m, sb / m, c);
                self.line_to(a, mid, depth);
                self.out.point(x2, y2, 0);
                self.line_to(mid, b, depth);
            }
        }
    }

    fn line_point(&mut self, lambda: f64, phi: f64) {
        let c = cartesian(lambda, phi);
        let p = self.proj.project_raw(lambda, phi);
        let next = (p[0], p[1], lambda, c[0], c[1], c[2]);
        self.line_to(self.prev, next, MAX_DEPTH);
        self.prev = next;
        self.out.point(p[0], p[1], 0);
    }
}

const COS_MIN_DISTANCE: f64 = 0.8660254037844387;

impl Stream for Resample<'_> {
    fn point(&mut self, lambda: f64, phi: f64, _: u8) {
        if !self.in_line {
            let p = self.proj.project_raw(lambda, phi);
            self.out.point(p[0], p[1], 0);
        } else if self.first {
            self.line_point(lambda, phi);
            self.start = self.prev;
            self.first = false;
        } else {
            self.line_point(lambda, phi);
        }
    }
    fn line_start(&mut self) {
        self.prev.0 = f64::NAN;
        self.in_line = true;
        self.first = self.ring;
        self.out.line_start();
    }
    fn line_end(&mut self) {
        if self.ring && !self.first {
            self.line_to(self.prev, self.start, MAX_DEPTH);
        }
        self.in_line = false;
        self.out.line_end();
    }
    fn polygon_start(&mut self) {
        self.out.polygon_start();
        self.ring = true;
    }
    fn polygon_end(&mut self) {
        self.out.polygon_end();
        self.ring = false;
    }
}

// The clip circle is the visible hemisphere: radius 90°.
const RADIUS: f64 = FRAC_PI_2;

fn visible(lambda: f64, phi: f64) -> bool {
    lambda.cos() * phi.cos() > RADIUS.cos()
}

/// d3's circleStream: points along the clip circle from `from` to `to` (or all the way round).
fn circle_stream(stream: &mut dyn Stream, direction: f64, from: Option<[f64; 2]>, to: Option<[f64; 2]>) {
    let delta = 2.0 * RADIANS;
    let (cr, sr) = (RADIUS.cos(), RADIUS.sin());
    let step = direction * delta;
    let (mut t0, t1) = match (from, to) {
        (Some(f), Some(t)) => {
            let a = circle_radius(cr, f);
            let b = circle_radius(cr, t);
            (a, b)
        }
        _ => (RADIUS + direction * TAU, RADIUS - step / 2.0),
    };
    if from.is_some() && (if direction > 0.0 { t0 < t1 } else { t0 > t1 }) {
        t0 += direction * TAU;
    }
    let mut t = t0;
    while if direction > 0.0 { t > t1 } else { t < t1 } {
        let p = spherical([cr, -sr * t.cos(), -sr * t.sin()]);
        stream.point(p[0], p[1], 0);
        t -= step;
    }
}

fn circle_radius(cos_radius: f64, point: [f64; 2]) -> f64 {
    let mut p = cartesian(point[0], point[1]);
    p[0] -= cos_radius;
    let p = normalize(p);
    let radius = acos(-p[1]);
    ((if -p[2] < 0.0 { -radius } else { radius }) + TAU - EPSILON) % TAU
}

/// Where the great circle from a to b crosses the clip circle, toward b.
fn intersect(a: [f64; 2], b: [f64; 2]) -> Option<[f64; 2]> {
    let pa = cartesian(a[0], a[1]);
    let pb = cartesian(b[0], b[1]);
    let n1 = [1.0, 0.0, 0.0];
    let n2 = cross(pa, pb);
    let n2n2 = dot(n2, n2);
    let n1n2 = n2[0];
    let determinant = n2n2 - n1n2 * n1n2;
    if determinant == 0.0 {
        return Some(a);
    }
    let cr = RADIUS.cos();
    let c1 = cr * n2n2 / determinant;
    let c2 = -cr * n1n2 / determinant;
    let u = cross(n1, n2);
    let big_a = add(scale(n1, c1), scale(n2, c2));
    let w = dot(big_a, u);
    let uu = dot(u, u);
    let t2 = w * w - uu * (dot(big_a, big_a) - 1.0);
    if t2 < 0.0 {
        return None;
    }
    let t = t2.sqrt();
    let q = add(scale(u, (-w - t) / uu), big_a);
    Some(spherical(q))
}

/// The line clipper for one line or ring: passes on the visible stretches, flagging the exit points.
#[derive(Default)]
struct ClipLine {
    point0: Option<[f64; 2]>,
    v0: bool,
    v00: bool,
    clean: u8,
}

impl ClipLine {
    fn line_start(&mut self) {
        self.v00 = false;
        self.v0 = false;
        self.clean = 1;
    }

    fn point(&mut self, stream: &mut dyn Stream, lambda: f64, phi: f64) {
        let point1 = [lambda, phi];
        let v = visible(lambda, phi);
        if self.point0.is_none() {
            self.v00 = v;
            self.v0 = v;
            if v {
                stream.line_start();
            }
        }
        if v != self.v0 {
            self.clean = 0;
            let p0 = self.point0.unwrap_or(point1);
            if v {
                stream.line_start();
                if let Some(p2) = intersect(point1, p0) {
                    stream.point(p2[0], p2[1], 0);
                    self.point0 = Some(p2);
                }
            } else if let Some(p2) = intersect(p0, point1) {
                stream.point(p2[0], p2[1], 2);
                stream.line_end();
                self.point0 = Some(p2);
            }
        }
        if v && self.point0.is_none_or(|p0| !point_equal(p0, point1)) {
            stream.point(point1[0], point1[1], 0);
        }
        self.point0 = Some(point1);
        self.v0 = v;
    }

    fn line_end(&mut self, stream: &mut dyn Stream) {
        if self.v0 {
            stream.line_end();
        }
        self.point0 = None;
    }

    fn clean(&self) -> u8 {
        self.clean | (((self.v00 && self.v0) as u8) << 1)
    }
}

/// Collects a ring's clipped stretches for rejoining.
#[derive(Default)]
struct Buffer {
    lines: Vec<Vec<[f64; 3]>>,
}

impl Stream for Buffer {
    fn point(&mut self, x: f64, y: f64, m: u8) {
        if let Some(l) = self.lines.last_mut() {
            l.push([x, y, m as f64]);
        }
    }
    fn line_start(&mut self) {
        self.lines.push(Vec::new());
    }
    fn line_end(&mut self) {}
    fn polygon_start(&mut self) {}
    fn polygon_end(&mut self) {}
}

fn longitude(p: [f64; 2]) -> f64 {
    if p[0].abs() <= PI { p[0] } else { p[0].signum() * ((p[0].abs() + PI) % TAU - PI) }
}

/// d3's polygonContains on the sphere.
fn polygon_contains(polygon: &[Vec<[f64; 2]>], point: [f64; 2]) -> bool {
    let lambda = longitude(point);
    let mut phi = point[1];
    let sin_phi = phi.sin();
    let normal = [lambda.sin(), -lambda.cos(), 0.0];
    let mut angle = 0.0;
    let mut winding = 0i64;
    let mut sum = 0.0;
    let mut comp = 0.0;
    if sin_phi == 1.0 {
        phi = FRAC_PI_2 + EPSILON;
    } else if sin_phi == -1.0 {
        phi = -FRAC_PI_2 - EPSILON;
    }
    for ring in polygon {
        let m = ring.len();
        if m == 0 {
            continue;
        }
        let mut point0 = ring[m - 1];
        let mut lambda0 = longitude(point0);
        let phi0 = point0[1] / 2.0 + FRAC_PI_4;
        let (mut sin_phi0, mut cos_phi0) = (phi0.sin(), phi0.cos());
        for &point1 in ring {
            let lambda1 = longitude(point1);
            let phi1 = point1[1] / 2.0 + FRAC_PI_4;
            let (sin_phi1, cos_phi1) = (phi1.sin(), phi1.cos());
            let delta = lambda1 - lambda0;
            let sign = if delta >= 0.0 { 1.0 } else { -1.0 };
            let abs_delta = sign * delta;
            let antimeridian = abs_delta > PI;
            let k = sin_phi0 * sin_phi1;
            // Kahan-compensated, like d3's Adder.
            let term = (k * sign * abs_delta.sin()).atan2(cos_phi0 * cos_phi1 + k * abs_delta.cos());
            let y = term - comp;
            let t = sum + y;
            comp = (t - sum) - y;
            sum = t;
            angle += if antimeridian { delta + sign * TAU } else { delta };
            if antimeridian ^ (lambda0 >= lambda) ^ (lambda1 >= lambda) {
                let arc = normalize(cross(cartesian(point0[0], point0[1]), cartesian(point1[0], point1[1])));
                let inter = normalize(cross(normal, arc));
                let phi_arc = (if antimeridian ^ (delta >= 0.0) { -1.0 } else { 1.0 }) * asin(inter[2]);
                if phi > phi_arc || (phi == phi_arc && (arc[0] != 0.0 || arc[1] != 0.0)) {
                    winding += if antimeridian ^ (delta >= 0.0) { 1 } else { -1 };
                }
            }
            lambda0 = lambda1;
            sin_phi0 = sin_phi1;
            cos_phi0 = cos_phi1;
            point0 = point1;
        }
    }
    ((angle < -EPSILON) || (angle < EPSILON && sum < -EPSILON2)) ^ ((winding & 1) != 0)
}

fn compare_intersection(a: [f64; 3], b: [f64; 3]) -> f64 {
    let key = |p: [f64; 3]| if p[0] < 0.0 { p[1] - FRAC_PI_2 - EPSILON } else { FRAC_PI_2 - p[1] };
    key(a) - key(b)
}

#[derive(Clone)]
struct Node {
    x: [f64; 3],
    segment: Option<usize>,
    other: usize,
    entry: bool,
    visited: bool,
    next: usize,
    prev: usize,
}

/// d3's clipRejoin: joins the clipped stretches of a polygon's rings along the clip circle.
fn rejoin(segments: &mut [Vec<[f64; 3]>], start_inside: bool, stream: &mut dyn Stream) {
    let mut subject: Vec<Node> = Vec::new();
    let mut clip: Vec<Node> = Vec::new();
    for (si, segment) in segments.iter_mut().enumerate() {
        let n = segment.len();
        if n < 2 {
            continue;
        }
        let n = n - 1;
        let (p0, mut p1) = (segment[0], segment[n]);
        if point_equal([p0[0], p0[1]], [p1[0], p1[1]]) {
            if p0[2] == 0.0 && p1[2] == 0.0 {
                stream.line_start();
                for p in &segment[..n] {
                    stream.point(p[0], p[1], 0);
                }
                stream.line_end();
                continue;
            }
            p1[0] += 2.0 * EPSILON;
            segment[n][0] = p1[0];
        }
        let s = subject.len();
        let c = clip.len();
        subject.push(Node { x: p0, segment: Some(si), other: c, entry: true, visited: false, next: 0, prev: 0 });
        clip.push(Node { x: p0, segment: None, other: s, entry: false, visited: false, next: 0, prev: 0 });
        subject.push(Node { x: p1, segment: Some(si), other: c + 1, entry: false, visited: false, next: 0, prev: 0 });
        clip.push(Node { x: p1, segment: None, other: s + 1, entry: true, visited: false, next: 0, prev: 0 });
    }
    if subject.is_empty() {
        return;
    }
    let mut order: Vec<usize> = (0..clip.len()).collect();
    order.sort_by(|a, b| {
        compare_intersection(clip[*a].x, clip[*b].x).partial_cmp(&0.0).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut sorted: Vec<Node> = order.iter().map(|i| clip[*i].clone()).collect();
    for (new_i, node) in sorted.iter().enumerate() {
        subject[node.other].other = new_i;
    }
    let link = |list: &mut Vec<Node>| {
        let n = list.len();
        for i in 0..n {
            list[i].next = (i + 1) % n;
            list[(i + 1) % n].prev = i;
        }
    };
    link(&mut subject);
    link(&mut sorted);
    let mut inside = start_inside;
    for node in sorted.iter_mut() {
        inside = !inside;
        node.entry = inside;
    }
    let mut clip = sorted;

    // (is_subject, index)
    loop {
        let mut current = (true, 0usize);
        let get = |subject: &Vec<Node>, clip: &Vec<Node>, c: (bool, usize)| -> Node {
            if c.0 { subject[c.1].clone() } else { clip[c.1].clone() }
        };
        while get(&subject, &clip, current).visited {
            current = (true, subject[current.1].next);
            if current.1 == 0 {
                return;
            }
        }
        let mut points = get(&subject, &clip, current).segment;
        stream.line_start();
        let mut is_subject = true;
        loop {
            let node = get(&subject, &clip, current);
            if current.0 {
                subject[current.1].visited = true;
                clip[node.other].visited = true;
            } else {
                clip[current.1].visited = true;
                subject[node.other].visited = true;
            }
            if node.entry {
                if is_subject {
                    if let Some(si) = points {
                        for p in &segments[si] {
                            stream.point(p[0], p[1], 0);
                        }
                    }
                } else {
                    let next = get(&subject, &clip, (current.0, node.next));
                    circle_stream(stream, 1.0, Some([node.x[0], node.x[1]]), Some([next.x[0], next.x[1]]));
                }
                current = (current.0, node.next);
            } else {
                if is_subject {
                    let prev = get(&subject, &clip, (current.0, node.prev));
                    points = prev.segment;
                    if let Some(si) = points {
                        for p in segments[si].iter().rev() {
                            stream.point(p[0], p[1], 0);
                        }
                    }
                } else {
                    let prev = get(&subject, &clip, (current.0, node.prev));
                    circle_stream(stream, -1.0, Some([node.x[0], node.x[1]]), Some([prev.x[0], prev.x[1]]));
                }
                current = (current.0, node.prev);
            }
            let here = get(&subject, &clip, current);
            current = (!current.0, here.other);
            points = get(&subject, &clip, current).segment;
            is_subject = !is_subject;
            if get(&subject, &clip, current).visited {
                break;
            }
        }
        stream.line_end();
    }
}

/// d3's clip for the visible hemisphere, between the rotation and the resampler.
struct Clip<'a> {
    sink: &'a mut dyn Stream,
    line: ClipLine,
    ring_line: ClipLine,
    ring_buffer: Buffer,
    in_polygon: bool,
    polygon_started: bool,
    polygon: Vec<Vec<[f64; 2]>>,
    segments: Vec<Vec<[f64; 3]>>,
    ring: Vec<[f64; 2]>,
}

impl<'a> Clip<'a> {
    fn new(sink: &'a mut dyn Stream) -> Self {
        Clip {
            sink,
            line: ClipLine::default(),
            ring_line: ClipLine::default(),
            ring_buffer: Buffer::default(),
            in_polygon: false,
            polygon_started: false,
            polygon: Vec::new(),
            segments: Vec::new(),
            ring: Vec::new(),
        }
    }

    fn point(&mut self, lambda: f64, phi: f64) {
        if self.in_polygon {
            self.ring.push([lambda, phi]);
            self.ring_line.point(&mut self.ring_buffer, lambda, phi);
        } else {
            self.line.point(self.sink, lambda, phi);
        }
    }

    fn line_start(&mut self) {
        if self.in_polygon {
            self.ring_line.line_start();
            self.ring.clear();
        } else {
            self.line.line_start();
        }
    }

    fn line_end(&mut self) {
        if !self.in_polygon {
            self.line.line_end(self.sink);
            return;
        }
        let first = self.ring[0];
        self.ring.push(first);
        self.ring_line.point(&mut self.ring_buffer, first[0], first[1]);
        self.ring_line.line_end(&mut self.ring_buffer);
        let clean = self.ring_line.clean();
        let mut ring_segments = std::mem::take(&mut self.ring_buffer.lines);
        self.ring.pop();
        self.polygon.push(std::mem::take(&mut self.ring));
        let n = ring_segments.len();
        if n == 0 {
            return;
        }
        if clean & 1 != 0 {
            let segment = &ring_segments[0];
            let m = segment.len().saturating_sub(1);
            if m > 0 {
                self.start_polygon();
                self.sink.line_start();
                for p in &segment[..m] {
                    self.sink.point(p[0], p[1], 0);
                }
                self.sink.line_end();
            }
            return;
        }
        if n > 1 && clean & 2 != 0 {
            let last = ring_segments.pop().unwrap_or_default();
            let first = ring_segments.remove(0);
            ring_segments.push([last, first].concat());
        }
        self.segments.extend(ring_segments.into_iter().filter(|s| s.len() > 1));
    }

    fn start_polygon(&mut self) {
        if !self.polygon_started {
            self.sink.polygon_start();
            self.polygon_started = true;
        }
    }

    fn polygon_start(&mut self) {
        self.in_polygon = true;
        self.segments.clear();
        self.polygon.clear();
    }

    fn polygon_end(&mut self) {
        self.in_polygon = false;
        let start_inside = polygon_contains(&self.polygon, [0.0, -RADIUS]);
        if !self.segments.is_empty() {
            self.start_polygon();
            let mut segments = std::mem::take(&mut self.segments);
            rejoin(&mut segments, start_inside, self.sink);
        } else if start_inside {
            self.start_polygon();
            self.sink.line_start();
            circle_stream(self.sink, 1.0, None, None);
            self.sink.line_end();
        }
        if self.polygon_started {
            self.sink.polygon_end();
            self.polygon_started = false;
        }
        self.polygon.clear();
    }
}
