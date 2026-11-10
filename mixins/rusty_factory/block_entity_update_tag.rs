fn rusty_factory_block_entity_update_tag(
    pos: [i32; 3],
    entity: &horizon_client::containers::ModBlockEntity,
    tag: &mut Option<horizon_world::nbt::Compound>,
) {
    crate::game::rusty_factory_glue::update_tag(pos, entity, tag);
}
