use godot::prelude::*;
use logger::Logger;
use tracing::level_filters::LevelFilter;

mod logger;
mod websocket;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {
    fn on_level_init(level: InitLevel) {
        if level == InitLevel::Scene {
            let logger = Logger {
                max_level: LevelFilter::DEBUG,
            };
            tracing::subscriber::set_global_default(logger).unwrap();
        }
    }
}

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct RandTest {
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for RandTest {
    fn ready(&mut self) {}
}
