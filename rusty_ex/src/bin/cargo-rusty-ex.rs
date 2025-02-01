#![feature(rustc_private)]

fn main() {
    env_logger::init();
    rusty_ex::instrument::cli_main(rusty_ex::RustcEx);
}
