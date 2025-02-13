mod client;
mod message;
mod server;

pub use client::Client;
pub use message::ServerMessage;
pub use server::Server;

pub fn default_polling_timer() -> godot::obj::Gd<godot::classes::Timer> {
    let mut timer = <godot::classes::Timer as godot::obj::NewAlloc>::new_alloc();

    timer.set_one_shot(false);
    timer.set_autostart(true);
    timer.set_wait_time(0.1);
    timer.set_timer_process_callback(godot::classes::timer::TimerProcessCallback::PHYSICS);

    timer
}
