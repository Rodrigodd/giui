use std::sync::{Arc, Mutex};

use instant::Duration;

use crate::{font::Fonts, Behaviour, Context, Gui, Id, InputFlags, MouseInfo};
use crate::{MouseButton, MouseEvent};

struct TestClickCount {
    list: Arc<Mutex<Vec<u8>>>,
}
impl Behaviour for TestClickCount {
    fn input_flags(&self) -> InputFlags {
        InputFlags::MOUSE
    }

    fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
        self.list.lock().unwrap().push(mouse.click_count);
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

    gui.mouse_enter(0);
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

    gui.mouse_enter(0);
    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_exit(0);

    mock_instant::MockClock::advance(Duration::from_millis(100));

    gui.mouse_enter(0);
    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_exit(0);

    mock_instant::MockClock::advance(Duration::from_millis(100));

    gui.mouse_enter(0);
    gui.mouse_moved(0, 50.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);
    gui.mouse_up(0, MouseButton::Left);
    gui.mouse_exit(0);

    mock_instant::MockClock::advance(Duration::from_millis(1000));

    gui.mouse_enter(0);
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

    struct TestMouseEvent {
        list: Arc<Mutex<Vec<(u64, MouseEvent, bool)>>>,
    }
    impl Behaviour for TestMouseEvent {
        fn input_flags(&self) -> InputFlags {
            InputFlags::MOUSE
        }

        fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
            let e = (mouse.id, mouse.event, mouse.buttons.left.pressed());
            self.list.lock().unwrap().push(e);
        }
    }

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

    gui.mouse_enter(1);
    gui.mouse_moved(1, 25.0, 50.0);
    gui.mouse_down(1, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(1000));

    gui.mouse_enter(2);
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

        fn on_mouse_event(&mut self, mouse: MouseInfo, this: Id, ctx: &mut Context) {
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

    gui.mouse_enter(0);
    gui.mouse_moved(0, 25.0, 50.0);
    gui.mouse_down(0, MouseButton::Left);

    mock_instant::MockClock::advance(Duration::from_millis(1000));

    gui.mouse_enter(2);
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

    gui.mouse_enter(1);
    gui.mouse_moved(1, 25.0, 50.0);
    gui.mouse_down(1, MouseButton::Left);

    gui.mouse_enter(2);
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

/*
 left: `[(1, Enter, false), (1, Moved, false), (1, Down(Left), true), (2, Enter, false),
right: `[(1, Enter, false), (1, Moved, false), (1, Down(Left), true), (2, Enter, false),

         (2, Moved, f alse), (2, Down(Left), true), (1, Enter, false), (1, Moved, false),
         (2, Moved, f alse), (2, Down(Left), true), (1, Moved, true), (2, Moved, true),

         (2, Moved, true), (2, Up(Left), fa lse), (2, Exit, false), (1, Moved, false),
         (2, Up(Left), false), (2, Exit, fals e), (1, Moved, true), (1, Up(Left), false),

         (1, Up(Left), false), (1, Exit, false)]`,
         (1, Exit, false)]`', src\gui\test.rs:330:5
*/
