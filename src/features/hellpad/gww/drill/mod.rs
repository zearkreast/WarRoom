//! The stratagem drill: stratagem codes keyed against a burning fuse, in classic, custom loadout and permit
//! colour modes, with the best scores kept on this device.

mod audio;

use crate::features::hellpad::gww::blocks::{HeadingRef, banner, crumb};
use crate::features::hellpad::gww::nav::{Rich, open_in_gww};
use crate::features::map::stratagem_art::{icon_of, permit_color};
use crate::js::{self, fmt_date, num};
use crate::state::{SettingsData, settings};
use crate::ui::icons::{self, Icon};
use crate::ui::primitives::{Empty, Keycap};
use audio::{Audio, March, audio};
use leptos::html;
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;
use warroom_core::drill::{
    Drill, DrillScore, FUSE, MODES, Mode, Phase, Press, best, classic_pool, custom_pool, multiplier, permit_pool,
    pool_label,
};
use warroom_core::format::fmt_int;
use warroom_core::gww_nav::{DRILL_TITLE, GwwPage};
use warroom_core::stratagems::{Arrow, KINDS, Kind, PERMITS, Permit, STRATAGEMS, Stratagem, arrow_of, by_id};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Fuse seconds under which the alarm ticks.
const LOW_FUSE: f64 = 3.0;
/// How long "Time's up" shows before the results.
const OVER_MS: u64 = 1400;

pub fn open_drill() {
    open_in_gww(GwwPage::Site { id: "drill".into() });
}

fn permits_of(saved: &[String]) -> Vec<Permit> {
    PERMITS.into_iter().filter(|p| saved.iter().any(|s| s == p.as_str())).collect()
}

fn pool_of(d: &SettingsData) -> Vec<&'static Stratagem> {
    match d.drill_mode {
        Mode::Classic => classic_pool(),
        Mode::Custom => custom_pool(&d.drill_picks),
        Mode::Permits => permit_pool(&permits_of(&d.drill_permits)),
    }
}

fn sfx(play: impl FnOnce(&Audio)) {
    if let Some(a) = audio::current() {
        play(&a);
    }
}

fn apply_levels() {
    if let Some(a) = audio::current() {
        settings().untracked(|d| a.set_levels(d.drill_music, d.drill_effects));
    }
}

fn tempo(round: u32) -> f64 {
    (132.0 + 4.0 * round.saturating_sub(1) as f64).min(168.0)
}

fn icon_img(s: &Stratagem, size: u32) -> impl IntoView + use<> {
    icon_of(s).map(|src| view! { <img src=src alt="" width=size height=size draggable="false" /> })
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Lobby,
    Run,
    Over,
}

#[derive(Clone, PartialEq)]
struct Finished {
    score: DrillScore,
    best_streak: u32,
    accuracy: f64,
    place: Option<usize>,
    was_best: bool,
}

pub fn drill_page(heading: HeadingRef) -> AnyView {
    view! { <DrillPage heading=heading /> }.into_any()
}

