fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS")
        .expect("Can't parse target os env variable")
        != "linux"
    {
        return;
    }

    pkg_config::Config::new()
        .probe("x11")
        .expect("X11 is needed for linking");

    cc::Build::new()
        .file("libxdo/xdo.c")
        .file("libxdo/xdo_search.c")
        .static_flag(true)
        .warnings(false)
        .compile("xdo");
}
