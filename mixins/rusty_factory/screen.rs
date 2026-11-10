fn rusty_factory_screen(
    block: &str,
    screen: &mut Option<horizon_client::mod_api::NativeScreen>,
) {
    crate::game::rusty_factory_glue::screen(block, screen);
}
