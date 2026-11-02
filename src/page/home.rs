use leptos::web_sys::HtmlElement;
use leptos::{ev, prelude::*};
use std::f32::consts::FRAC_1_SQRT_2;
use wasm_bindgen::prelude::*;

use crate::component::{Footer, Header};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date)]
    fn now() -> f64;
}

// Must match home.css: body perspective, .gl-back translateZ and max size.
const PERSP: f32 = 1000.0;
const SPRITE_Z: f32 = 4800.0;
const SPRITE_VMIN: f32 = 0.6;
const SPRITE_PX: f32 = 1280.0;

thread_local! {
    static CAM: std::cell::RefCell<Option<CamState>> =
        const { std::cell::RefCell::new(None) };
    static MOUSE: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0.0, 0.0)) };
}

struct CamState {
    id: i32,
    tick: Closure<dyn Fn()>,
    keys: std::collections::HashSet<String>,
    cur: [f32; 5],
    tgt: [f32; 5],
    lw: f32,
    lh: f32,
    back_vis: bool,
}

fn axes(pitch: f32, yaw: f32) -> [[f32; 3]; 2] {
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    [[-sy * cp, sp, -cy * cp], [cy, 0.0, -sy]]
}

fn set_style(id: &str, prop: &str, val: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        let el: HtmlElement = el.unchecked_into();
        let _ = el.style().set_property(prop, val);
    }
}

