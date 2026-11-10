fn rusty_factory_right_click_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    canceled: &mut bool,
) {
    crate::game::rusty_factory_glue::right_click(game, pos, canceled);
}
