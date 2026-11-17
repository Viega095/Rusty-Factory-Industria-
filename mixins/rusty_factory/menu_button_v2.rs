fn rusty_factory_menu_button_v2(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
    id: i32,
    carried: &mut Option<crate::items::ItemStack>,
) {
    crate::game::rusty_factory_glue::menu_button_v2(game, pos, entity, id, carried);
}
