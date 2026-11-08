use leptos::web_sys::HtmlElement;
use leptos::{ev, prelude::*};
use parry3d::math::{Pose, Rotation, Vector};
use parry3d::query::contact;
use parry3d::shape::{Capsule, Cuboid};
use std::f32::consts::{FRAC_1_SQRT_2, FRAC_PI_2, PI, TAU};
use wasm_bindgen::prelude::*;

use crate::component::{Footer, Header};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Date)]
    fn now() -> f64;
}

// Must match home.css: body perspective, .gl-back translateZ and max size.
const PERSP: f32 = 740.0;
const SPRITE_Z: f32 = 4800.0;
const SPRITE_VMIN: f32 = 0.6;
const SPRITE_PX: f32 = 1280.0;

// Shooting range booth center & dimensions in #app-center-relative 3D space (2x linear scale)
const RANGE_X: f32 = -3200.0;
const RANGE_Y: f32 = 0.0;
const RANGE_Z: f32 = 3600.0;
const RANGE_W: f32 = 1680.0;
const RANGE_H: f32 = 760.0;
const RANGE_D: f32 = 2400.0;
const TARGET_Z: f32 = RANGE_Z - RANGE_D * 0.5;
const GRID_COLS: usize = 6;
const GRID_ROWS: usize = 5;
const GRID_CELLS: usize = GRID_COLS * GRID_ROWS;
const GRID_STEP: f32 = 120.0;
const TARGET_R: f32 = 46.0;

// Parry3D (Dimforge 3D collision & physics geometry) character capsule constants (units: px, seconds)
const CAPSULE_EYE_H: f32 = 220.0;
const CAPSULE_HEAD_R: f32 = 50.0;
const B2_RADIUS: f32 = 26.0;
const CAPSULE_HALF_SEG: f32 = (CAPSULE_EYE_H + CAPSULE_HEAD_R) * 0.5 - B2_RADIUS; // 109.0
const CAPSULE_CENTER_OFFSET_Y: f32 = (CAPSULE_EYE_H - CAPSULE_HEAD_R) * 0.5; // +85.0
const B2_GRAVITY: f32 = 1550.0;
const B2_JUMP_V: f32 = -810.0;
const COYOTE_TIME: f32 = 0.10;
const JUMP_BUFFER_TIME: f32 = 0.10;
const B2_MOVE_SPEED: f32 = 510.0;
const B2_GROUND_ACCEL: f32 = 14.0;
const B2_AIR_ACCEL: f32 = 6.5;
const B2_GROUND_FRICTION: f32 = 12.0;
const B2_AIR_DRAG: f32 = 1.2;
const FLOOR_EYE_Y: f32 = RANGE_Y + RANGE_H * 0.5 - CAPSULE_EYE_H;
const CEIL_EYE_Y: f32 = RANGE_Y - RANGE_H * 0.5 + CAPSULE_HEAD_R;

// Colossal Jump Map chamber far to the right of the homepage panel
const JM_X: f32 = 4800.0;
const JM_Y: f32 = -3400.0;
const JM_Z: f32 = 4800.0;
const JM_W: f32 = 6400.0;
const JM_H: f32 = 7600.0;
const JM_D: f32 = 6400.0;
const JM_FLOOR_Y: f32 = JM_Y + JM_H * 0.5;
const JM_FLOOR_EYE_Y: f32 = JM_FLOOR_Y - CAPSULE_EYE_H;
const JM_CEIL_EYE_Y: f32 = JM_Y - JM_H * 0.5 + CAPSULE_HEAD_R;
const JM_PLATS: usize = 72;
const JM_SCENERY: usize = 14;

#[derive(Clone, Copy)]
struct Plat {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    d: f32,
    h: f32,
    rx: f32,
    ry: f32,
    rz: f32,
    zone: u8,
}

fn rand_f32(rng: &mut u32) -> f32 {
    *rng = xorshift(*rng);
    (*rng as f32) * (1.0 / 4_294_967_295.0)
}

fn gen_jm_platforms(mut rng: u32) -> [Plat; JM_PLATS] {
    let mut out = [Plat {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
        d: 0.0,
        h: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: 0.0,
        zone: 0,
    }; JM_PLATS];
    let mut theta = rand_f32(&mut rng) * TAU;
    let mut cur_y = JM_FLOOR_Y;
    let mut prev_x = JM_X + theta.cos() * 1750.0;
    let mut prev_z = JM_Z + theta.sin() * 1750.0;
    let mut i = 0;
    while i < JM_PLATS {
        let zone = (i / 12) as u8;
        let is_rest = i % 12 == 11 || i + 1 == JM_PLATS;
        if i > 0 {
            theta += 0.23 + rand_f32(&mut rng) * 0.055;
            let r = 1620.0 + rand_f32(&mut rng) * 520.0;
            let tx = JM_X + theta.cos() * r;
            let tz = JM_Z + theta.sin() * r;
            let dx = tx - prev_x;
            let dz = tz - prev_z;
            let len = (dx * dx + dz * dz).sqrt().max(1.0);
            let step_d = 415.0 + rand_f32(&mut rng) * 65.0;
            prev_x = (prev_x + dx / len * step_d).clamp(JM_X - 2450.0, JM_X + 2450.0);
            prev_z = (prev_z + dz / len * step_d).clamp(JM_Z - 2450.0, JM_Z + 2450.0);
        }
        cur_y -= 92.0 + rand_f32(&mut rng) * 14.0;
        let (w, d, h) = if is_rest {
            (330.0, 330.0, 150.0)
        } else {
            let base = 205.0 - (zone as f32) * 4.0;
            (
                base + rand_f32(&mut rng) * 75.0,
                base + rand_f32(&mut rng) * 75.0,
                95.0 + rand_f32(&mut rng) * 135.0,
            )
        };
        let (rx, ry, rz) = if i + 1 == JM_PLATS {
            (0.0, rand_f32(&mut rng) * 0.8 - 0.4, 0.0)
        } else {
            (
                (rand_f32(&mut rng) - 0.5) * 0.24,
                (rand_f32(&mut rng) - 0.5) * 1.2,
                (rand_f32(&mut rng) - 0.5) * 0.24,
            )
        };
        out[i] = Plat {
            x: prev_x,
            y: cur_y,
            z: prev_z,
            w,
            d,
            h,
            rx,
            ry,
            rz,
            zone,
        };
        i += 1;
    }
    out
}

