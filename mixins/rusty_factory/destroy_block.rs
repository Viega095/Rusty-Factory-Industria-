fn rusty_factory_destroy_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
) {
    crate::game::rusty_factory_glue::destroy_block(game, pos);
}
