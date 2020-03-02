use rkd::config::*;
use rkd::*;

fn main() {
    env_logger::init();
    let c = Cfg::new("/tmp/temp_conf");
    let kb = c.parse().unwrap();
    run_rkd(kb);
}
