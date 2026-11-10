fn rusty_factory_world_primitives(
    game: &crate::game::Game<'_>,
    partial: f32,
    camera: [f64; 3],
    primitives: &mut horizon_client::world_primitives::WorldPrimitives,
) {
    crate::game::rusty_factory_glue::world_primitives(game, partial, camera, primitives);
}
