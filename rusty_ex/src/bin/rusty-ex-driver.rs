#![feature(rustc_private)]

fn main() {
    env_logger::init();
    rusty_ex::instrument::driver_main(rusty_ex::RustcEx);
}
