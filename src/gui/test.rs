use std::sync::{Arc, Mutex};

use instant::Duration;

use crate::widgets::{List, ListBuilder, ListViewLayout, ScrollView, ViewLayout};
use crate::{
    font::Fonts, Behaviour, Context, Gui, Id, InputFlags, MouseButton, MouseEvent, MouseInfo,
};

struct TestClickCount {
    list: Arc<Mutex<Vec<u8>>>,
}
impl Behaviour for TestClickCount {
    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, _this: Id, _ctx: &mut Context) {
        self.list.lock().unwrap().push(mouse.click_count);
    }
}

struct TestMouseEvent {
    list: Arc<Mutex<Vec<(u64, MouseEvent, bool)>>>,
}
impl Behaviour for TestMouseEvent {
    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, _this: Id, _ctx: &mut Context) {
        let e = (mouse.id, mouse.event, mouse.buttons.left.pressed());
        self.list.lock().unwrap().push(e);
    }
}

struct Mousable;
impl Behaviour for Mousable {
    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }
}

trait Take: Default {
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}
impl<T: Default> Take for T {}

fn init_logger() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init();
}

#[test]
fn mouse_click() {
    init_logger();

    let mut gui = Gui::new(100.0, 100.0, 1.0, Fonts::new());

    let list = Arc::new(Mutex::new(Vec::new()));
    gui.create_control()
        .margins([30.0, 30.0, -30.0, -30.0])
        .behaviour(TestClickCount { list: list.clone() })
        .build(&mut gui);

    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(100));

    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(100));

    gui.mouse_moved(0, 20.0, 50.0); // move out of the control
    gui.mouse_moved(0, 50.0, 50.0); // move back in
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(100));

    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(100));

    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(1000));

    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);

    gui.mouse_exit(0);

    #[rustfmt::skip]
    assert_eq!(
        list.lock().unwrap().take().as_slice(),
        &[
            0, 0, 1, 1,
            2, 2, 
            2, 0, 0, 1, 1,
            2, 2,
            3, 3,
            1, 1,
            1,
        ]
    );
}

/// On touch devices, the mouse enters and exits the screen frequently. Make sure that double
/// clicks works.
#[test]
fn mouse_click_touch() {
    init_logger();

    let mut gui = Gui::new(100.0, 100.0, 1.0, Fonts::new());

    let list = Arc::new(Mutex::new(Vec::new()));
    gui.create_control()
        .margins([30.0, 30.0, -30.0, -30.0])
        .behaviour(TestClickCount { list: list.clone() })
        .build(&mut gui);

    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_exit(0);

    mock_instant::MockClock::advance(Duration::from_millis(100));

    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_exit(0);

    mock_instant::MockClock::advance(Duration::from_millis(100));

    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_exit(0);

    mock_instant::MockClock::advance(Duration::from_millis(1000));

    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);

    gui.mouse_moved(0, 20.0, 50.0); // move out of the control
    gui.mouse_moved(0, 50.0, 50.0); // move back in
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_exit(0);

    #[rustfmt::skip]
    assert_eq!(
        list.lock().unwrap().take().as_slice(),
        &[
            0, 0, 1, 1, 1,
            1, 1, 2, 2, 2,
            2, 2, 3, 3, 3,
            0, 0, 1, 1, 
            1, 0, 0, 1, 1, 1,
        ]
    );
}

/// Should be possible for two mouses to interact with two controls independently.
#[test]
fn multi_touch() {
    init_logger();

    let mut gui = Gui::new(100.0, 100.0, 1.0, Fonts::new());

    let list_a = Arc::new(Mutex::new(Vec::new()));
    gui.create_control()
        .margins([10.0, 10.0, -10.0, -10.0])
        .anchors([0.0, 0.0, 0.5, 1.0])
        .behaviour(TestMouseEvent {
            list: list_a.clone(),
        })
        .build(&mut gui);

    let list_b = Arc::new(Mutex::new(Vec::new()));
    gui.create_control()
        .margins([10.0, 10.0, -10.0, -10.0])
        .anchors([0.5, 0.0, 1.0, 1.0])
        .behaviour(TestMouseEvent {
            list: list_b.clone(),
        })
        .build(&mut gui);

    gui.mouse_moved(1, 25.0, 50.0);
    gui.mouse_down(1, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(1000));

    gui.mouse_moved(2, 75.0, 50.0);
    gui.mouse_down(2, MouseButton::Left);

    gui.mouse_moved(1, 25.0, 50.0);
    gui.mouse_moved(2, 75.0, 50.0);

    gui.mouse_up(2, MouseButton::Left);
    gui.mouse_exit(2);

    gui.mouse_moved(1, 25.0, 50.0);

    gui.mouse_up(1, MouseButton::Left);
    gui.mouse_exit(1);

    #[rustfmt::skip]
    assert_eq!(
        list_a.lock().unwrap().take().as_slice(),
        &[
            (1, MouseEvent::Enter, false),
            (1, MouseEvent::Moved, false),
            (1, MouseEvent::Down(MouseButton::Left), true),
            (1, MouseEvent::Moved, true),
            (1, MouseEvent::Moved, true),
            (1, MouseEvent::Up(MouseButton::Left), false),
            (1, MouseEvent::Exit, false),
        ]
    );

    #[rustfmt::skip]
    assert_eq!(
        list_b.lock().unwrap().take().as_slice(),
        &[
            (2, MouseEvent::Enter, false),
            (2, MouseEvent::Moved, false),
            (2, MouseEvent::Down(MouseButton::Left), true),
            (2, MouseEvent::Moved, true),
            (2, MouseEvent::Up(MouseButton::Left), false),
            (2, MouseEvent::Exit, false),
        ]
    );
}

