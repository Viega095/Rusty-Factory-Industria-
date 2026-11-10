fn rusty_factory_embedded_content(enabled: &std::collections::BTreeSet<String>) {
    if !enabled.contains("rusty_factory") {
        return;
    }
    static CONTENT: &[u8] = horizon_client::native_data!("rusty_factory", "rusty_factory-content.hmod");
    let mods_dir = std::env::current_dir().unwrap_or_default().join("horizon-mods");
    let file = mods_dir.join("rusty_factory.hmod");
    let same = std::fs::read(&file).is_ok_and(|on_disk| on_disk == CONTENT);
    if !same {
        if let Err(e) = std::fs::create_dir_all(&mods_dir).and_then(|()| std::fs::write(&file, CONTENT)) {
            log::warn!("Rusty Factory: embedded content could not be written to {}: {e}", file.display());
        }
    }
}
