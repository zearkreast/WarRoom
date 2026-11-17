//! Galactic Campaign pictures. The game's API names each picture only by id, so the ones storage lacks are
//! looked up on a MediaWiki whose Campaigns page lists every campaign with a `{{Campaign}}` template.

use crate::http::{Request, Transport};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use warroom_core::snapshot::{EpisodeLive, campaign_art_names, order_art_names};

pub const BUCKET: &str = "campaign-art";
/// The widest the app shows a picture.
const WIDTH: u32 = 752;
/// The bucket's file_size_limit; keep in sync with 0013_campaign_art.sql.
const MAX_BYTES: usize = 1_048_576;
const TIMEOUT_MS: u64 = 30_000;
const USER_AGENT: &str = "warroom-collector (+https://github.com/zearkreast/WarRoom)";

/// A picture the app looks for: a campaign's cover, or the picture of its `order`-th Major Order.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    /// Storage names the app tries in turn, without `.webp`; a new picture is stored under the first.
    pub names: Vec<String>,
    pub campaign: String,
    pub order: Option<usize>,
}

/// Every picture the app shows for these campaigns. An order sharing its campaign's picture isn't shown
/// apart from the cover, so it needs none of its own.
pub fn slots(episodes: &[EpisodeLive]) -> Vec<Slot> {
    let mut out = Vec::new();
    for e in episodes {
        out.push(Slot { names: campaign_art_names(e), campaign: e.title.clone(), order: None });
        for (i, p) in e.phases.iter().enumerate() {
            if p.picture.is_some() && p.picture == e.picture {
                continue;
            }
            out.push(Slot { names: order_art_names(e, i + 1), campaign: e.title.clone(), order: Some(i + 1) });
        }
    }
    out
}

/// Slots with none of their names in storage, one per name to store: the game reuses pictures.
pub fn missing(slots: &[Slot], stored: &HashSet<String>) -> Vec<Slot> {
    let mut seen = HashSet::new();
    slots
        .iter()
        .filter(|s| !s.names.iter().any(|n| stored.contains(n)))
        .filter(|s| seen.insert(s.names[0].clone()))
        .cloned()
        .collect()
}

/// A campaign as the wiki lists it: its banner and each Major Order's picture, as file names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WikiCampaign {
    pub name: String,
    pub banner: Option<String>,
    /// By order, from the first.
    pub orders: Vec<Option<String>>,
}

impl WikiCampaign {
    pub fn file(&self, order: Option<usize>) -> Option<&str> {
        match order {
            None => self.banner.as_deref(),
            Some(n) => self.orders.get(n - 1)?.as_deref(),
        }
    }
}

fn key(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ").to_uppercase()
}

pub fn find<'a>(page: &'a [WikiCampaign], campaign: &str) -> Option<&'a WikiCampaign> {
    page.iter().find(|c| key(&c.name) == key(campaign))
}

/// The text of the template call starting at `at` (just after its `{{`), up to its matching `}}`.
fn template_body(text: &str, at: usize) -> &str {
    let bytes = text.as_bytes();
    let (mut depth, mut i) = (1, at);
    while i + 1 < bytes.len() {
        match &bytes[i..i + 2] {
            b"{{" => {
                depth += 1;
                i += 2;
            }
            b"}}" => {
                depth -= 1;
                if depth == 0 {
                    return &text[at..i];
                }
                i += 2;
            }
            _ => i += 1,
        }
    }
    &text[at..]
}

/// A template call's named parameters, split on the `|`s outside nested templates and links.
fn params(body: &str) -> HashMap<String, String> {
    let mut parts = Vec::new();
    let (mut depth, mut start, mut i) = (0i32, 0, 0);
    let bytes = body.as_bytes();
    while i < bytes.len() {
        match bytes.get(i..i + 2) {
            Some(b"{{") | Some(b"[[") => {
                depth += 1;
                i += 2;
                continue;
            }
            Some(b"}}") | Some(b"]]") => {
                depth -= 1;
                i += 2;
                continue;
            }
            _ => {}
        }
        if bytes[i] == b'|' && depth == 0 {
            parts.push(&body[start..i]);
            start = i + 1;
        }
        i += 1;
    }
    parts.push(&body[start..]);
    parts
        .into_iter()
        .skip(1)
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (k.trim().to_lowercase(), v.trim().to_string()))
        .collect()
}