#[test]
fn multi_touch_one_control() {
    init_logger();

    struct TestMouseEvent {
        list: Arc<Mutex<Vec<(u64, MouseEvent, bool)>>>,
    }
    impl Behaviour for TestMouseEvent {
        fn input_flags(&self) -> InputFlags {
            InputFlags::MOUSE
        }

        fn on_mouse_event(&mut self, mouse: MouseInfo, _this: Id, _ctx: &mut Context) {
            let e = (mouse.id, mouse.event, mouse.buttons.left.pressed());
            self.list.lock().unwrap().push(e);
        }
    }

    let mut gui = Gui::new(100.0, 100.0, 1.0, Fonts::new());

    let list = Arc::new(Mutex::new(Vec::new()));
    gui.create_control()
        .margins([10.0, 10.0, -10.0, -10.0])
        .behaviour(TestMouseEvent { list: list.clone() })
        .build(&mut gui);

    gui.mouse_moved(0, 25.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(1000));

    gui.mouse_moved(2, 75.0, 50.0);
    gui.mouse_down(2, MouseButton::Left);

    gui.mouse_moved(0, 25.0, 50.0);
    gui.mouse_moved(2, 75.0, 50.0);

    gui.mouse_up(2, MouseButton::Left);
    gui.mouse_exit(2);

    gui.mouse_moved(0, 25.0, 50.0);

    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_exit(0);

    #[rustfmt::skip]
    assert_eq!(
        list.lock().unwrap().take().as_slice(),
        &[
            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Down(MouseButton::Left), true),
            (2, MouseEvent::Enter, false),
            (2, MouseEvent::Moved, false),
            (2, MouseEvent::Down(MouseButton::Left), true),
            (0, MouseEvent::Moved, true),
            (2, MouseEvent::Moved, true),
            (2, MouseEvent::Up(MouseButton::Left), false),
            (2, MouseEvent::Exit, false),
            (0, MouseEvent::Moved, true),
            (0, MouseEvent::Up(MouseButton::Left), false),
            (0, MouseEvent::Exit, false),
        ]
    );

    gui.mouse_moved(1, 25.0, 50.0);
    gui.mouse_down(1, MouseButton::Left);

    gui.mouse_moved(2, 75.0, 50.0);
    gui.mouse_down(2, MouseButton::Left);

    gui.mouse_moved(1, 25.0, 50.0);
    gui.mouse_moved(2, 75.0, 50.0);

    gui.mouse_up(2, MouseButton::Left);
    gui.mouse_exit(2);

    gui.mouse_moved(1, 25.0, 50.0);

    gui.mouse_up(1, MouseButton::Left);
    gui.mouse_exit(1);

    #[rustfmt::skip]
    assert_eq!(
        list.lock().unwrap().take().as_slice(),
        &[
            (1, MouseEvent::Enter, false),
            (1, MouseEvent::Moved, false),
            (1, MouseEvent::Down(MouseButton::Left), true),
            (2, MouseEvent::Enter, false),
            (2, MouseEvent::Moved, false),
            (2, MouseEvent::Down(MouseButton::Left), true),
            (1, MouseEvent::Moved, true),
            (2, MouseEvent::Moved, true),
            (2, MouseEvent::Up(MouseButton::Left), false),
            (2, MouseEvent::Exit, false),
            (1, MouseEvent::Moved, true),
            (1, MouseEvent::Up(MouseButton::Left), false),
            (1, MouseEvent::Exit, false),
        ]
    );
}

