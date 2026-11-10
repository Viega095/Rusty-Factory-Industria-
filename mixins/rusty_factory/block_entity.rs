fn rusty_factory_block_entity(
    block: &str,
    entity: &mut Option<(String, horizon_world::nbt::Compound)>,
) {
    crate::game::rusty_factory_glue::block_entity(block, entity);
}
