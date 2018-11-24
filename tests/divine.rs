use divination::divination;

#[divination(Cargo.toml)]
#[derive(Debug)]
pub struct Config;

#[test]
fn read_the_tea() {
    let conf = Config::parse().unwrap();
    let _ = conf.dependencies.get("syn");
    let _ = conf.lib.get("proc-macro");
    let _ = conf.package.get("name");
}
