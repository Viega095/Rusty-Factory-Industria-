fn rusty_factory_screen_tooltip(
    block: &str,
    data: &[i32],
    at: [f32; 2],
    lines: &mut Vec<String>,
) {
    crate::game::rusty_factory_glue::screen_tooltip(block, data, at, lines);
}
