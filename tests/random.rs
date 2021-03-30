use core::f32;

use crui::{
    layouts::{GridLayout, HBoxLayout, MarginLayout, RatioLayout, VBoxLayout},
    Behaviour, Context, Id, InputFlags, MouseEvent, MouseInfo, GUI,
};
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};

#[derive(Default)]
struct AssertInvariant {
    started: bool,
    is_over: bool,
    is_active: bool,
    focus: bool,
}
impl Behaviour for AssertInvariant {
    fn on_start(&mut self, _this: Id, _ctx: &mut Context) {
        assert!(!self.started);
        self.started = true;
    }

    fn on_active(&mut self, _this: Id, _ctx: &mut Context) {
        assert!(self.started);
        assert!(!self.is_active);
        self.is_active = true;
    }

    fn on_deactive(&mut self, _this: Id, _ctx: &mut Context) {
        assert!(self.started);
        assert!(self.is_active);
        assert!(!self.is_over);
        self.is_active = false;
    }

    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, _this: Id, _ctx: &mut Context) {
        assert!(self.started);
        match mouse.event {
            MouseEvent::Enter => {
                assert!(!self.is_over);
                self.is_over = true;
            }
            MouseEvent::Exit => {
                assert!(self.is_over);
                self.is_over = false;
            }
            MouseEvent::Down(_) => {}
            MouseEvent::Up(_) => {}
            MouseEvent::Moved { .. } => {
                assert!(self.is_over);
            }
            MouseEvent::None => {}
        }
    }

    fn on_focus_change(&mut self, focus: bool, _this: Id, _ctx: &mut Context) {
        assert!(self.started);
        assert!(self.focus != focus);
        self.focus = focus;
    }
}

fn rexp(rng: &mut SmallRng, max: f64) -> f64 {
    (rng.gen::<f64>() * max.ln()).exp()
}

fn build_random_gui(gui: &mut GUI, rng: &mut SmallRng) -> Vec<Id> {
    let mut ids = Vec::new();
    let total = rexp(rng, 1000.0) as usize;
    let active_chance = rng.gen();
    for _ in 0..total {
        let builder = gui
            .create_control()
            .active(rng.gen_bool(active_chance))
            .min_size([rexp(rng, 1000.0) as f32, rexp(rng, 1000.0) as f32])
            .anchors(rng.gen())
            .margins([
                rexp(rng, 100.0) as f32,
                rexp(rng, 100.0) as f32,
                rexp(rng, 100.0) as f32,
                rexp(rng, 100.0) as f32,
            ])
            .behaviour(AssertInvariant::default())
            .parent(*ids.choose(rng).unwrap_or(&Id::ROOT_ID));
        let builder = match rng.gen_range(0..=5) {
            0 => builder,
            1 => builder.layout(VBoxLayout::new(
                2.0,
                [2.0, 2.0, 2.0, 2.0],
                rng.gen_range(-1..=1),
            )),
            2 => builder.layout(HBoxLayout::new(
                2.0,
                [2.0, 2.0, 2.0, 2.0],
                rng.gen_range(-1..=1),
            )),
            3 => builder.layout(MarginLayout::new([2.0, 2.0, 2.0, 2.0])),
            4 => builder.layout(GridLayout::new(
                [2.0, 2.0],
                [2.0, 2.0, 2.0, 2.0],
                rng.gen_range(1..10),
            )),
            5 => builder.layout(RatioLayout::new(
                10.0 / (rng.gen::<f32>() * 99.0 + 1.0),
                (rng.gen_range(-1..=1), rng.gen_range(-1..=1)),
            )),
            _ => unreachable!(),
        };
        ids.push(builder.build());
    }
    ids
}

fn from_seed(seed: u64) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let rng = &mut rng;
    let width = rexp(rng, 10000.0) as f32;
    let height = rexp(rng, 10000.0) as f32;
    let mut gui = GUI::new(width, height, Vec::new());
    let ids = build_random_gui(&mut gui, rng);

    let mut mouse_x = rng.gen::<f32>() * width;
    let mut mouse_y = rng.gen::<f32>() * height;
    for _ in 0..2000 {
        if rng.gen_bool(0.05) {
            mouse_x = rng.gen::<f32>() * width;
            mouse_y = rng.gen::<f32>() * height;
            gui.mouse_moved(mouse_x, mouse_y);
        }
        if rng.gen() {
            mouse_x += rexp(rng, width as f64 / 2.0) as f32;
            mouse_y += rexp(rng, height as f64 / 2.0) as f32;
            gui.mouse_moved(mouse_x, mouse_y);
        }
        if rng.gen() {
            gui.deactive_control(*ids.choose(rng).unwrap());
        }
        if rng.gen() {
            gui.active_control(*ids.choose(rng).unwrap());
        }
    }
}

#[test]
fn random() {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let seed: u64 = rng.gen();
        println!("testing for seed {:08x}...", seed);
        from_seed(seed);
    }
}
