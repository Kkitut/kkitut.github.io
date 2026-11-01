use leptos::{ev, prelude::*};
use wasm_bindgen::prelude::*;

use crate::component::{Footer, Header};

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_namespace = Date)]
    fn now() -> f64;
}

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
}

fn cam_frame() {
    CAM.with(|c| {
        let mut slot = c.borrow_mut();
        let Some(st) = slot.as_mut() else { return };
        let locked = document().pointer_lock_element().is_some();
        if !locked {
            let (mx, my) = MOUSE.with(|m| m.get());
            st.tgt[3] = (-my * 0.21).clamp(-1.05, 1.05);
            st.tgt[4] = -mx * 0.28;
        }
        // camera-space movement (FPS: yaw -= dx, pitch -= dy)
        let (sy, cy) = st.tgt[4].sin_cos();
        let (sp, cp) = st.tgt[3].sin_cos();
        let step = 7.0;
        if st.keys.contains("KeyW") {
            st.tgt[0] += -sy * cp * step;
            st.tgt[1] += sp * step;
            st.tgt[2] += -cy * cp * step;
        }
        if st.keys.contains("KeyS") {
            st.tgt[0] += sy * cp * step;
            st.tgt[1] += -sp * step;
            st.tgt[2] += cy * cp * step;
        }
        if st.keys.contains("KeyD") {
            st.tgt[0] += cy * step;
            st.tgt[2] += -sy * step;
        }
        if st.keys.contains("KeyA") {
            st.tgt[0] += -cy * step;
            st.tgt[2] += sy * step;
        }
        if st.keys.contains("Space") {
            st.tgt[1] += step;
        }
        if st.keys.contains("KeyC") {
            st.tgt[1] -= step;
        }
        for i in 0..5 {
            st.cur[i] += (st.tgt[i] - st.cur[i]) * 0.12;
        }
        // view matrix = R^T * T(-pos), pivoted about screen center:
        // t = D - R^T * (D + C), D = viewport center
        let (sy, cy) = st.cur[4].sin_cos();
        let (sp, cp) = st.cur[3].sin_cos();
        let (px, py, pz) = (st.cur[0], st.cur[1], st.cur[2]);
        let vw = window().inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        let vh = window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
        let (dx, dy) = (vw * 0.5, vh * 0.5);
        let (qx, qy, qz) = (dx + px, dy + py, pz);
        let t0 = dx - (cy * qx + -sy * qz);
        let t1 = dy - (sy * sp * qx + cp * qy + cy * sp * qz);
        let t2 = -(sy * cp * qx + -sp * qy + cy * cp * qz);
        if let Some(el) = document().get_element_by_id("app") {
            let el: leptos::web_sys::HtmlElement = el.unchecked_into();
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
        });
    });
    if let Some(el) = document().get_element_by_id("app") {
        let el: leptos::web_sys::HtmlElement = el.unchecked_into();
        el.request_pointer_lock();
    }
    cam_frame();
}

fn cam_stop() {
    let _ = document().exit_pointer_lock();
    CAM.with(|p| {
        if let Some(st) = p.borrow_mut().take() {
            let _ = window().cancel_animation_frame(st.id);
            if let Some(el) = document().get_element_by_id("app") {
                let el: leptos::web_sys::HtmlElement = el.unchecked_into();
                let _ = el.style().set_property("transform", "");
            }
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
            let _ = b.style().set_property(
                "background",
                if party.get() { "#000" } else { "" },
            );
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

    window_event_listener(ev::click, move |_| {
        if !party.get() {
            return;
        }
        if document().pointer_lock_element().is_some() {
            return;
        }
        if let Some(el) = document().get_element_by_id("app") {
            let el: leptos::web_sys::HtmlElement = el.unchecked_into();
            el.request_pointer_lock();
        }
    });

    window_event_listener(ev::mousemove, move |e| {
        if party.get() && document().pointer_lock_element().is_some() {
            CAM.with(|c| {
                if let Some(st) = c.borrow_mut().as_mut() {
                    st.tgt[4] -= e.movement_x() as f32 * 0.0025;
                    st.tgt[3] =
                        (st.tgt[3] - e.movement_y() as f32 * 0.0025).clamp(-1.05, 1.05);
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
            </div>
        </>
    }
}
