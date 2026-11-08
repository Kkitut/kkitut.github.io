use leptos::{html::Canvas, prelude::*};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast};

const SPAN: f64 = 500.0;
const SEGMENTS: usize = 220;

const SIM_W: usize = 180;
const SIM_H: usize = 400;

struct WaveSim {
    width: usize,
    height: usize,
    buf1: Vec<f32>,
    buf2: Vec<f32>,
    damping: Vec<f32>,
    pixels: Vec<u8>,
}

impl WaveSim {
    fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let margin_y = 16;
        let mut damping = vec![0.994f32; size];

        for y in 0..height {
            let dist_y = y.min(height - 1 - y);
            let damp_y = if dist_y < margin_y {
                let factor = dist_y as f32 / margin_y as f32;
                0.75 + 0.244 * factor
            } else {
                0.994
            };
            for x in 0..width {
                // No margin on left and right (좌우 벽끝에서 튕기게, 마진 없이)
                damping[y * width + x] = damp_y;
            }
        }

        Self {
            width,
            height,
            buf1: vec![0.0; size],
            buf2: vec![0.0; size],
            damping,
            pixels: vec![0; size * 4],
        }
    }

    fn touch(&mut self, cx: usize, cy: usize, radius: usize, strength: f32) {
        let r_i = radius as isize;
        let w = self.width as isize;
        let h = self.height as isize;
        let cx_i = cx as isize;
        let cy_i = cy as isize;

        for dy in -r_i..=r_i {
            for dx in -r_i..=r_i {
                let d2 = (dx * dx + dy * dy) as f32;
                let r2 = (radius * radius) as f32;
                if d2 <= r2 {
                    let nx = cx_i + dx;
                    let ny = cy_i + dy;
                    if nx >= 0 && nx < w && ny > 0 && ny < h - 1 {
                        let idx = (ny * w + nx) as usize;
                        let weight = (-d2 / (2.0 * (radius as f32 * 0.45).powi(2))).exp();
                        self.buf1[idx] += strength * weight;
                    }
                }
            }
        }
    }

    fn step(&mut self) {
        let w = self.width;
        let h = self.height;

        for y in 1..(h - 1) {
            let row = y * w;
            for x in 0..w {
                let idx = row + x;
                // Neumann boundary reflection on left (x=0) and right (x=w-1) walls (no margin)
                let left = if x > 0 { self.buf1[idx - 1] } else { self.buf1[idx + 1] };
                let right = if x < w - 1 { self.buf1[idx + 1] } else { self.buf1[idx - 1] };
                let sum = left
                    + right
                    + self.buf1[idx - w]
                    + self.buf1[idx + w];

                // alpha = 0.22 gives ~3x slower propagation speed
                let next = 1.12 * self.buf1[idx] - self.buf2[idx] + 0.22 * sum;
                self.buf2[idx] = next * self.damping[idx];
            }
        }

        std::mem::swap(&mut self.buf1, &mut self.buf2);
    }

    fn sample(&self, gx: f32, gy: f32) -> (f32, f32, f32) {
        let x = (gx as usize).clamp(0, self.width - 1);
        let y = (gy as usize).clamp(1, self.height - 2);
        let idx = y * self.width + x;

        let height = self.buf1[idx];
        let prev_x = if x > 0 { self.buf1[idx - 1] } else { self.buf1[idx] };
        let next_x = if x < self.width - 1 { self.buf1[idx + 1] } else { self.buf1[idx] };
        let slope_x = (next_x - prev_x) * 0.5;
        let slope_y = (self.buf1[idx + self.width] - self.buf1[idx - self.width]) * 0.5;

        (height, slope_x, slope_y)
    }

    fn render_to_pixels(&mut self) {
        let w = self.width;
        let h = self.height;

        for y in 1..(h - 1) {
            let row = y * w;
            for x in 0..w {
                let idx = row + x;
                let px = idx * 4;

                let dx = if x == 0 {
                    self.buf1[idx + 1] - self.buf1[idx]
                } else if x == w - 1 {
                    self.buf1[idx] - self.buf1[idx - 1]
                } else {
                    (self.buf1[idx + 1] - self.buf1[idx - 1]) * 0.5
                };
                let dy = (self.buf1[idx + w] - self.buf1[idx - w]) * 0.5;
                let val = self.buf1[idx];

                let grad_mag = (dx * dx + dy * dy).sqrt();
                let highlight = (-0.4 * dx - 0.7 * dy).max(0.0) * 2.2;
                let energy = (grad_mag * 0.055 + highlight * 0.05 + val.abs() * 0.007).clamp(0.0, 1.0);

                if energy > 0.015 {
                    let a = (energy * 230.0) as u8;
                    let r = (energy * 180.0) as u8;
                    let g = (energy * 225.0) as u8;
                    let b = (energy * 255.0) as u8;

                    self.pixels[px] = r;
                    self.pixels[px + 1] = g;
                    self.pixels[px + 2] = b;
                    self.pixels[px + 3] = a;
                } else {
                    self.pixels[px] = 0;
                    self.pixels[px + 1] = 0;
                    self.pixels[px + 2] = 0;
                    self.pixels[px + 3] = 0;
                }
            }
        }
    }
}

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

    // 2D Wave Equation Fluid Simulator
    let sim: Rc<RefCell<WaveSim>> = Rc::new(RefCell::new(WaveSim::new(SIM_W, SIM_H)));
    let canvas_ref = NodeRef::<Canvas>::new();
    let (show_scene1_content, set_show_scene1_content) = signal(false);

    // Scene 1: 10s idle hint and arrow unlock/flash
    let (show_scene1_hint, set_show_scene1_hint) = signal(false);
    let (has_clicked_scene1, set_has_clicked_scene1) = signal(false);
    let (arrow1_unlocked, set_arrow1_unlocked) = signal(false);
    let (arrow1_flash, set_arrow1_flash) = signal(false);
    let last_a1_hit = Rc::new(Cell::new(0.0f64));

    // Scene 2: Leaf 3D transform and bobbed state
    let leaf_state = Rc::new(Cell::new((0.0f32, 0.0f32, 0.0f32)));
    let (leaf_transform, set_leaf_transform) = signal(String::from("scale(1)"));
    let (leaf_bobbed, set_leaf_bobbed) = signal(false);

    // Smooth scroll control shared with click handlers
    let (scroll_tgt, set_scroll_tgt) = signal(0.0f64);
    let (scroll_wheeling, set_scroll_wheeling) = signal(false);

    let (raf_id, set_raf_id) = signal(None::<i32>);

    let sim_effect = sim.clone();

    Effect::new(move |_| {
        let start = Rc::new(Cell::new(None::<f64>));
        let cur_scroll = Rc::new(Cell::new(window().scroll_y().unwrap_or(0.0)));
        set_scroll_tgt.set(cur_scroll.get());

        let cur_wheel = cur_scroll.clone();

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

                // Lock scroll past Scene 1 until Arrow 1 is struck by a wave
                let is_unlocked = arrow1_unlocked.get_untracked();
                let max_allowed = if is_unlocked { max_scroll } else { vh };

                if !scroll_wheeling.get_untracked() {
                    let sy = window().scroll_y().unwrap_or(0.0);
                    cur_wheel.set(sy);
                    set_scroll_tgt.set(sy);
                    set_scroll_wheeling.set(true);
                }

                let next = (scroll_tgt.get_untracked() + ev.delta_y()).clamp(0.0, max_allowed);
                set_scroll_tgt.set(next);
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

        let sim_ref = sim_effect.clone();
        let last_a1_hit_ref = last_a1_hit.clone();
        let leaf_state_ref = leaf_state.clone();

        let cb: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> = Rc::new(RefCell::new(None));
        let cb_clone = cb.clone();

        *cb.borrow_mut() = Some(Closure::new(move |now: f64| {
            let t0 = start.get().unwrap_or_else(|| {
                start.set(Some(now));
                now
            });
            let t = (now - t0) * 0.001;

            // Smooth scroll interpolation
            if scroll_wheeling.get_untracked() {
                let c = cur_scroll.get();
                let tg = scroll_tgt.get_untracked();
                let diff = tg - c;
                if diff.abs() > 0.35 {
                    let next = c + diff * 0.12;
                    cur_scroll.set(next);
                    window().scroll_to_with_x_and_y(0.0, next);
                } else {
                    cur_scroll.set(tg);
                    window().scroll_to_with_x_and_y(0.0, tg);
                    set_scroll_wheeling.set(false);
                }
            }

            let vh = window()
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);
            let cur_sy = window().scroll_y().unwrap_or(0.0);

            // Trigger prologue animation when scrolling down by a full screen height (~100dvh)
            if !revealed.get_untracked() {
                if cur_sy >= vh * 0.94 {
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

                // Show "물 위를 건드리면.." strictly after title movement finishes (1.05s)
                if elapsed >= 1.05 && !show_scene1_content.get_untracked() {
                    set_show_scene1_content.set(true);
                }

                // Scene 1: 10s idle hint if user hasn't clicked yet after scene 1 content appeared
                if show_scene1_content.get_untracked()
                    && !has_clicked_scene1.get_untracked()
                    && !show_scene1_hint.get_untracked()
                {
                    if elapsed >= 11.05 {
                        set_show_scene1_hint.set(true);
                    }
                }

                // Arrow 1 physical wave strike & unlock (at bottom of Section 2: y = 194.0)
                let (a1_h, _, _) = sim_ref.borrow().sample(SIM_W as f32 * 0.5, 194.0);
                if a1_h.abs() > 2.0 {
                    set_arrow1_flash.set(true);
                    set_arrow1_unlocked.set(true);
                    last_a1_hit_ref.set(now);
                } else if now - last_a1_hit_ref.get() > 480.0 {
                    set_arrow1_flash.set(false);
                }

                // Scene 2: Leaf pure physical response (centered in Section 3: y = 250.0)
                let (raw_h, raw_gx, raw_gy) = sim_ref.borrow().sample(SIM_W as f32 * 0.5, 250.0);
                let (prev_h, prev_gx, prev_gy) = leaf_state_ref.get();

                let h = prev_h + (raw_h - prev_h) * 0.32;
                let gx = prev_gx + (raw_gx - prev_gx) * 0.32;
                let gy = prev_gy + (raw_gy - prev_gy) * 0.32;
                leaf_state_ref.set((h, gx, gy));

                let tz = (h * 1.5).clamp(-55.0, 65.0);
                let tx = (gx * 1.8).clamp(-22.0, 22.0);
                let ty = (gy * 1.8).clamp(-22.0, 22.0);
                let rx = (-gy * 3.2).clamp(-42.0, 42.0);
                let ry = (gx * 3.2).clamp(-42.0, 42.0);
                let rz = ((gx - gy) * 1.2).clamp(-30.0, 30.0);

                set_leaf_transform.set(format!(
                    "translate3d({:.1}px, {:.1}px, {:.1}px) rotateX({:.2}deg) rotateY({:.2}deg) rotateZ({:.2}deg)",
                    tx, ty, tz, rx, ry, rz
                ));

                if raw_h.abs() > 2.0 || raw_gx.abs() > 0.8 || raw_gy.abs() > 0.8 {
                    if !leaf_bobbed.get_untracked() {
                        set_leaf_bobbed.set(true);
                    }
                }
            }

            // Intro 2D Wave animation timings
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

            // 2D Wave equation simulation step & rendering to canvas
            if let Some(canvas) = canvas_ref.get_untracked() {
                if canvas.width() != SIM_W as u32 || canvas.height() != SIM_H as u32 {
                    canvas.set_width(SIM_W as u32);
                    canvas.set_height(SIM_H as u32);
                }

                if let Ok(Some(ctx_val)) = canvas.get_context("2d") {
                    if let Ok(ctx) = ctx_val.dyn_into::<leptos::web_sys::CanvasRenderingContext2d>() {
                        if promoted.get_untracked() {
                            let mut s = sim_ref.borrow_mut();
                            s.step();
                            s.render_to_pixels();

                            if let Ok(img_data) = leptos::web_sys::ImageData::new_with_u8_clamped_array_and_sh(
                                wasm_bindgen::Clamped(&s.pixels),
                                SIM_W as u32,
                                SIM_H as u32,
                            ) {
                                let _ = ctx.put_image_data(&img_data, 0.0, 0.0);
                            }
                        } else {
                            ctx.clear_rect(0.0, 0.0, SIM_W as f64, SIM_H as f64);
                        }
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

    let on_pointer_down = {
        let sim = sim.clone();
        move |ev: leptos::web_sys::PointerEvent| {
            if !promoted.get_untracked() {
                return;
            }
            let sy = window().scroll_y().unwrap_or(0.0);
            let vh = window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(1.0);
            let vw = window().inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1.0);
            let client_y = ev.client_y() as f64;
            let world_y = client_y + sy;

            // Only allow ripple clicks in Part 2 ("물 위를 건드리면..") and Part 3 ("나뭇잎")
            if world_y < 1.0 * vh - 20.0 || world_y >= 3.0 * vh - 20.0 {
                return;
            }

            let total_h = 4.0 * vh;
            let gx = ((ev.client_x() as f64 / vw) * SIM_W as f64).clamp(0.0, (SIM_W - 1) as f64) as usize;
            let gy = ((world_y / total_h) * SIM_H as f64).clamp(0.0, (SIM_H - 1) as f64) as usize;

            sim.borrow_mut().touch(gx, gy, 4, 150.0);

            set_has_clicked_scene1.set(true);
            set_show_scene1_hint.set(false);
        }
    };

    let scroll_to_y = move |factor: f64| {
        let vh = window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(1.0);
        set_scroll_tgt.set(factor * vh);
        set_scroll_wheeling.set(true);
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

            <main class="wave-main" on:pointerdown=on_pointer_down>
                <canvas
                    node_ref=canvas_ref
                    class="wave-canvas"
                    class:active=move || promoted.get()
                />

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

                    // Scene 1: 물 위를 건드리면..
                    {move || {
                        if promoted.get() {
                            let on_arrow1_click = move |e: leptos::web_sys::MouseEvent| {
                                e.stop_propagation();
                                scroll_to_y(2.0);
                            };

                            view! {
                                <div
                                    class="wave-scene-content wave-scene-1-content"
                                    class:in=move || show_scene1_content.get()
                                >
                                    <div class="wave-scene-text">
                                        <p class="wave-lead-text">"물 위를 건드리면.."</p>
                                        <p class="wave-sub-text">"건드린 부분은 동심원을 그리며 물결이 퍼져나간다."</p>
                                        <div class="wave-hint-slot">
                                            <span
                                                class="wave-hint-text"
                                                class:in=move || show_scene1_hint.get()
                                            >
                                                "(클릭해봐요..!)"
                                            </span>
                                        </div>
                                    </div>

                                    <div
                                        class="wave-scene-arrow wave-scene-arrow-1"
                                        class:unlocked=move || arrow1_unlocked.get()
                                        class:flash=move || arrow1_flash.get()
                                        on:click=on_arrow1_click
                                        title="아래로 스크롤"
                                    >
                                        <span>"↓"</span>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span/> }.into_any()
                        }
                    }}
                </section>

                // Subsequent scenes appear once promoted
                {move || {
                    if promoted.get() {
                        let on_arrow2_click = move |e: leptos::web_sys::MouseEvent| {
                            e.stop_propagation();
                            scroll_to_y(3.0);
                        };

                        view! {
                            <>
                                // Scene 2: 나뭇잎 (Leaf)
                                <section class="wave-scene wave-scene-2">
                                    <div class="wave-scene-text">
                                        <p class="wave-lead-text">"수면 위에 나뭇잎 하나가 떠 있다고 생각해보자"</p>
                                        <p class="wave-sub-text">"물결이 지나갈 때 나뭇잎은 파도를 타고 저 멀리 떠내려갈까.."</p>
                                    </div>

                                    <div class="wave-leaf-stage">
                                        <div
                                            class="wave-leaf-anchor"
                                            style:transform=move || leaf_transform.get()
                                        >
                                            <svg
                                                class="wave-leaf-svg"
                                                viewBox="0 0 100 140"
                                                width="110"
                                                height="154"
                                                aria-label="나뭇잎"
                                            >
                                                // Outline-only minimalist leaf (no fill, no center line, bold stroke)
                                                <path
                                                    d="M 50 6 C 16 32, 16 108, 50 134 C 84 108, 84 32, 50 6 Z"
                                                    fill="none"
                                                    stroke="#ffffff"
                                                    stroke-width="4.0"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                />
                                            </svg>
                                        </div>
                                    </div>

                                    <p
                                        class="wave-reveal-text"
                                        class:in=move || leaf_bobbed.get()
                                    >
                                        "나뭇잎은 그저 제자리에서 위아래로 몇 번 출렁거릴 뿐이다."
                                    </p>

                                    <div
                                        class="wave-scene-arrow wave-scene-arrow-2"
                                        class:in=move || leaf_bobbed.get()
                                        on:click=on_arrow2_click
                                        title="아래로 스크롤"
                                    >
                                        <span>"↓"</span>
                                    </div>
                                </section>

                                // Scene 3: 본질 (이것이 파동이다)
                                <section class="wave-scene wave-scene-3">
                                    <div class="wave-scene-text wave-final-box">
                                        <p class="wave-body-line">"물도 제자리에서 흔들렸을 뿐이다."</p>
                                        <p class="wave-lead-text">"하지만 무언가는 먼 위치로 거침없이 나아간다."</p>

                                        <div class="wave-core-statement">
                                            <p class="wave-core-text">
                                                "물질은 움직이지 않았는데, 흔들림만이 공간을 넘어가는 것"
                                            </p>
                                        </div>

                                        <div class="wave-definition">
                                            <h2 class="wave-def-title">"이것이 파동이다."</h2>
                                            <p class="wave-body-line">"질량 덩어리를 던지지 않고도"</p>
                                            <p class="wave-body-line">"사건의 상태와 에너지를 저편으로 전달하는 것."</p>
                                        </div>

                                        <p class="wave-wip-notice">"(아직 쓰는 중...)"</p>
                                    </div>
                                </section>
                            </>
                        }.into_any()
                    } else {
                        view! { <span/> }.into_any()
                    }
                }}
            </main>
        </>
    }
}