#[test]
fn drag_scroll_view() {
    init_logger();

    let mut gui = Gui::new(100.0, 100.0, 1.0, Fonts::new());

    let [scroll_view, view, content, h_bar, h_handle, v_bar, v_handle] =
        [(); 7].map(|_| gui.reserve_id());

    gui.create_control_reserved(scroll_view)
        .behaviour_and_layout(ScrollView::new(
            view,
            content,
            Some((h_bar, h_handle)),
            Some((v_bar, v_handle)),
        ))
        .build(&mut gui);

    gui.create_control_reserved(view)
        .layout(ViewLayout::new(true, true))
        .parent(scroll_view)
        .build(&mut gui);

    gui.create_control_reserved(content)
        .parent(view)
        .min_size([200.0, 200.0])
        .build(&mut gui);

    gui.create_control_reserved(h_bar)
        .parent(scroll_view)
        .build(&mut gui);
    gui.create_control_reserved(h_handle)
        .parent(h_bar)
        .build(&mut gui);

    gui.create_control_reserved(v_bar)
        .parent(scroll_view)
        .build(&mut gui);
    gui.create_control_reserved(v_handle)
        .parent(v_bar)
        .build(&mut gui);

    assert_eq!(
        gui.get_context().get_rect(content),
        [0.0, 0.0, 200.0, 200.0]
    );

    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_moved(0, 50.0, 40.0);

    assert_eq!(
        gui.get_context().get_rect(content),
        [0.0, -10.0, 200.0, 190.0]
    );

    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_moved(0, 46.0, 40.0);

    assert_eq!(
        gui.get_context().get_rect(content),
        [0.0, -10.0, 200.0, 190.0]
    );

    gui.mouse_moved(0, 31.0, 40.0);

    assert_eq!(
        gui.get_context().get_rect(content),
        [-15.0, -10.0, 185.0, 190.0]
    );

    gui.mouse_moved(0, 46.0, 50.0);
    gui.mouse_up(0, MouseButton::Left);

    assert_eq!(
        gui.get_context().get_rect(content),
        [0.0, 0.0, 200.0, 200.0]
    );

    gui.create_control()
        .parent(content)
        .margins([40.0, 40.0, -140.0, -140.0])
        .behaviour(Mousable)
        .build(&mut gui);

    gui.mouse_moved(0, 51.0, 51.0);

    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_moved(0, 1.0, 1.0);

    assert_eq!(
        gui.get_context().get_rect(content),
        [-50.0, -50.0, 150.0, 150.0]
    );
}

#[test]
fn drag_list_view() {
    init_logger();

    struct MyListBuilder;
    impl ListBuilder for MyListBuilder {
        fn item_count(&mut self, _: &mut dyn crate::BuilderContext) -> usize {
            usize::max_value()
        }

        fn create_item<'a>(
            &mut self,
            index: usize,
            _list_id: Id,
            cb: crate::ControlBuilder,
            _ctx: &mut dyn crate::BuilderContext,
        ) -> crate::ControlBuilder {
            cb.min_size([15.0, 15.0])
                // only for testing, changes nothing in the layout
                .margins([index as f32; 4])
        }
    }

    let mut gui = Gui::new(100.0, 100.0, 1.0, Fonts::new());

    let [list, view, h_bar, h_handle, v_bar, v_handle] = [(); 6].map(|_| gui.reserve_id());

    gui.create_control_reserved(list)
        .behaviour_and_layout(List::new(
            0.0,
            10.0,
            [10.0; 4],
            view,
            v_bar,
            v_handle,
            h_bar,
            h_handle,
            MyListBuilder,
        ))
        .build(&mut gui);

    gui.create_control_reserved(view)
        .layout(ListViewLayout::new(true, true))
        .parent(list)
        .build(&mut gui);

    gui.create_control_reserved(h_bar)
        .parent(list)
        .build(&mut gui);
    gui.create_control_reserved(h_handle)
        .parent(h_bar)
        .build(&mut gui);

    gui.create_control_reserved(v_bar)
        .parent(list)
        .build(&mut gui);
    gui.create_control_reserved(v_handle)
        .parent(v_bar)
        .build(&mut gui);

    let get_items_rects = |ctx: &mut Context| {
        let items = ctx.get_active_children(view);
        items
            .into_iter()
            .map(|item| ctx.get_rect(item))
            .collect::<Vec<_>>()
    };

    assert_eq!(
        &get_items_rects(&mut gui.get_context()),
        &[
            [10.0, 10.0, 90.0, 25.0],
            [10.0, 35.0, 90.0, 50.0],
            [10.0, 60.0, 90.0, 75.0],
            [10.0, 85.0, 90.0, 100.0]
        ]
    );

    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_moved(0, 50.0, 40.0);

    assert_eq!(
        &get_items_rects(&mut gui.get_context()),
        &[
            [10.0, 0.0, 90.0, 15.0],
            [10.0, 25.0, 90.0, 40.0],
            [10.0, 50.0, 90.0, 65.0],
            [10.0, 75.0, 90.0, 90.0],
            [10.0, 100.0, 90.0, 115.0],
        ]
    );

    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_moved(0, 40.0, 40.0);

    assert_eq!(
        &get_items_rects(&mut gui.get_context()),
        &[
            [10.0, 0.0, 90.0, 15.0],
            [10.0, 25.0, 90.0, 40.0],
            [10.0, 50.0, 90.0, 65.0],
            [10.0, 75.0, 90.0, 90.0],
            [10.0, 100.0, 90.0, 115.0]
        ]
    );

    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_moved(0, 40.0, 36.0);

    assert_eq!(
        &get_items_rects(&mut gui.get_context()),
        &[
            [10.0, 0.0, 90.0, 15.0],
            [10.0, 25.0, 90.0, 40.0],
            [10.0, 50.0, 90.0, 65.0],
            [10.0, 75.0, 90.0, 90.0],
            [10.0, 100.0, 90.0, 115.0],
        ]
    );

    gui.mouse_moved(0, 40.0, 21.0);

    assert_eq!(
        &get_items_rects(&mut gui.get_context()),
        &[
            [10.0, -15.0, 90.0, 0.0],
            [10.0, 10.0, 90.0, 25.0],
            [10.0, 35.0, 90.0, 50.0],
            [10.0, 60.0, 90.0, 75.0],
            [10.0, 85.0, 90.0, 100.0],
        ]
    );

    gui.mouse_moved(0, 40.0, 46.0);
    gui.mouse_up(0, MouseButton::Left);

    assert_eq!(
        &get_items_rects(&mut gui.get_context()),
        &[
            [10.0, 10.0, 90.0, 25.0],
            [10.0, 35.0, 90.0, 50.0],
            [10.0, 60.0, 90.0, 75.0],
            [10.0, 85.0, 90.0, 100.0]
        ]
    );
}

