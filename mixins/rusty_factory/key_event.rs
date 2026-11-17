fn rusty_factory_key_event(
    game: &mut crate::game::Game<'_>,
    key: &str,
    pressed: bool,
) {
    crate::game::rusty_factory_glue::key_event(game, key, pressed);
}