#[component]
fn DrillPage(heading: HeadingRef) -> impl IntoView {
    let screen = RwSignal::new(Screen::Lobby);
    let finished = RwSignal::new(None::<Finished>);
    let run = StoredValue::new((Vec::<&'static Stratagem>::new(), Mode::Classic, String::new()));
    let start = Callback::new(move |()| {
        let (pool, mode, label) = settings().untracked(|d| {
            let permits = permits_of(&d.drill_permits);
            (pool_of(d), d.drill_mode, pool_label(d.drill_mode, &d.drill_picks, &permits))
        });
        if pool.is_empty() {
            return;
        }
        if let Some(a) = audio() {
            a.wake();
        }
        apply_levels();
        run.set_value((pool, mode, label));
        screen.set(Screen::Run);
    });
    let over = Callback::new(move |f: Finished| {
        finished.set(Some(f));
        screen.set(Screen::Over);
    });
    let lobby = Callback::new(move |()| screen.set(Screen::Lobby));
    view! {
        <article class="gww-page wide dr-page">
            {banner(
                Some(view! { <Icon icon=icons::GAMEPAD_2 size=40.0 stroke_width=1.5 /> }.into_any()),
                vec![crumb("G.W.W.", Some("home")), crumb("High Command", None), crumb("Training", None)],
                DRILL_TITLE.into(),
                Some(view! {
                    <Rich text="Key stratagem codes before the fuse burns out. Every code you complete buys time back; each round deals more codes and burns the fuse faster. The codes are those of the Hellpad's [[app:stratagems|stratagem collection]]." />
                }
                .into_any()),
                heading,
                false,
            )}
            {move || match screen.get() {
                Screen::Lobby => view! { <Lobby start=start /> }.into_any(),
                Screen::Run => {
                    let (pool, mode, label) = run.get_value();
                    view! { <Run pool=pool mode=mode label=label over=over quit=lobby /> }.into_any()
                }
                Screen::Over => finished.get().map(|f| view! { <Results f=f again=start lobby=lobby /> }.into_any()).unwrap_or_else(|| ().into_any()),
            }}
        </article>
    }
}

// ---------- Lobby ----------

fn mode_card(m: Mode, now: Memo<Mode>) -> impl IntoView {
    let top = Memo::new(move |_| settings().get(|d| best(&d.drill_scores, m).map(|s| s.score)));
    let (icon, text) = match m {
        Mode::Classic => (icons::ZAP, "Every stratagem on file, dealt at random."),
        Mode::Custom => (icons::CROSSHAIR, "Only the stratagems you pick, to drill your own loadout."),
        Mode::Permits => (icons::SHIELD_HALF, "Every stratagem of the permit colours you pick."),
    };
    view! {
        <button
            class=move || format!("dr-mode {}", if now.get() == m { "on" } else { "" })
            aria-pressed=move || (now.get() == m).to_string()
            on:click=move |_| settings().set(|d| d.drill_mode = m)
        >
            <Icon icon=icon size=22.0 stroke_width=1.75 />
            <b class="dr-mode-name">{m.label()}</b>
            <span class="dr-mode-text">{text}</span>
            <span class="dr-mode-best caps num">
                {move || top.get().map_or("No score yet".to_string(), |s| format!("Best {}", fmt_int(s as f64)))}
            </span>
        </button>
    }
}

#[component]
fn Picker() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let q = Memo::new(move |_| query.with(|s| s.trim().to_lowercase()));
    let picks = Memo::new(move |_| settings().get(|d| d.drill_picks.clone()));
    let count = Memo::new(move |_| picks.with(|p| custom_pool(p).len()));
    let set_kind = move |k: Kind, on: bool| {
        settings().set(|d| {
            for s in STRATAGEMS.iter().filter(|s| s.kind == k) {
                let has = d.drill_picks.iter().any(|p| p == s.id);
                if on && !has {
                    d.drill_picks.push(s.id.into());
                } else if !on && has {
                    d.drill_picks.retain(|p| p != s.id);
                }
            }
        })
    };
    let group = move |k: Kind| {
        let all = Memo::new(move |_| {
            picks.with(|p| STRATAGEMS.iter().filter(|s| s.kind == k).all(|s| p.iter().any(|x| x == s.id)))
        });
        let shown = Memo::new(move |_| {
            q.with(|q| STRATAGEMS.iter().any(|s| s.kind == k && s.name.to_lowercase().contains(q.as_str())))
        });
        view! {
            <div class="dr-pick-group" style=format!("--p: {};", permit_color(k.permit())) hidden=move || !shown.get()>
                <div class="dr-pick-head caps">
                    <span>{k.label()}</span>
                    <button
                        class="dr-text-btn"
                        aria-label=move || format!("{} all {}", if all.get() { "Drop" } else { "Pick" }, k.label().to_lowercase())
                        on:click=move |_| set_kind(k, !all.get_untracked())
                    >
                        {move || if all.get() { "None" } else { "All" }}
                    </button>
                </div>
                <ul class="dr-picks">
                    {STRATAGEMS
                        .iter()
                        .filter(|s| s.kind == k)
                        .map(|s| {
                            let on = Memo::new(move |_| picks.with(|p| p.iter().any(|x| x == s.id)));
                            let hide = Memo::new(move |_| q.with(|q| !s.name.to_lowercase().contains(q.as_str())));
                            view! {
                                <li hidden=move || hide.get()>
                                    <button
                                        class=move || format!("dr-pick {}", if on.get() { "on" } else { "" })
                                        aria-pressed=move || on.get().to_string()
                                        aria-label=s.name
                                        title=s.name
                                        on:click=move |_| settings().toggle_drill_pick(s.id)
                                    >
                                        {icon_img(s, 40)}
                                    </button>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
        }
    };
    view! {
        <section class="dr-panel" aria-label="Custom loadout">
            <header class="dr-panel-head">
                <b class="dr-panel-title caps">
                    "Your loadout "
                    <span class="num">{move || count.get()}</span>
                </b>
                <label class="dr-filter">
                    <Icon icon=icons::SEARCH size=14.0 />
                    <input
                        type="search"
                        placeholder="Find a stratagem"
                        aria-label="Find a stratagem"
                        prop:value=move || query.get()
                        on:input=move |e| query.set(event_target_value(&e))
                    />
                </label>
                <button class="dr-text-btn" disabled=move || count.get() == 0 on:click=move |_| settings().set(|d| d.drill_picks.clear())>
                    "Clear"
                </button>
            </header>
            <p class="dr-note">"Tap stratagems to add them; the crosshair on the stratagem collection's cards picks them too."</p>
            {KINDS.map(group).collect_view()}
        </section>
    }
}

#[component]
fn PermitPicker() -> impl IntoView {
    let chosen = Memo::new(move |_| settings().get(|d| permits_of(&d.drill_permits)));
    let toggle = move |p: Permit| {
        settings().set(|d| match d.drill_permits.iter().position(|s| s == p.as_str()) {
            Some(i) => {
                d.drill_permits.remove(i);
            }
            None => d.drill_permits.push(p.as_str().into()),
        })
    };
    view! {
        <section class="dr-panel" aria-label="Permit colours">
            <div class="dr-permits">
                {PERMITS
                    .map(|p| {
                        let n = STRATAGEMS.iter().filter(|s| s.permit() == p).count();
                        let on = Memo::new(move |_| chosen.with(|c| c.contains(&p)));
                        view! {
                            <button
                                class=move || format!("dr-permit {}", if on.get() { "on" } else { "" })
                                style=format!("--p: {};", permit_color(p))
                                aria-pressed=move || on.get().to_string()
                                on:click=move |_| toggle(p)
                            >
                                <span class="dr-permit-swatch" aria-hidden="true"></span>
                                <b>{p.colour()}</b>
                                <span class="caps num">{format!("{} · {n}", p.label())}</span>
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn SoundToggles() -> impl IntoView {
    let music = Memo::new(move |_| settings().get(|d| d.drill_music));
    let effects = Memo::new(move |_| settings().get(|d| d.drill_effects));
    let flip = move |f: fn(&mut SettingsData)| {
        settings().set(f);
        apply_levels();
    };
    view! {
        <span class="dr-sound">
            <button
                class="dr-icon-btn"
                aria-pressed=move || music.get().to_string()
                aria-label="Music"
                title=move || if music.get() { "Music on" } else { "Music off" }
                on:click=move |_| flip(|d| d.drill_music = !d.drill_music)
            >
                <Icon icon=icons::MUSIC size=16.0 />
            </button>
            <button
                class="dr-icon-btn"
                aria-pressed=move || effects.get().to_string()
                aria-label="Sound effects"
                title=move || if effects.get() { "Sound effects on" } else { "Sound effects off" }
                on:click=move |_| flip(|d| d.drill_effects = !d.drill_effects)
            >
                {move || view! { <Icon icon=if effects.get() { icons::VOLUME_2 } else { icons::VOLUME_X } size=16.0 /> }}
            </button>
        </span>
    }
}

/// This device's best scores for a mode; the one finished at `mark` is highlighted.
#[component]
fn Board(#[prop(into)] mode: Signal<Mode>, #[prop(optional)] mark: Option<f64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        let m = mode.get();
        settings().get(|d| d.drill_scores.iter().filter(|s| s.mode == m).cloned().collect::<Vec<_>>())
    });
    view! {
        <section class="dr-board">
            <h3>"Best scores · " {move || mode.get().label()}</h3>
            {move || {
                let list = rows.get();
                if list.is_empty() {
                    return view! { <Empty>"No drill finished in this mode yet. Scores stay on this device."</Empty> }.into_any();
                }
                view! {
                    <table class="dr-table num">
                        <thead>
                            <tr>
                                <th scope="col">"#"</th>
                                <th scope="col">"Score"</th>
                                <th scope="col">"Round"</th>
                                <th scope="col">"Codes"</th>
                                <th scope="col" class="dr-col-pool">"Pool"</th>
                                <th scope="col">"Date"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {list
                                .into_iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    let marked = if mark == Some(s.at) { "mark" } else { "" };
                                    view! {
                                    <tr class=marked>
                                        <td>{i + 1}</td>
                                        <td class="dr-col-score">{fmt_int(s.score as f64)}</td>
                                        <td>{s.round}</td>
                                        <td>{s.called}</td>
                                        <td class="dr-col-pool">{s.pool}</td>
                                        <td>{fmt_date(s.at)}</td>
                                    </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                }
                .into_any()
            }}
        </section>
    }
}

fn controls() -> impl IntoView {
    view! {
        <p class="dr-help">
            <Icon icon=icons::GAMEPAD_2 size=14.0 />
            <Keycap k="W" />
            <Keycap k="A" />
            <Keycap k="S" />
            <Keycap k="D" />
            " or the arrow keys, or the pad on a touch screen. "
            <Keycap k="Esc" />
            " pauses."
        </p>
    }
}

#[component]
fn Lobby(start: Callback<()>) -> impl IntoView {
    let mode = Memo::new(move |_| settings().get(|d| d.drill_mode));
    let size = Memo::new(move |_| settings().get(|d| pool_of(d).len()));
    view! {
        <div class="dr-lobby">
            <div class="dr-modes" role="group" aria-label="Mode">
                {MODES.map(|m| mode_card(m, mode)).collect_view()}
            </div>
            {move || match mode.get() {
                Mode::Classic => view! {
                    <p class="dr-note">
                        {format!("All {} stratagems, from the Orbital Precision Strike to the objectives' drills.", STRATAGEMS.len())}
                    </p>
                }
                .into_any(),
                Mode::Custom => view! { <Picker /> }.into_any(),
                Mode::Permits => view! { <PermitPicker /> }.into_any(),
            }}
            <div class="dr-go">
                <button class="cta-btn dr-start" disabled=move || size.get() == 0 on:click=move |_| start.run(())>
                    <Icon icon=icons::PLAY size=15.0 stroke_width=2.0 />
                    "Start drill"
                </button>
                <span class="dr-go-pool num">
                    {move || match (size.get(), mode.get()) {
                        (0, Mode::Custom) => "Pick at least one stratagem.".to_string(),
                        (0, _) => "Pick at least one colour.".to_string(),
                        (1, _) => "1 stratagem in the pool".to_string(),
                        (n, _) => format!("{n} stratagems in the pool"),
                    }}
                </span>
                <SoundToggles />
            </div>
            {controls()}
            <Board mode=mode />
        </div>
    }
}

// ---------- The run ----------

#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    /// Whole seconds left on the countdown.
    Ready(u32),
    Play,
    Cleared,
    Over,
}

fn stage_of(p: Phase) -> Stage {
    match p {
        Phase::Ready(s) => Stage::Ready(s.ceil().max(1.0) as u32),
        Phase::Play => Stage::Play,
        Phase::Cleared(_) => Stage::Cleared,
        Phase::Over => Stage::Over,
    }
}

fn seed() -> u64 {
    (js_sys::Math::random() * u64::MAX as f64) as u64
}

/// Replays a CSS animation by taking its class off and putting it back.
fn replay(el: Option<web_sys::HtmlElement>, class: &str) {
    let Some(el) = el else { return };
    let list = el.class_list();
    let _ = list.remove_1(class);
    let _ = el.offset_width();
    let _ = list.add_1(class);
}

const PAD: [(Arrow, &str); 4] =
    [(Arrow::Up, "Up"), (Arrow::Left, "Left"), (Arrow::Down, "Down"), (Arrow::Right, "Right")];

#[component]
fn Run(
    pool: Vec<&'static Stratagem>,
    mode: Mode,
    label: String,
    over: Callback<Finished>,
    quit: Callback<()>,
) -> impl IntoView {
    let drill = StoredValue::new(Drill::new(pool, seed()));
    let rev = RwSignal::new(0u32);
    let stage = RwSignal::new(Stage::Ready(2));
    let fuse = RwSignal::new(FUSE);
    let paused = RwSignal::new(false);
    let gained = RwSignal::new(None::<(u32, u32)>);
    let march = StoredValue::new(None::<March>);
    let ending = StoredValue::new(None::<TimeoutHandle>);
    let root = NodeRef::<html::Div>::new();
    let code_el = NodeRef::<html::Div>::new();
    let resume_btn = NodeRef::<html::Button>::new();
    let best_before = settings().untracked(|d| best(&d.drill_scores, mode).map_or(0, |s| s.score));
    let label = StoredValue::new(label);

    let read = move |f: fn(&Drill) -> u32| {
        rev.track();
        drill.with_value(f)
    };
    let round = Memo::new(move |_| read(|d| d.round));
    let score = Memo::new(move |_| read(|d| d.score));
    let streak = Memo::new(move |_| read(|d| d.streak));
    let left = Memo::new(move |_| read(|d| d.queue.len() as u32));
    let keyed = Memo::new(move |_| read(|d| d.keyed as u32));
    let current = Memo::new(move |_| {
        rev.track();
        drill.with_value(|d| d.current().map(|s| s.id))
    });
    let upcoming = Memo::new(move |_| {
        rev.track();
        drill.with_value(|d| d.queue.iter().skip(1).take(6).map(|s| s.id).collect::<Vec<_>>())
    });
    let bonus = Memo::new(move |_| {
        rev.track();
        drill.with_value(|d| d.last_bonus)
    });

    let start_music = move || {
        if let Some(a) = audio::current() {
            a.tempo.set(tempo(drill.with_value(|d| d.round)));
            march.set_value(Some(March::start(a)));
        }
    };
    let finish = move || {
        march.set_value(None);
        sfx(audio::over);
        let d = drill.get_value();
        let score = DrillScore {
            mode,
            score: d.score,
            round: d.round,
            called: d.called,
            at: js::now(),
            pool: label.get_value(),
        };
        let place = settings().record_drill(score.clone());
        let was_best = d.score > best_before;
        if was_best {
            sfx(audio::best);
        }
        let f = Finished { score, best_streak: d.best_streak, accuracy: d.accuracy(), place, was_best };
        ending.set_value(set_timeout_with_handle(move || over.run(f), Duration::from_millis(OVER_MS)).ok());
    };
    let enter = move |to: Stage| {
        let was = stage.get_untracked();
        if to == was {
            return;
        }
        match to {
            Stage::Ready(_) => {
                if was == Stage::Cleared
                    && let Some(a) = audio::current()
                {
                    a.tempo.set(tempo(drill.with_value(|d| d.round)));
                }
                sfx(|a| audio::count(a, false));
            }
            Stage::Play => sfx(|a| audio::count(a, true)),
            Stage::Cleared => sfx(audio::cleared),
            Stage::Over => finish(),
        }
        stage.set(to);
    };

    let press = move |a: Arrow| {
        if paused.get_untracked() {
            return;
        }
        let Some((r, at)) = drill.try_update_value(|d| (d.press(a), d.keyed)) else { return };
        match r {
            Press::Ignored => return,
            Press::Hit => sfx(|s| audio::key(s, at.saturating_sub(1))),
            Press::Miss => {
                sfx(audio::miss);
                replay(code_el.get_untracked().map(|e| e.unchecked_into()), "miss");
            }
            Press::Called(p) => {
                sfx(audio::called);
                gained.update(|g| *g = Some((p, g.map_or(0, |g| g.1) + 1)));
            }
            Press::Cleared(p, _) => {
                gained.update(|g| *g = Some((p, g.map_or(0, |g| g.1) + 1)));
                enter(Stage::Cleared);
            }
        }
        rev.update(|n| *n += 1);
    };

    let set_paused = move |p: bool| {
        if paused.get_untracked() == p || matches!(stage.get_untracked(), Stage::Over) {
            return;
        }
        paused.set(p);
        if p {
            march.set_value(None);
        } else {
            start_music();
            if let Some(r) = root.get_untracked() {
                let _ = r.focus();
            }
        }
    };

    let last = Rc::new(Cell::new(None::<f64>));
    let alarm_at = Rc::new(Cell::new(0.0));
    let stop = js::animation_loop(move |t| {
        let dt = last.replace(Some(t)).map_or(0.0, |l| ((t - l) / 1000.0).clamp(0.0, 0.1));
        if paused.get_untracked() {
            return true;
        }
        let Some((phase, f)) = drill.try_update_value(|d| {
            d.tick(dt);
            (d.phase, d.fuse)
        }) else {
            return false;
        };
        fuse.set(f);
        let to = stage_of(phase);
        if to != stage.get_untracked() {
            enter(to);
            rev.update(|n| *n += 1);
        }
        if to == Stage::Over {
            return false;
        }
        if to == Stage::Play && f < LOW_FUSE && t - alarm_at.get() > 500.0 {
            alarm_at.set(t);
            sfx(audio::alarm);
        }
        true
    });

    let on_key = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        if e.ctrl_key() || e.meta_key() || e.alt_key() {
            return;
        }
        let in_field = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .and_then(|el| el.closest("input, textarea, select").ok().flatten())
            .is_some();
        if in_field || js::document().query_selector("dialog[open]").ok().flatten().is_some() {
            return;
        }
        let key = e.key();
        if key == "Escape" || key.eq_ignore_ascii_case("p") {
            e.prevent_default();
            set_paused(!paused.get_untracked());
            return;
        }
        let Some(a) = arrow_of(&key) else { return };
        e.prevent_default();
        if !e.repeat() {
            press(a);
        }
    });
    let on_hide = Closure::<dyn Fn()>::new(move || {
        if js::document().visibility_state() == web_sys::VisibilityState::Hidden {
            set_paused(true);
        }
    });
    let w = js::window();
    let _ = w.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
    let _ = js::document().add_event_listener_with_callback("visibilitychange", on_hide.as_ref().unchecked_ref());
    let held = SendWrapper::new((on_key, on_hide, stop));
    on_cleanup(move || {
        let (on_key, on_hide, stop) = &*held;
        stop.set(true);
        let _ = js::window().remove_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
        let _ =
            js::document().remove_event_listener_with_callback("visibilitychange", on_hide.as_ref().unchecked_ref());
        let _ = march.try_update_value(|m| *m = None);
        if let Some(Some(h)) = ending.try_get_value() {
            h.clear();
        }
    });

    start_music();
    sfx(|a| audio::count(a, false));
    Effect::new(move |_| {
        if let Some(r) = root.get() {
            let opts = web_sys::FocusOptions::new();
            opts.set_prevent_scroll(true);
            let _ = r.focus_with_options(&opts);
            let into = web_sys::ScrollIntoViewOptions::new();
            into.set_block(web_sys::ScrollLogicalPosition::Center);
            r.scroll_into_view_with_scroll_into_view_options(&into);
        }
    });
    Effect::new(move |_| {
        if paused.get()
            && let Some(b) = resume_btn.get()
        {
            let _ = b.focus();
        }
    });

    let tone = move || current.get().and_then(by_id).map_or("var(--cta)", |s| permit_color(s.permit()));
    let announce = move || {
        current.get().and_then(by_id).map_or(String::new(), |s| {
            format!("{}: {}", s.name, s.code.iter().map(|a| a.as_str()).collect::<Vec<_>>().join(", "))
        })
    };
    view! {
        <div class="dr-run" node_ref=root tabindex="-1" aria-label="Stratagem drill" style=move || format!("--p: {};", tone())>
            <div class="dr-hud num">
                <div class="dr-hud-cell">
                    <span class="caps">"Round"</span>
                    <b>{move || round.get()}</b>
                    <span class="dr-hud-sub caps">{move || format!("{} left", left.get())}</span>
                </div>
                <div class="dr-hud-cell dr-hud-score">
                    <span class="caps">"Score"</span>
                    <b>{move || fmt_int(score.get() as f64)}</b>
                    {move || gained.get().map(|(p, _)| view! { <span class="dr-gain" aria-hidden="true">{format!("+{p}")}</span> })}
                </div>
                <div class="dr-hud-cell">
                    <span class="caps">"Streak"</span>
                    <b>{move || streak.get()}</b>
                    <span class=move || format!("dr-mult caps {}", if streak.get() >= 5 { "on" } else { "" })>
                        {move || format!("x{}", multiplier(streak.get()))}
                    </span>
                </div>
                <div class="dr-hud-cell">
                    <span class="caps">"Best"</span>
                    <b>{move || fmt_int(best_before.max(score.get()) as f64)}</b>
                </div>
                <div class="dr-hud-tools">
                    <SoundToggles />
                    <button class="dr-icon-btn" aria-label="Pause" title="Pause (Esc)" on:click=move |_| set_paused(true)>
                        <Icon icon=icons::PAUSE size=16.0 />
                    </button>
                </div>
            </div>
            <div class="dr-stage">
                <div class="dr-lane">
                    <div class="dr-now">{move || current.get().and_then(by_id).map(|s| icon_img(s, 96))}</div>
                    <ol class="dr-queue" aria-label="Up next">
                        {move || {
                            upcoming
                                .get()
                                .into_iter()
                                .filter_map(by_id)
                                .enumerate()
                                .map(|(i, s)| view! {
                                    <li style=format!("--i: {i}; --p: {};", permit_color(s.permit())) title=s.name>
                                        {icon_img(s, 48)}
                                    </li>
                                })
                                .collect_view()
                        }}
                    </ol>
                </div>
                <div class="dr-name">{move || current.get().and_then(by_id).map(|s| s.name)}</div>
                <div class="dr-code" node_ref=code_el>
                    {move || {
                        current.get().and_then(by_id).map(|s| {
                            s.code
                                .iter()
                                .enumerate()
                                .map(|(i, a)| view! {
                                    <i class=move || format!("dr-arrow {} {}", a.as_str(), if (i as u32) < keyed.get() { "on" } else { "" })></i>
                                })
                                .collect_view()
                        })
                    }}
                </div>
                <div class=move || format!("dr-fuse {}", if fuse.get() < LOW_FUSE { "low" } else { "" })>
                    <i style=move || format!("transform: scaleX({});", num(fuse.get() / FUSE))></i>
                </div>
                <span class="sr-only" aria-live="polite">{announce}</span>
                {move || match stage.get() {
                    Stage::Ready(n) => Some(view! {
                        <div class="dr-overlay">
                            <span class="caps">{format!("Round {}", round.get())}</span>
                            <b class="dr-count num">{n}</b>
                            <span class="caps">{format!("{} codes · get ready", left.get())}</span>
                        </div>
                    }
                    .into_any()),
                    Stage::Cleared => bonus.get().map(|b| {
                        let flawless = if b.flawless > 0 { "" } else { "missed" };
                        view! {
                        <div class="dr-overlay">
                            <span class="caps">{format!("Round {} cleared", b.round)}</span>
                            <dl class="dr-bonus num">
                                <div>
                                    <dt>"Round bonus"</dt>
                                    <dd>{format!("+{}", b.clear)}</dd>
                                </div>
                                <div>
                                    <dt>"Time bonus"</dt>
                                    <dd>{format!("+{}", b.time)}</dd>
                                </div>
                                <div class=flawless>
                                    <dt>"Flawless bonus"</dt>
                                    <dd>{if b.flawless > 0 { format!("+{}", b.flawless) } else { "Missed".into() }}</dd>
                                </div>
                            </dl>
                        </div>
                        }
                        .into_any()
                    }),
                    Stage::Over => Some(view! {
                        <div class="dr-overlay over">
                            <b class="dr-count caps">"Time's up"</b>
                        </div>
                    }
                    .into_any()),
                    Stage::Play => None,
                }}
                <Show when=move || paused.get()>
                    <div class="dr-overlay paused" role="dialog" aria-label="Paused">
                        <b class="dr-count caps">"Paused"</b>
                        <div class="dr-go">
                            <button class="cta-btn" node_ref=resume_btn on:click=move |_| set_paused(false)>
                                <Icon icon=icons::PLAY size=15.0 stroke_width=2.0 />
                                "Resume"
                            </button>
                            <button class="dr-text-btn" on:click=move |_| quit.run(())>
                                "Quit to the lobby"
                            </button>
                        </div>
                    </div>
                </Show>
            </div>
            <div class="dr-pad" role="group" aria-label="Stratagem keys">
                {PAD
                    .map(|(a, name)| view! {
                        <button
                            class=format!("dr-pad-key {}", a.as_str())
                            aria-label=name
                            on:pointerdown=move |e| {
                                e.prevent_default();
                                press(a);
                            }
                            on:click=move |e| {
                                if e.detail() == 0 {
                                    press(a);
                                }
                            }
                        >
                            <i class=format!("hp-key on {}", a.as_str())></i>
                        </button>
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

// ---------- Results ----------

#[component]
fn Results(f: Finished, again: Callback<()>, lobby: Callback<()>) -> impl IntoView {
    let again_btn = NodeRef::<html::Button>::new();
    Effect::new(move |_| {
        if let Some(b) = again_btn.get() {
            let _ = b.focus();
        }
    });
    let s = &f.score;
    let place = match f.place {
        Some(1) => "Top of this device's board.".to_string(),
        Some(n) => format!("Number {n} on this device's board."),
        None => "Off the board: this device's best ten are higher.".to_string(),
    };
    view! {
        <div class="dr-results">
            <div class=format!("dr-final {}", if f.was_best { "best" } else { "" })>
                <span class="caps">{if f.was_best { "New best" } else { "Drill over" }}</span>
                <b class="dr-final-score num">{fmt_int(s.score as f64)}</b>
                <span class="dr-final-pool">{format!("{} · {}", s.mode.label(), s.pool)}</span>
            </div>
            <dl class="dr-final-stats num">
                <div>
                    <dt>"Round reached"</dt>
                    <dd>{s.round}</dd>
                </div>
                <div>
                    <dt>"Codes keyed"</dt>
                    <dd>{s.called}</dd>
                </div>
                <div>
                    <dt>"Best streak"</dt>
                    <dd>{f.best_streak}</dd>
                </div>
                <div>
                    <dt>"Accuracy"</dt>
                    <dd>{format!("{}%", (f.accuracy * 100.0).round())}</dd>
                </div>
            </dl>
            <p class="dr-note">{place}</p>
            <div class="dr-go">
                <button class="cta-btn" node_ref=again_btn on:click=move |_| again.run(())>
                    <Icon icon=icons::ROTATE_CW size=15.0 stroke_width=2.0 />
                    "Drill again"
                </button>
                <button class="dr-text-btn" on:click=move |_| lobby.run(())>
                    "Change mode"
                </button>
                <SoundToggles />
            </div>
            <Board mode=Signal::stored(s.mode) mark=s.at />
        </div>
    }
}
