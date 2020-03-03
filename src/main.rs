use rkd::config::*;
use rkd::*;
use std::env;

fn main() {
    env_logger::init();
    let mut args = env::args();
    let cfg_file = args.skip(1).next().expect("Provide a path to config file");
    let c = Cfg::new(cfg_file);
    let kb = c.parse().unwrap();
    run_rkd(kb);
}
