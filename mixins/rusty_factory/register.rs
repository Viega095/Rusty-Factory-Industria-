fn rusty_factory_register(commands: &mut Vec<horizon_client::mod_api::NativeCommand>) {
    crate::game::rusty_factory_glue::commands(commands);
}