fn cam_frame() {
    CAM.with(|c| {
        let mut slot = c.borrow_mut();
        let Some(st) = slot.as_mut() else { return };
        let locked = document().pointer_lock_element().is_some();
        if !locked {
            let (mx, my) = MOUSE.with(|m| m.get());
            st.tgt[3] = (my * 0.21).clamp(-1.05, 1.05);
            st.tgt[4] = -mx * 0.28;
        }
        // camera-space FPS movement
        let [go, side] = axes(st.tgt[3], st.tgt[4]);
        let step = 7.0;
        if st.keys.contains("KeyW") {
            st.tgt[0] += go[0] * step;
            st.tgt[1] += go[1] * step;
            st.tgt[2] += go[2] * step;
        }
        if st.keys.contains("KeyS") {
            st.tgt[0] -= go[0] * step;
            st.tgt[1] -= go[1] * step;
            st.tgt[2] -= go[2] * step;
        }
        if st.keys.contains("KeyD") {
            st.tgt[0] += side[0] * step;
            st.tgt[2] += side[2] * step;
        }
        if st.keys.contains("KeyA") {
            st.tgt[0] -= side[0] * step;
            st.tgt[2] -= side[2] * step;
        }
        if st.keys.contains("Space") {
            st.tgt[1] -= step;
        }
        if st.keys.contains("KeyC") {
            st.tgt[1] += step;
        }
        // no easing: 1:1 response (CSS handles party on/off)
        let vw = window().inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        let vh = window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        let mut changed = vw != st.lw || vh != st.lh;
        for i in 0..5 {
            if st.cur[i] != st.tgt[i] {
                changed = true;
                break;
            }
        }
        if changed {
            st.cur = st.tgt;
            st.lw = vw;
            st.lh = vh;
            // View matrix V(X) = M * (X - C) + D: rotation M (columns
            // below), translated about the eye D. Culling replicates
            // M * S + t exactly (rows, not R^T — t carries the pivot).
            let (sy, cy) = st.cur[4].sin_cos();
            let (sp, cp) = st.cur[3].sin_cos();
            let (px, py, pz) = (st.cur[0], st.cur[1], st.cur[2]);
            let (ex, ey, ez) = (vw * 0.5, vh * 0.5, PERSP);
            let (qx, qy, qz) = (ex + px, ey + py, ez + pz);
            let t0 = ex - (cy * qx + -sy * qz);
            let t1 = ey - (sy * sp * qx + cp * qy + cy * sp * qz);
            let t2 = ez - (sy * cp * qx + -sp * qy + cy * cp * qz);
            if let Some(el) = document().get_element_by_id("app") {
                let el: HtmlElement = el.unchecked_into();
                let _ = el.style().set_property(
                    "transform",
                    &format!(
                        "matrix3d({:.5},{:.5},{:.5},0,{:.5},{:.5},{:.5},0,{:.5},{:.5},{:.5},0,{:.1},{:.1},{:.1},1)",
                        cy, sy * sp, sy * cp,
                        0.0, cp, -sp,
                        -sy, cy * sp, cy * cp,
                        t0, t1, t2
                    ),
                );
                // Manual frustum culling: the sprite is double-sided, so
                // the browser can't cull it. Exact sphere-vs-frustum test
                // with the camera as a pinhole at the eye.
                let sr = (vw.min(vh) * SPRITE_VMIN).min(SPRITE_PX) * FRAC_1_SQRT_2 + 1.0;
                let sw = el.client_width() as f32 * 0.5;
                let sh = el.client_height() as f32 * 0.5;
                let vx = cy * sw - sy * SPRITE_Z + t0;
                let vy = sy * sp * sw + cp * sh + cy * sp * SPRITE_Z + t1;
                let vz = sy * cp * sw - sp * sh + cy * cp * SPRITE_Z + t2;
                // frustum planes through the eye; slopes from the
                // perspective origin (body center), clipped to viewport
                let (ox, oy) = document().body().map(|b| (b.client_width() as f32 * 0.5, b.client_height() as f32 * 0.5)).unwrap_or((ex, ey));
                let ax = ox / PERSP;
                let bx = (vw - ox) / PERSP;
                let ay = oy / PERSP;
                let by = (vh - oy) / PERSP;
                let vis = vz - PERSP < sr
                    && (vx - ox - ax * (vz - PERSP)) / (1.0 + ax * ax).sqrt() > -sr
                    && (ox - vx - bx * (vz - PERSP)) / (1.0 + bx * bx).sqrt() > -sr
                    && (vy - oy - ay * (vz - PERSP)) / (1.0 + ay * ay).sqrt() > -sr
                    && (oy - vy - by * (vz - PERSP)) / (1.0 + by * by).sqrt() > -sr;
                if vis != st.back_vis {
                    st.back_vis = vis;
                    // visibility (not display): no relayout, keeps the decoded image
                    set_style("gl-back", "visibility", if vis { "" } else { "hidden" });
                }
            }
        }
        if let Ok(id) =
            window().request_animation_frame(st.tick.as_js_value().unchecked_ref())
        {
            st.id = id;
        }
    });
}

fn cam_start() {
    CAM.with(|c| {
        if c.borrow().is_some() {
            return;
        }
        let tick: Closure<dyn Fn()> = Closure::own(cam_frame);
        *c.borrow_mut() = Some(CamState {
            id: -1,
            tick,
            keys: std::collections::HashSet::new(),
            cur: [0.0; 5],
            tgt: [0.0; 5],
            lw: 0.0,
            lh: 0.0,
            back_vis: true,
        });
    });
    if let Some(el) = document().get_element_by_id("app") {
        let el: HtmlElement = el.unchecked_into();
        el.request_pointer_lock();
    }
    cam_frame();
}

fn cam_stop() {
    let _ = document().exit_pointer_lock();
    CAM.with(|p| {
        if let Some(st) = p.borrow_mut().take() {
            let _ = window().cancel_animation_frame(st.id);
            set_style("app", "transform", "");
            set_style("gl-back", "visibility", "");
        }
    });
}