thread_local! {
    static CAM: std::cell::RefCell<Option<CamState>> =
        const { std::cell::RefCell::new(None) };
    static MOUSE: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0.0, 0.0)) };
    static INIT_TARGETS: std::cell::Cell<Option<[usize; 3]>> = const { std::cell::Cell::new(None) };
    static INIT_PLATS: std::cell::Cell<Option<[Plat; JM_PLATS]>> = const { std::cell::Cell::new(None) };
}

fn page_jm_platforms() -> [Plat; JM_PLATS] {
    INIT_PLATS.with(|c| {
        if let Some(p) = c.get() {
            return p;
        }
        let p = gen_jm_platforms(seed_from_ms(now()) ^ 0x6C8E_9CF5);
        c.set(Some(p));
        p
    })
}

fn jm_scenery() -> [Plat; JM_SCENERY] {
    let mut out = [Plat {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
        d: 0.0,
        h: 0.0,
        rx: 0.0,
        ry: 0.0,
        rz: 0.0,
        zone: 0,
    }; JM_SCENERY];
    // 6 tiered central monolith spire blocks rising through the center of the spiral
    let mut i = 0;
    while i < 6 {
        let fi = i as f32;
        let size = 920.0 - fi * 95.0;
        let h = 1120.0;
        out[i] = Plat {
            x: JM_X,
            y: JM_FLOOR_Y - (fi + 1.0) * 1080.0,
            z: JM_Z,
            w: size,
            d: size,
            h,
            rx: (fi * 1.7).sin() * 0.04,
            ry: fi * 0.38,
            rz: (fi * 2.3).cos() * 0.04,
            zone: i as u8,
        };
        i += 1;
    }
    // 8 weathered perimeter desert megalith pillars around the outer floor
    let mut j = 0;
    while j < 8 {
        let fj = j as f32;
        let ang = fj * (TAU / 8.0) + 0.35;
        let dist = 2650.0 + (fj * 1.9).sin() * 180.0;
        let h = 520.0 + ((j * 173) % 880) as f32;
        let w = 240.0 + ((j * 97) % 140) as f32;
        out[6 + j] = Plat {
            x: JM_X + ang.cos() * dist,
            y: JM_FLOOR_Y - h + 40.0,
            z: JM_Z + ang.sin() * dist,
            w,
            d: w * 0.85,
            h,
            rx: (fj * 1.3).sin() * 0.08,
            ry: ang + 0.4,
            rz: (fj * 2.1).cos() * 0.08,
            zone: (j % 3) as u8,
        };
        j += 1;
    }
    out
}

fn seed_from_ms(ms: f64) -> u32 {
    let n = ms as u64;
    let mixed = ((n as u32) ^ ((n >> 32) as u32)).wrapping_mul(0x85EB_CA6B) ^ 0xA5A5_5A5A;
    xorshift(xorshift(mixed))
}

fn page_init_targets() -> [usize; 3] {
    INIT_TARGETS.with(|c| {
        if let Some(t) = c.get() {
            return t;
        }
        let mut seed = seed_from_ms(now());
        let mut t = [usize::MAX; 3];
        for i in 0..3 {
            respawn_target(&mut t, i, &mut seed);
        }
        c.set(Some(t));
        t
    })
}

struct CamState {
    id: i32,
    tick: Closure<dyn Fn()>,
    keys: std::collections::HashSet<String>,
    cur: [f32; 5],
    tgt: [f32; 5],
    lw: f32,
    lh: f32,
    last_t: f64,
    back_vis: bool,
    in_range: bool,
    jm_inside: bool,
    jm_active: bool,
    jm_sig: WriteSignal<bool>,
    jm_clear_sig: WriteSignal<bool>,
    vel: [f32; 3],
    grounded: bool,
    coyote: f32,
    jump_buffer: f32,
    targets: [usize; 3],
    rng: u32,
    hits: u32,
    shots: u32,
}

fn cell_offset(cell: usize) -> (f32, f32) {
    let col = (cell % GRID_COLS) as f32 - (GRID_COLS as f32 - 1.0) * 0.5;
    let row = (cell / GRID_COLS) as f32 - (GRID_ROWS as f32 - 1.0) * 0.5;
    (col * GRID_STEP, row * GRID_STEP)
}

fn in_range_box(px: f32, py: f32, pz: f32) -> bool {
    let ox = px;
    let oy = py;
    let oz = PERSP + pz;
    (ox - RANGE_X).abs() <= RANGE_W * 0.5
        && (oy - RANGE_Y).abs() <= RANGE_H * 0.5
        && (oz - RANGE_Z).abs() <= RANGE_D * 0.5
}

fn xorshift(mut x: u32) -> u32 {
    if x == 0 {
        x = 0x9E37_79B9;
    }
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

fn respawn_target(targets: &mut [usize; 3], hit_idx: usize, rng: &mut u32) {
    let old = targets[hit_idx];
    let mut candidates = [0usize; GRID_CELLS];
    let mut count = 0;
    for c in 0..GRID_CELLS {
        if c != old && !targets.contains(&c) {
            candidates[count] = c;
            count += 1;
        }
    }
    if count > 0 {
        *rng = xorshift(*rng);
        targets[hit_idx] = candidates[(*rng as usize) % count];
    }
}

fn hit_test_targets(
    px: f32,
    py: f32,
    pz: f32,
    pitch: f32,
    yaw: f32,
    targets: &[usize; 3],
) -> Option<usize> {
    let [go, _] = axes(pitch, yaw);
    if go[2].abs() < 1e-5 {
        return None;
    }
    let oz = PERSP + pz;
    let t = (TARGET_Z - oz) / go[2];
    if t <= 0.0 {
        return None;
    }
    let hx = px + t * go[0] - RANGE_X;
    let hy = py + t * go[1] - RANGE_Y;
    for (i, &cell) in targets.iter().enumerate() {
        let (cx, cy) = cell_offset(cell);
        let dx = hx - cx;
        let dy = hy - cy;
        if dx * dx + dy * dy <= TARGET_R * TARGET_R {
            return Some(i);
        }
    }
    None
}

fn target_transform(cell: usize) -> String {
    let (cx, cy) = cell_offset(cell);
    format!("translate(calc(-50% + {cx:.0}px), calc(-50% + {cy:.0}px))")
}

fn sync_target_dom(idx: usize, cell: usize) {
    let id = match idx {
        0 => "sr-t0",
        1 => "sr-t1",
        _ => "sr-t2",
    };
    set_style(id, "transform", &target_transform(cell));
}

fn set_text(id: &str, text: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        el.set_text_content(Some(text));
    }
}

