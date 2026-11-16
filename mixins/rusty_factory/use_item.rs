fn rusty_factory_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &crate::items::ItemStack,
    claimed: &mut bool,
) {
    crate::game::rusty_factory_glue::use_item(game, stack, claimed);
}
