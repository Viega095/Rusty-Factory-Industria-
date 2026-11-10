fn rusty_factory_container(
    block: &str,
    container: &mut Option<horizon_client::containers::ModContainer>,
) {
    crate::game::rusty_factory_glue::container(block, container);
}