fn sync_score_dom(hits: u32, shots: u32) {
    let acc = if shots == 0 {
        0.0
    } else {
        (f64::from(hits) / f64::from(shots)) * 100.0
    };
    set_text("sr-hits", &format!("C : {hits}"));
    set_text("sr-shots", &format!("B : {shots}"));
    set_text("sr-acc", &format!("A : {acc:06.3}%"));
}

fn axes(pitch: f32, yaw: f32) -> [[f32; 3]; 2] {
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    [[-sy * cp, sp, -cy * cp], [cy, 0.0, -sy]]
}

// Engine-style free look: wrap to [-PI, PI] instead of clamping, so the
// angles stay bounded (no float precision loss / inf) while spinning forever.
fn wrap_angle(a: f32) -> f32 {
    (a + PI).rem_euclid(TAU) - PI
}

fn set_style(id: &str, prop: &str, val: &str) {
    if let Some(el) = document().get_element_by_id(id) {
        let el: HtmlElement = el.unchecked_into();
        let _ = el.style().set_property(prop, val);
    }
}

/// Frustum culling for the far sprite (double-sided, so the browser
/// can't cull it). Mirrors the matrix t computed in cam_frame — keep
/// in sync. Exact sphere-vs-frustum test, camera as pinhole at the eye.
fn sprite_visible(
    vw: f32,
    vh: f32,
    sw: f32,
    sh: f32,
    ox: f32,
    oy: f32,
    yaw: f32,
    pitch: f32,
    px: f32,
    py: f32,
    pz: f32,
) -> bool {
    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    let (ex, ey, ez) = (vw * 0.5, vh * 0.5, PERSP);
    let (qx, qy, qz) = (sw + px, sh + py, ez + pz);
    let t0 = ex - (cy * qx + -sy * qz);
    let t1 = ey - (sy * sp * qx + cp * qy + cy * sp * qz);
    let t2 = ez - (sy * cp * qx + -sp * qy + cy * cp * qz);
    let vx = cy * sw - sy * SPRITE_Z + t0;
    let vy = sy * sp * sw + cp * sh + cy * sp * SPRITE_Z + t1;
    let vz = sy * cp * sw - sp * sh + cy * cp * SPRITE_Z + t2;
    let sr = (vw.min(vh) * SPRITE_VMIN).min(SPRITE_PX) * FRAC_1_SQRT_2 + 1.0;
    let ax = ox / PERSP;
    let bx = (vw - ox) / PERSP;
    let ay = oy / PERSP;
    let by = (vh - oy) / PERSP;
    vz - PERSP < sr
        && (vx - ox - ax * (vz - PERSP)) / (1.0 + ax * ax).sqrt() > -sr
        && (ox - vx - bx * (vz - PERSP)) / (1.0 + bx * bx).sqrt() > -sr
        && (vy - oy - ay * (vz - PERSP)) / (1.0 + ay * ay).sqrt() > -sr
        && (oy - vy - by * (vz - PERSP)) / (1.0 + by * by).sqrt() > -sr
}

fn in_jm_box(px: f32, py: f32, pz: f32, active: bool) -> bool {
    let ox = px;
    let oy = py;
    let oz = PERSP + pz;
    let scale = if active { 0.52 } else { 0.5 };
    (ox - JM_X).abs() <= JM_W * scale
        && (oy - JM_Y).abs() <= JM_H * 0.5
        && (oz - JM_Z).abs() <= JM_D * scale
}

fn set_jm_active(active: bool, sig: WriteSignal<bool>, clear_sig: WriteSignal<bool>) {
    sig.set(active);
    if !active {
        clear_sig.set(false);
    }
    if let Some(el) = document().get_element_by_id("jm-room") {
        let _ = el.set_attribute("class", if active { "jm-room jm-active" } else { "jm-room" });
    }
    if let Some(b) = document().body() {
        let bg = if active {
            "linear-gradient(to bottom,#4d79a6 0px,#7da5cc calc(var(--hz,50vh) - 480px),#b5cde0 calc(var(--hz,50vh) - 130px),#dce4e8 var(--hz,50vh),#cfc1ad calc(var(--hz,50vh) + 55px),#bfa684 calc(var(--hz,50vh) + 210px),#b49a76 100%)"
        } else {
            "#000"
        };
        let _ = b.style().set_property("background", bg);
    }
}

// Parry3D 3D GJK/EPA capsule-vs-oriented-cuboid contact solver
fn b2_collide_platform(
    p: &Plat,
    pos: &mut [f32; 3],
    vel: &mut [f32; 3],
    grounded: &mut bool,
) -> bool {
    let hx = p.w * 0.5;
    let hy = p.h * 0.5;
    let hz = p.d * 0.5;
    let max_r = hx.hypot(hz) + hy + B2_RADIUS + 40.0;
    if (pos[0] - p.x).abs() > max_r || (PERSP + pos[2] - p.z).abs() > max_r {
        return false;
    }
    let capsule = Capsule::new_y(CAPSULE_HALF_SEG, B2_RADIUS);
    let cuboid = Cuboid::new(Vector::new(hx, hy, hz));
    let pos_player = Pose::translation(pos[0], pos[1] + CAPSULE_CENTER_OFFSET_Y, PERSP + pos[2]);
    let rot = Rotation::from_rotation_y(p.ry)
        * Rotation::from_rotation_x(p.rx)
        * Rotation::from_rotation_z(p.rz);
    let center = Vector::new(p.x, p.y, p.z) + rot * Vector::new(0.0, hy, 0.0);
    let pos_plat = Pose::from_parts(center, rot);
    let Ok(Some(c)) = contact(&pos_player, &capsule, &pos_plat, &cuboid, 0.0) else {
        return false;
    };
    if c.dist > 0.0 {
        return false;
    }
    let n = c.normal2;
    let depth = -c.dist;

    if n.y < -0.5 {
        // Resolve penetration purely along vertical Y on walkable ground to eliminate horizontal downhill slip
        pos[1] -= depth / (-n.y);
        *grounded = true;
        if vel[1] > 0.0 {
            vel[1] = 0.0;
        }
        return true;
    }

    if n.y.abs() <= 0.5 {
        let h_len = n.x.hypot(n.z);
        let nx = n.x / h_len;
        let nz = n.z / h_len;
        pos[0] += nx * depth;
        pos[2] += nz * depth;
        let vn = vel[0] * nx + vel[2] * nz;
        if vn < 0.0 {
            vel[0] -= nx * vn;
            vel[2] -= nz * vn;
        }
        return false;
    }

    pos[0] += n.x * depth;
    pos[1] += n.y * depth;
    pos[2] += n.z * depth;

    let vn = vel[0] * n.x + vel[1] * n.y + vel[2] * n.z;
    if vn < 0.0 {
        vel[0] -= n.x * vn;
        vel[1] -= n.y * vn;
        vel[2] -= n.z * vn;
    }
    false
}

