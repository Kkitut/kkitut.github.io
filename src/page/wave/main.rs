use leptos::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, closure::Closure};

const SPAN: f64 = 500.0;
const SEGMENTS: usize = 220;

thread_local! {
    static WHEEL_CB: RefCell<Option<Closure<dyn FnMut(leptos::web_sys::WheelEvent)>>> =
        const { RefCell::new(None) };
}

fn out_back(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    1.0 + 2.70158 * t.powi(3) + 1.70158 * t.powi(2)
}

fn out_circ(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    (1.0 - t.powi(2)).sqrt()
}

fn in_out_circ(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        0.5 * (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt())
    } else {
        0.5 * ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0)
    }
}

#[component]
pub fn Wave() -> impl IntoView {
    let (center_r, set_center_r) = signal(0.0f64);
    let (center_op, set_center_op) = signal(0.0f64);
    let (line_span, set_line_span) = signal(0.0f64);
    let (line_op, set_line_op) = signal(0.0f64);
    let (wave_t, set_wave_t) = signal(0.0f64);
    let (wave_y, set_wave_y) = signal(0.0f64);

    let (en_y, set_en_y) = signal(150.0f64);
    let (en_scale, set_en_scale) = signal(1.0f64);
    let (en_op, set_en_op) = signal(0.0f64);

    let (ko_y, set_ko_y) = signal(160.0f64);
    let (ko_op, set_ko_op) = signal(0.0f64);

    let (arrow_y, set_arrow_y) = signal(12.0f64);
    let (arrow_op, set_arrow_op) = signal(0.0f64);

    let (revealed, set_revealed) = signal(false);
    let (reveal_at_ms, set_reveal_at_ms) = signal(None::<f64>);
    let (show_hint, set_show_hint) = signal(false);

    // Page 2 -> Title promotion state when clicking "파동이라 함은 무엇일까."
    let (promoted, set_promoted) = signal(false);
    let (promote_start_ms, set_promote_start_ms) = signal(None::<f64>);
    let (promote_from, set_promote_from) = signal((0.0f64, 0.0f64));
    let (promote_u, set_promote_u) = signal(0.0f64);
    let (dot_gone, set_dot_gone) = signal(false);

    let (raf_id, set_raf_id) = signal(None::<i32>);

    Effect::new(move |_| {
        let start = Rc::new(Cell::new(None::<f64>));
        let cur_scroll = Rc::new(Cell::new(window().scroll_y().unwrap_or(0.0)));
        let tgt_scroll = Rc::new(Cell::new(cur_scroll.get()));
        let wheeling = Rc::new(Cell::new(false));

        let tgt_wheel = tgt_scroll.clone();
        let cur_wheel = cur_scroll.clone();
        let wheeling_flag = wheeling.clone();

        let wheel_cb = Closure::<dyn FnMut(leptos::web_sys::WheelEvent)>::new(
            move |ev: leptos::web_sys::WheelEvent| {
                ev.prevent_default();
                let vh = window()
                    .inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let doc_h = document()
                    .document_element()
                    .map(|el| el.scroll_height() as f64)
                    .unwrap_or(0.0);
                let max_scroll = (doc_h - vh).max(0.0);

                if !wheeling_flag.get() {
                    let sy = window().scroll_y().unwrap_or(0.0);
                    cur_wheel.set(sy);
                    tgt_wheel.set(sy);
                    wheeling_flag.set(true);
                }

                let next = (tgt_wheel.get() + ev.delta_y()).clamp(0.0, max_scroll);
                tgt_wheel.set(next);
            },
        );

        let opts = leptos::web_sys::AddEventListenerOptions::new();
        opts.set_passive(false);
        let _ = window().add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            wheel_cb.as_ref().unchecked_ref(),
            &opts,
        );
        WHEEL_CB.with(|slot| *slot.borrow_mut() = Some(wheel_cb));

        let cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let cb_clone = cb.clone();

        *cb.borrow_mut() = Some(Closure::new(move |now: f64| {
            let t0 = start.get().unwrap_or_else(|| {
                start.set(Some(now));
                now
            });
            let t = (now - t0) * 0.001;

            // Smooth scroll interpolation
            if wheeling.get() {
                let c = cur_scroll.get();
                let tg = tgt_scroll.get();
                let diff = tg - c;
                if diff.abs() > 0.35 {
                    let next = c + diff * 0.12;
                    cur_scroll.set(next);
                    window().scroll_to_with_x_and_y(0.0, next);
                } else {
                    cur_scroll.set(tg);
                    window().scroll_to_with_x_and_y(0.0, tg);
                    wheeling.set(false);
                }
            }

            // Trigger prologue animation 0.3s after scrolling down by a full screen height (~100dvh)
            if !revealed.get_untracked() {
                let vh = window()
                    .inner_height()
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let sy = window().scroll_y().unwrap_or(0.0);
                if sy >= vh * 0.94 {
                    let trigger_ms = reveal_at_ms.get_untracked().unwrap_or_else(|| {
                        let target = now + 300.0;
                        set_reveal_at_ms.set(Some(target));
                        target
                    });
                    if now >= trigger_ms {
                        set_revealed.set(true);
                    }
                }
            } else if !promoted.get_untracked() && !show_hint.get_untracked() {
                if let Some(trigger_ms) = reveal_at_ms.get_untracked() {
                    // 10s after "파동이라 함은 무엇일까." finishes appearing (2.75s delay + 0.95s anim + 10.0s = 13.7s)
                    if now - trigger_ms >= 13_700.0 {
                        set_show_hint.set(true);
                    }
                }
            }

            if promoted.get_untracked() {
                let p0 = promote_start_ms.get_untracked().unwrap_or_else(|| {
                    set_promote_start_ms.set(Some(now));
                    now
                });
                let elapsed = (now - p0) * 0.001;
                set_promote_u.set(in_out_circ(elapsed / 1.05));
                if elapsed >= 1.15 && !dot_gone.get_untracked() {
                    set_dot_gone.set(true);
                }
            }

            let base_r = 4.0;

            if t < 1.0 {
                set_center_r.set(0.0);
                set_center_op.set(0.0);
                set_line_op.set(0.0);
            } else if t < 1.3 {
                set_center_r.set(base_r * out_back((t - 1.0) / 0.3));
                set_center_op.set(1.0);
            } else if t < 1.5 {
                set_center_r.set(base_r);
                set_center_op.set(1.0);
            } else if t < 2.6 {
                let u = out_circ((t - 1.5) / 1.1);
                set_center_r.set(base_r * (1.0 + 5.0 * u));
            } else if t < 2.8 {
                set_center_r.set(base_r * 6.0);
            } else if t < 3.8 {
                let u = out_circ((t - 2.8) / 1.0);
                set_center_r.set(base_r * (6.0 - 5.0 * u));
            } else if t < 4.55 {
                let u = out_circ((t - 3.8) / 0.75);
                set_center_r.set(base_r);
                set_center_op.set(0.0);
                set_line_span.set(u);
                set_line_op.set(1.0);
            } else {
                let wt = t - 4.55;
                set_center_op.set(0.0);
                set_line_span.set(1.0);
                set_line_op.set(1.0);
                set_wave_t.set(wt);

                if t < 7.0 {
                    set_wave_y.set(0.0);
                    set_en_op.set(0.0);
                    set_ko_op.set(0.0);
                } else if t < 8.6 {
                    let u = in_out_circ((t - 7.0) / 1.6);
                    set_wave_y.set(0.0);
                    set_en_y.set(190.0 - 72.0 * u);
                    set_en_scale.set(1.0);
                    set_en_op.set(u);
                    set_ko_op.set(0.0);
                } else if t < 9.96 {
                    let u = in_out_circ((t - 8.6) / 1.36);
                    set_wave_y.set(-122.0 * u);
                    set_en_y.set(118.0 * (1.0 - u));
                    set_en_scale.set(1.0);
                    set_en_op.set(1.0);
                    set_ko_op.set(0.0);
                } else {
                    let p = ((t - 9.96) / 1.2).clamp(0.0, 1.0);
                    let ub = out_back(p);
                    let u = in_out_circ(p);

                    set_wave_y.set(-122.0 - 44.0 * ub);
                    set_en_y.set(-48.0 * ub);
                    set_en_scale.set(1.0 - 0.52 * u);
                    set_en_op.set(1.0 - 0.55 * u);

                    set_ko_y.set(150.0 * (1.0 - u) + 20.0 * u);
                    set_ko_op.set(u);

                    if t > 11.3 {
                        let at = t - 11.3;
                        let fade = in_out_circ((at / 1.5).clamp(0.0, 1.0));
                        let breath = (at * 1.8).sin();
                        set_arrow_op.set(fade * (0.45 + 0.2 * breath));
                        set_arrow_y.set((1.0 - fade) * 14.0 + breath * 7.0);
                    } else {
                        set_arrow_op.set(0.0);
                    }
                }
            }

            if let Some(f) = cb_clone.borrow().as_ref() {
                if let Ok(id) = window().request_animation_frame(f.as_ref().unchecked_ref()) {
                    set_raf_id.set(Some(id));
                }
            }
        }));

        if let Some(f) = cb.borrow().as_ref() {
            if let Ok(id) = window().request_animation_frame(f.as_ref().unchecked_ref()) {
                set_raf_id.set(Some(id));
            }
        }
    });

    on_cleanup(move || {
        if let Some(id) = raf_id.get_untracked() {
            let _ = window().cancel_animation_frame(id);
        }
        WHEEL_CB.with(|slot| {
            if let Some(cb) = slot.borrow_mut().take() {
                let _ = window().remove_event_listener_with_callback(
                    "wheel",
                    cb.as_ref().unchecked_ref(),
                );
            }
        });
    });

    let on_question_click = move |_| {
        if !revealed.get_untracked() || promoted.get_untracked() {
            return;
        }
        if let (Some(sec), Some(btn)) = (
            document().get_element_by_id("wave-slide-2"),
            document().get_element_by_id("wave-question-btn"),
        ) {
            let sr = sec.get_bounding_client_rect();
            let br = btn.get_bounding_client_rect();
            let cur_cx = (br.left() - sr.left()) + br.width() * 0.5;
            let cur_cy = (br.top() - sr.top()) + br.height() * 0.5;
            let tgt_cx = sr.width() * 0.5;
            let tgt_cy = 96.0;
            set_promote_from.set((cur_cx - tgt_cx, cur_cy - tgt_cy));
        }
        set_promoted.set(true);
    };

    let path_d = move || {
        let wt = wave_t.get();
        let span = line_span.get();
        let ext = (SPAN + 12.0) * span;

        if wt <= 0.0 {
            return format!("M {:.1} 0.0 L {:.1} 0.0", -ext, ext);
        }

        let mut d = String::with_capacity(SEGMENTS * 18);

        for i in 0..=SEGMENTS {
            let norm = i as f64 / SEGMENTS as f64;
            let x = -ext + norm * (ext * 2.0);

            let lt = wt - norm * 1.1;

            let y = if lt <= 0.0 {
                0.0
            } else {
                let attack = out_circ((lt / 0.225).clamp(0.0, 1.0));
                let settle = out_circ(((lt - 0.175) / 1.15).clamp(0.0, 1.0));
                let surge = attack * (1.0 - settle);

                let amp = (36.0 + 108.0 * surge) * attack;
                let local_bunch = 2.4 * surge;
                let phase = (norm - 0.5) * std::f64::consts::TAU * 3.96
                    - local_bunch
                    - wt * 3.36;

                -amp * phase.sin()
            };

            if i == 0 {
                d.push_str(&format!("M {x:.1} {y:.1}"));
            } else {
                d.push_str(&format!(" L {x:.1} {y:.1}"));
            }
        }
        d
    };

    view! {
        <>
            <link rel="stylesheet" href="/style/page/wave/main.css"/>

            <main class="wave-main">
                <section class="wave-intro">
                    <svg
                        class="wave-stage"
                        viewBox="-500 -160 1000 320"
                        aria-label="파동 도입 애니메이션"
                        style:transform=move || format!("translateY({:.2}px)", wave_y.get())
                    >
                        <circle
                            cx="0"
                            cy="0"
                            r=move || format!("{:.2}", center_r.get())
                            fill="#fff"
                            opacity=move || format!("{:.3}", center_op.get())
                        />

                        <path
                            d=path_d
                            fill="none"
                            stroke="#fff"
                            stroke-width="8"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            opacity=move || format!("{:.3}", line_op.get())
                        />
                    </svg>

                    <div
                        class="wave-title-en"
                        style:transform=move || {
                            format!(
                                "translate(-50%, calc(-50% + {:.2}px)) scale({:.3})",
                                en_y.get(),
                                en_scale.get()
                            )
                        }
                        style:opacity=move || format!("{:.3}", en_op.get())
                    >
                        "Wave"
                    </div>

                    <h1
                        class="wave-title-ko"
                        style:transform=move || {
                            format!("translate(-50%, calc(-50% + {:.2}px))", ko_y.get())
                        }
                        style:opacity=move || format!("{:.3}", ko_op.get())
                    >
                        "파동"
                    </h1>

                    <div
                        class="wave-scroll-arrow"
                        style:transform=move || {
                            format!("translate(-50%, calc(-50% + {:.2}px))", arrow_y.get())
                        }
                        style:opacity=move || format!("{:.3}", arrow_op.get())
                        aria-hidden="true"
                    >
                        "↓"
                    </div>
                </section>

                <section
                    id="wave-slide-2"
                    class="wave-prologue"
                    class:in=move || revealed.get()
                    class:promoted=move || promoted.get()
                >
                    <div class="wave-prologue-inner">
                        <p class="wave-fade wave-statement">
                            "현대에서 전파는 없어서는 안될 중요한 핵심으로 자리잡았다."
                        </p>

                        <p class="wave-fade wave-turn" style="transition-delay: 0.9s">
                            "그런데..."
                        </p>

                        <p class="wave-fade wave-ask" style="transition-delay: 1.85s">
                            "그러한 전파는 어떤 것일까."
                        </p>

                        <div class="wave-fade wave-question-slot" style="transition-delay: 2.75s">
                            <button
                                id="wave-question-btn"
                                type="button"
                                class="wave-question-btn"
                                class:is-title=move || promoted.get()
                                on:click=on_question_click
                                style:transform=move || {
                                    if !promoted.get() {
                                        String::new()
                                    } else {
                                        let (dx, dy) = promote_from.get();
                                        let u = promote_u.get();
                                        let x = dx * (1.0 - u);
                                        let y = dy * (1.0 - u);
                                        format!("translate(calc(-50% + {x:.1}px), calc(-50% + {y:.1}px))")
                                    }
                                }
                            >
                                <span class="wave-question-ring" aria-hidden="true"/>
                                <span
                                    class="wave-question-text"
                                    style:font-size=move || {
                                        if !promoted.get() {
                                            String::new()
                                        } else {
                                            let s = 1.0 + promote_u.get();
                                            format!("calc(clamp(16px, 2.4vw, 26px) * {s:.3})")
                                        }
                                    }
                                >
                                    "파동이라 함은 무엇일까"
                                    <span class="wave-question-dot" class:gone=move || dot_gone.get()>
                                        "."
                                    </span>
                                </span>
                            </button>

                            <p
                                class="wave-click-hint"
                                class:show=move || show_hint.get() && !promoted.get()
                            >
                                "클릭해 보세요 . ."
                            </p>
                        </div>
                    </div>
                </section>
            </main>
        </>
    }
}
