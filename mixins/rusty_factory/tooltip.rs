fn rusty_factory_tooltip(
    game: &crate::game::Game<'_>,
    stack: &horizon_items::ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    crate::game::rusty_factory_glue::tooltip(game, stack, lines);
}