fn b2_step_player(
    dt: f32,
    wish: [f32; 2],
    in_range: bool,
    jm_active: bool,
    plats: &[Plat; JM_PLATS],
    pos: &mut [f32; 3],
    vel: &mut [f32; 3],
    grounded: &mut bool,
    coyote: &mut f32,
    jump_buffer: &mut f32,
) -> (bool, bool) {
    let mut touched_jm_floor = false;
    let mut reached_goal = false;
    let step_count = ((dt / (1.0 / 120.0)).ceil() as usize).clamp(4, 16);
    let sub_dt = dt / step_count as f32;
    let mut any_grounded = false;
    for _ in 0..step_count {
        // 1. Symplectic Euler velocity step + linear damping
        vel[1] += B2_GRAVITY * sub_dt;
        if wish[0] != 0.0 || wish[1] != 0.0 {
            let accel = if *grounded { B2_GROUND_ACCEL } else { B2_AIR_ACCEL };
            let k = (accel * sub_dt).min(1.0);
            vel[0] += (wish[0] * B2_MOVE_SPEED - vel[0]) * k;
            vel[2] += (wish[1] * B2_MOVE_SPEED - vel[2]) * k;
        } else {
            let damp = if *grounded { B2_GROUND_FRICTION } else { B2_AIR_DRAG };
            let inv = 1.0 / (1.0 + sub_dt * damp);
            vel[0] *= inv;
            vel[2] *= inv;
            if vel[0].abs() < 0.5 {
                vel[0] = 0.0;
            }
            if vel[2].abs() < 0.5 {
                vel[2] = 0.0;
            }
        }

        // 2. Symplectic Euler position step
        pos[0] += vel[0] * sub_dt;
        pos[1] += vel[1] * sub_dt;
        pos[2] += vel[2] * sub_dt;

        // 3. Parry3D 3D GJK/EPA capsule collision solving (2 iterations)
        let mut step_grounded = false;
        let (floor_y, ceil_y) = if in_range {
            (FLOOR_EYE_Y, CEIL_EYE_Y)
        } else {
            (JM_FLOOR_EYE_Y, JM_CEIL_EYE_Y)
        };
        if !in_range && jm_active {
            let scenery = jm_scenery();
            for _ in 0..2 {
                for (i, p) in plats.iter().enumerate() {
                    if (p.y - pos[1]).abs() > 550.0 {
                        continue;
                    }
                    let hit = b2_collide_platform(p, pos, vel, &mut step_grounded);
                    if hit && i + 1 == JM_PLATS {
                        reached_goal = true;
                    }
                }
                for s in &scenery {
                    b2_collide_platform(s, pos, vel, &mut step_grounded);
                }
            }
        }
        if pos[1] >= floor_y {
            pos[1] = floor_y;
            if vel[1] > 0.0 {
                vel[1] = 0.0;
            }
            step_grounded = true;
            if !in_range {
                touched_jm_floor = true;
            }
        }
        if pos[1] < ceil_y {
            pos[1] = ceil_y;
            if vel[1] < 0.0 {
                vel[1] = 0.0;
            }
        }
        if step_grounded && wish[0] == 0.0 && wish[1] == 0.0 {
            vel[0] = 0.0;
            vel[2] = 0.0;
        }
        if step_grounded {
            any_grounded = true;
            *coyote = COYOTE_TIME;
            if *jump_buffer > 0.0 {
                vel[1] = B2_JUMP_V;
                any_grounded = false;
                *coyote = 0.0;
                *jump_buffer = 0.0;
            }
        } else {
            *coyote = (*coyote - sub_dt).max(0.0);
        }
    }
    *grounded = any_grounded;
    (touched_jm_floor, reached_goal)
}