#[test]
fn lock_cursor() {
    init_logger();

    let mut gui = Gui::new(100.0, 100.0, 1.0, Fonts::new());

    let list = Arc::new(Mutex::new(Vec::new()));

    let ids = (0..4)
        .map(|i| {
            let x = (i % 2) as f32;
            let y = (i / 2) as f32;
            let list = list.clone();
            gui.create_control()
                .margins([-10.0, -10.0, 10.0, 10.0])
                .anchors([
                    0.25 + 0.5 * x,
                    0.25 + 0.5 * y,
                    0.25 + 0.5 * x,
                    0.25 + 0.5 * y,
                ])
                .behaviour(TestMouseEvent { list })
                .build(&mut gui)
        })
        .collect::<Vec<Id>>();

    let get_ids_rects = |ctx: &mut Context| {
        ids.into_iter()
            .map(|id| ctx.get_rect(id))
            .collect::<Vec<_>>()
    };

    assert_eq!(
        get_ids_rects(&mut gui.get_context()),
        &[
            [15.0, 15.0, 35.0, 35.0],
            [65.0, 15.0, 85.0, 35.0],
            [15.0, 65.0, 35.0, 85.0],
            [65.0, 65.0, 85.0, 85.0],
        ]
    );

    gui.mouse_moved(0, 25.0, 25.0);
    gui.mouse_moved(0, 75.0, 25.0);
    gui.mouse_moved(0, 75.0, 75.0);
    gui.mouse_moved(0, 25.0, 75.0);
    gui.mouse_moved(0, 25.0, 25.0);

    gui.get_context().lock_cursor(true, 0);

    gui.mouse_moved(0, 25.0, 25.0);
    gui.mouse_moved(0, 75.0, 25.0);
    gui.mouse_moved(0, 75.0, 75.0);
    gui.mouse_moved(0, 25.0, 75.0);
    gui.mouse_moved(0, 25.0, 25.0);

    gui.get_context().lock_cursor(false, 0);

    gui.mouse_moved(0, 25.0, 25.0);
    gui.mouse_moved(0, 75.0, 25.0);
    gui.mouse_moved(0, 75.0, 75.0);
    gui.mouse_moved(0, 25.0, 75.0);
    gui.mouse_moved(0, 25.0, 25.0);

    #[rustfmt::skip]
    assert_eq!(
        list.lock().unwrap().take().as_slice(),
        &[
            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Exit, false),

            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Exit, false),

            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Exit, false),

            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Exit, false),

            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),


            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Moved, false),


            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Exit, false),

            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Exit, false),

            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Exit, false),

            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
            (0, MouseEvent::Exit, false),

            (0, MouseEvent::Enter, false),
            (0, MouseEvent::Moved, false),
        ]
    );
}
