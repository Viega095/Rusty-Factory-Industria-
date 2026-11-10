fn rusty_factory_list(mods: &mut Vec<horizon_client::mod_api::NativeMod>) {
    crate::game::rusty_factory_glue::listed(mods);
}
