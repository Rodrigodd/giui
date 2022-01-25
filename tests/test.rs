use std::sync::Mutex;

use crui::Gui;

use log::{Metadata, Record};

struct SimpleLogger(Mutex<Vec<String>>);

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
        self.0.lock().unwrap().push(record.args().to_string());
    }

    fn flush(&self) {}
}

#[test]
fn double_start() {
    let logs = Mutex::new(Vec::new());
    let logger = Box::leak(Box::new(SimpleLogger(logs)));
    log::set_logger(logger).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let mut gui = Gui::new(100.0, 100.0, Default::default());

    let parent = gui.reserve_id();

    gui.create_control().parent(parent).build(&mut gui);

    gui.create_control_reserved(parent)
        .child(&mut gui, |cb, _| cb)
        .build(&mut gui);

    let logs = logger.0.lock().unwrap();
    println!("{:#?}", logs);
    assert!(logs
        .iter()
        .any(|x| x.as_str().starts_with("delayed start of 1:2")));
}