fn cam_frame() {
    CAM.with(|c| {
        let mut slot = c.borrow_mut();
        let Some(st) = slot.as_mut() else { return };
        let t_now = now();
        let elapsed = ((t_now - st.last_t) * 0.001) as f32;
        st.last_t = t_now;
        let dt = elapsed.clamp(1.0 / 240.0, 0.2);
        let locked = document().pointer_lock_element().is_some();
        if !locked {
            let (mx, my) = MOUSE.with(|m| m.get());
            st.tgt[3] = my * 0.21;
            st.tgt[4] = -mx * 0.28;
        }
        let inside_range = in_range_box(st.tgt[0], st.tgt[1], st.tgt[2]);
        let inside_jm = in_jm_box(st.tgt[0], st.tgt[1], st.tgt[2], st.jm_active);
        if inside_jm != st.jm_inside {
            st.jm_inside = inside_jm;
            if !inside_jm && st.jm_active {
                st.jm_active = false;
                set_jm_active(false, st.jm_sig, st.jm_clear_sig);
            }
        }
        if inside_range || inside_jm {
            let (sy, cy) = st.tgt[4].sin_cos();
            let fwd = [-sy, -cy];
            let right = [cy, -sy];
            let mut wx = 0.0f32;
            let mut wz = 0.0f32;
            if st.keys.contains("KeyW") {
                wx += fwd[0];
                wz += fwd[1];
            }
            if st.keys.contains("KeyS") {
                wx -= fwd[0];
                wz -= fwd[1];
            }
            if st.keys.contains("KeyD") {
                wx += right[0];
                wz += right[1];
            }
            if st.keys.contains("KeyA") {
                wx -= right[0];
                wz -= right[1];
            }
            let len = (wx * wx + wz * wz).sqrt();
            let wish = if len > 1e-4 {
                [wx / len, wz / len]
            } else {
                [0.0, 0.0]
            };
            let plats = page_jm_platforms();
            st.jump_buffer = (st.jump_buffer - dt).max(0.0);
            let mut pos = [st.tgt[0], st.tgt[1], st.tgt[2]];
            let (touched, reached_goal) = b2_step_player(
                dt,
                wish,
                inside_range,
                st.jm_active,
                &plats,
                &mut pos,
                &mut st.vel,
                &mut st.grounded,
                &mut st.coyote,
                &mut st.jump_buffer,
            );
            st.tgt[0] = pos[0];
            st.tgt[1] = pos[1];
            st.tgt[2] = pos[2];
            if touched && !st.jm_active {
                st.jm_active = true;
                st.lw = 0.0;
                set_jm_active(true, st.jm_sig, st.jm_clear_sig);
            }
            if reached_goal {
                st.jm_clear_sig.set(true);
            } else if touched {
                st.jm_clear_sig.set(false);
            }
        } else {
            // Free-fly camera-space FPS movement outside the range & jump map
            let step = 420.0 * dt;
            st.vel = [0.0; 3];
            st.grounded = false;
            st.coyote = 0.0;
            st.jump_buffer = 0.0;
            let [go, side] = axes(st.tgt[3], st.tgt[4]);
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
            if let Some(el) = document().get_element_by_id("app") {
                let el: HtmlElement = el.unchecked_into();
                let sw = el.client_width() as f32 * 0.5;
                let sh = el.client_height() as f32 * 0.5;
                let (sy, cy) = st.cur[4].sin_cos();
                let (sp, cp) = st.cur[3].sin_cos();
                let (px, py, pz) = (st.cur[0], st.cur[1], st.cur[2]);
                let (ex, ey, ez) = (vw * 0.5, vh * 0.5, PERSP);
                let (qx, qy, qz) = (sw + px, sh + py, ez + pz);
                let t0 = ex - (cy * qx + -sy * qz);
                let t1 = ey - (sy * sp * qx + cp * qy + cy * sp * qz);
                let t2 = ez - (sy * cp * qx + -sp * qy + cy * cp * qz);
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
                // Perspective origin is the body center, not the viewport.
                if st.jm_active {
                    if let Some(b) = document().body() {
                        let hz = ey + (-PERSP * st.cur[3].tan()).clamp(-4000.0, 4000.0);
                        let _ = b.style().set_property("--hz", &format!("{hz:.1}px"));
                    }
                }
                let (ox, oy) = document().body().map(|b| (b.client_width() as f32 * 0.5, b.client_height() as f32 * 0.5)).unwrap_or((ex, ey));
                let vis = sprite_visible(vw, vh, sw, sh, ox, oy, st.cur[4], st.cur[3], px, py, pz);
                if vis != st.back_vis && document().get_element_by_id("gl-back").is_some() {
                    st.back_vis = vis;
                    // visibility (not display): no relayout, keeps the decoded image
                    set_style("gl-back", "visibility", if vis { "" } else { "hidden" });
                }
                let inside = in_range_box(px, py, pz);
                if inside != st.in_range {
                    st.in_range = inside;
                    let vis_str = if inside { "visible" } else { "hidden" };
                    set_style("sr-crosshair", "visibility", vis_str);
                    set_style("sr-board", "visibility", vis_str);
                    if !inside {
                        st.hits = 0;
                        st.shots = 0;
                        sync_score_dom(0, 0);
                    }
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

fn cam_start(jm_sig: WriteSignal<bool>, jm_clear_sig: WriteSignal<bool>) {
    let init_targets = page_init_targets();
    CAM.with(|c| {
        if c.borrow().is_some() {
            return;
        }
        let tick: Closure<dyn Fn()> = Closure::own(cam_frame);
        let seed = seed_from_ms(now());
        *c.borrow_mut() = Some(CamState {
            id: -1,
            tick,
            keys: std::collections::HashSet::new(),
            cur: [0.0; 5],
            tgt: [0.0; 5],
            lw: 0.0,
            lh: 0.0,
            last_t: now(),
            back_vis: false,
            in_range: false,
            jm_inside: false,
            jm_active: false,
            jm_sig,
            jm_clear_sig,
            vel: [0.0; 3],
            grounded: false,
            coyote: 0.0,
            jump_buffer: 0.0,
            targets: init_targets,
            rng: seed,
            hits: 0,
            shots: 0,
        });
    });
    let vw = window().inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(1920.0) as f32;
    let vh = window().inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(1080.0) as f32;
    let aw = vw.clamp(1280.0, 2200.0);
    let ah = vh.clamp(720.0, 1300.0);
    set_style("app", "width", &format!("{aw:.0}px"));
    set_style("app", "height", &format!("{ah:.0}px"));
    set_style("app", "min-height", "0");
    for (i, &cell) in init_targets.iter().enumerate() {
        sync_target_dom(i, cell);
    }
    sync_score_dom(0, 0);
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
            set_style("app", "width", "");
            set_style("app", "height", "");
            set_style("app", "min-height", "");
            set_style("gl-back", "visibility", "");
            set_style("sr-crosshair", "visibility", "hidden");
            set_style("sr-board", "visibility", "hidden");
            set_jm_active(false, st.jm_sig, st.jm_clear_sig);
        }
    });
}

#[component]
pub fn Home() -> impl IntoView {
    let (tilt, set_tilt) = signal((0.0f64, 0.0f64));
    let (shift, set_shift) = signal(false);
    let (party, set_party) = signal(false);
    let (party_init, set_party_init) = signal(false);
    let (jm_vis, set_jm_vis) = signal(false);
    let (jm_clear, set_jm_clear) = signal(false);
    let presses = RwSignal::new(Vec::<f64>::new());

    Effect::new(move |_| {
        if let Some(b) = document().body() {
            let _ = b
                .style()
                .set_property("background", if party.get() { "#000" } else { "" });
        }
    });

    // SPA: leaving the page doesn't reload, so reset what party mode changed
    on_cleanup(move || {
        cam_stop();
        if let Some(b) = document().body() {
            let _ = b.style().set_property("background", "");
        }
    });

    let key_h = window_event_listener(ev::keydown, move |e| {
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
                        if on {
                            set_party_init.set(true);
                            set_party.set(true);
                            cam_start(set_jm_vis, set_jm_clear);
                        } else {
                            set_party.set(false);
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
                    if !e.repeat() && (st.in_range || st.jm_inside) {
                        if st.grounded || st.coyote > 0.0 {
                            st.vel[1] = B2_JUMP_V;
                            st.grounded = false;
                            st.coyote = 0.0;
                            st.jump_buffer = 0.0;
                        } else {
                            st.jump_buffer = JUMP_BUFFER_TIME;
                        }
                    }
                }
                st.keys.insert(e.code());
            }
        });
    });

    let keyup_h = window_event_listener(ev::keyup, move |e| {
        if e.key() == "Shift" {
            set_shift.set(false);
        }
        CAM.with(|f| {
            if let Some(st) = f.borrow_mut().as_mut() {
                st.keys.remove(&e.code());
            }
        });
    });

    let wheel_h = window_event_listener(ev::wheel, move |e| {
        if !party.get() {
            return;
        }
        CAM.with(|f| {
            if let Some(st) = f.borrow_mut().as_mut() {
                if st.in_range || st.jm_inside {
                    return;
                }
                let [go, _] = axes(st.tgt[3], st.tgt[4]);
                let d = -(e.delta_y() as f32) * 0.4;
                st.tgt[0] += go[0] * d;
                st.tgt[1] += go[1] * d;
                st.tgt[2] += go[2] * d;
            }
        });
    });

    let click_h = window_event_listener(ev::mousedown, move |_| {
        if !party.get() {
            return;
        }
        if document().pointer_lock_element().is_none() {
            if let Some(el) = document().get_element_by_id("app") {
                let el: HtmlElement = el.unchecked_into();
                el.request_pointer_lock();
            }
            return;
        }
        CAM.with(|c| {
            let mut slot = c.borrow_mut();
            let Some(st) = slot.as_mut() else { return };
            if !st.in_range {
                return;
            }
            st.shots += 1;
            if let Some(hit_idx) = hit_test_targets(
                st.cur[0],
                st.cur[1],
                st.cur[2],
                st.cur[3],
                st.cur[4],
                &st.targets,
            ) {
                st.hits += 1;
                let mut rng = st.rng;
                respawn_target(&mut st.targets, hit_idx, &mut rng);
                st.rng = rng;
                sync_target_dom(hit_idx, st.targets[hit_idx]);
            }
            sync_score_dom(st.hits, st.shots);
        });
    });

    let mouse_h = window_event_listener(ev::mousemove, move |e| {
        if party.get() && document().pointer_lock_element().is_some() {
            CAM.with(|c| {
                if let Some(st) = c.borrow_mut().as_mut() {
                    st.tgt[4] = wrap_angle(st.tgt[4] - e.movement_x() as f32 * 0.0025);
                    st.tgt[3] = (st.tgt[3] + e.movement_y() as f32 * 0.0025)
                        .clamp(-FRAC_PI_2, FRAC_PI_2);
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
    on_cleanup(move || {
        key_h.remove();
        keyup_h.remove();
        wheel_h.remove();
        click_h.remove();
        mouse_h.remove();
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

            <div
                id="app"
                class=move || match (party.get(), jm_vis.get()) {
                    (true, true) => "home-party home-jm",
                    (true, false) => "home-party",
                    _ => "",
                }
            >
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

                {move || {
                    if !party_init.get() {
                        return None;
                    }
                    let init_t = page_init_targets();
                    let t0 = format!("transform:{};", target_transform(init_t[0]));
                    let t1 = format!("transform:{};", target_transform(init_t[1]));
                    let t2 = format!("transform:{};", target_transform(init_t[2]));
                    Some(view! {
                        <>
                            <link
                                rel="stylesheet"
                                href="https://fonts.googleapis.com/css2?family=Geist+Mono:wght@400;500;700&display=swap"
                            />
                            <img
                                id="gl-back"
                                class="gl-back"
                                src="/res/image/kkitut_ks.png"
                                style="visibility:hidden;"
                                alt=""
                            />

                            <div class="sr-booth" aria-hidden="true">
                                <div class="sr-face sr-floor"/>
                                <div class="sr-face sr-ceiling"/>
                                <div class="sr-face sr-left"/>
                                <div class="sr-face sr-right"/>
                                <div class="sr-face sr-front"/>
                                <div class="sr-face sr-back">
                                    <div class="sr-grid">
                                        <div id="sr-t0" class="sr-target" style=t0/>
                                        <div id="sr-t1" class="sr-target" style=t1/>
                                        <div id="sr-t2" class="sr-target" style=t2/>
                                    </div>
                                </div>
                            </div>

                            <div id="jm-room" class="jm-room" aria-hidden="true">
                                <div class="jm-face jm-floor"/>
                                <div class="jm-face jm-ceiling"/>
                                <div class="jm-face jm-left"/>
                                <div class="jm-face jm-right"/>
                                <div class="jm-face jm-back"/>
                                <div class="jm-face jm-front"/>
                                {move || {
                                    if !jm_vis.get() {
                                        return None;
                                    }
                                    let plats = page_jm_platforms();
                                    let scenery = jm_scenery();
                                    let last = plats[JM_PLATS - 1];
                                    let cx = last.x - JM_X;
                                    let cy = last.y - JM_Y - 210.0;
                                    let cz = last.z - JM_Z;
                                    let crown_st = format!(
                                        "--px:{cx:.0}px;--py:{cy:.0}px;--pz:{cz:.0}px;"
                                    );
                                    let plat_views = plats
                                        .into_iter()
                                        .chain(scenery)
                                        .enumerate()
                                        .map(|(i, p)| {
                                            let lx = p.x - JM_X;
                                            let ly = p.y - JM_Y;
                                            let lz = p.z - JM_Z;
                                            let id = format!("jm-p{i}");
                                            let cls = format!("jm-plat jm-z{}", p.zone);
                                            let st = format!(
                                                "width:{:.0}px;height:{:.0}px;--ph:{:.0}px;--px:{lx:.0}px;--py:{ly:.0}px;--pz:{lz:.0}px;--rx:{:.3}rad;--ry:{:.3}rad;--rz:{:.3}rad;",
                                                p.w, p.d, p.h, p.rx, p.ry, p.rz,
                                            );
                                            view! {
                                                <div id=id class=cls style=st>
                                                    <div class="jm-plat-s"/>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    let cloud_views = (0..10usize)
                                        .map(|i| {
                                            let fi = i as f32;
                                            let ang = fi * 1.95 + 0.4;
                                            let r = 1050.0 + (fi * 1.3).sin().abs() * 1650.0;
                                            let lx = ang.cos() * r;
                                            let lz = ang.sin() * r;
                                            let ly = 900.0 - fi * 440.0;
                                            let w = (220.0 + (i % 4) as f32 * 55.0) * 0.96;
                                            let d = (150.0 + (i % 3) as f32 * 45.0) * 0.96;
                                            let dur = 16.0 + (i % 5) as f32 * 3.5;
                                            let st = format!(
                                                "width:{w:.0}px;height:{d:.0}px;--cx:{lx:.0}px;--cy:{ly:.0}px;--cz:{lz:.0}px;animation-duration:{dur:.1}s;"
                                            );
                                            view! {
                                                <div class="jm-cloud" style=st>
                                                    <div class="jm-cloud-b"/>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    Some(view! {
                                        <>
                                            {plat_views}
                                            {cloud_views}
                                            {move || {
                                                if jm_clear.get() {
                                                    None
                                                } else {
                                                    Some(view! {
                                                        <div class="jm-plat jm-crown jm-z5" style=crown_st.clone()>
                                                            <div class="jm-plat-s"/>
                                                        </div>
                                                    })
                                                }
                                            }}
                                        </>
                                    })
                                }}
                                {move || {
                                    if !jm_clear.get() {
                                        return None;
                                    }
                                    let last = page_jm_platforms()[JM_PLATS - 1];
                                    let gx = last.x - JM_X;
                                    let gy = last.y - JM_Y;
                                    let gz = last.z - JM_Z;
                                    let origin = format!("transform:translate3d({gx:.0}px,{gy:.0}px,{gz:.0}px);");
                                    let colors = ["#ff595e", "#ffca3a", "#8ac926", "#1982c4", "#6a4c93", "#ff924c"];
                                    let items = (0..72usize)
                                        .map(|i| {
                                            let fi = i as f32;
                                            let phi = fi * 2.39996;
                                            let elev = ((i * 29 % 72) as f32 / 72.0) * 1.65 - 0.35;
                                            let speed = 420.0 + ((i * 53) % 760) as f32;
                                            let (se, ce) = elev.sin_cos();
                                            let (sp, cp) = phi.sin_cos();
                                            let tx = cp * ce * speed;
                                            let ty = -se * speed;
                                            let tz = sp * ce * speed;
                                            let rx = (i as i32 * 137) % 720 - 360;
                                            let ry = (i as i32 * 193) % 720 - 360;
                                            let rz = (i as i32 * 251) % 720 - 360;
                                            let col = colors[i % colors.len()];
                                            let st = format!(
                                                "--c:{col};--tx:{tx:.0}px;--ty:{ty:.0}px;--tz:{tz:.0}px;--rx:{rx}deg;--ry:{ry}deg;--rz:{rz}deg;"
                                            );
                                            view! { <div class="jm-burst-p" style=st/> }
                                        })
                                        .collect::<Vec<_>>();
                                    Some(view! {
                                        <div class="jm-burst" style=origin>
                                            {items}
                                        </div>
                                    })
                                }}
                            </div>
                        </>
                    })
                }}
            </div>

            {move || {
                party_init.get().then(|| {
                    view! {
                        <>
                            <div id="sr-crosshair" class="sr-crosshair" aria-hidden="true"/>
                            <div id="sr-board" class="sr-board" aria-hidden="true">
                                <div class="sr-board-head">"STAT"</div>
                                <div class="sr-board-row" id="sr-hits">"C : 0"</div>
                                <div class="sr-board-row" id="sr-shots">"B : 0"</div>
                                <div class="sr-board-row" id="sr-acc">"A : 00.000%"</div>
                            </div>
                        </>
                    }
                })
            }}
        </>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // Viewport the browser automation measured: 800x894, app fills it.
    const VW: f32 = 800.0;
    const VH: f32 = 894.0;

    fn vis(yaw: f32, pitch: f32, px: f32, py: f32, pz: f32) -> bool {
        sprite_visible(VW, VH, VW * 0.5, VH * 0.5, VW * 0.5, VH * 0.5, yaw, pitch, px, py, pz)
    }

    #[test]
    fn hidden_behind_camera_at_start() {
        assert!(!vis(0.0, 0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn shown_when_turned_around() {
        assert!(vis(PI, 0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn hidden_at_exact_side() {
        assert!(!vis(PI / 2.0, 0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn shown_at_side_edge() {
        // Regression: the shifted-frustum build hid these.
        assert!(vis(PI - 0.55, 0.0, 2000.0, 0.0, 0.0));
        assert!(vis(PI + 0.55, 0.0, -2000.0, 0.0, 0.0));
    }

    #[test]
    fn shown_up_close() {
        assert!(vis(PI, 0.0, 0.0, 0.0, 3000.0));
    }

    #[test]
    fn hidden_when_fully_outside() {
        assert!(!vis(PI, 0.0, 0.0, 0.0, 5000.0));
        assert!(!vis(PI + 0.62, 0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn angles_wrap_without_clamping() {
        // Spinning forever stays bounded: no float growth, same rotation.
        assert!((wrap_angle(3.0 * PI) - -PI).abs() < 1e-6);
        assert!((wrap_angle(-3.0 * PI) - -PI).abs() < 1e-6);
        for a in [0.0, 1.05, 2.5, 10.0, 100.0, -7.3] {
            let w = wrap_angle(a);
            assert!(w >= -PI && w <= PI);
            assert!((w.sin() - a.sin()).abs() < 1e-5);
            assert!((w.cos() - a.cos()).abs() < 1e-5);
        }
    }

    #[test]
    fn move_axes_are_unit_and_orthogonal() {
        let [f, r] = axes(0.0, 0.0);
        assert_eq!(f, [0.0, 0.0, -1.0]);
        assert_eq!(r, [1.0, 0.0, 0.0]);

        let [f, r] = axes(0.5, 1.2);
        let len = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((len(f) - 1.0).abs() < 1e-6);
        assert!((len(r) - 1.0).abs() < 1e-6);
        assert!((f[0] * r[0] + f[1] * r[1] + f[2] * r[2]).abs() < 1e-6);
    }

    #[test]
    fn shooting_range_hit_and_respawn() {
        assert!(!in_range_box(0.0, 0.0, 0.0));
        // Eye at (RANGE_X, RANGE_Y, RANGE_Z) -> pz = RANGE_Z - PERSP
        let pz = RANGE_Z - PERSP;
        assert!(in_range_box(RANGE_X, RANGE_Y, pz));

        // Center of 6x5 grid (col 2 -> cx = -60, row 2 -> cy = 0). Aiming straight at (RANGE_X - 60, RANGE_Y) hits cell 14.
        let mut targets = [1usize, 14, 27];
        assert_eq!(
            hit_test_targets(RANGE_X - 60.0, RANGE_Y, pz, 0.0, 0.0, &targets),
            Some(1)
        );

        let mut rng = 12345u32;
        for _ in 0..20 {
            respawn_target(&mut targets, 1, &mut rng);
            assert_ne!(targets[1], targets[0]);
            assert_ne!(targets[1], targets[2]);
            assert!(targets[1] < GRID_CELLS);
        }

        // Verify f64 Date.now() (> u32::MAX) does not saturate to a constant seed
        let s1 = seed_from_ms(1_794_090_400_000.0);
        let s2 = seed_from_ms(1_794_090_400_017.0);
        assert_ne!(s1, s2);
    }

    #[test]
    fn jump_map_course_is_reachable() {
        let p_a = gen_jm_platforms(seed_from_ms(1_794_090_400_000.0));
        let p_b = gen_jm_platforms(seed_from_ms(1_794_090_400_123.0));
        assert!((p_a[0].x - p_b[0].x).abs() > 1.0 || (p_a[0].z - p_b[0].z).abs() > 1.0);

        let plats = p_a;
        let max_apex = (B2_JUMP_V * B2_JUMP_V) / (2.0 * B2_GRAVITY);
        for i in 0..JM_PLATS {
            let p = plats[i];
            assert!(p.x - p.w * 0.5 > JM_X - JM_W * 0.5);
            assert!(p.x + p.w * 0.5 < JM_X + JM_W * 0.5);
            assert!(p.z - p.d * 0.5 > JM_Z - JM_D * 0.5);
            assert!(p.z + p.d * 0.5 < JM_Z + JM_D * 0.5);
            if i + 1 < JM_PLATS {
                let next = plats[i + 1];
                let climb = p.y - next.y;
                assert!(climb > 0.0 && climb < max_apex * 0.7);
                let dist = ((next.x - p.x).powi(2) + (next.z - p.z).powi(2)).sqrt();
                assert!(dist > 340.0 && dist < 520.0);

                // Verify Parry3D solver can jump from tilted platform i and land on tilted platform i+1
                let dx = next.x - p.x;
                let dz = next.z - p.z;
                let wish = [dx / dist, dz / dist];
                let mut pos = [p.x, p.y - CAPSULE_EYE_H - 12.0, p.z - PERSP];
                let mut vel = [wish[0] * B2_MOVE_SPEED, B2_JUMP_V, wish[1] * B2_MOVE_SPEED];
                let mut grounded = false;
                let mut coyote = 0.0;
                let mut jump_buf = 0.0;
                let mut landed_next = false;
                let mut goal_hit = false;
                for _ in 0..85 {
                    let (_, reached) = b2_step_player(
                        1.0 / 60.0,
                        wish,
                        false,
                        true,
                        &plats,
                        &mut pos,
                        &mut vel,
                        &mut grounded,
                        &mut coyote,
                        &mut jump_buf,
                    );
                    if reached {
                        goal_hit = true;
                    }
                    if grounded && (pos[1] - (next.y - CAPSULE_EYE_H)).abs() < 45.0 {
                        landed_next = true;
                        break;
                    }
                }
                assert!(landed_next, "failed 3D jump from {i} to {}", i + 1);
                if i + 2 == JM_PLATS {
                    assert!(goal_hit, "final platform did not trigger goal celebration");
                }
            }
        }

        // Verify all 14 scenery structures (6 central monolith tiers + 8 outer pillars) are inside bounds and block the player on all 4 sides + top
        let scenery = jm_scenery();
        for (idx, s) in scenery.iter().enumerate() {
            assert!(in_jm_box(s.x, JM_FLOOR_EYE_Y, s.z - PERSP, false));
            for [dx, dz] in [[1.0f32, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
                let start_r = s.w.max(s.d) + 120.0;
                let test_y = (s.y + s.h * 0.5).min(JM_FLOOR_EYE_Y);
                let mut pos = [s.x - dx * start_r, test_y, s.z - dz * start_r - PERSP];
                let mut vel = [dx * B2_MOVE_SPEED, 0.0, dz * B2_MOVE_SPEED];
                let mut grounded = true;
                let mut coyote = 0.1;
                let mut jump_buf = 0.0;
                for _ in 0..60 {
                    b2_step_player(
                        1.0 / 60.0,
                        [dx, dz],
                        false,
                        true,
                        &plats,
                        &mut pos,
                        &mut vel,
                        &mut grounded,
                        &mut coyote,
                        &mut jump_buf,
                    );
                }
                let rem_dist = (pos[0] - s.x).hypot(PERSP + pos[2] - s.z);
                assert!(
                    rem_dist > s.w.min(s.d) * 0.35,
                    "scenery {idx} failed side collision from ({dx}, {dz}): dist={rem_dist}"
                );
            }
            let mut top_pos = [s.x, s.y - CAPSULE_EYE_H - 10.0, s.z - PERSP];
            let mut top_vel = [0.0; 3];
            let mut top_grounded = false;
            let mut top_coyote = 0.0;
            let mut top_jump_buf = 0.0;
            for _ in 0..60 {
                b2_step_player(
                    1.0 / 60.0,
                    [0.0, 0.0],
                    false,
                    true,
                    &plats,
                    &mut top_pos,
                    &mut top_vel,
                    &mut top_grounded,
                    &mut top_coyote,
                    &mut top_jump_buf,
                );
            }
            assert!(
                top_grounded && (top_pos[1] - (s.y - CAPSULE_EYE_H)).abs() < 30.0,
                "scenery {idx} failed top landing: grounded={top_grounded}, y={}, expected={}",
                top_pos[1],
                s.y - CAPSULE_EYE_H
            );
        }

        // Verify standing still on tilted platforms does not slide or drift downhill
        for idx in [3, 7, 15, 23] {
            let p = plats[idx];
            let mut pos = [p.x, p.y - CAPSULE_EYE_H, p.z - PERSP];
            let mut vel = [0.0; 3];
            let mut grounded = true;
            let mut coyote = 0.10;
            let mut jump_buf = 0.0;
            for _ in 0..120 {
                b2_step_player(
                    1.0 / 60.0,
                    [0.0, 0.0],
                    false,
                    true,
                    &plats,
                    &mut pos,
                    &mut vel,
                    &mut grounded,
                    &mut coyote,
                    &mut jump_buf,
                );
            }
            assert!(
                (pos[0] - p.x).abs() < 1e-4 && (PERSP + pos[2] - p.z).abs() < 1e-4,
                "player drifted while stationary on tilted platform {idx}: dx={}, dz={}",
                pos[0] - p.x,
                PERSP + pos[2] - p.z
            );
        }

        // Verify jump buffer: pressing jump slightly before landing triggers jump upon landing
        let mut b_pos = [JM_X + 1200.0, JM_FLOOR_EYE_Y - 2.0, JM_Z + 1200.0 - PERSP];
        let mut b_vel = [0.0, 250.0, 0.0];
        let mut b_grounded = false;
        let mut b_coyote = 0.0;
        let mut b_buf = JUMP_BUFFER_TIME;
        b2_step_player(
            1.0 / 60.0,
            [0.0, 0.0],
            false,
            true,
            &plats,
            &mut b_pos,
            &mut b_vel,
            &mut b_grounded,
            &mut b_coyote,
            &mut b_buf,
        );
        assert_eq!(b_buf, 0.0);
        assert!(b_vel[1] < -700.0, "buffered jump did not trigger on landing");

        // Verify jump map exit boundary: does not require 2x distance to exit
        assert!(in_jm_box(JM_X + JM_W * 0.49, JM_Y, JM_Z - PERSP, true));
        assert!(!in_jm_box(JM_X + JM_W * 0.55, JM_Y, JM_Z - PERSP, true));
    }
}
