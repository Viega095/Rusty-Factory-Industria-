fn rusty_factory_place_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
) {
    crate::game::rusty_factory_glue::place_block(game, pos);
}