fn file_name(value: Option<&String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

pub fn parse_campaigns(wikitext: &str) -> Vec<WikiCampaign> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(found) = wikitext[from..].find("{{Campaign") {
        let at = from + found + 2;
        from = at;
        let after = &wikitext[at + "Campaign".len()..];
        // Skips `{{Campaign MO` and other templates whose name starts the same.
        if !after.trim_start().starts_with('|') {
            continue;
        }
        let p = params(template_body(wikitext, at));
        let Some(name) = file_name(p.get("name")) else { continue };
        let mut orders = Vec::new();
        for n in 1.. {
            let Some(order) = p.get(&format!("{n}_mo")) else { break };
            let image = order.find("{{").and_then(|s| params(template_body(order, s + 2)).remove("image"));
            orders.push(file_name(image.as_ref()));
        }
        out.push(WikiCampaign { name, banner: file_name(p.get("banner_image")), orders });
    }
    out
}

/// Resizes a picture to at most the app's width and encodes it as lossy WebP within the bucket's limit.
pub fn encode(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("unreadable picture: {e}"))?;
    let img =
        if img.width() > WIDTH { img.resize(WIDTH, u32::MAX, image::imageops::FilterType::Lanczos3) } else { img };
    let rgb = img.to_rgb8();
    for quality in [80.0, 65.0, 50.0] {
        let out = webp::Encoder::from_rgb(&rgb, rgb.width(), rgb.height()).encode(quality);
        if out.len() <= MAX_BYTES {
            return Ok(out.to_vec());
        }
    }
    Err("picture over 1 MB even at low quality".into())
}

