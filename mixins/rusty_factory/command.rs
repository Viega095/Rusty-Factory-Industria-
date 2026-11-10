fn rusty_factory_command(
    game: &mut crate::game::Game<'_>,
    command: &str,
    ran: &mut bool,
) {
    crate::game::rusty_factory_glue::perform(game, command, ran);
}