#[component]
pub fn Home() -> impl IntoView {
    let (tilt, set_tilt) = signal((0.0f64, 0.0f64));
    let (shift, set_shift) = signal(false);
    let (party, set_party) = signal(false);
    let presses = RwSignal::new(Vec::<f64>::new());

    Effect::new(move |_| {
        if let Some(b) = document().body() {
            let _ = b
                .style()
                .set_property("background", if party.get() { "#000" } else { "" });
        }
    });

    on_cleanup(cam_stop);

    window_event_listener(ev::keydown, move |e| {
        if e.key() == "Shift" {
            set_shift.set(true);
            if !e.repeat() {
                let t = now();
                presses.update(|v| {
                    v.push(t);
                    v.retain(|p| t - p < 1000.0);
                    if v.len() >= 6 {
                        v.clear();
                        let on = !party.get();
                        set_party.set(on);
                        if on {
                            cam_start();
                        } else {
                            cam_stop();
                        }
                    }
                });
            }
        }
        CAM.with(|f| {
            if let Some(st) = f.borrow_mut().as_mut() {
                if e.code() == "Space" {
                    e.prevent_default();
                }
                st.keys.insert(e.code());
            }
        });
    });

    window_event_listener(ev::keyup, move |e| {
        if e.key() == "Shift" {
            set_shift.set(false);
        }
        CAM.with(|f| {
            if let Some(st) = f.borrow_mut().as_mut() {
                st.keys.remove(&e.code());
            }
        });
    });

    window_event_listener(ev::wheel, move |e| {
        if !party.get() {
            return;
        }
        CAM.with(|f| {
            if let Some(st) = f.borrow_mut().as_mut() {
                let [go, _] = axes(st.tgt[3], st.tgt[4]);
                let d = -(e.delta_y() as f32) * 0.4;
                st.tgt[0] += go[0] * d;
                st.tgt[1] += go[1] * d;
                st.tgt[2] += go[2] * d;
            }
        });
    });

    window_event_listener(ev::click, move |_| {
        if !party.get() {
            return;
        }
        if document().pointer_lock_element().is_some() {
            return;
        }
        if let Some(el) = document().get_element_by_id("app") {
            let el: HtmlElement = el.unchecked_into();
            el.request_pointer_lock();
        }
    });

    window_event_listener(ev::mousemove, move |e| {
        if party.get() && document().pointer_lock_element().is_some() {
            CAM.with(|c| {
                if let Some(st) = c.borrow_mut().as_mut() {
                    st.tgt[4] -= e.movement_x() as f32 * 0.0025;
                    st.tgt[3] = (st.tgt[3] + e.movement_y() as f32 * 0.0025).clamp(-1.05, 1.05);
                }
            });
            return;
        }
        let w = window()
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let h = window()
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let x = (f64::from(e.client_x()) / w - 0.5) * 2.0;
        let y = (f64::from(e.client_y()) / h - 0.5) * 2.0;
        MOUSE.with(|m| m.set((x as f32, y as f32)));
        set_tilt.set((x, y));
    });

    let style = move || {
        if party.get() {
            return String::from("transform: translateZ(90px) scale(0.91);");
        }
        let (x, y) = tilt.get();
        let (rx, ry) = if shift.get() {
            (-y * 36.0, x * 36.0)
        } else {
            (0.0, 0.0)
        };
        format!("transform: perspective(700px) rotateX({rx:.2}deg) rotateY({ry:.2}deg);")
    };

    view! {
        <>
            <link rel="stylesheet" href="/style/common.css"/>
            <link rel="stylesheet" href="/style/header.css"/>
            <link rel="stylesheet" href="/style/footer.css"/>
            <link rel="stylesheet" href="/style/page/home.css"/>

            <div id="app" class=move || if party.get() { "home-party" } else { "" }>
                <Header/>

                <main class="home-main">
                    <img
                        src="/res/image/Kkitut_NL.svg"
                        alt="Kkitut"
                        class="home-logo"
                        style=style
                    />
                </main>

                <Footer/>

                <img
                    id="gl-back"
                    class="gl-back"
                    src="/res/image/kkitut_ks.png"
                    loading="lazy"
                    alt=""
                />
            </div>
        </>
    }
}