fn encode_component(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// The wiki's API, at `api` (its `api.php`).
pub struct Wiki {
    api: String,
    transport: Box<dyn Transport>,
}

impl Wiki {
    pub fn new(api: &str, transport: Box<dyn Transport>) -> Wiki {
        Wiki { api: api.trim().to_string(), transport }
    }

    fn get(&self, url: &str) -> Result<Vec<u8>, String> {
        let res = self
            .transport
            .send(Request {
                method: "GET",
                url: url.to_string(),
                headers: vec![("User-Agent", USER_AGENT.into())],
                body: None,
                timeout_ms: TIMEOUT_MS,
            })
            .map_err(|e| format!("picture source: {e}"))?;
        if !res.ok() {
            return Err(format!("picture source: {}", res.status));
        }
        Ok(res.body)
    }

    fn query(&self, params: &str) -> Result<Value, String> {
        let body = self.get(&format!("{}?{params}&format=json&formatversion=2", self.api))?;
        serde_json::from_slice(&body).map_err(|e| format!("picture source: {e}"))
    }

    pub fn campaigns(&self) -> Result<Vec<WikiCampaign>, String> {
        let page = self.query("action=parse&page=Campaigns&prop=wikitext")?;
        let text =
            page.pointer("/parse/wikitext").and_then(Value::as_str).ok_or("picture source: no Campaigns page")?;
        Ok(parse_campaigns(text))
    }

    /// Where each of `files` can be downloaded, by the file name as given; files the wiki lacks are left out.
    pub fn file_urls(&self, files: &[&str]) -> Result<HashMap<String, String>, String> {
        if files.is_empty() {
            return Ok(HashMap::new());
        }
        let titles = files.iter().map(|f| format!("File:{f}")).collect::<Vec<_>>().join("|");
        let v = self.query(&format!("action=query&prop=imageinfo&iiprop=url&titles={}", encode_component(&titles)))?;
        // The wiki answers under its own spelling of each title (underscores as spaces, a capital first letter).
        let mut asked: HashMap<String, String> = files.iter().map(|f| (format!("File:{f}"), f.to_string())).collect();
        for n in v.pointer("/query/normalized").and_then(Value::as_array).into_iter().flatten() {
            if let (Some(from), Some(to)) = (n["from"].as_str(), n["to"].as_str())
                && let Some(f) = asked.remove(from)
            {
                asked.insert(to.to_string(), f);
            }
        }
        Ok(v.pointer("/query/pages")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|p| {
                let file = asked.get(p["title"].as_str()?)?;
                let url = p.pointer("/imageinfo/0/url")?.as_str()?;
                Some((file.clone(), url.to_string()))
            })
            .collect())
    }

    pub fn download(&self, url: &str) -> Result<Vec<u8>, String> {
        self.get(url)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use warroom_core::snapshot::EpisodePhase;

    pub const PAGE: &str = "=== Serene Guardian ===
{{Campaign
 | name          = Serene  Guardian
 | banner_image  = Header SG.png
 | reward_text   = [[M-104 Incinerator FRV]]
 | 1_mo =
 {{Campaign MO
  | name     = Assurance
  | image    = Header PE B.png
  | briefing = Hold [[Alamak VII|ALAMAK VII]] = now, {{Tooltip|a|b}}
 }}
 | 2_mo = {{Campaign MO
  | image =
 }}
 | 3_mo =
}}
{{Campaign MO | image = stray.png }}
{{Campaigns nav}}
";

    /// A PNG wider than the app shows.
    pub fn png() -> Vec<u8> {
        let img = image::RgbImage::from_fn(1000, 460, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 90]));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    pub fn episode(id: i64, title: &str, picture: Option<i64>, orders: &[Option<i64>]) -> EpisodeLive {
        EpisodeLive {
            id,
            title: title.into(),
            picture,
            phases: orders.iter().map(|&picture| EpisodePhase { picture, ..Default::default() }).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn reads_each_campaigns_banner_and_order_pictures_past_nested_templates_and_links() {
        let page = parse_campaigns(PAGE);
        assert_eq!(
            page,
            [WikiCampaign {
                name: "Serene  Guardian".into(),
                banner: Some("Header SG.png".into()),
                orders: vec![Some("Header PE B.png".into()), None, None],
            }]
        );
        let found = find(&page, "SERENE GUARDIAN").unwrap();
        assert_eq!(
            (found.file(None), found.file(Some(1)), found.file(Some(2))),
            (Some("Header SG.png"), Some("Header PE B.png"), None)
        );
        assert!(find(&page, "MENTAL FORTITUDE").is_none());
    }

    #[test]
    fn needs_one_picture_per_name_and_none_where_any_name_is_stored() {
        let list = [episode(1, "A", Some(10), &[Some(10), Some(20), None]), episode(2, "B", None, &[Some(20)])];
        let all = slots(&list);
        assert_eq!(
            all.iter().map(|s| (s.names[0].as_str(), s.order)).collect::<Vec<_>>(),
            [("10", None), ("20", Some(2)), ("campaign-1-3", Some(3)), ("campaign-2", None), ("20", Some(1))]
        );
        let stored: HashSet<String> = ["campaign-1".to_string(), "campaign-2".to_string()].into();
        assert_eq!(
            missing(&all, &stored).iter().map(|s| (s.campaign.as_str(), s.names[0].as_str())).collect::<Vec<_>>(),
            [("A", "20"), ("A", "campaign-1-3")]
        );
    }

    #[test]
    fn encodes_a_lossy_webp_no_wider_than_the_app_shows() {
        let out = encode(&png()).unwrap();
        assert_eq!((&out[..4], &out[8..12]), (&b"RIFF"[..], &b"WEBP"[..]));
        let img = webp::Decoder::new(&out).decode().unwrap();
        assert_eq!((img.width(), img.height()), (752, 346));
        assert!(out.len() < 100_000, "{} bytes", out.len());
        assert!(encode(b"not a picture").is_err());
    }
}
