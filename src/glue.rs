// Horizon engine glue for Rusty Factory (`crate::game::rusty_factory_glue`).
// Connects Horizon's native block entity hooks, 3D rendering, world primitives,
// container menus, screens, and player interactions to `runtime.rs`.

use crate::host_compat as hc;
use crate::runtime as rules;
use mod_api::*;
use rules::{MachineKind, MachineState, MachineTier, StepStatus};

const STATE_TAG: &str = "rusty_machine";
const BTN_TOGGLE_POWER: i32 = 0;
const BTN_MODE_OR_SIZE: i32 = 1;
const BTN_QUARRY_RESET: i32 = 2;

const POWER_BTN_RECT: [f32; 4] = [136.0, 18.0, 34.0, 14.0];
const MODE_BTN_RECT: [f32; 4] = [136.0, 62.0, 34.0, 14.0];
const RESET_BTN_RECT: [f32; 4] = [98.0, 18.0, 34.0, 14.0];

fn state_key() -> String {
    STATE_TAG.to_owned()
}

pub(crate) fn load_state(entity: &impl hc::HasNbt) -> Option<MachineState> {
    hc::machine_state_from_tag(entity.nbt().get(&STATE_TAG.into())?)
}

pub(crate) fn store_state(entity: &mut impl hc::HasNbt, state: &MachineState) {
    entity
        .nbt_mut()
        .insert(STATE_TAG.into(), hc::machine_state_to_tag(state));
}

pub(crate) fn all_block_entity_blocks() -> &'static [&'static str] {
    &[
        rules::BLOCK_GENERATOR,
        rules::BLOCK_BIO_GENERATOR,
        rules::BLOCK_SOLAR,
        rules::BLOCK_ENERGY_BANK,
        rules::BLOCK_CABLE_WHITE,
        rules::BLOCK_CABLE_COPPER,
        rules::BLOCK_CABLE_GOLD,
        rules::BLOCK_CABLE_RED,
        rules::BLOCK_ITEM_PIPE,
        rules::BLOCK_ITEM_PIPE_MK2,
        rules::BLOCK_ITEM_PIPE_MK3,
        rules::BLOCK_EXTRACTOR,
        rules::BLOCK_CRUSHER,
        rules::BLOCK_FURNACE,
        rules::BLOCK_PRESS,
        rules::BLOCK_ASSEMBLER,
        rules::BLOCK_VULCANIZER,
        rules::BLOCK_SAP_EXTRACTOR,
        rules::BLOCK_QUARRY,
        rules::BLOCK_QUARRY_MK2,
        rules::BLOCK_QUARRY_MK3,
        rules::BLOCK_HARVESTER,
        rules::BLOCK_BREAKER,
        rules::BLOCK_PLACER,
        rules::BLOCK_CONVEYOR,
        rules::BLOCK_CONVEYOR_MK2,
        rules::BLOCK_CONVEYOR_MK3,
        rules::BLOCK_ENERGY_MONITOR,
        rules::BLOCK_FLUID_TANK,
        rules::BLOCK_STEAM_BOILER,
        rules::BLOCK_FLUID_PUMP,
        rules::BLOCK_TRASH_CAN,
        rules::BLOCK_ENERGY_RELAY,
        rules::BLOCK_CHARGING_PAD,
        rules::BLOCK_FLUID_PIPE,
        rules::BLOCK_FLUID_PIPE_MK2,
        rules::BLOCK_FLUID_PIPE_MK3,
        rules::BLOCK_ITEM_SPLITTER,
        rules::BLOCK_ALLOY_SMELTER,
        rules::BLOCK_UPGRADE_STATION,
        rules::BLOCK_GEOTHERMAL,
        rules::BLOCK_STEAM_TURBINE,
        rules::BLOCK_TESSERACT,
        rules::BLOCK_INDUSTRIAL_SILO,
        rules::BLOCK_SILO_CONTROLLER,
        rules::BLOCK_SILO_CRAFTING_TERMINAL,
        rules::BLOCK_NUCLEAR_REACTOR,
        rules::BLOCK_WIND_TURBINE,
        rules::BLOCK_HYDROPONIC_CHAMBER,
        rules::BLOCK_LASER_TURRET,
        rules::BLOCK_WIRELESS_CHARGER,
        rules::BLOCK_TELEPORTER_PAD,
        rules::BLOCK_RECYCLER,
        rules::BLOCK_RUBBER_SAP_LOG,
        rules::BLOCK_RUBBER_SAPLING,
    ]
}

pub(crate) fn block_entity(block: &str) -> Option<Compound> {
    let is_tree_tick_block = matches!(block, rules::BLOCK_RUBBER_SAP_LOG | rules::BLOCK_RUBBER_SAPLING);
    if MachineKind::from_block_id(block).is_some() || is_tree_tick_block {
        let mut nbt = Compound::default();
        if let Some(kind) = MachineKind::from_block_id(block) {
            nbt.insert(
                STATE_TAG.into(),
                hc::machine_state_to_tag(&MachineState::new(kind)),
            );
        }
        Some(nbt)
    } else {
        None
    }
}

pub(crate) fn block_entity_ticks(block: &str) -> bool {
    MachineKind::from_block_id(block).is_some()
        || matches!(block, rules::BLOCK_RUBBER_SAP_LOG | rules::BLOCK_RUBBER_SAPLING)
}

pub(crate) fn update_tag(_pos: [i32; 3], entity: &ModBlockEntity) -> Option<Compound> {
    if MachineKind::from_block_id(entity.id()).is_some() {
        Some(entity.nbt.clone())
    } else {
        None
    }
}

pub(crate) fn container(block: &str) -> Option<ModContainer> {
    let kind = MachineKind::from_block_id(block)?;
    match kind {
        MachineKind::CoalGenerator
        | MachineKind::BioGenerator
        | MachineKind::SteamBoiler
        | MachineKind::SolarGenerator
        | MachineKind::VoidTrashCan
        | MachineKind::ChargingPad
        | MachineKind::FluidTank
        | MachineKind::FluidPump
        | MachineKind::GeothermalGenerator
        | MachineKind::SteamTurbine
        | MachineKind::IndustrialSilo
        | MachineKind::NuclearReactor => Some(ModContainer {
            slots: 1,
            faces: std::array::from_fn(|_| vec![0]),
            inputs: vec![0],
            outputs: vec![0],
            menu: Some(rules::MENU_SINGLE.to_owned()),
        }),
        MachineKind::SiloController => Some(ModContainer {
            slots: 2,
            faces: std::array::from_fn(|_| vec![0, 1]),
            inputs: vec![0],
            outputs: vec![1],
            menu: Some("rusty_factory:silo_controller".to_owned()),
        }),
        MachineKind::SiloCraftingTerminal => Some(ModContainer {
            slots: 10,
            faces: std::array::from_fn(|_| vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]),
            inputs: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            outputs: vec![9],
            menu: Some("rusty_factory:silo_crafting_terminal".to_owned()),
        }),
        MachineKind::EnergyMonitor
        | MachineKind::EnergyRelay
        | MachineKind::ConveyorBelt
        | MachineKind::ItemPipe
        | MachineKind::FluidPipe
        | MachineKind::ItemSplitter
        | MachineKind::UpgradeStation => None,
        MachineKind::EnergyBank => Some(ModContainer {
            slots: 1,
            faces: std::array::from_fn(|_| vec![0]),
            inputs: vec![0],
            outputs: vec![0],
            menu: Some(rules::MENU_SINGLE.to_owned()),
        }),
        _ => Some(ModContainer {
            slots: rules::SLOTS,
            faces: std::array::from_fn(|_| {
                vec![rules::SLOT_INPUT, rules::SLOT_FUEL, rules::SLOT_OUTPUT]
            }),
            inputs: vec![rules::SLOT_INPUT, rules::SLOT_FUEL],
            outputs: vec![rules::SLOT_OUTPUT],
            menu: Some(rules::MENU.to_owned()),
        }),
    }
}

fn fuel_burn_ticks(host: &dyn Host, stack: &ItemStack) -> Option<i32> {
    match hc::item_id(host, stack).as_str() {
        rules::ITEM_BATTERY => Some(-1),
        rules::ITEM_BIO_FUEL => Some(3200),
        rules::ITEM_RAW_SAP => Some(800),
        rules::BLOCK_RUBBER_SAPLING
        | rules::BLOCK_RUBBER_LOG
        | rules::BLOCK_RUBBER_SAP_LOG
        | rules::BLOCK_RUBBER_PLANKS => Some(400),
        rules::BLOCK_RUBBER_LEAVES => Some(100),
        "minecraft:coal" | "minecraft:charcoal" => Some(1600),
        "minecraft:coal_block" => Some(16000),
        "minecraft:blaze_rod" => Some(2400),
        "minecraft:lava_bucket" => Some(20000),
        "minecraft:dried_kelp_block" => Some(4000),
        "minecraft:oak_planks"
        | "minecraft:spruce_planks"
        | "minecraft:birch_planks"
        | "minecraft:wheat"
        | "minecraft:sugar_cane"
        | "minecraft:apple"
        | "minecraft:bread"
        | "minecraft:wheat_seeds"
        | "minecraft:oak_leaves"
        | "minecraft:oak_sapling"
        | "minecraft:Stick"
        | "minecraft:stick" => Some(300),
        _ => None,
    }
}

/// Checks if a block at `pos` is a valid item destination (vanilla chest/hopper/furnace/barrel or mod machine/belt).
fn is_item_container_block(block_id: &str) -> bool {
    matches!(
        block_id,
        "minecraft:chest"
            | "minecraft:trapped_chest"
            | "minecraft:barrel"
            | "minecraft:hopper"
            | "minecraft:furnace"
            | "minecraft:blast_furnace"
            | "minecraft:smoker"
            | "minecraft:dropper"
            | "minecraft:dispenser"
            | "minecraft:shulker_box"
            | rules::BLOCK_CRUSHER
            | rules::BLOCK_FURNACE
            | rules::BLOCK_PRESS
            | rules::BLOCK_ASSEMBLER
            | rules::BLOCK_VULCANIZER
            | rules::BLOCK_SAP_EXTRACTOR
            | rules::BLOCK_PLACER
            | rules::BLOCK_GENERATOR
            | rules::BLOCK_BIO_GENERATOR
            | rules::BLOCK_TRASH_CAN
            | rules::BLOCK_CHARGING_PAD
            | rules::BLOCK_ALLOY_SMELTER
            | rules::BLOCK_INDUSTRIAL_SILO
            | rules::BLOCK_SILO_CONTROLLER
            | rules::BLOCK_TESSERACT
            | rules::BLOCK_RECYCLER
    )
}

/// Grows a full Rubber Tree (`rubber_log`, `rubber_sap_log` with golden sap cavities, and `rubber_leaves`) at `pos`.
fn grow_rubber_tree(host: &mut dyn ServerHost, pos: [i32; 3]) {
    let dim = host.dimension();
    let seed = hc::game_time(host) ^ ((pos[0] as i64).wrapping_mul(31) ^ (pos[2] as i64));
    for (p, block_id, opt_prop) in rules::rubber_tree_blocks(pos, seed) {
        let cur = hc::block_id_at(host, p);
        let can_place = p == pos
            || matches!(
                cur.as_str(),
                "minecraft:air"
                    | "minecraft:cave_air"
                    | "minecraft:void_air"
                    | "minecraft:short_grass"
                    | "minecraft:tall_grass"
                    | rules::BLOCK_RUBBER_LEAVES
                    | "minecraft:oak_leaves"
            );
        if !can_place {
            continue;
        }
        if hc::set_block_by_id(host, p, block_id) {
            if let Some((k, v)) = opt_prop {
                host.set_block_property(p, k, v);
            }
            if block_id == rules::BLOCK_RUBBER_SAP_LOG {
                host.set_block_property(p, "lit", "true");
            }
        }
    }
}

/// Scans all connected `industrial_silo`s / `silo_controller`s and adjacent chests/barrels.
fn collect_silo_network(host: &dyn Host, start: [i32; 3]) -> (Vec<[i32; 3]>, Vec<[i32; 3]>) {
    let mut silos = Vec::new();
    let mut chests = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut q = std::collections::VecDeque::new();
    visited.insert(start);
    q.push_back(start);

    while let Some(cur) = q.pop_front() {
        if silos.len() + chests.len() >= 128 {
            break;
        }
        for [dx, dy, dz] in rules::NEIGHBORS_6 {
            let nb = [cur[0] + dx, cur[1] + dy, cur[2] + dz];
            if !visited.insert(nb) {
                continue;
            }
            let b = hc::block_id_at(host, nb);
            if b == rules::BLOCK_INDUSTRIAL_SILO {
                silos.push(nb);
                q.push_back(nb);
            } else if b == rules::BLOCK_SILO_CONTROLLER {
                q.push_back(nb);
            } else if matches!(b.as_str(), "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:barrel") {
                chests.push(nb);
            }
        }
    }
    (silos, chests)
}

/// Aggregates all items across connected Silos and nearby Chests, sorted by total count descending.
fn aggregate_silo_network(host: &dyn Host, start: [i32; 3]) -> (Vec<(String, u32, bool)>, usize, usize) {
    let (silos, chests) = collect_silo_network(host, start);
    let mut map: std::collections::BTreeMap<String, (u32, bool)> = std::collections::BTreeMap::new();

    for sp in &silos {
        if let Some(st) = hc::read_machine_state(host, *sp) {
            if !st.belt_item.is_empty() && st.belt_count > 0 {
                let e = map.entry(st.belt_item).or_insert((0, true));
                e.0 = e.0.saturating_add(st.belt_count);
                e.1 = true;
            }
        }
    }
    let mut list: Vec<(String, u32, bool)> = map.into_iter().map(|(k, (v, s))| (k, v, s)).collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    (list, silos.len(), chests.len())
}

fn sorted_silo_network(
    host: &dyn Host,
    start: [i32; 3],
    sort_by_name: bool,
    ascending: bool,
) -> (Vec<(String, u32, bool)>, usize, usize) {
    let (mut list, silos_n, chests_n) = aggregate_silo_network(host, start);
    list.sort_by(|a, b| {
        let ord = if sort_by_name {
            let sa = a.0.split(':').nth(1).unwrap_or(&a.0);
            let sb = b.0.split(':').nth(1).unwrap_or(&b.0);
            sa.cmp(sb).then_with(|| a.1.cmp(&b.1))
        } else {
            a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0))
        };
        if ascending { ord } else { ord.reverse() }
    });
    (list, silos_n, chests_n)
}

static TERMINAL_SEARCH: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
static TERMINAL_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn get_terminal_search() -> String {
    TERMINAL_SEARCH.lock().map(|s| s.clone()).unwrap_or_default()
}

fn filtered_silo_network(
    host: &dyn Host,
    start: [i32; 3],
    sort_by_name: bool,
    ascending: bool,
    query: &str,
) -> (Vec<(String, u32, bool)>, usize, usize) {
    let (mut list, silos_n, chests_n) = sorted_silo_network(host, start, sort_by_name, ascending);
    let q = query.trim().to_ascii_lowercase();
    if !q.is_empty() {
        list.retain(|(id, _, _)| {
            let short = id.split(':').nth(1).unwrap_or(id).to_ascii_lowercase();
            short.contains(&q)
        });
    }
    (list, silos_n, chests_n)
}

pub(crate) fn key_event(host: &mut dyn ClientHost, key: &str, pressed: bool) {
    if !pressed {
        return;
    }
    if !host.mod_screen().is_some() {
        TERMINAL_OPEN.store(false, std::sync::atomic::Ordering::Relaxed);
        return;
    }
    if !TERMINAL_OPEN.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let k = key.strip_prefix("key.keyboard.").unwrap_or(key);
    if let Ok(mut s) = TERMINAL_SEARCH.lock() {
        if k.eq_ignore_ascii_case("backspace") {
            s.pop();
        } else if k.eq_ignore_ascii_case("space") {
            if s.len() < 20 {
                s.push(' ');
            }
        } else if k.len() == 1 {
            if let Some(ch) = k.chars().next() {
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                    if s.len() < 20 {
                        s.push(ch.to_ascii_lowercase());
                    }
                }
            }
        }
    }
}

fn match_terminal_craft(host: &dyn Host, grid: &[Option<ItemStack>]) -> Option<(&'static str, u32)> {
    let ids: [Option<String>; 9] = std::array::from_fn(|i| {
        grid.get(i)
            .and_then(|s| s.as_ref())
            .map(|s| hc::item_id(host, s))
    });
    let g = |i: usize| ids[i].as_deref();
    let non_empty: Vec<(usize, &str)> = (0..9).filter_map(|i| g(i).map(|id| (i, id))).collect();
    if non_empty.is_empty() {
        return None;
    }
    // 9 identical items -> storage block
    if non_empty.len() == 9 && non_empty.iter().all(|(_, id)| *id == non_empty[0].1) {
        return match non_empty[0].1 {
            "minecraft:iron_ingot" => Some(("minecraft:iron_block", 1)),
            "minecraft:gold_ingot" => Some(("minecraft:gold_block", 1)),
            "minecraft:diamond" => Some(("minecraft:diamond_block", 1)),
            "minecraft:emerald" => Some(("minecraft:emerald_block", 1)),
            "minecraft:redstone" => Some(("minecraft:redstone_block", 1)),
            "minecraft:coal" => Some(("minecraft:coal_block", 1)),
            "minecraft:lapis_lazuli" => Some(("minecraft:lapis_block", 1)),
            "minecraft:copper_ingot" => Some(("minecraft:copper_block", 1)),
            _ => None,
        };
    }
    // 1 item -> unpack block or logs -> planks
    if non_empty.len() == 1 {
        return match non_empty[0].1 {
            "minecraft:iron_block" => Some(("minecraft:iron_ingot", 9)),
            "minecraft:gold_block" => Some(("minecraft:gold_ingot", 9)),
            "minecraft:diamond_block" => Some(("minecraft:diamond", 9)),
            "minecraft:emerald_block" => Some(("minecraft:emerald", 9)),
            "minecraft:redstone_block" => Some(("minecraft:redstone", 9)),
            "minecraft:coal_block" => Some(("minecraft:coal", 9)),
            "minecraft:lapis_block" => Some(("minecraft:lapis_lazuli", 9)),
            "minecraft:copper_block" => Some(("minecraft:copper_ingot", 9)),
            "minecraft:oak_log" | "minecraft:spruce_log" | "minecraft:birch_log" => Some(("minecraft:oak_planks", 4)),
            rules::BLOCK_RUBBER_LOG | rules::BLOCK_RUBBER_SAP_LOG => Some((rules::BLOCK_RUBBER_PLANKS, 4)),
            _ => None,
        };
    }
    // 8 surrounding + 1 center
    if non_empty.len() == 8 && g(4).is_none() {
        let ring = [0, 1, 2, 3, 5, 6, 7, 8];
        if ring.iter().all(|&i| g(i) == g(0)) {
            if matches!(g(0), Some("minecraft:cobblestone" | "minecraft:cobbled_deepslate")) {
                return Some(("minecraft:furnace", 1));
            }
            if g(0).is_some_and(|s| s.ends_with("_planks")) {
                return Some(("minecraft:chest", 1));
            }
        }
    }
    // 2x2 planks -> crafting_table
    if non_empty.len() == 4 && non_empty.iter().all(|(_, id)| id.ends_with("_planks")) {
        let idxs: Vec<usize> = non_empty.iter().map(|(i, _)| *i).collect();
        if matches!(idxs.as_slice(), [0, 1, 3, 4] | [1, 2, 4, 5] | [3, 4, 6, 7] | [4, 5, 7, 8]) {
            return Some(("minecraft:crafting_table", 1));
        }
    }
    // 2 vertical planks -> 4 sticks
    if non_empty.len() == 2 && non_empty[0].1.ends_with("_planks") && non_empty[1].1.ends_with("_planks")
        && non_empty[1].0 == non_empty[0].0 + 3
    {
        return Some(("minecraft:stick", 4));
    }
    // Coal/charcoal above stick -> 4 torches
    if non_empty.len() == 2
        && matches!(non_empty[0].1, "minecraft:coal" | "minecraft:charcoal")
        && non_empty[1].1 == "minecraft:stick"
        && non_empty[1].0 == non_empty[0].0 + 3
    {
        return Some(("minecraft:torch", 4));
    }
    // 8 coal_block + obsidian center -> 1 diamond
    if non_empty.len() == 9 && g(4) == Some("minecraft:obsidian")
        && [0, 1, 2, 3, 5, 6, 7, 8].iter().all(|&i| g(i) == Some("minecraft:coal_block"))
    {
        return Some(("minecraft:diamond", 1));
    }
    // 4 iron/steel + 1 steel_ingot center -> steel_gear
    if non_empty.len() == 5 && g(4) == Some(rules::ITEM_STEEL_INGOT)
        && [1, 3, 5, 7].iter().all(|&i| g(i) == Some("minecraft:iron_ingot"))
    {
        return Some((rules::ITEM_STEEL_GEAR, 2));
    }
    // 8 iron_ingot + 1 redstone center -> machine_casing
    if non_empty.len() == 9 && g(4) == Some("minecraft:redstone")
        && [0, 1, 2, 3, 5, 6, 7, 8].iter().all(|&i| g(i) == Some("minecraft:iron_ingot"))
    {
        return Some((rules::BLOCK_CASING, 1));
    }
    // 6 iron_ingot top/bottom + 3 glass middle -> 6 item_pipe
    if non_empty.len() == 9
        && [0, 1, 2, 6, 7, 8].iter().all(|&i| g(i) == Some("minecraft:iron_ingot"))
        && [3, 4, 5].iter().all(|&i| g(i) == Some("minecraft:glass"))
    {
        return Some((rules::BLOCK_ITEM_PIPE, 6));
    }
    // 3 rubber_bar top + 6 iron_ingot -> 6 conveyor_belt
    if non_empty.len() == 9
        && [0, 1, 2].iter().all(|&i| matches!(g(i), Some(rules::ITEM_RUBBER_BAR | rules::ITEM_RUBBER_SHEET)))
        && [3, 4, 5, 6, 7, 8].iter().all(|&i| g(i) == Some("minecraft:iron_ingot"))
    {
        return Some((rules::BLOCK_CONVEYOR, 6));
    }
    // 3 copper_ingot middle -> 6 copper_cable
    if non_empty.len() == 3 && [3, 4, 5].iter().all(|&i| g(i) == Some("minecraft:copper_ingot")) {
        return Some((rules::BLOCK_CABLE, 6));
    }
    // 4 diamond corners + 4 obsidian/gold + blaze_powder center -> netherite_seed_core
    if non_empty.len() == 9
        && [0, 2, 6, 8].iter().all(|&i| g(i) == Some("minecraft:diamond"))
        && [1, 7].iter().all(|&i| g(i) == Some("minecraft:obsidian"))
        && [3, 5].iter().all(|&i| g(i) == Some("minecraft:gold_ingot"))
        && g(4) == Some("minecraft:blaze_powder")
    {
        return Some((rules::ITEM_NETHERITE_SEED_CORE, 1));
    }
    // 3 iron_ingot in V -> bucket
    if non_empty.len() == 3 && g(3) == Some("minecraft:iron_ingot") && g(5) == Some("minecraft:iron_ingot") && g(7) == Some("minecraft:iron_ingot") {
        return Some(("minecraft:bucket", 1));
    }
    // 5 iron_ingot in V + chest center -> hopper
    if non_empty.len() == 6 && g(4) == Some("minecraft:chest")
        && [0, 2, 3, 5, 7].iter().all(|&i| g(i) == Some("minecraft:iron_ingot"))
    {
        return Some(("minecraft:hopper", 1));
    }
    // 6 glass (top 2 rows) -> 16 glass_pane
    if non_empty.len() == 6 && [0, 1, 2, 3, 4, 5].iter().all(|&i| g(i) == Some("minecraft:glass")) {
        return Some(("minecraft:glass_pane", 16));
    }
    // 6 iron_ingot (top 2 rows) -> 16 iron_bars
    if non_empty.len() == 6 && [0, 1, 2, 3, 4, 5].iter().all(|&i| g(i) == Some("minecraft:iron_ingot")) {
        return Some(("minecraft:iron_bars", 16));
    }
    // 6 copper_ingot top/bottom + 3 glass middle -> 6 fluid_pipe
    if non_empty.len() == 9
        && [0, 1, 2, 6, 7, 8].iter().all(|&i| g(i) == Some("minecraft:copper_ingot"))
        && [3, 4, 5].iter().all(|&i| g(i) == Some("minecraft:glass"))
    {
        return Some((rules::BLOCK_FLUID_PIPE, 6));
    }
    // 8 iron_plate + chest center -> silo_upgrade_5k
    if non_empty.len() == 9 && g(4) == Some("minecraft:chest")
        && [0, 1, 2, 3, 5, 6, 7, 8].iter().all(|&i| g(i) == Some(rules::ITEM_IRON_PLATE))
    {
        return Some((rules::ITEM_SILO_UPGRADE_5K, 1));
    }
    None
}

/// Pulls 1 unit of `target_item` from the connected Silo/Chest network for auto-refill crafting.
fn pull_one_from_silo_network(host: &mut dyn ServerHost, pos: [i32; 3], target_item: &str) -> bool {
    let (silos, chests) = collect_silo_network(host, pos);
    for &sp in &silos {
        let mut took = false;
        hc::with_mod_block_entity(host, sp, |sent| {
            if let Some(mut sst) = load_state(sent) {
                if sst.belt_item == target_item && sst.belt_count > 0 {
                    sst.belt_count -= 1;
                    if sst.belt_count == 0 {
                        sst.belt_item.clear();
                    }
                    store_state(sent, &sst);
                    took = true;
                }
            }
        });
        if took {
            host.block_entity_sync(sp);
            return true;
        }
    }
    for &cp in &chests {
        if hc::take_one_matching_from_container(host, cp, target_item) {
            return true;
        }
    }
    false
}

/// Routes `stack` through a `silo_controller` into matching Silos, then empty Silos, then connected Chests.
fn insert_into_silo_controller(host: &mut dyn ServerHost, pos: [i32; 3], mut stack: ItemStack) -> Option<ItemStack> {
    let (silos, chests) = collect_silo_network(host, pos);
    let item_str = hc::item_id(host, &stack);
    // 1. Silos already holding this exact item
    for &sp in &silos {
        let matches_item = hc::read_machine_state(host, sp)
            .is_some_and(|st| st.belt_item == item_str);
        if matches_item {
            match insert_into_silo(host, sp, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        }
    }
    // 2. Empty Silos
    for &sp in &silos {
        let is_empty = hc::read_machine_state(host, sp)
            .is_some_and(|st| st.belt_item.is_empty());
        if is_empty {
            match insert_into_silo(host, sp, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        }
    }
    // 3. Connected Chests
    for &cp in &chests {
        match hc::insert_item_rem(host, cp, Direction::Up, stack) {
            None => return None,
            Some(rest) => stack = rest,
        }
    }
    Some(stack)
}

/// Inserts `stack` into an `industrial_silo` ONLY if it is empty or already holds that exact item type.
fn insert_into_silo(host: &mut dyn ServerHost, pos: [i32; 3], stack: ItemStack) -> Option<ItemStack> {
    if hc::block_id_at(host, pos) != rules::BLOCK_INDUSTRIAL_SILO {
        return Some(stack);
    }
    let item_id = hc::item_id(host, &stack);
    let mut rem = stack.count();
    let changed = hc::with_mod_block_entity(host, pos, |entity| {
        let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::IndustrialSilo));
        if !st.belt_item.is_empty() && st.belt_item != item_id {
            return false;
        }
        let cap = st.silo_capacity.max(2_048);
        let space = cap.saturating_sub(st.belt_count);
        let add = rem.min(space);
        if add == 0 {
            return false;
        }
        st.belt_item = item_id.clone();
        st.belt_count += add;
        rem -= add;
        store_state(entity, &st);
        true
    });
    if changed == Some(true) {
        host.block_entity_sync(pos);
    }
    if rem == 0 {
        None
    } else {
        hc::make_stack(host, &hc::item_id(host, &stack), rem)
    }
}

/// Inserts `stack` onto a conveyor belt's `MachineState` if it is empty or can draw more of the same item.
/// Returns `None` if all items were accepted, or `Some(leftover)`.
fn insert_into_conveyor(host: &mut dyn ServerHost, pos: [i32; 3], stack: ItemStack) -> Option<ItemStack> {
    let block = hc::block_id_at(host, pos);
    let Some((_, max_batch)) = rules::conveyor_tier(&block) else {
        return Some(stack);
    };
    let mut accepted = 0u32;
    let item_id = hc::item_id(host, &stack);
    let changed = hc::with_mod_block_entity(host, pos, |entity| {
        let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ConveyorBelt));
        if !st.enabled {
            return false;
        }
        if st.belt_item.is_empty() {
            let take = stack.count().min(max_batch);
            st.belt_item = item_id;
            st.belt_count = take;
            st.progress = 0;
            accepted = take;
            store_state(entity, &st);
            true
        } else if st.belt_item == item_id && st.belt_count < max_batch {
            let space = max_batch - st.belt_count;
            let take = stack.count().min(space);
            st.belt_count += take;
            accepted = take;
            store_state(entity, &st);
            true
        } else {
            false
        }
    });
    if changed == Some(true) && accepted > 0 {
        host.block_entity_sync(pos);
        if stack.count() > accepted {
            let mut rest = stack;
            let _ = rest.set_count(rest.count().saturating_sub(accepted).max(1));
            Some(rest)
        } else {
            None
        }
    } else {
        Some(stack)
    }
}

/// Inserts `stack` into a Create-style Filtered Round-Robin Item Splitter (`rusty_factory:item_splitter`),
/// routing matching items to filtered faces (Left / Front / Right) and dropping into open air if unattached!
fn insert_into_splitter(host: &mut dyn ServerHost, pos: [i32; 3], stack: ItemStack) -> Option<ItemStack> {
    if hc::block_id_at(host, pos) != rules::BLOCK_ITEM_SPLITTER {
        return Some(stack);
    }
    let facing = hc::block_property_at(host, pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let out_offsets = [
        rules::conveyor_exit_offset(&facing, "turn_left"),
        rules::conveyor_exit_offset(&facing, "straight"),
        rules::conveyor_exit_offset(&facing, "turn_right"),
    ];
    let mut start_idx = 0usize;
    let mut filters = [String::new(), String::new(), String::new()];
    hc::with_mod_block_entity(host, pos, |entity| {
        if let Some(st) = load_state(entity) {
            start_idx = (st.progress as usize) % 3;
            filters = st.splitter_filters;
        }
    });

    let item_id = hc::item_id(host, &stack);
    let has_matching_filter = filters.iter().any(|f| !f.is_empty() && *f == item_id);

    for step in 0..3 {
        let idx = (start_idx + step) % 3;
        let f = &filters[idx];
        if has_matching_filter {
            if f != &item_id {
                continue;
            }
        } else if !f.is_empty() {
            continue;
        }

        let off = out_offsets[idx];
        let target = [pos[0] + off[0], pos[1] + off[1], pos[2] + off[2]];
        let tb = hc::block_id_at(host, target);
        let is_air = matches!(tb.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air");
        let leftover = if rules::is_conveyor_block(&tb) {
            insert_into_conveyor(host, target, stack.clone())
        } else if rules::is_item_pipe(&tb) {
            insert_into_pipe(host, pos, target, stack.clone(), None)
        } else if tb == rules::BLOCK_INDUSTRIAL_SILO {
            insert_into_silo(host, target, stack.clone())
        } else if tb == rules::BLOCK_SILO_CONTROLLER {
            insert_into_silo_controller(host, target, stack.clone())
        } else if is_item_container_block(&tb) {
            match hc::insert_item_rem(host, target, Direction::Up, stack.clone()) {
                None => None,
                Some(r) => hc::insert_item_rem(host, target, Direction::North, r),
            }
        } else if is_air {
            let target_below = [target[0], target[1] - 1, target[2]];
            if rules::is_conveyor_block(&hc::block_id_at(host, target_below)) {
                insert_into_conveyor(host, target_below, stack.clone())
            } else if has_matching_filter {
                let drop_pos = [
                    pos[0] as f64 + 0.5 + off[0] as f64 * 0.65,
                    pos[1] as f64 + 0.30,
                    pos[2] as f64 + 0.5 + off[2] as f64 * 0.65,
                ];
                let vel = [off[0] as f64 * 0.12, 0.06, off[2] as f64 * 0.12];
                hc::spawn_item(host, stack.clone(), drop_pos);
                None
            } else {
                Some(stack.clone())
            }
        } else {
            Some(stack.clone())
        };
        if leftover.as_ref().map_or(0, |r| r.count()) < stack.count() {
            let next_idx = ((idx + 1) % 3) as i32;
            hc::with_mod_block_entity(host, pos, |entity| {
                let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ItemSplitter));
                st.progress = next_idx;
                store_state(entity, &st);
            });
            return leftover;
        }
    }
    Some(stack)
}

/// Inserts `stack` into an `item_pipe`'s `ModBlockEntity` so it visibly travels through the glass tube.
fn insert_into_pipe(
    host: &mut dyn ServerHost,
    from_pos: [i32; 3],
    pipe_pos: [i32; 3],
    stack: ItemStack,
    target_pos: Option<[i32; 3]>,
) -> Option<ItemStack> {
    if !rules::is_item_pipe(&hc::block_id_at(host, pipe_pos)) {
        return Some(stack);
    }
    let from_off = [from_pos[0] - pipe_pos[0], from_pos[1] - pipe_pos[1], from_pos[2] - pipe_pos[2]];
    let from_dir = rules::dir_index_from_offset(from_off);
    let next_step = rules::find_next_pipe_step(
        pipe_pos,
        target_pos,
        |p| rules::is_item_pipe(&hc::block_id_at(host, p)) && p != from_pos,
        |p| p != from_pos && (is_item_container_block(&hc::block_id_at(host, p)) || rules::is_conveyor_block(&hc::block_id_at(host, p))),
    );
    let (to_dir, final_target) = if let Some((step, tgt)) = next_step {
        let to_off = [step[0] - pipe_pos[0], step[1] - pipe_pos[1], step[2] - pipe_pos[2]];
        (rules::dir_index_from_offset(to_off), Some(tgt))
    } else {
        (rules::opposite_dir_index(from_dir), target_pos)
    };

    let item_id = hc::item_id(host, &stack);
    let inserted = hc::with_mod_block_entity(host, pipe_pos, |entity| {
        let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ItemPipe));
        if !st.belt_item.is_empty() {
            return false;
        }
        st.belt_item = item_id.clone();
        st.belt_count = stack.count();
        st.progress = 0;
        st.pipe_from = from_dir;
        st.pipe_to = to_dir;
        if let Some(t) = final_target {
            st.pipe_target = t;
            st.pipe_has_target = true;
        } else {
            st.pipe_has_target = false;
        }
        store_state(entity, &st);
        true
    });
    if inserted == Some(true) {
        host.block_entity_sync(pipe_pos);
        None
    } else {
        Some(stack)
    }
}

/// Routes `stack` from `source_pos` into connected `rusty_factory:item_pipe` networks (step-by-step visible travel)
/// or directly into adjacent containers / conveyor belts.
/// Returns the remaining stack (`None` if all items were inserted).
fn route_item_from(host: &mut dyn ServerHost, source_pos: [i32; 3], mut stack: ItemStack) -> Option<ItemStack> {
    // 1. Try direct adjacent containers first (except conveyors)
    for d in rules::DIRS {
        let adj = [source_pos[0] + d[0], source_pos[1] + d[1], source_pos[2] + d[2]];
        let blk = hc::block_id_at(host, adj);
        if blk == rules::BLOCK_INDUSTRIAL_SILO {
            match insert_into_silo(host, adj, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        } else if blk == rules::BLOCK_SILO_CONTROLLER {
            match insert_into_silo_controller(host, adj, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        } else if !rules::is_conveyor_block(&blk) && is_item_container_block(&blk) {
            match hc::insert_item_rem(host, adj, Direction::Up, stack) {
                None => return None,
                Some(rest) => {
                    stack = match hc::insert_item_rem(host, adj, Direction::North, rest) {
                        None => return None,
                        Some(r2) => r2,
                    };
                }
            }
        }
    }

    // 2. Try adjacent item_pipes if the network has a reachable target
    let targets = rules::find_connected_item_targets(
        source_pos,
        |p| rules::is_item_pipe(&hc::block_id_at(host, p)),
        |p| p != source_pos && (is_item_container_block(&hc::block_id_at(host, p)) || rules::is_conveyor_block(&hc::block_id_at(host, p)) || hc::block_id_at(host, p) == rules::BLOCK_ITEM_SPLITTER),
    );
    if !targets.is_empty() {
        for d in rules::DIRS {
            let adj = [source_pos[0] + d[0], source_pos[1] + d[1], source_pos[2] + d[2]];
            if rules::is_item_pipe(&hc::block_id_at(host, adj)) {
                if insert_into_pipe(host, source_pos, adj, stack.clone(), None).is_none() {
                    return None;
                }
            }
        }
    }

    // 3. Try adjacent conveyor belts
    for d in rules::DIRS {
        let adj = [source_pos[0] + d[0], source_pos[1] + d[1], source_pos[2] + d[2]];
        if rules::is_conveyor_block(&hc::block_id_at(host, adj)) {
            match insert_into_conveyor(host, adj, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        }
    }

    Some(stack)
}

/// Auto-ejects up to `max_count` items from `entity.items[SLOT_OUTPUT]` into connected pipes/belts/chests.
fn try_auto_eject_output(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    max_count: u32,
) {
    let Some(out_stack) = entity.items.get(rules::SLOT_OUTPUT).cloned().flatten() else {
        return;
    };
    let send_count = out_stack.count().min(max_count).max(1);
    let mut to_send = out_stack.clone();
    let _ = to_send.set_count(send_count);
    let leftover = route_item_from(host, pos, to_send);
    let sent = send_count - leftover.as_ref().map_or(0, |r| r.count());
    if sent > 0 {
        if let Some(Some(slot_ref)) = entity.items.get_mut(rules::SLOT_OUTPUT) {
            if slot_ref.count() > sent {
                let _ = slot_ref.set_count(slot_ref.count() - sent);
            } else {
                entity.items[rules::SLOT_OUTPUT] = None;
            }
        }
    }
}

/// Distributes FE from a Generator, Solar Panel, or Energy Bank across connected copper cables and adjacent machines.
/// If `source_is_bank` is true, it will not send energy into another Energy Bank (preventing loops).
fn distribute_energy(
    host: &mut dyn ServerHost,
    gen_pos: [i32; 3],
    available: i32,
    source_is_bank: bool,
) -> i32 {
    if available <= 0 {
        return 0;
    }
    let targets = rules::find_connected_consumers(
        gen_pos,
        |p| rules::is_cable(&hc::block_id_at(host, p)),
        |p| {
            let Some(k) = MachineKind::from_block_id(&hc::block_id_at(host, p)) else {
                return false;
            };
            if source_is_bank && k == MachineKind::EnergyBank {
                return false;
            }
            k.is_energy_consumer()
        },
    );

    let mut remaining = available;
    for target_pos in targets {
        if remaining <= 0 {
            break;
        }
        let Some(kind) = MachineKind::from_block_id(&hc::block_id_at(host, target_pos)) else {
            continue;
        };
        let mut transferred = 0;
        hc::with_mod_block_entity(host, target_pos, |target_entity| {
            let mut st = load_state(target_entity).unwrap_or_else(|| MachineState::new(kind));
            let space = (kind.max_energy() - st.energy).max(0);
            let send = remaining.min(space).min(300);
            if send > 0 {
                st.energy += send;
                transferred = send;
                store_state(target_entity, &st);
            }
        });
        remaining -= transferred;
    }

    available - remaining
}

/// Updates a machine block's `lit` blockstate property (`true` when actively working, `false` otherwise)
/// so its block model switches to the animated `*_on` texture and emits light.
fn update_lit_state(host: &mut dyn ServerHost, pos: [i32; 3], working: bool) {
    let want = if working { "true" } else { "false" };
    if hc::block_property_at(host, pos, "lit").as_deref() != Some(want) {
        host.set_block_property(pos, "lit", want);
    }
}

/// Simulates the Pneumatic Item Pipe (`rusty_factory:item_pipe`):
/// Advances the travelling item smoothly from `pipe_from` to `pipe_to`,
/// then hands it off to the next pipe, conveyor belt, or destination container.
fn tick_item_pipe(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    state: &mut MachineState,
) {
    if state.belt_item.is_empty() {
        return;
    }
    let pipe_ticks = rules::item_pipe_ticks(&entity.id());
    if state.progress < pipe_ticks {
        state.progress += 1;
        store_state(entity, state);
        host.block_entity_sync(pos);
        if state.progress < pipe_ticks {
            return;
        }
    }

    let from_off = rules::offset_from_dir_index(state.pipe_from);
    let from_pos = [pos[0] + from_off[0], pos[1] + from_off[1], pos[2] + from_off[2]];
    let preferred = if state.pipe_has_target { Some(state.pipe_target) } else { None };
    let is_pipe_target = |p: [i32; 3]| {
        p != from_pos
            && (is_item_container_block(&hc::block_id_at(host, p))
                || rules::is_conveyor_block(&hc::block_id_at(host, p))
                || hc::block_id_at(host, p) == rules::BLOCK_ITEM_SPLITTER)
    };
    let next_step = rules::find_next_pipe_step(
        pos,
        preferred,
        |p| rules::is_item_pipe(&hc::block_id_at(host, p)) && p != from_pos,
        is_pipe_target,
    );

    let Some(stack) = hc::make_stack(host, &state.belt_item, state.belt_count.max(1)) else { return; };

    // If there is no target at all in front of this pipe, check if this is an open pipe end (air) and drop cleanly
    let Some((step_pos, final_target)) = next_step else {
        let opp = rules::offset_from_dir_index(rules::opposite_dir_index(state.pipe_from));
        let open_pos = [pos[0] + opp[0], pos[1] + opp[1], pos[2] + opp[2]];
        if matches!(
            hc::block_id_at(host, open_pos).as_str(),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ) {
            let drop_pos = [
                pos[0] as f64 + 0.5 + opp[0] as f64 * 0.6,
                pos[1] as f64 + 0.35 + opp[1] as f64 * 0.4,
                pos[2] as f64 + 0.5 + opp[2] as f64 * 0.6,
            ];
            let vel = [opp[0] as f64 * 0.12, 0.06, opp[2] as f64 * 0.12];
            hc::spawn_item(host, stack, drop_pos);
            state.belt_item.clear();
            state.belt_count = 0;
            state.progress = 0;
            state.pipe_has_target = false;
            store_state(entity, state);
            host.block_entity_sync(pos);
        }
        return;
    };

    let step_block = hc::block_id_at(host, step_pos);
    let leftover = if rules::is_item_pipe(&step_block) {
        insert_into_pipe(host, pos, step_pos, stack.clone(), Some(final_target))
    } else if step_block == rules::BLOCK_ITEM_SPLITTER {
        insert_into_splitter(host, step_pos, stack.clone())
    } else if step_block == rules::BLOCK_INDUSTRIAL_SILO {
        insert_into_silo(host, step_pos, stack.clone())
    } else if step_block == rules::BLOCK_SILO_CONTROLLER {
        insert_into_silo_controller(host, step_pos, stack.clone())
    } else if rules::is_conveyor_block(&step_block) {
        insert_into_conveyor(host, step_pos, stack.clone())
    } else {
        match hc::insert_item_rem(host, step_pos, Direction::Up, stack.clone()) {
            None => None,
            Some(rest) => hc::insert_item_rem(host, step_pos, Direction::North, rest),
        }
    };

    match leftover {
        None => {
            state.belt_item.clear();
            state.belt_count = 0;
            state.progress = 0;
            state.pipe_has_target = false;
            store_state(entity, state);
            host.block_entity_sync(pos);
        }
        Some(rest) if rest.count() < state.belt_count => {
            state.belt_count = rest.count();
            store_state(entity, state);
            host.block_entity_sync(pos);
        }
        Some(_) => {
            // Destination chest/target is full! Try any other reachable target in the network (including branching back!)
            state.pipe_has_target = false;
            let all_targets = rules::find_connected_item_targets(
                pos,
                |p| rules::is_item_pipe(&hc::block_id_at(host, p)),
                |p| {
                    p != final_target
                        && p != step_pos
                        && (is_item_container_block(&hc::block_id_at(host, p))
                            || rules::is_conveyor_block(&hc::block_id_at(host, p))
                            || hc::block_id_at(host, p) == rules::BLOCK_ITEM_SPLITTER)
                },
            );
            let mut rerouted = false;
            for alt in all_targets {
                if let Some((alt_step, _)) = rules::find_next_pipe_step(
                    pos,
                    Some(alt),
                    |p| rules::is_item_pipe(&hc::block_id_at(host, p)),
                    |p| p == alt,
                ) {
                    let alt_blk = hc::block_id_at(host, alt_step);
                    let rem = if rules::is_item_pipe(&alt_blk) {
                        insert_into_pipe(host, pos, alt_step, stack.clone(), Some(alt))
                    } else if alt_blk == rules::BLOCK_ITEM_SPLITTER {
                        insert_into_splitter(host, alt_step, stack.clone())
                    } else if alt_blk == rules::BLOCK_INDUSTRIAL_SILO {
                        insert_into_silo(host, alt_step, stack.clone())
                    } else if alt_blk == rules::BLOCK_SILO_CONTROLLER {
                        insert_into_silo_controller(host, alt_step, stack.clone())
                    } else if rules::is_conveyor_block(&alt_blk) {
                        insert_into_conveyor(host, alt_step, stack.clone())
                    } else {
                        match hc::insert_item_rem(host, alt_step, Direction::Up, stack.clone()) {
                            None => None,
                            Some(r) => hc::insert_item_rem(host, alt_step, Direction::North, r),
                        }
                    };
                    if rem.is_none() {
                        state.belt_item.clear();
                        state.belt_count = 0;
                        state.progress = 0;
                        state.pipe_has_target = false;
                        store_state(entity, state);
                        host.block_entity_sync(pos);
                        rerouted = true;
                        break;
                    } else if let Some(r) = rem {
                        if r.count() < state.belt_count {
                            state.belt_count = r.count();
                            store_state(entity, state);
                            host.block_entity_sync(pos);
                            rerouted = true;
                            break;
                        }
                    }
                }
            }
            if !rerouted {
                store_state(entity, state);
            }
        }
    }
}

/// Simulates the Tiered Conveyor Belt (straight, turn_left, turn_right, up, down):
/// 1. Picks up dropped items in the world lying on top of the belt.
/// 2. Optionally pulls items from a container behind the belt (`auto_io`).
/// 3. Smoothly advances `state.progress` from `0` to `max_ticks` (20 / 10 / 5 by tier).
/// 4. At the end of the animation, pushes into the next belt/pipe/container, OR drops into the world if at the end!
fn tick_conveyor(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    state: &mut MachineState,
) {
    let dim = host.dimension();
    let (max_ticks, max_batch) = rules::conveyor_tier(&entity.id()).unwrap_or((20, 1));
    let facing = hc::block_property_at(host, pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let shape = hc::block_property_at(host, pos, "shape")
        .unwrap_or_else(|| "straight".to_owned());
    let front_off = rules::conveyor_exit_offset(&facing, &shape);
    let (_, back_off) = rules::conveyor_offsets(&facing);
    let front_pos = [
        pos[0] + front_off[0],
        pos[1] + front_off[1],
        pos[2] + front_off[2],
    ];
    let back_pos = [
        pos[0] + back_off[0],
        pos[1] + (if shape == "down" { 1 } else { 0 }) + back_off[1],
        pos[2] + back_off[2],
    ];

    // 1. Pick up dropped item entities lying on top of this belt if empty
    if state.belt_item.is_empty() {
        let picked = host.level(dim).and_then(|level| {
            let ents = level.entities_in(
                host,
                [pos[0] as f64 + 0.1, pos[1] as f64, pos[2] as f64 + 0.1],
                [pos[0] as f64 + 0.9, pos[1] as f64 + 1.15, pos[2] as f64 + 0.9],
            )?;
            for e in ents {
                if let Some(ie) = host.item_entity(e) {
                    if let Some(st) = ie.stack(host) {
                        return Some((ie, st));
                    }
                }
            }
            None
        });
        if let Some((ie, st)) = picked {
            let take = st.count().min(max_batch);
            state.belt_item = hc::item_id(host, &st);
            state.belt_count = take;
            state.progress = 0;
            let rem = st.shrink(take);
            if let Some(r) = rem { let _ = ie.set_stack(host, r); } else { let _ = ie.set_never_pickup(host); }
            store_state(entity, state);
            host.block_entity_sync(pos);
            return;
        }
    }

    // 2. If Auto-Pull (`state.auto_io`) is enabled and belt is empty, pull up to `max_batch` from container behind
    if state.auto_io && state.belt_item.is_empty() {
        let back_block = hc::block_id_at(host, back_pos);
        if !rules::is_conveyor_block(&back_block) && is_item_container_block(&back_block) {
            if let Some(pulled) = hc::extract_item_at(host, back_pos, Some(Direction::Down), max_batch) {
                state.belt_item = hc::item_id(host, &pulled);
                state.belt_count = pulled.count();
                state.progress = 0;
                store_state(entity, state);
                host.block_entity_sync(pos);
                return;
            }
        }
    }

    if state.belt_item.is_empty() {
        return;
    }

    // 3. Advance smooth travel progress up to `max_ticks`
    if state.progress < max_ticks {
        state.progress += 1;
        store_state(entity, state);
        host.block_entity_sync(pos);
        // If an inline Belt Filter is attached and item does NOT match filter (whitelist/blacklist), deflect to the side!
        if state.progress >= (max_ticks / 2).max(1)
            && state.belt_filter_attached
            && !state.splitter_filters[0].is_empty()
            && !belt_filter_matches(&state.splitter_filters[0], &state.belt_item, state.belt_filter_blacklist)
        {
            let Some(stack) = hc::make_stack(host, &state.belt_item, state.belt_count.max(1)) else { return; };
            let side_offs = [
                rules::conveyor_exit_offset(&facing, "turn_right"),
                rules::conveyor_exit_offset(&facing, "turn_left"),
            ];
            let mut deflected = false;
            for soff in side_offs {
                let sp = [pos[0] + soff[0], pos[1], pos[2] + soff[2]];
                let sb = hc::block_id_at(host, sp);
                if rules::is_conveyor_block(&sb) {
                    if insert_into_conveyor(host, sp, stack.clone()).is_none() {
                        deflected = true;
                        break;
                    }
                } else if is_item_container_block(&sb) {
                    if hc::insert_item_rem(host, sp, Direction::Up, stack.clone()).is_none() {
                        deflected = true;
                        break;
                    }
                } else if matches!(sb.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") {
                    let drop_pos = [
                        pos[0] as f64 + 0.5 + soff[0] as f64 * 0.68,
                        pos[1] as f64 + 0.30,
                        pos[2] as f64 + 0.5 + soff[2] as f64 * 0.68,
                    ];
                    let vel = [soff[0] as f64 * 0.12, 0.06, soff[2] as f64 * 0.12];
                    hc::spawn_item(host, stack.clone(), drop_pos);
                    deflected = true;
                    break;
                }
            }
            if deflected {
                state.belt_item.clear();
                state.belt_count = 0;
                state.progress = 0;
                store_state(entity, state);
                host.block_entity_sync(pos);
                return;
            }
        }
        // If an Item Splitter sits directly on top of this belt, feed it once item reaches center
        if state.progress >= (max_ticks / 2).max(1) {
            let above_pos = [pos[0], pos[1] + 1, pos[2]];
            if hc::block_id_at(host, above_pos) == rules::BLOCK_ITEM_SPLITTER {
                let Some(stack) = hc::make_stack(host, &state.belt_item, state.belt_count.max(1)) else { return; };
                let leftover = insert_into_splitter(host, above_pos, stack);
                if leftover.is_none() {
                    state.belt_item.clear();
                    state.belt_count = 0;
                    state.progress = 0;
                    store_state(entity, state);
                    host.block_entity_sync(pos);
                    return;
                } else if let Some(rest) = leftover {
                    if rest.count() < state.belt_count {
                        state.belt_count = rest.count();
                        store_state(entity, state);
                        host.block_entity_sync(pos);
                    }
                }
            }
        }
        if state.progress < max_ticks {
            return;
        }
    }

    // 4. Animation finished: push forward into next belt/pipe/container, OR drop into world at the end!
    let Some(stack) = hc::make_stack(host, &state.belt_item, state.belt_count.max(1)) else { return; };
    let front_block = hc::block_id_at(host, front_pos);
    let mut dropped_at_end = false;
    let leftover = if rules::is_conveyor_block(&front_block) {
        insert_into_conveyor(host, front_pos, stack)
    } else if front_block == rules::BLOCK_ITEM_SPLITTER {
        insert_into_splitter(host, front_pos, stack)
    } else if rules::is_item_pipe(&front_block) {
        insert_into_pipe(host, pos, front_pos, stack, None)
    } else if front_block == rules::BLOCK_INDUSTRIAL_SILO {
        insert_into_silo(host, front_pos, stack)
    } else if front_block == rules::BLOCK_SILO_CONTROLLER {
        insert_into_silo_controller(host, front_pos, stack)
    } else if is_item_container_block(&front_block) {
        match hc::insert_item_rem(host, front_pos, Direction::Up, stack) {
            None => None,
            Some(r) => hc::insert_item_rem(host, front_pos, Direction::North, r),
        }
    } else if front_block == "minecraft:air" || front_block == "minecraft:cave_air" || front_block == "minecraft:void_air" {
        let drop_pos = [
            pos[0] as f64 + 0.5 + front_off[0] as f64 * 0.68,
            pos[1] as f64 + if shape == "up" { 1.12 } else { 0.32 },
            pos[2] as f64 + 0.5 + front_off[2] as f64 * 0.68,
        ];
        let vel = [
            front_off[0] as f64 * 0.10,
            0.06,
            front_off[2] as f64 * 0.10,
        ];
        hc::spawn_item(host, stack, drop_pos);
        dropped_at_end = true;
        None
    } else {
        // Blocked by a solid non-container block at the end: check if there's a belt 1 block below (step-down)
        let below_front = [front_pos[0], front_pos[1] - 1, front_pos[2]];
        if rules::is_conveyor_block(&hc::block_id_at(host, below_front)) {
            insert_into_conveyor(host, below_front, stack)
        } else {
            Some(stack)
        }
    };

    if dropped_at_end || leftover.is_none() {
        state.belt_item.clear();
        state.belt_count = 0;
        state.progress = 0;
        store_state(entity, state);
        host.block_entity_sync(pos);
    } else if let Some(rest) = leftover {
        if rest.count() < state.belt_count {
            state.belt_count = rest.count();
            store_state(entity, state);
            host.block_entity_sync(pos);
        }
    }
}

/// Simulates the Automated Industrial Quarry (`rusty_factory:quarry`):
/// Excavates blocks in a `5x5`, `7x7`, or `9x9` area behind the Quarry down to Bedrock,
/// moves the 3D gantry & laser to the mined block, and outputs drops into its buffer or pipes/chests!
fn tick_quarry(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
    fortune_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(host, entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    let (tier_radius, tier_speed, tier_fortune) = rules::quarry_tier(&entity.id());
    if state.quarry_radius < tier_radius {
        state.quarry_radius = tier_radius;
    }
    let cost = rules::effective_energy_cost(MachineKind::Quarry, speed_upgrades);
    let base_needed = rules::effective_work_ticks(MachineKind::Quarry, speed_upgrades);
    let needed = (base_needed / tier_speed as i32).max(1);

    // Check if output slot is completely full before mining
    if let Some(existing) = entity.items[rules::SLOT_OUTPUT].as_ref() {
        let max_s = host.max_stack_size(existing.item()).unwrap_or(64);
        if existing.count() >= max_s {
            if state.auto_io {
                try_auto_eject_output(host, pos, entity, 8);
            }
            if entity.items[rules::SLOT_OUTPUT]
                .as_ref()
                .is_some_and(|s| s.count() >= max_s)
            {
                return (StepStatus::OutputFull, 0);
            }
        }
    }

    state.energy -= cost;
    state.progress += 1;

    if state.progress >= needed {
        state.progress = 0;
        let facing = hc::block_property_at(host, pos, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let side = state.quarry_side_len();
        let layer_area = side * side;
        let dim = host.dimension();
        let (min_x, max_x, min_z, max_z) =
            rules::quarry_area_bounds(pos, &facing, state.quarry_radius);

        // 0. Surface flattening: before digging underground, clear any blocks above `pos[1]` inside the quarry frame!
        let mut cleared_surface = false;
        if state.quarry_y_offset == 0 {
            'surface: for sy in (pos[1]..=pos[1] + 5).rev() {
                for sz in min_z..=max_z {
                    for sx in min_x..=max_x {
                        let sp = [sx, sy, sz];
                        if sp == pos {
                            continue;
                        }
                        let b = hc::block_id_at(host, sp);
                        if let Some((drop_item, base_count)) = rules::quarry_drop_for_block(&b) {
                            hc::set_block_by_id(host, sp, "minecraft:air");
                            state.quarry_mined += 1;
                            state.last_target = sp;
                            let produced = hc::make_stack(host, &drop_item, base_count).expect("valid item");
                            store_or_route_output(host, pos, entity, state.auto_io, produced);
                            store_state(entity, state);
                            host.block_entity_sync(pos);
                            cleared_surface = true;
                            break 'surface;
                        }
                    }
                }
            }
        }

        // Scan up to 512 positions to skip over empty air/water/bedrock layers immediately
        for _ in 0..if cleared_surface { 0 } else { 512 } {
            let target_pos = rules::quarry_target_pos(pos, &facing, state);
            if target_pos[1] <= -60 {
                // Reached bottom of the world; wrap back to top layer
                state.quarry_y_offset = 0;
                state.quarry_cursor = 0;
                break;
            }

            // Advance cursor for next step
            state.quarry_cursor += 1;
            if state.quarry_cursor >= layer_area {
                state.quarry_cursor = 0;
                state.quarry_y_offset += 1;
            }

            let block_id = hc::block_id_at(host, target_pos);
            if let Some((drop_item, base_count)) = rules::quarry_drop_for_block(&block_id) {
                hc::set_block_by_id(host, target_pos, "minecraft:air");
                state.quarry_mined += 1;
                state.last_target = target_pos;

                let bonus = if block_id.ends_with("_ore") || block_id == "minecraft:ancient_debris" {
                    (fortune_upgrades + tier_fortune).min(6)
                } else {
                    0
                };
                let produced = hc::make_stack(host, &drop_item, base_count + bonus).expect("valid item");

                store_or_route_output(host, pos, entity, state.auto_io, produced);
                store_state(entity, state);
                host.block_entity_sync(pos);
                break;
            }
        }
    }

    if state.auto_io && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(host, pos, entity, 4);
    }

    (StepStatus::Working, -cost)
}

/// Inserts `produced` into connected pipes/containers (if `auto_io`) or `SLOT_OUTPUT`.
fn store_or_route_output(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    auto_io: bool,
    produced: impl Into<Option<ItemStack>>,
) {
    let Some(produced) = produced.into() else { return; };
    let mut rest = Some(produced);
    if auto_io {
        rest = route_item_from(host, pos, rest.take().unwrap());
    }
    if let Some(leftover) = rest {
        match entity.items[rules::SLOT_OUTPUT].as_mut() {
            Some(existing) if &*hc::item_id(host, &existing) == &*hc::item_id(host, &leftover) => {
                let max_s = host.max_stack_size(existing.item()).unwrap_or(64);
                let _ = existing.set_count((existing.count() + leftover.count()).min(max_s));
            }
            None => {
                entity.items[rules::SLOT_OUTPUT] = Some(leftover);
            }
            _ => {}
        }
    }
}

/// Simulates the 3D Automated Crop & Tree Harvester (`rusty_factory:auto_harvester`):
/// Scans a configurable `5x5`, `7x7`, or `9x9` area behind it, replants mature crops (`age=0`),
/// harvests sugar cane / bamboo / melons / pumpkins / wood logs, and routes drops into pipes/chests!
fn tick_harvester(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
    fortune_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(host, entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    let cost = rules::effective_energy_cost(MachineKind::AutoHarvester, speed_upgrades);
    let needed = rules::effective_work_ticks(MachineKind::AutoHarvester, speed_upgrades);

    state.energy -= cost;
    state.progress += 1;

    if state.progress >= needed {
        state.progress = 0;
        let facing = hc::block_property_at(host, pos, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let side = state.quarry_side_len();
        let layer_area = side * side;
        let dim = host.dimension();

        for _ in 0..8 {
            let target_pos = rules::harvester_target_pos(pos, &facing, state);
            state.quarry_cursor += 1;
            if state.quarry_cursor >= layer_area {
                state.quarry_cursor = 0;
                state.quarry_y_offset = (state.quarry_y_offset + 1) % 2;
            }

            let block_id = hc::block_id_at(host, target_pos);
            let age_prop = hc::block_property_at(host, target_pos, "age");
            if let Some((drop_item, base_count, replant)) =
                rules::harvester_drop_for_block(&block_id, age_prop.as_deref())
            {
                // For sugar cane / bamboo / cactus at base Y, only break if there is another one above or we are at Y+1
                if matches!(
                    block_id.as_str(),
                    "minecraft:sugar_cane" | "minecraft:bamboo" | "minecraft:cactus"
                ) && target_pos[1] == pos[1]
                {
                    let above = [target_pos[0], target_pos[1] + 1, target_pos[2]];
                    if hc::block_id_at(host, above) == block_id {
                        hc::set_block_by_id(host, above, "minecraft:air");
                        state.last_target = above;
                    } else {
                        continue;
                    }
                } else if replant {
                    // Replant mature crop back to growth stage 0 (or stage 1 for sweet berry bushes)
                    let reset_age = if block_id == "minecraft:sweet_berry_bush" {
                        "1"
                    } else {
                        "0"
                    };
                    host.set_block_property(target_pos, "age", reset_age);
                    state.last_target = target_pos;
                } else {
                    hc::set_block_by_id(host, target_pos, "minecraft:air");
                    state.last_target = target_pos;
                }

                state.quarry_mined += 1;
                let produced = hc::make_stack(host, &drop_item, base_count + fortune_upgrades.min(4)).expect("valid item");
                store_or_route_output(host, pos, entity, state.auto_io, produced);
                store_state(entity, state);
                host.block_entity_sync(pos);
                break;
            }
        }
    }

    if state.auto_io && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(host, pos, entity, 4);
    }

    (StepStatus::Working, -cost)
}

/// Simulates the Automated Pneumatic Pipe Extractor (`rusty_factory:item_extractor`):
/// Automatically pulls items from adjacent Chests/Barrels/Furnaces/Hoppers and routes them
/// through connected `rusty_factory:item_pipe` networks or `conveyor_belt`s!
/// If `SLOT_INPUT` contains an item, it acts as a Smart Filter (only extracting matching items).
fn tick_extractor(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    state: &mut MachineState,
    speed_upgrades: u32,
) -> StepStatus {
    let outcome = rules::step_machine(state, None, None, None, speed_upgrades);
    if outcome.status != StepStatus::Working {
        return outcome.status;
    }

    if state.progress == 0 {
        let dim = host.dimension();
        let filter_item = entity.items[rules::SLOT_INPUT]
            .as_ref()
            .map(|s| hc::item_id(host, &s));
        let mut pipe_batch = 4u32;
        for [dx, dy, dz] in rules::NEIGHBORS_6 {
            let b = hc::block_id_at(host, [pos[0] + dx, pos[1] + dy, pos[2] + dz]);
            if rules::is_item_pipe(&b) {
                pipe_batch = pipe_batch.max(rules::item_pipe_batch(&b));
            }
        }
        let batch_size = ((2 + speed_upgrades.min(8) * 2).min(16)).max(pipe_batch);

        let targets = rules::find_connected_item_targets(
            pos,
            |p| rules::is_item_pipe(&hc::block_id_at(host, p)),
            |p| is_item_container_block(&hc::block_id_at(host, p)),
        );

        for [dx, dy, dz] in rules::NEIGHBORS_6 {
            let adj = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let adj_block = hc::block_id_at(host, adj);
            if adj_block == rules::BLOCK_INDUSTRIAL_SILO {
                let mut take_info: Option<(String, u32)> = None;
                hc::with_mod_block_entity(host, adj, |sent| {
                    if let Some(mut sst) = load_state(sent) {
                        let matches_filter = filter_item
                            .as_deref()
                            .is_none_or(|f| f == sst.belt_item);
                        if matches_filter && !sst.belt_item.is_empty() && sst.belt_count > 0 {
                            let take = sst.belt_count.min(batch_size);
                            take_info = Some((sst.belt_item.clone(), take));
                            sst.belt_count -= take;
                            if sst.belt_count == 0 {
                                sst.belt_item.clear();
                            }
                            store_state(sent, &sst);
                        }
                    }
                });
                let extracted = take_info.and_then(|(id, take)| hc::make_stack(host, &id, take));
                if let Some(st_out) = extracted {
                    host.block_entity_sync(adj);
                    if let Some(rest) = route_item_from(host, pos, st_out) {
                        let _ = insert_into_silo(host, adj, rest);
                    } else {
                        state.quarry_mined += 1;
                    }
                    return StepStatus::Working;
                }
            } else if matches!(
                adj_block.as_str(),
                "minecraft:chest"
                    | "minecraft:trapped_chest"
                    | "minecraft:barrel"
                    | "minecraft:hopper"
                    | "minecraft:furnace"
                    | "minecraft:blast_furnace"
                    | "minecraft:smoker"
                    | "minecraft:shulker_box"
            ) {
                for &dest in &targets {
                    if dest == adj {
                        continue;
                    }
                    let moved = hc::move_items(host,
                        adj,
                        Direction::Down,
                        dest,
                        Direction::Up,
                        filter_item.as_deref(),
                        batch_size,
                    );
                    if moved > 0 {
                        state.quarry_mined += moved as i32;
                        return StepStatus::Working;
                    }
                }
            }
        }
    }

    StepStatus::Working
}

/// Simulates the Directional Block Breaker (`rusty_factory:block_breaker`):
/// Mines the block directly in front of its `facing` face (ideal for cobblestone/obsidian generators).
fn tick_breaker(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
    fortune_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(host, entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    let facing = hc::block_property_at(host, pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let (front_off, _) = rules::conveyor_offsets(&facing);
    let target_pos = [
        pos[0] + front_off[0],
        pos[1] + front_off[1],
        pos[2] + front_off[2],
    ];
    let block_id = hc::block_id_at(host, target_pos);
    let Some((drop_item, base_count)) = rules::quarry_drop_for_block(&block_id) else {
        state.progress = 0;
        return (StepStatus::Idle, 0);
    };

    let cost = rules::effective_energy_cost(MachineKind::BlockBreaker, speed_upgrades);
    let needed = rules::effective_work_ticks(MachineKind::BlockBreaker, speed_upgrades);
    state.energy -= cost;
    state.progress += 1;

    if state.progress >= needed {
        state.progress = 0;
        let dim = host.dimension();
        hc::set_block_by_id(host, target_pos, "minecraft:air");
        state.quarry_mined += 1;
        let produced = hc::make_stack(host, &drop_item, base_count + fortune_upgrades.min(4)).expect("valid item");
        store_or_route_output(host, pos, entity, state.auto_io, produced);
    }

    if state.auto_io && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(host, pos, entity, 4);
    }

    (StepStatus::Working, -cost)
}

/// Simulates the Automated Block Placer (`rusty_factory:block_placer`):
/// Places block items from `SLOT_INPUT` into the world directly in front of its `facing` face when air.
fn tick_placer(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(host, entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    let Some(in_stack) = entity.items[rules::SLOT_INPUT].clone() else {
        state.progress = 0;
        return (StepStatus::Idle, 0);
    };
    let place_block_id = hc::item_id(host, &in_stack);
    if host.block_key(&place_block_id).is_none() {
        state.progress = 0;
        return (StepStatus::Idle, 0);
    }

    let facing = hc::block_property_at(host, pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let (front_off, _) = rules::conveyor_offsets(&facing);
    let target_pos = [
        pos[0] + front_off[0],
        pos[1] + front_off[1],
        pos[2] + front_off[2],
    ];
    let current_block = hc::block_id_at(host, target_pos);
    if !matches!(
        current_block.as_str(),
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water"
    ) {
        state.progress = 0;
        return (StepStatus::Idle, 0);
    }

    let cost = rules::effective_energy_cost(MachineKind::BlockPlacer, speed_upgrades);
    let needed = rules::effective_work_ticks(MachineKind::BlockPlacer, speed_upgrades);
    state.energy -= cost;
    state.progress += 1;

    if state.progress >= needed {
        state.progress = 0;
        if hc::set_block_by_id(host, target_pos, &place_block_id) {
            state.quarry_mined += 1;
            entity.items[rules::SLOT_INPUT] = in_stack.shrink(1);
        }
    }

    (StepStatus::Working, -cost)
}

/// Simulates the Automated Sap Extractor (`rusty_factory:sap_extractor`):
/// Scans adjacent blocks (and vertical trunk columns around it) for `rubber_sap_log` or `rubber_log`,
/// extracts `raw_sap` without destroying the tree, and refills sap cavities over time!
fn tick_sap_extractor(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
    fortune_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(host, entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    // Look for any Rubber Tree trunk block around the extractor (horizontal neighbors + up to 4 blocks up)
    let mut found_trunk: Option<([i32; 3], bool)> = None;
    'search: for [dx, _, dz] in [[-1, 0, 0], [1, 0, 0], [0, 0, -1], [0, 0, 1]] {
        for dy in 0..=4 {
            let p = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let bid = hc::block_id_at(host, p);
            if bid == rules::BLOCK_RUBBER_SAP_LOG {
                let has_sap = hc::block_property_at(host, p, "lit").as_deref() == Some("true");
                found_trunk = Some((p, has_sap));
                if has_sap {
                    break 'search;
                }
            } else if bid == rules::BLOCK_RUBBER_LOG && found_trunk.is_none() {
                found_trunk = Some((p, false));
            }
        }
    }

    let Some((trunk_pos, is_active_cavity)) = found_trunk else {
        state.progress = 0;
        return (StepStatus::Idle, 0);
    };

    let cost = rules::effective_energy_cost(MachineKind::SapExtractor, speed_upgrades);
    let needed = rules::effective_work_ticks(MachineKind::SapExtractor, speed_upgrades);
    state.energy -= cost;
    state.progress += 1;

    if state.progress >= needed {
        state.progress = 0;
        if is_active_cavity {
            host.set_block_property(trunk_pos, "lit", "false");
        }
        let base_sap = if is_active_cavity { 3 } else { 1 };
        let sap_count = base_sap + fortune_upgrades.min(4);
        state.quarry_mined += sap_count as i32;
        state.last_target = trunk_pos;

        let produced = hc::make_stack(host, rules::ITEM_RAW_SAP.into(), sap_count).expect("valid item");
        store_or_route_output(host, pos, entity, state.auto_io, produced);
        store_state(entity, state);
        host.block_entity_sync(pos);
    }

    if state.auto_io && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(host, pos, entity, 4);
    }

    (StepStatus::Working, -cost)
}

fn consume_one_fuel(host: &dyn Host, entity: &mut ModBlockEntityView<'_>) {
    let slot_idx = if entity.items.len() == 1 { 0 } else { rules::SLOT_FUEL };
    if let Some(fuel_stack) = entity.items.get(slot_idx).and_then(|s| s.clone()) {
        let is_lava = hc::item_id(host, &fuel_stack) == "minecraft:lava_bucket";
        if fuel_stack.count() > 1 {
            entity.items[slot_idx] = fuel_stack.shrink(1);
        } else {
            entity.items[slot_idx] = if is_lava {
                hc::make_stack(host, "minecraft:bucket", 1)
            } else {
                None
            };
        }
    }
}

pub(crate) fn tick(host: &mut dyn ServerHost, pos: [i32; 3], entity: &mut ModBlockEntityView<'_>) {
    let block_id = hc::block_id_at(host, pos);
    tick_player_equipment(host);

    // 1. Rubber Sapling automatic growth tick (grows into a full Rubber Tree after ~8 seconds)
    if block_id == rules::BLOCK_RUBBER_SAPLING {
        let t = entity.data.first().copied().unwrap_or(0) + 1;
        if t >= 160 {
            grow_rubber_tree(host, pos);
        } else {
            entity.data = vec![t];
        }
        return;
    }

    // 2. Rubber Sap Log Cavity natural sap regeneration tick (refills golden resin after ~10 seconds)
    if entity.id() == rules::BLOCK_RUBBER_SAP_LOG {
        let is_full = hc::block_property_at(host, pos, "lit").as_deref() == Some("true");
        if !is_full {
            let t = entity.data.first().copied().unwrap_or(0) + 1;
            if t >= 200 {
                host.set_block_property(pos, "lit", "true");
                entity.data = vec![0];
            } else {
                entity.data = vec![t];
            }
        }
        return;
    }

    let Some(kind) = MachineKind::from_block_id(&entity.id()) else {
        return;
    };
    let min_slots = match kind {
        MachineKind::CoalGenerator
        | MachineKind::BioGenerator
        | MachineKind::EnergyBank
        | MachineKind::SolarGenerator
        | MachineKind::ChargingPad
        | MachineKind::VoidTrashCan
        | MachineKind::FluidTank
        | MachineKind::FluidPump => 1,
        MachineKind::EnergyMonitor
        | MachineKind::EnergyRelay
        | MachineKind::ConveyorBelt
        | MachineKind::ItemPipe
        | MachineKind::FluidPipe
        | MachineKind::ItemSplitter => 0,
        _ => rules::SLOTS,
    };
    if entity.items.len() < min_slots {
        entity.items.resize(min_slots, None);
    }

    let mut state = load_state(entity).unwrap_or_else(|| MachineState::new(kind));
    let speed_upgrades = entity.items.get(rules::SLOT_UPGRADE)
        .and_then(|s| s.as_ref())
        .filter(|s| &*hc::item_id(host, &s) == rules::ITEM_SPEED_UPGRADE)
        .map_or(0, |s| s.count());
    let fortune_upgrades = entity.items.get(rules::SLOT_UPGRADE)
        .and_then(|s| s.as_ref())
        .filter(|s| &*hc::item_id(host, &s) == rules::ITEM_FORTUNE_UPGRADE)
        .map_or(0, |s| s.count());

    if kind == MachineKind::FluidPipe {
        if state.fluid_amount > 0 {
            state.fluid_amount = (state.fluid_amount - 5).max(0);
            if state.fluid_amount == 0 {
                state.fluid_kind.clear();
            }
            store_state(entity, &state);
            host.block_entity_sync(pos);
        }
        return;
    }

    if kind == MachineKind::ItemSplitter {
        if state.enabled && state.auto_io && (hc::game_time(host) % 5 == 0) {
            let dim = host.dimension();
            let facing = hc::block_property_at(host, pos, "facing")
                .unwrap_or_else(|| "north".to_owned());
            let (_, back_off) = rules::conveyor_offsets(&facing);
            let back_pos = [pos[0] + back_off[0], pos[1] + back_off[1], pos[2] + back_off[2]];
            let back_block = hc::block_id_at(host, back_pos);
            if !rules::is_conveyor_block(&back_block) && is_item_container_block(&back_block) {
                if let Some(pulled) = hc::extract_item_at(host, back_pos, Some(Direction::Down), 1) {
                    if let Some(rest) = insert_into_splitter(host, pos, pulled) {
                        let _ = hc::insert_item_rem(host, back_pos, Direction::Up, rest);
                    }
                }
            }
        }
        return;
    }

    if kind == MachineKind::ItemPipe {
        tick_item_pipe(host, pos, entity, &mut state);
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::ConveyorBelt {
        if state.enabled {
            tick_conveyor(host, pos, entity, &mut state);
        }
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::ItemExtractor {
        let status = tick_extractor(host, pos, entity, &mut state, speed_upgrades);
        update_lit_state(host, pos, status == StepStatus::Working);
        let period = rules::effective_work_ticks(kind, speed_upgrades).max(1);
        let pct = ((state.progress * 100) / period).clamp(0, 100);
        entity.data = vec![
            status as i32,
            0,
            1,
            pct,
            0,
            i32::from(state.enabled),
            i32::from(state.auto_io),
            0,
            0,
            state.quarry_mined,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::VoidTrashCan {
        if entity.items.iter().any(|s| s.is_some()) {
            entity.items = vec![None];
            update_lit_state(host, pos, true);
        } else {
            update_lit_state(host, pos, false);
        }
        entity.data = vec![StepStatus::Working as i32, 0, 1, 0, 0, 1, 0, 0, 0, 0];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::FluidPump {
        let mut status = StepStatus::Idle;
        let mut net_rate = 0;
        let mut detected_fluid: Option<&'static str> = None;
        for [dx, dy, dz] in rules::NEIGHBORS_6 {
            let check_p = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let b = hc::block_id_at(host, check_p);
            if matches!(b.as_str(), "minecraft:water" | "minecraft:flowing_water") {
                detected_fluid = Some("water");
                break;
            } else if matches!(b.as_str(), "minecraft:lava" | "minecraft:flowing_lava") {
                detected_fluid = Some("lava");
                break;
            }
        }
        if let Some(fk) = detected_fluid {
            state.fluid_kind = fk.to_owned();
        } else {
            state.fluid_kind.clear();
        }
        if state.enabled {
            let cost = 15;
            if let Some(fk) = detected_fluid {
                if state.energy >= cost {
                    state.energy -= cost;
                    net_rate = -cost;
                    status = StepStatus::Working;
                    let pump_mb = 100 + (speed_upgrades.min(8) as i32) * 50;
                    let mut tank_targets: Vec<[i32; 3]> = Vec::new();
                    let mut pipe_bonus = 0i32;
                    for [dx, dy, dz] in rules::NEIGHBORS_6 {
                        let target = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
                        let tb = hc::block_id_at(host, target);
                        if tb == rules::BLOCK_FLUID_TANK {
                            tank_targets.push(target);
                        } else if rules::is_fluid_pipe(&tb) {
                            pipe_bonus = pipe_bonus.max(rules::fluid_pipe_rate(&tb));
                            hc::with_mod_block_entity(host, target, |p_ent| {
                                let mut p_st = load_state(p_ent).unwrap_or_else(|| MachineState::new(MachineKind::FluidPipe));
                                p_st.fluid_kind = fk.to_owned();
                                p_st.fluid_amount = rules::fluid_pipe_cap(&tb);
                                store_state(p_ent, &p_st);
                            });
                            host.block_entity_sync(target);
                            let tanks = rules::find_connected_item_targets(
                                target,
                                |p| rules::is_fluid_pipe(&hc::block_id_at(host, p)),
                                |p| hc::block_id_at(host, p) == rules::BLOCK_FLUID_TANK,
                            );
                            tank_targets.extend(tanks);
                        }
                    }
                    let effective_pump_mb = pump_mb.max(pipe_bonus);
                    for tank_pos in tank_targets {
                        let mut filled_now = false;
                        hc::with_mod_block_entity(host, tank_pos, |t_ent| {
                            let mut t_st = load_state(t_ent).unwrap_or_else(|| MachineState::new(MachineKind::FluidTank));
                            if (t_st.fluid_kind.is_empty() || t_st.fluid_kind == fk)
                                && t_st.fluid_amount < rules::FLUID_TANK_MAX_MB
                            {
                                t_st.fluid_kind = fk.to_owned();
                                t_st.fluid_amount = (t_st.fluid_amount + effective_pump_mb).min(rules::FLUID_TANK_MAX_MB);
                                store_state(t_ent, &t_st);
                                filled_now = true;
                            }
                        });
                        if filled_now {
                            host.set_block_property(tank_pos, "lit", "true");
                            break;
                        }
                    }
                } else {
                    status = StepStatus::NoPower;
                }
            }
        }
        update_lit_state(host, pos, status == StepStatus::Working);
        let fcode = rules::fluid_code(&state.fluid_kind);
        let rate_mb = if status == StepStatus::Working {
            100 + (speed_upgrades.min(8) as i32) * 50
        } else {
            0
        };
        entity.data = vec![
            status as i32,
            state.energy,
            kind.max_energy(),
            if status == StepStatus::Working { 100 } else { 0 },
            net_rate,
            i32::from(state.enabled),
            0,
            fcode,
            rate_mb,
            0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::UpgradeStation {
        return;
    }

    if kind == MachineKind::GeothermalGenerator {
        // Consume lava_bucket in slot 0 or pull lava from adjacent FluidTank/FluidPipe
        if let Some(Some(slot0)) = entity.items.first_mut() {
            if hc::item_id(host, &slot0).as_str() == "minecraft:lava_bucket" && state.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB {
                state.fluid_kind = "lava".to_owned();
                state.fluid_amount += 1000;
                if let Some(b) = hc::make_stack(host, "minecraft:bucket", 1) {
                    *slot0 = b;
                }
            }
        }
        if state.fluid_amount < 500 {
            for d in rules::DIRS {
                let adj = [pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]];
                if matches!(hc::block_id_at(host, adj).as_str(), rules::BLOCK_FLUID_TANK | rules::BLOCK_FLUID_PIPE | rules::BLOCK_FLUID_PIPE_MK2 | rules::BLOCK_FLUID_PIPE_MK3) {
                    let mut pulled = 0;
                    hc::with_mod_block_entity(host, adj, |other| {
                        if let Some(mut ost) = load_state(other) {
                            if ost.fluid_kind == "lava" && ost.fluid_amount > 0 {
                                let take = ost.fluid_amount.min(250);
                                ost.fluid_amount -= take;
                                if ost.fluid_amount == 0 {
                                    ost.fluid_kind.clear();
                                }
                                store_state(other, &ost);
                                pulled = take;
                            }
                        }
                    });
                    if pulled > 0 {
                        state.fluid_kind = "lava".to_owned();
                        state.fluid_amount += pulled;
                        break;
                    }
                }
            }
        }
        let mut net_rate = 0;
        if state.enabled && state.fluid_kind == "lava" && state.fluid_amount >= 10 && state.energy < kind.max_energy() {
            state.fluid_amount -= 10;
            if state.fluid_amount == 0 {
                state.fluid_kind.clear();
            }
            let produced_fe = 120.min(kind.max_energy() - state.energy);
            state.energy += produced_fe;
            net_rate = produced_fe;
        }
        if state.energy > 0 {
            let sent = distribute_energy(host, pos, state.energy.min(240), false);
            state.energy -= sent;
        }
        update_lit_state(host, pos, net_rate > 0);
        entity.data = vec![
            if net_rate > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            (state.fluid_amount * 100 / rules::FLUID_TANK_MAX_MB).clamp(0, 100),
            net_rate,
            i32::from(state.enabled),
            0, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::SteamTurbine {
        if state.fluid_amount < 500 {
            for d in rules::DIRS {
                let adj = [pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]];
                if matches!(hc::block_id_at(host, adj).as_str(), rules::BLOCK_STEAM_BOILER | rules::BLOCK_FLUID_TANK | rules::BLOCK_FLUID_PIPE | rules::BLOCK_FLUID_PIPE_MK2 | rules::BLOCK_FLUID_PIPE_MK3) {
                    let mut pulled = 0;
                    hc::with_mod_block_entity(host, adj, |other| {
                        if let Some(mut ost) = load_state(other) {
                            if ost.fluid_kind == "steam" && ost.fluid_amount > 0 {
                                let take = ost.fluid_amount.min(300);
                                ost.fluid_amount -= take;
                                if ost.fluid_amount == 0 {
                                    ost.fluid_kind.clear();
                                }
                                store_state(other, &ost);
                                pulled = take;
                            }
                        }
                    });
                    if pulled > 0 {
                        state.fluid_kind = "steam".to_owned();
                        state.fluid_amount += pulled;
                        break;
                    }
                }
            }
        }
        let mut net_rate = 0;
        if state.enabled && state.fluid_kind == "steam" && state.fluid_amount >= 15 && state.energy < kind.max_energy() {
            state.fluid_amount -= 15;
            if state.fluid_amount == 0 {
                state.fluid_kind.clear();
            }
            let produced_fe = 160.min(kind.max_energy() - state.energy);
            state.energy += produced_fe;
            net_rate = produced_fe;
        }
        if state.energy > 0 {
            let sent = distribute_energy(host, pos, state.energy.min(320), false);
            state.energy -= sent;
        }
        update_lit_state(host, pos, net_rate > 0);
        entity.data = vec![
            if net_rate > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            (state.fluid_amount * 100 / rules::FLUID_TANK_MAX_MB).clamp(0, 100),
            net_rate,
            i32::from(state.enabled),
            0, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::Recycler {
        let mut working = false;
        if state.enabled && state.energy >= 10 && entity.items[rules::SLOT_INPUT].is_some() {
            state.energy -= 10;
            state.progress += 1;
            working = true;
            if state.progress >= 20 {
                state.progress = 0;
                if let Some(in_stack) = entity.items[rules::SLOT_INPUT].as_mut() {
                    if in_stack.count() > 1 {
                        let _ = in_stack.set_count(in_stack.count().saturating_sub(1).max(1));
                    } else {
                        entity.items[rules::SLOT_INPUT] = None;
                    }
                }
                const SCRAP_DROPS: [&str; 6] = [
                    rules::ITEM_IRON_DUST,
                    rules::ITEM_COPPER_DUST,
                    rules::ITEM_SILICON,
                    rules::ITEM_SULFUR_DUST,
                    rules::ITEM_RAW_SAP,
                    rules::ITEM_GOLD_DUST,
                ];
                let pick = SCRAP_DROPS[(state.quarry_mined as usize) % SCRAP_DROPS.len()];
                state.quarry_mined += 1;
                let prod = hc::make_stack(host, pick, 1);
                store_or_route_output(host, pos, entity, state.auto_io, prod);
            }
        }
        update_lit_state(host, pos, working);
        entity.data = vec![
            if working { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            (state.progress * 100 / 20).clamp(0, 100),
            if working { -10 } else { 0 },
            i32::from(state.enabled),
            i32::from(state.auto_io),
            0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::IndustrialSilo {
        let cap = state.silo_capacity.max(2_048);
        if let Some(Some(slot0)) = entity.items.first_mut() {
            let in_id = hc::item_id(host, &slot0);
            if state.belt_item.is_empty() || state.belt_item == in_id {
                let space = cap.saturating_sub(state.belt_count);
                let add = slot0.count().min(space);
                if add > 0 {
                    state.belt_item = in_id;
                    state.belt_count += add;
                    if slot0.count() > add {
                        let _ = slot0.set_count(slot0.count().saturating_sub(add).max(1));
                    } else {
                        entity.items[0] = None;
                    }
                }
            }
        }
        let pct = ((state.belt_count as u64 * 100) / cap.max(1) as u64).min(100) as i32;
        entity.data = vec![
            if state.belt_count > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.belt_count as i32,
            cap as i32,
            pct,
            0, 1, 1, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::SiloController {
        if let Some(in_stack) = entity.items.first_mut().and_then(|s| s.take()) {
            if let Some(rest) = insert_into_silo_controller(host, pos, in_stack) {
                entity.items[0] = Some(rest);
            }
        }
        let search = get_terminal_search();
        let (list, silos_n, chests_n) = filtered_silo_network(host, pos, state.auto_io, state.enabled, &search);
        let max_page = (list.len().saturating_sub(1) / 27) as i32;
        if state.quarry_cursor > max_page {
            state.quarry_cursor = max_page;
        }
        let page = state.quarry_cursor.max(0) as usize;
        update_lit_state(host, pos, silos_n + chests_n > 0);
        entity.data = vec![
            if silos_n + chests_n > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            silos_n as i32,
            chests_n as i32,
            list.len() as i32,
            if state.auto_io { 1 } else { 0 },
            if state.enabled { 1 } else { 0 },
            page as i32,
            pos[0],
            pos[1],
            pos[2],
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::SiloCraftingTerminal {
        if entity.items.len() < 10 {
            entity.items.resize(10, None);
        }
        // Automatic crafting table behavior:
        // state.progress == 1 means `slot 9` currently holds an unconsumed crafting preview.
        let matched = match_terminal_craft(host, &entity.items[0..9]);
        if state.progress == 1 {
            if entity.items[9].is_none() {
                // Player took the crafted item! Consume 1 ingredient from each non-empty slot in 0..=8 and auto-refill from Silos.
                let mut depleted: Vec<(usize, String)> = Vec::new();
                for i in 0..9 {
                    if let Some(s) = entity.items[i].as_mut() {
                        if s.count() <= 1 {
                            depleted.push((i, hc::item_id(host, &s)));
                            entity.items[i] = None;
                        } else {
                            let _ = s.set_count(s.count().saturating_sub(1).max(1));
                        }
                    }
                }
                for (i, item_id) in depleted {
                    if pull_one_from_silo_network(host, pos, &item_id) {
                        entity.items[i] = hc::make_stack(host, &item_id, 1);
                    }
                }
                state.progress = 0;
            } else {
                // Update or clear preview if grid changed
                match matched {
                    Some((out_id, out_cnt)) => {
                        let same = entity.items[9]
                            .as_ref()
                            .is_some_and(|s| hc::item_id(host, s).as_str() == out_id && s.count() == out_cnt);
                        if !same {
                            entity.items[9] = hc::make_stack(host, out_id, out_cnt);
                        }
                    }
                    None => {
                        entity.items[9] = None;
                        state.progress = 0;
                    }
                }
            }
        }
        if state.progress == 0 && entity.items[9].is_none() {
            if let Some((out_id, out_cnt)) = match_terminal_craft(host, &entity.items[0..9]) {
                entity.items[9] = hc::make_stack(host, out_id, out_cnt);
                state.progress = 1;
            }
        }
        let search = get_terminal_search();
        let (list, silos_n, chests_n) = filtered_silo_network(host, pos, state.auto_io, state.enabled, &search);
        let max_page = (list.len().saturating_sub(1) / 27) as i32;
        if state.quarry_cursor > max_page {
            state.quarry_cursor = max_page;
        }
        let page = state.quarry_cursor.max(0) as usize;
        update_lit_state(host, pos, silos_n + chests_n > 0);
        entity.data = vec![
            if silos_n + chests_n > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            silos_n as i32,
            chests_n as i32,
            list.len() as i32,
            if state.auto_io { 1 } else { 0 },
            if state.enabled { 1 } else { 0 },
            page as i32,
            pos[0],
            pos[1],
            pos[2],
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::NuclearReactor {
        if state.burn_left <= 0 {
            if let Some(Some(slot0)) = entity.items.first_mut() {
                let fuel_ticks = match hc::item_id(host, &slot0).as_str() {
                    "minecraft:nether_star" => 6_000,
                    rules::ITEM_NETHERITE_SEED_CORE => 2_400,
                    "minecraft:diamond" | "minecraft:emerald" => 600,
                    "minecraft:redstone_block" | "minecraft:blaze_rod" => 300,
                    _ => 0,
                };
                if fuel_ticks > 0 {
                    state.burn_left = fuel_ticks;
                    state.burn_max = fuel_ticks;
                    if slot0.count() <= 1 {
                        entity.items[0] = None;
                    } else {
                        let _ = slot0.set_count(slot0.count().saturating_sub(1).max(1));
                    }
                }
            }
        }
        let mut net_rate = 0;
        if state.enabled && state.burn_left > 0 && state.energy < kind.max_energy() {
            state.burn_left -= 1;
            let prod = 1_000.min(kind.max_energy() - state.energy);
            state.energy += prod;
            net_rate = prod;
        }
        if state.energy > 0 {
            let sent = distribute_energy(host, pos, state.energy.min(2_000), false);
            state.energy -= sent;
        }
        update_lit_state(host, pos, net_rate > 0);
        let pct = if state.burn_max > 0 {
            (state.burn_left * 100 / state.burn_max).clamp(0, 100)
        } else {
            0
        };
        entity.data = vec![
            if net_rate > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            pct,
            net_rate,
            i32::from(state.enabled),
            0, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::WindTurbine {
        let alt_rate = (15 + ((pos[1] - 60).max(0) / 2)).min(120);
        let mut net_rate = 0;
        if state.enabled && state.energy < kind.max_energy() {
            let prod = alt_rate.min(kind.max_energy() - state.energy);
            state.energy += prod;
            net_rate = prod;
        }
        if state.energy > 0 {
            let sent = distribute_energy(host, pos, state.energy.min(240), false);
            state.energy -= sent;
        }
        update_lit_state(host, pos, net_rate > 0);
        let pct = ((alt_rate * 100) / 120).clamp(0, 100);
        entity.data = vec![
            if net_rate > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            pct,
            alt_rate,
            i32::from(state.enabled),
            0, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::HydroponicChamber {
        let period = rules::effective_work_ticks(kind, speed_upgrades).max(1);
        let cost = rules::effective_energy_cost(kind, speed_upgrades);
        let mut status = StepStatus::Idle;
        let out_item = entity.items.first().and_then(|s| s.as_ref()).and_then(|s| {
            rules::lookup_recipe(MachineKind::HydroponicChamber, hc::item_id(host, &s).as_str())
        });
        if state.enabled && out_item.is_some() {
            let out_full = entity.items.get(rules::SLOT_OUTPUT).and_then(|s| s.as_ref()).is_some_and(|s| s.count() >= 64);
            if out_full {
                status = StepStatus::OutputFull;
            } else if state.energy < cost {
                status = StepStatus::NoPower;
            } else {
                status = StepStatus::Working;
                state.energy -= cost;
                state.progress += 1;
                if state.progress >= period {
                    state.progress = 0;
                    if let Some(rec) = out_item {
                        if let Some(produced) = hc::make_stack(host, rec.item, rec.count) {
                            store_or_route_output(host, pos, entity, state.auto_io, produced);
                        }
                    }
                }
            }
        } else {
            state.progress = 0;
        }
        if state.auto_io && entity.items.get(rules::SLOT_OUTPUT).is_some_and(|s| s.is_some()) {
            try_auto_eject_output(host, pos, entity, 4);
        }
        update_lit_state(host, pos, status == StepStatus::Working);
        let pct = ((state.progress * 100) / period).clamp(0, 100);
        entity.data = vec![
            status as i32,
            state.energy,
            kind.max_energy(),
            pct,
            if status == StepStatus::Working { -cost } else { 0 },
            i32::from(state.enabled),
            i32::from(state.auto_io),
            0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::LaserTurret {
        let period = (15 - speed_upgrades as i32 * 2).max(5);
        let mut fired = false;
        if state.enabled && state.energy >= 40 {
            state.progress += 1;
            if state.progress >= period {
                state.progress = 0;
                state.energy -= 40;
                fired = true;
                let [cx, cy, cz] = pos;
                hc::run_command(host, &format!(
                    "execute positioned {cx}.5 {cy}.8 {cz}.5 run damage @e[type=!player,type=!item,type=!experience_orb,type=!armor_stand,distance=..16,limit=1,sort=nearest] 10 minecraft:lightning_bolt"
                ));
                hc::run_command(host, &format!(
                    "particle minecraft:electric_spark {cx}.5 {cy}.9 {cz}.5 0.6 0.6 0.6 0.1 8"
                ));
            }
        }
        update_lit_state(host, pos, state.enabled && state.energy >= 40);
        let pct = ((state.progress * 100) / period.max(1)).clamp(0, 100);
        entity.data = vec![
            if fired || (state.enabled && state.energy >= 40) { StepStatus::Working as i32 } else { StepStatus::NoPower as i32 },
            state.energy,
            kind.max_energy(),
            pct,
            if fired { -40 } else { 0 },
            i32::from(state.enabled),
            0, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::WirelessCharger {
        let mut charged_any = false;
        let pad_center = [pos[0] as f64 + 0.5, pos[1] as f64 + 0.5, pos[2] as f64 + 0.5];
        if state.enabled && state.energy >= 20 {
            for p in host.players().server_players() {
                let in_range = p.entity().position(host).is_some_and(|pp| {
                    let d2 = (pp[0] - pad_center[0]).powi(2)
                        + (pp[1] - pad_center[1]).powi(2)
                        + (pp[2] - pad_center[2]).powi(2);
                    d2 <= 32.0 * 32.0
                });
                if !in_range {
                    continue;
                }
                let living = LivingRef::new(p.entity());
                for eq in [EquipmentSlot::Head, EquipmentSlot::Chest, EquipmentSlot::Legs, EquipmentSlot::Feet, EquipmentSlot::OffHand] {
                    if state.energy < 20 { break; }
                    if let Some(mut st) = living.equipment(host, eq) {
                        let iid = hc::item_id(host, &st);
                        if let Some(cap) = rules::battery_capacity(&iid) {
                            let cur = get_item_energy(&st);
                            if cur < cap {
                                let add = 250.min(cap - cur).min(state.energy);
                                if add > 0 {
                                    set_item_energy(&mut st, cur + add);
                                    let _ = living.set_equipment(host, eq, Some(st));
                                    state.energy -= add;
                                    charged_any = true;
                                }
                            }
                        }
                    }
                }
                if let Some(slots) = p.inventory_slots(host) {
                    for (slot_idx, mut st) in slots {
                        if state.energy < 20 { break; }
                        let iid = hc::item_id(host, &st);
                        if let Some(cap) = rules::battery_capacity(&iid) {
                            let cur = get_item_energy(&st);
                            if cur < cap {
                                let add = 250.min(cap - cur).min(state.energy);
                                if add > 0 {
                                    set_item_energy(&mut st, cur + add);
                                    let _ = p.set_inventory_slot(host, slot_idx, Some(st));
                                    state.energy -= add;
                                    charged_any = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        update_lit_state(host, pos, charged_any);
        entity.data = vec![
            if charged_any { StepStatus::Working as i32 } else if state.energy < 20 { StepStatus::NoPower as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            if charged_any { 100 } else { 0 },
            if charged_any { -250 } else { 0 },
            i32::from(state.enabled),
            0, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::TeleporterPad {
        hc::record_server_be(pos, rules::BLOCK_TELEPORTER_PAD);
        if state.progress > 0 {
            state.progress -= 1;
        }
        let mut peers: Vec<[i32; 3]> = hc::SERVER_ACTIVE_BE
            .lock()
            .map(|m| {
                m.iter()
                    .filter(|(p, id)| id.as_str() == rules::BLOCK_TELEPORTER_PAD && **p != pos)
                    .map(|(p, _)| *p)
                    .collect()
            })
            .unwrap_or_default();
        peers.retain(|&p| hc::block_id_at(host, p) == rules::BLOCK_TELEPORTER_PAD);
        peers.sort();
        if state.progress == 0 && state.enabled && !peers.is_empty() && state.energy >= 1_000 {
            let crouching_player = host.players().server_players().into_iter().find(|&p| {
                let ent = p.entity();
                hc::is_crouching(host, ent)
                    && ent.position(host).is_some_and(|pp| {
                        (pp[0] - (pos[0] as f64 + 0.5)).abs() <= 0.85
                            && (pp[2] - (pos[2] as f64 + 0.5)).abs() <= 0.85
                            && (pp[1] - pos[1] as f64) >= 0.0
                            && (pp[1] - pos[1] as f64) <= 1.6
                    })
            });
            if let Some(who) = crouching_player {
                let dest = peers[0];
                state.energy -= 1_000;
                state.progress = 40;
                hc::run_command(host, &format!("tp @s {}.5 {} {}.5", dest[0], dest[1] + 1, dest[2]));
                hc::run_command(host, "playsound minecraft:entity.enderman.teleport player @s ~ ~ ~ 1.0 1.1");
                let _ = who.action_bar(host,
                    Text::literal(format!("🌀 [Teletransporte] Destino [{}, {}, {}] (-1,000 FE)", dest[0], dest[1], dest[2])),
                );
            }
        }
        update_lit_state(host, pos, !peers.is_empty() && state.energy >= 1_000);
        entity.data = vec![
            if !peers.is_empty() && state.energy >= 1_000 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            if state.progress > 0 { (state.progress * 100 / 40).clamp(0, 100) } else { 100 },
            peers.len() as i32,
            i32::from(state.enabled),
            0, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::Tesseract {
        hc::record_server_be(pos, rules::BLOCK_TESSERACT);
        // 1. Wireless cross-world energy sharing with other Tesseracts
        let mut peers: Vec<[i32; 3]> = hc::SERVER_ACTIVE_BE
            .lock()
            .map(|m| {
                m.iter()
                    .filter(|(p, id)| id.as_str() == rules::BLOCK_TESSERACT && **p != pos)
                    .map(|(p, _)| *p)
                    .collect()
            })
            .unwrap_or_default();
        peers.retain(|&p| hc::block_id_at(host, p) == rules::BLOCK_TESSERACT);
        if state.energy > 0 {
            let sent = distribute_energy(host, pos, state.energy.min(500), false);
            state.energy -= sent;
        }
        if !peers.is_empty() && state.energy > 100 {
            let share = (state.energy / (peers.len() as i32 + 1)).min(1_000);
            if share > 0 {
                for peer in &peers {
                    let mut accepted = 0;
                    hc::with_mod_block_entity(host, *peer, |pent| {
                        if let Some(mut pst) = load_state(pent) {
                            let room = kind.max_energy() - pst.energy;
                            let give = share.min(room);
                            if give > 0 {
                                pst.energy += give;
                                store_state(pent, &pst);
                                accepted = give;
                            }
                        }
                    });
                    state.energy -= accepted;
                }
            }
        }
        // 2. Wireless item teleportation from SLOT_INPUT to another Tesseract's SLOT_OUTPUT
        if !peers.is_empty() {
            if let Some(in_stack) = entity.items[rules::SLOT_INPUT].take() {
                let mut delivered = false;
                for peer in &peers {
                    hc::with_mod_block_entity(host, *peer, |pent| {
                        if pent.items[rules::SLOT_OUTPUT].is_none() {
                            pent.items[rules::SLOT_OUTPUT] = Some(in_stack.clone());
                            delivered = true;
                        }
                    });
                    if delivered {
                        break;
                    }
                }
                if !delivered {
                    entity.items[rules::SLOT_INPUT] = Some(in_stack);
                }
            }
        }
        if entity.items[rules::SLOT_OUTPUT].is_some() {
            if let Some(out) = entity.items[rules::SLOT_OUTPUT].take() {
                if let Some(rest) = route_item_from(host, pos, out) {
                    entity.items[rules::SLOT_OUTPUT] = Some(rest);
                }
            }
        }
        update_lit_state(host, pos, !peers.is_empty() && state.energy > 0);
        entity.data = vec![
            if !peers.is_empty() { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            (state.energy * 100 / kind.max_energy().max(1)).clamp(0, 100),
            0,
            i32::from(state.enabled),
            1, 0, 0, 0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::EnergyRelay {
        if state.enabled && state.energy > 0 {
            let sent = distribute_energy(host, pos, state.energy.min(500), false);
            state.energy -= sent;
        }
        update_lit_state(host, pos, state.enabled && state.energy > 0);
        let pct = (state.energy * 100) / kind.max_energy().max(1);
        entity.data = vec![1, state.energy, kind.max_energy(), pct, 0, i32::from(state.enabled), 0, 0, 0, 0];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::FluidTank {
        // Process bucket / fluid item in slot 0 inside Tank GUI
        if let Some(Some(slot0)) = entity.items.first_mut() {
            let item_id_str = hc::item_id(host, slot0);
            let item_id = item_id_str.as_str();
            if item_id == "minecraft:water_bucket"
                && (state.fluid_kind.is_empty() || state.fluid_kind == "water")
                && state.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB
            {
                state.fluid_kind = "water".to_owned();
                state.fluid_amount += 1000;
                if let Some(b) = hc::make_stack(host, "minecraft:bucket", 1) {
                    *slot0 = b;
                }
            } else if item_id == "minecraft:lava_bucket"
                && (state.fluid_kind.is_empty() || state.fluid_kind == "lava")
                && state.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB
            {
                state.fluid_kind = "lava".to_owned();
                state.fluid_amount += 1000;
                if let Some(b) = hc::make_stack(host, "minecraft:bucket", 1) {
                    *slot0 = b;
                }
            } else if item_id == rules::ITEM_RAW_SAP
                && (state.fluid_kind.is_empty() || state.fluid_kind == "sap")
                && state.fluid_amount + 250 <= rules::FLUID_TANK_MAX_MB
            {
                state.fluid_kind = "sap".to_owned();
                state.fluid_amount += 250;
                if slot0.count() > 1 {
                    let _ = slot0.set_count(slot0.count().saturating_sub(1).max(1));
                } else {
                    entity.items[0] = None;
                }
            } else if item_id == "minecraft:bucket" && slot0.count() == 1 && state.fluid_amount >= 1000 {
                if state.fluid_kind == "water" {
                    state.fluid_amount -= 1000;
                    if let Some(b) = hc::make_stack(host, "minecraft:water_bucket", 1) {
                        *slot0 = b;
                    }
                    if state.fluid_amount == 0 {
                        state.fluid_kind.clear();
                    }
                } else if state.fluid_kind == "lava" {
                    state.fluid_amount -= 1000;
                    if let Some(b) = hc::make_stack(host, "minecraft:lava_bucket", 1) {
                        *slot0 = b;
                    }
                    if state.fluid_amount == 0 {
                        state.fluid_kind.clear();
                    }
                }
            }
        }
        update_lit_state(host, pos, state.fluid_amount > 0);
        let pct = ((state.fluid_amount * 100) / rules::FLUID_TANK_MAX_MB).clamp(0, 100);
        let fcode = rules::fluid_code(&state.fluid_kind);
        entity.data = vec![
            if state.fluid_amount > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            0,
            1,
            pct,
            0,
            1,
            0,
            fcode,
            state.fluid_amount,
            rules::FLUID_TANK_MAX_MB,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::EnergyMonitor {
        store_state(entity, &state);
        return;
    }

    let is_gen = matches!(kind, MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler);
    let fuel_ticks = if is_gen {
        entity.items.first().and_then(|s| s.as_ref()).and_then(|s| fuel_burn_ticks(host, s))
    } else {
        entity.items.get(rules::SLOT_FUEL).and_then(|s| s.as_ref()).and_then(|s| fuel_burn_ticks(host, s))
    };

    if is_gen {
        let max_e = kind.max_energy();
        let outcome = rules::step_machine(&mut state, None, fuel_ticks, None, speed_upgrades);
        if outcome.consume_fuel {
            consume_one_fuel(host, entity);
        }
        let gen_rate = if outcome.status == StepStatus::Working {
            kind.energy_per_tick() + (speed_upgrades.min(8) as i32) * 10
        } else {
            0
        };
        if state.energy > 0 {
            let sent = distribute_energy(host, pos, state.energy.min(kind.energy_per_tick() * 4), false);
            state.energy -= sent;
        }
        update_lit_state(host, pos, outcome.status == StepStatus::Working);
        let pct = if state.burn_max > 0 {
            ((state.burn_left * 100) / state.burn_max).clamp(0, 100)
        } else {
            0
        };
        entity.data = vec![
            outcome.status as i32,
            state.energy,
            max_e,
            pct,
            gen_rate,
            i32::from(state.enabled),
            0,
            0,
            state.burn_left,
            state.burn_max,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::SolarGenerator {
        let is_array_3x3 = rules::is_solar_array_3x3(pos, |p| hc::block_id_at(host, p) == rules::BLOCK_SOLAR);
        let max_e = if is_array_3x3 { 20_000 } else { 2_000 };
        let mut net_rate = 0;
        let status = if !state.enabled {
            StepStatus::Paused
        } else {
            let (_block_light, sky_light) = hc::light_at(host, [pos[0], pos[1] + 1, pos[2]]);
            let day_time = hc::day_time(host);
            let (rain, _thunder) = hc::weather(host);
            let solar_gen = rules::solar_generation_rate(day_time, sky_light as u8, rain, speed_upgrades, is_array_3x3);
            if solar_gen > 0 {
                state.energy = (state.energy + solar_gen).min(max_e);
                net_rate = solar_gen;
            }
            if state.energy > 0 {
                let sent = distribute_energy(host, pos, state.energy.min(if is_array_3x3 { 600 } else { 200 }), false);
                state.energy -= sent;
            }
            // Recharging item placed in single centered slot 0 (Battery, Jetpack, Drill, Jet Boots, Mining Laser, Solar Helmet)
            if let Some(Some(item_stack)) = entity.items.first_mut() {
                let charge_cost = 40;
                if state.energy >= charge_cost {
                    if matches!(
                        hc::item_id(host, &item_stack).as_str(),
                        rules::ITEM_BATTERY
                            | rules::ITEM_JETPACK
                            | rules::ITEM_JET_BOOTS
                            | rules::ITEM_MINING_DRILL
                            | rules::ITEM_MINING_LASER
                            | rules::ITEM_SOLAR_HELMET
                    ) {
                        let dmg = item_stack.damage();
                        if dmg > 0 {
                            item_stack.set_damage(dmg.saturating_sub(1));
                            state.energy -= charge_cost;
                        }
                    }
                }
            }
            if solar_gen > 0 {
                StepStatus::Working
            } else {
                StepStatus::Idle
            }
        };
        update_lit_state(host, pos, status == StepStatus::Working);
        let pct = ((state.energy * 100) / max_e.max(1)).clamp(0, 100);
        entity.data = vec![
            status as i32,
            state.energy,
            max_e,
            pct,
            net_rate,
            i32::from(state.enabled),
            i32::from(state.auto_io),
            i32::from(is_array_3x3),
            0,
            0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::ChargingPad {
        // 1. Recharging slot for placed item:
        if let Some(Some(item_stack)) = entity.items.first_mut() {
            let charge_cost = 50;
            if state.energy >= charge_cost {
                if let Some(cap) = rules::battery_capacity(hc::item_id(host, &item_stack).as_str()) {
                    let cur = get_item_energy(item_stack);
                    if cur < cap {
                        let add = 500.min(cap - cur).min(state.energy);
                        if add > 0 {
                            set_item_energy(item_stack, cur + add);
                            state.energy -= add;
                        }
                    }
                    let dmg = item_stack.damage();
                    if dmg > 0 {
                        item_stack.set_damage(dmg.saturating_sub(2));
                        state.energy = (state.energy - charge_cost).max(0);
                    }
                }
            }
        }
        // 2. Standing induction wireless charging for player & entire inventory (armor, hotbar, inventory, offhand):
        let mut player_on_pad = false;
        let pad_center = [pos[0] as f64 + 0.5, pos[1] as f64 + 0.2, pos[2] as f64 + 0.5];
        if let Some(player) = host.players().acting_player() {
            if let Some(ppos) = player.entity().position(host) {
                let dx = (ppos[0] - pad_center[0]).abs();
                let dz = (ppos[2] - pad_center[2]).abs();
                let dy = ppos[1] - pos[1] as f64;
                if dx <= 0.9 && dz <= 0.9 && (0.0..=1.6).contains(&dy) {
                player_on_pad = true;
                if state.energy >= 20 {
                    state.energy -= 20;
                    if (hc::game_time(host) + pos[0] as i64) % 4 == 0 {
                        let phase = ((hc::game_time(host) % 20) as f64) * 0.09;
                        hc::run_command(host, &format!(
                            "particle minecraft:electric_spark {} {} {} 0.35 0.15 0.35 0.04 10",
                            pad_center[0], pos[1] as f64 + 1.05 + phase, pad_center[2]
                        ));
                        hc::run_command(host, &format!(
                            "particle minecraft:end_rod {} {} {} 0.25 0.1 0.25 0.02 3",
                            pad_center[0], pos[1] as f64 + 1.05 + phase, pad_center[2]
                        ));
                    }
                    if (hc::game_time(host) + pos[0] as i64) % 25 == 0 {
                        hc::run_command(host, "playsound minecraft:block.beacon.ambient player @s ~ ~ ~ 0.35 1.8");
                        if let Some(who) = host.players().acting_player() {
                            let _ = who.action_bar(host,
                                Text::literal("⚡ [Cargador Inductivo] Cargando todo tu inventario y armadura..."),
                            );
                        }
                    }
                }
            }
            }
        }
        if player_on_pad && state.energy > 0 {
            if let Some(who) = host.players().acting_player() {
                let living = LivingRef::new(who.entity());
                let mut transferred = 0;
                for eq in [EquipmentSlot::Head, EquipmentSlot::Chest, EquipmentSlot::Legs, EquipmentSlot::Feet] {
                    if transferred >= state.energy { break; }
                    if let Some(mut st) = living.equipment(host, eq) {
                        let iid = hc::item_id(host, &st);
                        if let Some(cap) = rules::battery_capacity(&iid) {
                            let cur = get_item_energy(&st);
                            let add = 500.min(cap - cur).min(state.energy - transferred);
                            if add > 0 {
                                set_item_energy(&mut st, cur + add);
                                let _ = living.set_equipment(host, eq, Some(st));
                                transferred += add;
                            }
                        }
                    }
                }
                if let Some(slots) = who.inventory_slots(host) {
                    for (idx, mut st) in slots {
                        if transferred >= state.energy { break; }
                        let iid = hc::item_id(host, &st);
                        if let Some(cap) = rules::battery_capacity(&iid) {
                            let cur = get_item_energy(&st);
                            let add = 500.min(cap - cur).min(state.energy - transferred);
                            if add > 0 {
                                set_item_energy(&mut st, cur + add);
                                let _ = who.set_inventory_slot(host, idx, Some(st));
                                transferred += add;
                            }
                        }
                    }
                }
                state.energy -= transferred;
            }
        }
        update_lit_state(host, pos, player_on_pad || entity.items.first().is_some_and(|s| s.is_some()));
        let pct = ((state.energy * 100) / kind.max_energy().max(1)).clamp(0, 100);
        entity.data = vec![
            if player_on_pad { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            state.energy,
            kind.max_energy(),
            pct,
            if player_on_pad { -25 } else { 0 },
            i32::from(state.enabled),
            0,
            0,
            0,
            0,
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::EnergyBank {
        let cluster = rules::find_battery_cluster(pos, |p| hc::block_id_at(host, p) == rules::BLOCK_ENERGY_BANK);
        let cluster_len = cluster.len() as i32;
        let cluster_max = cluster_len * 10_000;

        let prev_energy = state.energy;
        let outcome = rules::step_machine(&mut state, None, fuel_ticks, None, speed_upgrades);
        if outcome.consume_fuel {
            consume_one_fuel(host, entity);
        }
        let mut net_rate = state.energy - prev_energy;
        if state.enabled && state.auto_io && state.energy > 0 {
            let sent = distribute_energy(host, pos, state.energy.min(400), true);
            state.energy -= sent;
            if sent > 0 {
                net_rate = -sent;
            }
        }
        update_lit_state(host, pos, state.enabled && state.energy > 0);
        let pct = ((state.energy * 100) / kind.max_energy().max(1)).clamp(0, 100);
        entity.data = vec![
            outcome.status as i32,
            state.energy,
            cluster_max,
            pct,
            net_rate,
            i32::from(state.enabled),
            i32::from(state.auto_io),
            cluster_len,
            0,
            0,
        ];
        store_state(entity, &state);
        return;
    }

    if matches!(
        kind,
        MachineKind::Quarry
            | MachineKind::AutoHarvester
            | MachineKind::BlockBreaker
            | MachineKind::BlockPlacer
            | MachineKind::SapExtractor
    ) {
        let (status, rate) = match kind {
            MachineKind::Quarry => tick_quarry(
                host,
                pos,
                entity,
                &mut state,
                fuel_ticks,
                speed_upgrades,
                fortune_upgrades,
            ),
            MachineKind::AutoHarvester => tick_harvester(
                host,
                pos,
                entity,
                &mut state,
                fuel_ticks,
                speed_upgrades,
                fortune_upgrades,
            ),
            MachineKind::BlockBreaker => tick_breaker(
                host,
                pos,
                entity,
                &mut state,
                fuel_ticks,
                speed_upgrades,
                fortune_upgrades,
            ),
            MachineKind::BlockPlacer => {
                tick_placer(host, pos, entity, &mut state, fuel_ticks, speed_upgrades)
            }
            MachineKind::SapExtractor => tick_sap_extractor(
                host,
                pos,
                entity,
                &mut state,
                fuel_ticks,
                speed_upgrades,
                fortune_upgrades,
            ),
            _ => unreachable!(),
        };
        update_lit_state(host, pos, status == StepStatus::Working);
        let needed = rules::effective_work_ticks(kind, speed_upgrades).max(1);
        let pct = ((state.progress * 100) / needed).clamp(0, 100);
        let cur_y = if kind == MachineKind::Quarry {
            (pos[1] - 1 - state.quarry_y_offset).max(-60)
        } else {
            pos[1]
        };
        entity.data = vec![
            status as i32,
            state.energy,
            kind.max_energy(),
            pct,
            rate,
            i32::from(state.enabled),
            i32::from(state.auto_io),
            state.quarry_radius.clamp(2, 4),
            cur_y,
            state.quarry_mined,
        ];
        store_state(entity, &state);
        return;
    }

    if entity.items.len() < rules::SLOTS {
        entity.items.resize(rules::SLOTS, None);
    }

    let input_id = entity.items[rules::SLOT_INPUT]
        .as_ref()
        .map(|s| hc::item_id(host, &s));
    let output_info = entity.items[rules::SLOT_OUTPUT].as_ref().map(|s| {
        let max_stack = host.max_stack_size(s.item()).unwrap_or(64);
        (hc::item_id(host, &s), s.count(), max_stack)
    });

    let outcome = rules::step_machine(
        &mut state,
        input_id.as_deref(),
        fuel_ticks,
        output_info
            .as_ref()
            .map(|(id, c, m)| (id.as_str(), *c, *m)),
        speed_upgrades,
    );

    if outcome.consume_fuel {
        consume_one_fuel(host, entity);
    }

    if outcome.consume_input {
        if let Some(in_stack) = entity.items[rules::SLOT_INPUT].as_mut() {
            if in_stack.count() > 1 {
                let _ = in_stack.set_count(in_stack.count().saturating_sub(1).max(1));
            } else {
                entity.items[rules::SLOT_INPUT] = None;
            }
        }
    }

    if let Some(prod) = outcome.produced {
        let extra_yield = if matches!(kind, MachineKind::ElectricCrusher | MachineKind::Vulcanizer) {
            fortune_upgrades.min(4)
        } else {
            0
        };
        let total_count = prod.count + extra_yield;
        match entity.items[rules::SLOT_OUTPUT].as_mut() {
            Some(existing) if hc::item_id(host, existing) == prod.item => {
                let max_s = host.max_stack_size(existing.item()).unwrap_or(64);
                let _ = existing.set_count((existing.count() + total_count).min(max_s));
            }
            None => {
                entity.items[rules::SLOT_OUTPUT] = hc::make_stack(host, prod.item, total_count);
            }
            _ => {}
        }
    }

    let is_gen = matches!(kind, MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler);
    if state.auto_io && !is_gen && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(host, pos, entity, 4);
    }

    if is_gen && state.energy > 0 {
        let max_send = if kind == MachineKind::SteamBoiler { 600 } else { 280 };
        let sent = distribute_energy(host, pos, state.energy.min(max_send), false);
        state.energy -= sent;
    }


    update_lit_state(host, pos, outcome.status == StepStatus::Working);

    // Dynamic machine sound & particle effects
    if outcome.status == StepStatus::Working && ((hc::game_time(host) + (pos[0] as i64 * 11)) % 15 == 0) {
        let px = pos[0] as f64 + 0.5;
        let py = pos[1] as f64 + 1.05;
        let pz = pos[2] as f64 + 0.5;
        match kind {
            MachineKind::CoalGenerator | MachineKind::BioGenerator => {
                hc::run_command(host, &format!("particle minecraft:smoke {px} {py} {pz} 0.15 0.1 0.15 0.01 2"));
            }
            MachineKind::SteamBoiler => {
                hc::run_command(host, &format!("particle minecraft:cloud {px} {py} {pz} 0.15 0.15 0.15 0.02 2"));
            }
            MachineKind::ElectricCrusher | MachineKind::MechanicalPress => {
                hc::run_command(host, &format!("particle minecraft:crit {px} {py} {pz} 0.2 0.1 0.2 0.04 3"));
            }
            MachineKind::ElectricFurnace | MachineKind::AlloySmelter => {
                hc::run_command(host, &format!("particle minecraft:flame {px} {py} {pz} 0.15 0.08 0.15 0.01 2"));
            }
            _ => {}
        }
    }
    if outcome.status == StepStatus::Working && ((hc::game_time(host) + (pos[0] as i64 * 11)) % 40 == 0) {
        match kind {
            MachineKind::SteamBoiler => {
                hc::run_command(host, &format!("playsound minecraft:block.fire.extinguish block @a {} {} {} 0.3 1.2", pos[0], pos[1], pos[2]));
            }
            MachineKind::ElectricCrusher => {
                hc::run_command(host, &format!("playsound minecraft:block.grindstone.use block @a {} {} {} 0.35 0.9", pos[0], pos[1], pos[2]));
            }
            MachineKind::MechanicalPress => {
                hc::run_command(host, &format!("playsound minecraft:block.piston.extend block @a {} {} {} 0.4 0.8", pos[0], pos[1], pos[2]));
            }
            MachineKind::AlloySmelter => {
                hc::run_command(host, &format!("playsound minecraft:block.blastfurnace.fire_crackle block @a {} {} {} 0.35 1.1", pos[0], pos[1], pos[2]));
            }
            _ => {}
        }
    }


    let needed = rules::effective_work_ticks(kind, speed_upgrades).max(1);
    let pct = if is_gen {
        state.progress
    } else {
        ((state.progress * 100) / needed).clamp(0, 100)
    };

    let data = vec![
        outcome.status as i32,
        state.energy,
        kind.max_energy(),
        pct,
        outcome.net_energy_rate,
        i32::from(state.enabled),
        i32::from(state.auto_io),
        0,
        0,
        0,
    ];

    if entity.data != data {
        entity.data = data;
    }
    store_state(entity, &state);
}

/// 3D Block Entity Renderer (`native_block_entity_submit`):
/// Renders items smoothly gliding along Tiered Conveyor Belts (straight, left/right curves, up/down slopes)
/// AND travelling visibly inside glass Pneumatic Item Pipes!
pub(crate) fn submit(
    host: &dyn ClientHost,
    pos: [i32; 3],
    block: &str,
    state: BlockState,
    entity: &ModBlockEntity,
    draw: &mut dyn BlockEntityDrawContext,
) {
    if let Some(st) = load_state(entity) {
        hc::record_client_be(pos, block, st);
    }

    if block == rules::BLOCK_INDUSTRIAL_SILO {
        let Some(st) = load_state(entity) else {
            return;
        };
        if !st.belt_item.is_empty() && st.belt_count > 0 {
            let facing_str = hc::block_state_property(host, state, "facing");
            let facing = facing_str.as_deref().unwrap_or("north");
            let (front_off, _) = rules::conveyor_offsets(facing);
            let yaw = match facing {
                "north" => 0.0,
                "south" => 180.0,
                "east" => 270.0,
                "west" => 90.0,
                _ => 0.0,
            };
            let Some(stack) = hc::make_stack(host, &st.belt_item, 1) else { return; };
            let prev_light = draw.light();
            draw.set_light(240 | (240 << 8));
            draw.push();
            draw.translate([
                0.5 + front_off[0] as f32 * 0.535,
                0.60,
                0.5 + front_off[2] as f32 * 0.535,
            ]);
            draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
            draw.scale([0.44, 0.44, 0.08]);
            let _ = hc::draw_be_item_stack(draw,
                &stack,
                ItemDisplayContext::Fixed,
                None,
                pos[0].wrapping_mul(31) ^ pos[2],
            );
            draw.pop();
            draw.set_light(prev_light);
        }
        return;
    }

    if block == rules::BLOCK_ITEM_SPLITTER {
        let Some(st) = load_state(entity) else {
            return;
        };
        let facing_str = hc::block_state_property(host, state, "facing");
        let facing = facing_str.as_deref().unwrap_or("north");
        let side_shapes = ["turn_left", "straight", "turn_right"];
        for (i, shape_name) in side_shapes.iter().enumerate() {
            let filter_id = &st.splitter_filters[i];
            if filter_id.is_empty() {
                continue;
            }
            let off = rules::conveyor_exit_offset(facing, shape_name);
            let yaw = match off {
                [0, 0, -1] => 0.0,
                [0, 0, 1] => 180.0,
                [1, 0, 0] => 270.0,
                [-1, 0, 0] => 90.0,
                _ => 0.0,
            };
            let Some(stack) = hc::make_stack(host, filter_id, 1) else { return; };
            draw.push();
            draw.translate([
                0.5 + off[0] as f32 * 0.46,
                0.48,
                0.5 + off[2] as f32 * 0.46,
            ]);
            draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
            draw.scale([0.34, 0.34, 0.34]);
            let _ = hc::draw_be_item_stack(draw,
                &stack,
                ItemDisplayContext::Fixed,
                None,
                pos[0].wrapping_mul(31) ^ pos[2] ^ i as i32,
            );
            draw.pop();
        }
        return;
    }

    if rules::is_item_pipe(block) {
        return;
    }

    let Some((max_ticks, _)) = rules::conveyor_tier(block) else {
        return;
    };
    let Some(st) = load_state(entity) else {
        return;
    };
    // Render 3D filtered item icon floating above the belt filter arch
    if st.belt_filter_attached && !st.splitter_filters[0].is_empty() {
        let first_f = st.splitter_filters[0].split(',').next().unwrap_or("").trim();
        if !first_f.is_empty() {
            if let Some(fstack) = hc::make_stack(host, first_f, 1) {
                draw.push();
                draw.translate([0.5, 0.72, 0.5]);
                draw.scale([0.36, 0.36, 0.36]);
                let _ = hc::draw_be_item_stack(draw,
                    &fstack,
                    ItemDisplayContext::Fixed,
                    None,
                    pos[0].wrapping_mul(17) ^ pos[2],
                );
                draw.pop();
            }
        }
    }
    if st.belt_item.is_empty() {
        return;
    }
    let facing_str = hc::block_state_property(host, state, "facing");
    let facing = facing_str.as_deref().unwrap_or("north");
    let shape_str = hc::block_state_property(host, state, "shape");
    let shape = shape_str.as_deref().unwrap_or("straight");

    let t = ((st.progress as f32 + if st.enabled && st.progress < max_ticks { draw.partial() } else { 0.0 })
        / max_ticks as f32)
        .clamp(0.0, 1.0);

    let (_front_off, back_off) = rules::conveyor_offsets(facing);
    let exit_off = rules::conveyor_exit_offset(facing, shape);

    let (ix, iy, iz, yaw) = match shape {
        "turn_left" | "turn_right" => {
            let base_yaw = match facing {
                "north" => 0.0,
                "south" => 180.0,
                "east" => 270.0,
                "west" => 90.0,
                _ => 0.0,
            };
            if t < 0.5 {
                let u = t * 2.0;
                (
                    0.5 + back_off[0] as f32 * 0.45 * (1.0 - u),
                    0.28,
                    0.5 + back_off[2] as f32 * 0.45 * (1.0 - u),
                    base_yaw,
                )
            } else {
                let u = (t - 0.5) * 2.0;
                let turn_delta = if shape == "turn_left" { 90.0 } else { -90.0 };
                (
                    0.5 + exit_off[0] as f32 * 0.45 * u,
                    0.28,
                    0.5 + exit_off[2] as f32 * 0.45 * u,
                    base_yaw + turn_delta * u,
                )
            }
        }
        "up" => {
            let travel = 0.08 + t * 0.84;
            let (x, z, y_rot) = match facing {
                "north" => (0.5, 1.0 - travel, 0.0),
                "south" => (0.5, travel, 180.0),
                "east" => (travel, 0.5, 270.0),
                "west" => (1.0 - travel, 0.5, 90.0),
                _ => (0.5, 1.0 - travel, 0.0),
            };
            (x, 0.28 + t * 0.85, z, y_rot)
        }
        "down" => {
            let travel = 0.08 + t * 0.84;
            let (x, z, y_rot) = match facing {
                "north" => (0.5, 1.0 - travel, 0.0),
                "south" => (0.5, travel, 180.0),
                "east" => (travel, 0.5, 270.0),
                "west" => (1.0 - travel, 0.5, 90.0),
                _ => (0.5, 1.0 - travel, 0.0),
            };
            (x, 1.13 - t * 0.85, z, y_rot)
        }
        _ => {
            let travel = 0.12 + t * 0.76;
            let (x, z, y_rot) = match facing {
                "north" => (0.5, 1.0 - travel, 0.0),
                "south" => (0.5, travel, 180.0),
                "east" => (travel, 0.5, 270.0),
                "west" => (1.0 - travel, 0.5, 90.0),
                _ => (0.5, 1.0 - travel, 0.0),
            };
            (x, 0.28, z, y_rot)
        }
    };

    let Some(stack) = hc::make_stack(host, &st.belt_item, 1) else { return; };
    draw.push();
    draw.translate([ix, iy, iz]);
    draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
    draw.scale([0.46, 0.46, 0.46]);
    let _ = hc::draw_be_item_stack(draw,
        &stack,
        ItemDisplayContext::Fixed,
        None,
        pos[0].wrapping_mul(31) ^ pos[2],
    );
    draw.pop();
}

/// 3D World Primitives (`native_world_primitives`):
/// Renders the BuildCraft-style 3D Quarry Frame, Moving X/Z Gantry Bridge, Drill Head,
/// glowing Mining Laser beam, and floating Quarry Telemetry Hologram!
pub(crate) fn world_primitives(
    host: &dyn ClientHost,
    view: &WorldRenderView,
    draw: &mut dyn WorldDrawContext,
) {
    let partial = view.partial_tick;
    let camera = view.camera;
    let mut prim_buf = hc::WorldPrimitivesCompat::new();
    let primitives = &mut prim_buf;

    let cached: Vec<([i32; 3], String, MachineState)> = hc::CLIENT_VISIBLE_BE
        .lock()
        .map(|m| m.iter().map(|(&p, (id, st))| (p, id.clone(), st.clone())).collect())
        .unwrap_or_default();

    for (pos, be_id, st) in cached {
        let [x, y, z] = pos;
        let dx = x as f64 + 0.5 - camera[0];
        let dy = y as f64 + 0.5 - camera[1];
        let dz = z as f64 + 0.5 - camera[2];
        if dx * dx + dy * dy + dz * dz > 64.0 * 64.0 {
            continue;
        }
        if hc::block_id_at(host, pos) != be_id {
            continue;
        }

    if be_id == rules::BLOCK_INDUSTRIAL_SILO {
            let facing = hc::block_property_at(host, pos, "facing")
                .unwrap_or_else(|| "north".to_owned());
            let (front_off, _) = rules::conveyor_offsets(&facing);
            let cap = st.silo_capacity.max(2_048) as f64;
            let ratio = (st.belt_count as f64 / cap).clamp(0.0, 1.0);
            let bar_col = if ratio >= 0.90 {
                0xFFE6_3232 // Rojo: lleno / casi lleno
            } else if ratio >= 0.65 {
                0xFFE6_B822 // Amarillo: llenándose
            } else {
                0xFF38_D96B // Verde: mucha capacidad libre
            };
            let cx = x as f64 + 0.5 + front_off[0] as f64 * 0.51;
            let cz = z as f64 + 0.5 + front_off[2] as f64 * 0.51;
            let (rx, rz) = if front_off[0] != 0 { (0.015, 0.34) } else { (0.34, 0.015) };
            // Dark front display bezel
            primitives.filled_box([cx - rx, y as f64 + 0.16, cz - rz], [cx + rx, y as f64 + 0.28, cz + rz], 0xFF16_1A22);
            // Colored capacity bar (Green / Yellow / Red)
            let fill_w = (ratio.max(0.06)) * 0.30;
            let (fx, fz) = if front_off[0] != 0 { (0.02, fill_w) } else { (fill_w, 0.02) };
            primitives.filled_box([cx - fx, y as f64 + 0.18, cz - fz], [cx + fx, y as f64 + 0.26, cz + fz], bar_col);
            continue;
        }

        if be_id == rules::BLOCK_TESSERACT {
            let pulse = ((hc::game_time(host) as f64 + partial as f64) * 0.15).sin() * 0.04;
            let r = 0.22 + pulse;
            primitives.filled_box(
                [x as f64 + 0.5 - r, y as f64 + 0.5 - r, z as f64 + 0.5 - r],
                [x as f64 + 0.5 + r, y as f64 + 0.5 + r, z as f64 + 0.5 + r],
                0xCC38_E0FF,
            );
            primitives.box_outline([x as f64 + 0.12, y as f64 + 0.12, z as f64 + 0.12], [x as f64 + 0.88, y as f64 + 0.88, z as f64 + 0.88], 0xFF78_66FF, 1.8);
            continue;
        }

        if be_id == rules::BLOCK_GEOTHERMAL {
            
            if st.fluid_amount > 0 {
                primitives.filled_box(
                    [x as f64 + 0.24, y as f64 + 0.96, z as f64 + 0.24],
                    [x as f64 + 0.76, y as f64 + 1.03, z as f64 + 0.76],
                    0xEEFF_6618,
                );
            }
            continue;
        }

        if be_id == rules::BLOCK_STEAM_TURBINE {
            
            if st.fluid_amount > 0 {
                let spin = ((hc::game_time(host) / 2) % 2) == 0;
                let (rx, rz) = if spin { (0.32, 0.08) } else { (0.08, 0.32) };
                primitives.filled_box(
                    [x as f64 + 0.5 - rx, y as f64 + 0.98, z as f64 + 0.5 - rz],
                    [x as f64 + 0.5 + rx, y as f64 + 1.06, z as f64 + 0.5 + rz],
                    0xFFD0_F4FF,
                );
            }
            continue;
        }

        if rules::is_fluid_pipe(&be_id) {
            
            if st.fluid_amount > 0 && !st.fluid_kind.is_empty() {
                let col = match st.fluid_kind.as_str() {
                    "lava" => 0xEEFF_5511,
                    "sap" => 0xCCEE_9B15,
                    "steam" => 0x88D0_F4FF,
                    _ => 0xAA25_85E8,
                };
                primitives.filled_box(
                    [x as f64 + 0.36, y as f64 + 0.36, z as f64 + 0.36],
                    [x as f64 + 0.64, y as f64 + 0.64, z as f64 + 0.64],
                    col,
                );
            }
            continue;
        }

        if be_id == rules::BLOCK_FLUID_TANK {
            
            if st.fluid_amount > 0 {
                let ratio = (st.fluid_amount as f64 / rules::FLUID_TANK_MAX_MB as f64).clamp(0.05, 1.0);
                let top_y = y as f64 + 0.08 + 0.80 * ratio;
                let col = match st.fluid_kind.as_str() {
                    "lava" => 0xEEFF_5511,
                    "sap" => 0xCCEE_9B15,
                    "steam" => 0x88D0_F4FF,
                    _ => 0xAA25_85E8,
                };
                primitives.filled_box(
                    [x as f64 + 0.12, y as f64 + 0.08, z as f64 + 0.12],
                    [x as f64 + 0.88, top_y, z as f64 + 0.88],
                    col,
                );
            }
            continue;
        }

        if be_id == rules::BLOCK_HARVESTER {
            let st = Some(st.clone())
                .unwrap_or_else(|| MachineState::new(MachineKind::AutoHarvester));
            let pos = [x, y, z];
            let facing = hc::block_property_at(host, pos, "facing")
                .unwrap_or_else(|| "north".to_owned());
            let (min_x, max_x, min_z, max_z) =
                rules::quarry_area_bounds(pos, &facing, st.quarry_radius);
            let x0 = min_x as f64;
            let x1 = (max_x + 1) as f64;
            let z0 = min_z as f64;
            let z1 = (max_z + 1) as f64;
            let base_y = y as f64;
            let top_y = y as f64 + 2.0;

            // Agricultural perimeter frame
            primitives.box_outline([x0, base_y, z0], [x1, top_y, z1], 0xFF55_FF66, 2.5);
            if st.enabled && st.last_target != [0, 0, 0] {
                let tx = st.last_target[0] as f64 + 0.5;
                let ty = st.last_target[1] as f64 + 0.5;
                let tz = st.last_target[2] as f64 + 0.5;
                primitives.line(
                    [x as f64 + 0.5, y as f64 + 0.75, z as f64 + 0.5],
                    [tx, ty, tz],
                    0xFF55_FF88,
                    2.2,
                );
            }
            continue;
        }

        if rules::is_conveyor_block(&be_id) {
            { let st = st.clone();
                if st.belt_filter_attached {
                    let fx = x as f64;
                    let fy = y as f64;
                    let fz = z as f64;
                    // Sleek 3D brass & cyan filter arch mounted directly on the conveyor belt
                    primitives.filled_box([fx + 0.08, fy + 0.25, fz + 0.08], [fx + 0.20, fy + 0.58, fz + 0.20], 0xFFD9_A420);
                    primitives.filled_box([fx + 0.80, fy + 0.25, fz + 0.08], [fx + 0.92, fy + 0.58, fz + 0.20], 0xFFD9_A420);
                    primitives.filled_box([fx + 0.08, fy + 0.25, fz + 0.80], [fx + 0.20, fy + 0.58, fz + 0.92], 0xFFD9_A420);
                    primitives.filled_box([fx + 0.80, fy + 0.25, fz + 0.80], [fx + 0.92, fy + 0.58, fz + 0.92], 0xFFD9_A420);
                    let arch_col = if st.splitter_filters[0].is_empty() { 0xFF55_6070 } else { 0xFF38_F2FF };
                    primitives.filled_box([fx + 0.08, fy + 0.52, fz + 0.08], [fx + 0.92, fy + 0.64, fz + 0.92], arch_col);
                }
            }
            continue;
        }

        if !rules::is_quarry_block(&be_id) {
            continue;
        }

        let (tier_radius, _, _) = rules::quarry_tier(&be_id);
        let st = Some(st.clone()).unwrap_or_else(|| MachineState::new(MachineKind::Quarry));
        let eff_radius = st.quarry_radius.max(tier_radius);
        let pos = [x, y, z];
        let facing = hc::block_property_at(host, pos, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let (min_x, max_x, min_z, max_z) =
            rules::quarry_area_bounds(pos, &facing, eff_radius);

        let x0 = min_x as f64;
        let x1 = (max_x + 1) as f64;
        let z0 = min_z as f64;
        let z1 = (max_z + 1) as f64;
        let base_y = y as f64;
        let top_y = y as f64 + 3.25;
        let (beam_col_a, beam_col_b, accent_col) = match be_id.as_str() {
            rules::BLOCK_QUARRY_MK2 => (0xFF4A_76A4, 0xFFB8_8E2C, 0xFF5A_8EC0),
            rules::BLOCK_QUARRY_MK3 => (0xFF3E_8292, 0xFF1C_2026, 0xFF72_669E),
            _ => (0xFFC8_9826, 0xFF20_242B, 0xFFD0_8228),
        };
        let steel_dark = 0xFF26_2A32;
        let steel_mid = 0xFF46_4E5C;
        let steel_light = 0xFF6E_7888;

        // 1. 3D Industrial Construction Scaffolding (non-overlapping segmented beams)
        let draw_hazard_beam = |prims: &mut hc::WorldPrimitivesCompat, a: [f64; 3], b: [f64; 3], r: f64| {
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let dz = b[2] - a[2];
            let len = (dx * dx + dy * dy + dz * dz).sqrt().max(0.01);
            let segs = (len / 0.5).ceil().max(1.0) as usize;
            let rx = if dx.abs() > 0.001 { 0.0 } else { r };
            let ry = if dy.abs() > 0.001 { 0.0 } else { r };
            let rz = if dz.abs() > 0.001 { 0.0 } else { r };
            for i in 0..segs {
                let t0 = i as f64 / segs as f64;
                let t1 = (i + 1) as f64 / segs as f64;
                let p0 = [a[0] + dx * t0, a[1] + dy * t0, a[2] + dz * t0];
                let p1 = [a[0] + dx * t1, a[1] + dy * t1, a[2] + dz * t1];
                let bmin = [p0[0].min(p1[0]) - rx, p0[1].min(p1[1]) - ry, p0[2].min(p1[2]) - rz];
                let bmax = [p0[0].max(p1[0]) + rx, p0[1].max(p1[1]) + ry, p0[2].max(p1[2]) + rz];
                let col = if i % 2 == 0 { beam_col_a } else { beam_col_b };
                prims.filled_box(bmin, bmax, col);
            }
        };

        let anim_t = hc::game_time(host) as f64 + partial as f64;
        let beacon_col = if st.enabled && ((anim_t * 0.18) as i64).rem_euclid(2) == 0 {
            accent_col
        } else if st.enabled {
            0xFF52_A8B8
        } else {
            0xFF5A_2424
        };

        // 4 vertical corner pillars with heavy anchor footings, mid-collars & top beacons (zero overlap)
        let mid_y = base_y + (top_y - base_y) * 0.5;
        for &(cx, cz) in &[(x0, z0), (x1, z0), (x0, z1), (x1, z1)] {
            primitives.filled_box([cx - 0.19, base_y, cz - 0.19], [cx + 0.19, base_y + 0.22, cz + 0.19], steel_dark);
            primitives.filled_box([cx - 0.14, base_y + 0.22, cz - 0.14], [cx + 0.14, base_y + 0.34, cz + 0.14], steel_mid);
            draw_hazard_beam(primitives, [cx, base_y + 0.34, cz], [cx, mid_y - 0.08, cz], 0.085);
            primitives.filled_box([cx - 0.12, mid_y - 0.08, cz - 0.12], [cx + 0.12, mid_y + 0.08, cz + 0.12], steel_mid);
            draw_hazard_beam(primitives, [cx, mid_y + 0.08, cz], [cx, top_y - 0.16, cz], 0.085);
            primitives.filled_box([cx - 0.16, top_y - 0.16, cz - 0.16], [cx + 0.16, top_y + 0.16, cz + 0.16], steel_dark);
            primitives.filled_box([cx - 0.09, top_y + 0.16, cz - 0.09], [cx + 0.09, top_y + 0.28, cz + 0.09], beacon_col);
        }
        // 4 top perimeter hazard beams (inset 0.16 from corner housings)
        draw_hazard_beam(primitives, [x0 + 0.16, top_y, z0], [x1 - 0.16, top_y, z0], 0.085);
        draw_hazard_beam(primitives, [x0 + 0.16, top_y, z1], [x1 - 0.16, top_y, z1], 0.085);
        draw_hazard_beam(primitives, [x0, top_y, z0 + 0.16], [x0, top_y, z1 - 0.16], 0.085);
        draw_hazard_beam(primitives, [x1, top_y, z0 + 0.16], [x1, top_y, z1 - 0.16], 0.085);
        // 4 ground perimeter hazard bars (inset 0.19 from anchor footings)
        let gy = base_y + 0.08;
        draw_hazard_beam(primitives, [x0 + 0.19, gy, z0], [x1 - 0.19, gy, z0], 0.06);
        draw_hazard_beam(primitives, [x0 + 0.19, gy, z1], [x1 - 0.19, gy, z1], 0.06);
        draw_hazard_beam(primitives, [x0, gy, z0 + 0.19], [x0, gy, z1 - 0.19], 0.06);
        draw_hazard_beam(primitives, [x1, gy, z0 + 0.19], [x1, gy, z1 - 0.19], 0.06);

        // Power conduit pipe connecting the Quarry block (`pos`) to the frame
        let qx = x as f64 + 0.5;
        let qy = y as f64 + 0.5;
        let qz = z as f64 + 0.5;
        let attach_x = qx.clamp(x0, x1);
        let attach_z = qz.clamp(z0, z1);
        primitives.filled_box(
            [qx.min(attach_x) - 0.08, qy - 0.08, qz.min(attach_z) - 0.08],
            [qx.max(attach_x) + 0.08, qy + 0.08, qz.max(attach_z) + 0.08],
            steel_mid,
        );

        // 2. Compute current target block & Twin-Rail Overhead Gantry
        let target = if st.last_target != [0, 0, 0] {
            st.last_target
        } else {
            rules::quarry_target_pos(pos, &facing, &st)
        };
        let tx = (target[0] as f64 + 0.5).clamp(x0 + 0.5, x1 - 0.5);
        let tz = (target[2] as f64 + 0.5).clamp(z0 + 0.5, z1 - 0.5);
        let ty = (target[1] as f64 + 0.5).min(top_y - 0.8);

        // End-truck trolley wheels riding on the X0 and X1 rails
        primitives.filled_box([x0 - 0.14, top_y - 0.16, tz - 0.26], [x0 + 0.14, top_y + 0.14, tz + 0.26], steel_dark);
        primitives.filled_box([x1 - 0.14, top_y - 0.16, tz - 0.26], [x1 + 0.14, top_y + 0.14, tz + 0.26], steel_dark);
        // Twin-rail steel X-Bridge (inset 0.14 from end-trucks, split around carriage)
        let left_end = (tx - 0.34).max(x0 + 0.14);
        let right_start = (tx + 0.34).min(x1 - 0.14);
        if left_end > x0 + 0.14 {
            primitives.filled_box([x0 + 0.14, top_y - 0.09, tz - 0.18], [left_end, top_y + 0.05, tz - 0.07], steel_mid);
            primitives.filled_box([x0 + 0.14, top_y - 0.09, tz + 0.07], [left_end, top_y + 0.05, tz + 0.18], steel_mid);
        }
        if x1 - 0.14 > right_start {
            primitives.filled_box([right_start, top_y - 0.09, tz - 0.18], [x1 - 0.14, top_y + 0.05, tz - 0.07], steel_mid);
            primitives.filled_box([right_start, top_y - 0.09, tz + 0.07], [x1 - 0.14, top_y + 0.05, tz + 0.18], steel_mid);
        }

        // Multi-part Drill Head Carriage at `[tx, top_y, tz]`
        primitives.filled_box([tx - 0.34, top_y - 0.20, tz - 0.34], [tx + 0.34, top_y + 0.12, tz + 0.34], steel_dark);
        primitives.filled_box([tx - 0.24, top_y + 0.12, tz - 0.24], [tx + 0.24, top_y + 0.24, tz + 0.24], beam_col_a);
        primitives.filled_box([tx - 0.18, top_y - 0.36, tz - 0.18], [tx + 0.18, top_y - 0.20, tz + 0.18], steel_mid);

        // 3. Telescoping 3D Drill Pipe & Multi-Stage Animated Drill Head
        if st.enabled {
            let bob = (anim_t * 0.55).sin().abs() * 0.24;
            let spin_alt = ((anim_t * 0.50) as i64).rem_euclid(2) == 0;
            let drill_tip_y = ty + 0.06 - bob;
            let pipe_top = top_y - 0.36;
            let pipe_bot = drill_tip_y + 0.72;

            // Hydraulic upper sleeve + inner steel drill shaft (joined end-to-end, no overlap)
            if pipe_top > pipe_bot {
                let sleeve_bot = (pipe_top - 0.75).max(pipe_bot);
                primitives.filled_box([tx - 0.12, sleeve_bot, tz - 0.12], [tx + 0.12, pipe_top, tz + 0.12], steel_dark);
                if sleeve_bot > pipe_bot {
                    primitives.filled_box([tx - 0.075, pipe_bot, tz - 0.075], [tx + 0.075, sleeve_bot, tz + 0.075], steel_light);
                }
            }

            // Gearbox & hazard collar above cutter head
            primitives.filled_box([tx - 0.22, drill_tip_y + 0.50, tz - 0.22], [tx + 0.22, drill_tip_y + 0.72, tz + 0.22], beam_col_a);
            primitives.filled_box([tx - 0.16, drill_tip_y + 0.44, tz - 0.16], [tx + 0.16, drill_tip_y + 0.50, tz + 0.16], steel_dark);

            // Rotating 4-flute cutter head (crossed steel & carbide blades, non-overlapping arms)
            let (bx, bz) = if spin_alt { (0.30, 0.09) } else { (0.09, 0.30) };
            primitives.filled_box([tx - bx, drill_tip_y + 0.22, tz - bz], [tx + bx, drill_tip_y + 0.44, tz + bz], steel_light);
            if spin_alt {
                primitives.filled_box([tx - 0.09, drill_tip_y + 0.22, tz - 0.30], [tx + 0.09, drill_tip_y + 0.44, tz - 0.09], steel_dark);
                primitives.filled_box([tx - 0.09, drill_tip_y + 0.22, tz + 0.09], [tx + 0.09, drill_tip_y + 0.44, tz + 0.30], steel_dark);
            } else {
                primitives.filled_box([tx - 0.30, drill_tip_y + 0.22, tz - 0.09], [tx - 0.09, drill_tip_y + 0.44, tz + 0.09], steel_dark);
                primitives.filled_box([tx + 0.09, drill_tip_y + 0.22, tz - 0.09], [tx + 0.30, drill_tip_y + 0.44, tz + 0.09], steel_dark);
            }

            // Conical stepped diamond/energy drill tip
            let tip_col = if spin_alt { 0xFF52_A8B8 } else { accent_col };
            primitives.filled_box([tx - 0.14, drill_tip_y + 0.10, tz - 0.14], [tx + 0.14, drill_tip_y + 0.22, tz + 0.14], steel_mid);
            primitives.filled_box([tx - 0.08, drill_tip_y, tz - 0.08], [tx + 0.08, drill_tip_y + 0.10, tz + 0.08], tip_col);

            // Pulsing excavation collar around the target block
            primitives.box_outline(
                [target[0] as f64, target[1] as f64, target[2] as f64],
                [
                    target[0] as f64 + 1.0,
                    target[1] as f64 + 1.0,
                    target[2] as f64 + 1.0,
                ],
                tip_col,
                2.5,
            );
        }
    }

    // Ghost block + directional arrow preview when holding Conveyor Belt or Item Splitter
    if let (Some(held_stack), Some(player)) = (hc::local_held(host), host.local_player()) {
        let held_id = hc::item_id(host, &held_stack);
        let is_belt = rules::is_conveyor_block(&held_id);
        let is_splitter = held_id == rules::BLOCK_ITEM_SPLITTER;
        if is_belt || is_splitter {
            let (yaw, pitch) = hc::local_yaw_pitch(host);
            let yaw_rad = (yaw as f64).to_radians();
            let pitch_rad = (pitch as f64).to_radians();
            let dir = [
                -yaw_rad.sin() * pitch_rad.cos(),
                -pitch_rad.sin(),
                yaw_rad.cos() * pitch_rad.cos(),
            ];
            let pos_eye = player.entity().position(host).unwrap_or([0.0, 0.0, 0.0]);
            let eye = [pos_eye[0], pos_eye[1] + 1.62, pos_eye[2]];
            let mut prev_cell = [
                eye[0].floor() as i32,
                eye[1].floor() as i32,
                eye[2].floor() as i32,
            ];
            let mut hit_place: Option<[i32; 3]> = None;
            for step in 1..=90 {
                let t = step as f64 * 0.05;
                let px = eye[0] + dir[0] * t;
                let py = eye[1] + dir[1] * t;
                let pz = eye[2] + dir[2] * t;
                let cell = [px.floor() as i32, py.floor() as i32, pz.floor() as i32];
                if cell != prev_cell {
                    let blk = hc::block_id_at(host, cell);
                    if !matches!(
                        blk.as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    ) {
                        hit_place = Some(prev_cell);
                        break;
                    }
                    prev_cell = cell;
                }
            }
            if let Some(p) = hit_place {
                let is_crouching = hc::is_crouching(host, player.entity());
                let (yaw, _) = hc::local_yaw_pitch(host);
                let forward_facing = cardinal_facing_from_yaw(yaw);
                let facing = if is_crouching {
                    opposite_facing(forward_facing)
                } else {
                    forward_facing
                };
                let (front_off, _) = rules::conveyor_offsets(facing);
                let top_y = if is_belt { 0.25 } else { 0.625 };
                let (fill_col, line_col) = if is_crouching {
                    (0x44FF_A020, 0xEEFF_B830)
                } else {
                    (0x4430_E0FF, 0xEE40_F5FF)
                };
                let bmin = [p[0] as f64 + 0.0625, p[1] as f64, p[2] as f64 + 0.0625];
                let bmax = [
                    p[0] as f64 + 0.9375,
                    p[1] as f64 + top_y,
                    p[2] as f64 + 0.9375,
                ];
                primitives.filled_box(bmin, bmax, fill_col);
                primitives.box_outline(bmin, bmax, line_col, 2.0);
                // Directional flow arrow on top of the ghost block
                let cx = p[0] as f64 + 0.5;
                let cy = p[1] as f64 + top_y + 0.04;
                let cz = p[2] as f64 + 0.5;
                let fx = front_off[0] as f64;
                let fz = front_off[2] as f64;
                let tip = [cx + fx * 0.36, cy, cz + fz * 0.36];
                let tail = [cx - fx * 0.30, cy, cz - fz * 0.30];
                primitives.line(tail, tip, line_col, 3.0);
                let wing1 = [cx + fx * 0.14 - fz * 0.18, cy, cz + fz * 0.14 + fx * 0.18];
                let wing2 = [cx + fx * 0.14 + fz * 0.18, cy, cz + fz * 0.14 - fx * 0.18];
                primitives.line(tip, wing1, line_col, 2.5);
                primitives.line(tip, wing2, line_col, 2.5);
            }
        }
    }
    prim_buf.flush(draw);
}

fn cardinal_facing_from_yaw(yaw: f32) -> &'static str {
    let y = yaw.rem_euclid(360.0);
    if !(45.0..315.0).contains(&y) {
        "south"
    } else if y < 135.0 {
        "west"
    } else if y < 225.0 {
        "north"
    } else {
        "east"
    }
}

fn opposite_facing(facing: &str) -> &'static str {
    match facing {
        "north" => "south",
        "south" => "north",
        "east" => "west",
        "west" => "east",
        _ => "north",
    }
}

fn compute_belt_shape(host: &dyn Host, pos: [i32; 3], facing: &str) -> &'static str {
    let (front_off, back_off) = rules::conveyor_offsets(facing);
    let front_up = [pos[0] + front_off[0], pos[1] + 1, pos[2] + front_off[2]];
    let back_up = [pos[0] + back_off[0], pos[1] + 1, pos[2] + back_off[2]];
    let front_pos = [pos[0] + front_off[0], pos[1], pos[2] + front_off[2]];

    if rules::is_conveyor_block(&hc::block_id_at(host, front_up)) {
        return "up";
    }
    if rules::is_conveyor_block(&hc::block_id_at(host, back_up)) {
        return "down";
    }

    let is_linkable_output = |p: [i32; 3], into_dir: [i32; 3]| -> bool {
        let b = hc::block_id_at(host, p);
        if rules::is_conveyor_block(&b) {
            let target_facing = hc::block_property_at(host, p, "facing")
                .unwrap_or_else(|| "north".to_owned());
            let (target_front, _) = rules::conveyor_offsets(&target_facing);
            // Do not turn into a belt that is pointing directly into us
            target_front != [-into_dir[0], 0, -into_dir[2]]
        } else {
            rules::is_item_pipe(&b) || b == rules::BLOCK_ITEM_SPLITTER || is_item_container_block(&b)
        }
    };

    if is_linkable_output(front_pos, front_off) {
        return "straight";
    }

    let left_off = rules::conveyor_exit_offset(facing, "turn_left");
    let right_off = rules::conveyor_exit_offset(facing, "turn_right");
    let left_pos = [pos[0] + left_off[0], pos[1], pos[2] + left_off[2]];
    let right_pos = [pos[0] + right_off[0], pos[1], pos[2] + right_off[2]];

    if is_linkable_output(left_pos, left_off) {
        "turn_left"
    } else if is_linkable_output(right_pos, right_off) {
        "turn_right"
    } else {
        "straight"
    }
}

/// Auto-orients Conveyor Belts & Item Splitters forward (Shift reverses) and auto-curves connected belts!
pub(crate) fn place_block(host: &mut dyn ServerHost, pos: [i32; 3]) {
    
    let block_id = hc::block_id_at(host, pos);
    if rules::is_quarry_block(&block_id) {
        let (tier_radius, _, _) = rules::quarry_tier(&block_id);
        hc::with_mod_block_entity(host, pos, |entity| {
            let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::Quarry));
            st.quarry_radius = tier_radius;
            store_state(entity, &st);
        });
        host.block_entity_sync(pos);
        return;
    }
    let is_belt = rules::is_conveyor_block(&block_id);
    let is_splitter = block_id == rules::BLOCK_ITEM_SPLITTER;
    if !is_belt && !is_splitter {
        return;
    }

    let is_crouching = host
        .players()
        .acting_player()
        .is_some_and(|p| hc::is_crouching(host, p.entity()));

    // `minecraft:furnace` template faces towards the player; flip so normal placement goes FORWARD (away from player),
    // and Shift (crouching) reverses direction!
    let raw_facing = hc::block_property_at(host, pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let facing = if is_crouching {
        raw_facing.clone()
    } else {
        opposite_facing(&raw_facing).to_owned()
    };
    if facing != raw_facing {
        host.set_block_property(pos, "facing", &facing);
    }

    if !is_belt {
        return;
    }

    let desired_shape = compute_belt_shape(host, pos, &facing);
    if desired_shape != "straight" {
        host.set_block_property(pos, "shape", desired_shape);
    }

    // Update adjacent belts that have an open front so they smoothly curve or ramp into this newly placed belt
    let neighbor_offsets = [
        [0, 0, -1],
        [0, 0, 1],
        [1, 0, 0],
        [-1, 0, 0],
        [0, -1, -1],
        [0, -1, 1],
        [1, -1, 0],
        [-1, -1, 0],
    ];
    for off in neighbor_offsets {
        let nb = [pos[0] + off[0], pos[1] + off[1], pos[2] + off[2]];
        if !rules::is_conveyor_block(&hc::block_id_at(host, nb)) {
            continue;
        }
        let nb_facing = hc::block_property_at(host, nb, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let nb_shape = hc::block_property_at(host, nb, "shape")
            .unwrap_or_else(|| "straight".to_owned());
        if nb_shape == "straight" {
            let updated = compute_belt_shape(host, nb, &nb_facing);
            if updated != nb_shape {
                host.set_block_property(nb, "shape", updated);
            }
        }
    }
}

/// Drops any item currently travelling inside an `item_pipe` or riding on a `conveyor_belt` when broken.
pub(crate) fn destroy_block(host: &mut dyn ServerHost, pos: [i32; 3]) {
    
    let block_id = hc::block_id_at(host, pos);
    if rules::is_item_pipe(&block_id) || rules::is_conveyor_block(&block_id) {
        let mut dropped_item: Option<(String, u32)> = None;
        hc::with_mod_block_entity(host, pos, |entity| {
            if let Some(mut st) = load_state(entity) {
                if !st.belt_item.is_empty() {
                    dropped_item = Some((st.belt_item.clone(), st.belt_count.max(1)));
                    st.belt_item.clear();
                    st.belt_count = 0;
                    store_state(entity, &st);
                }
            }
        });
        if let Some((item_id, count)) = dropped_item {
            { let _st = hc::make_stack(host, &item_id, count); hc::pop_resource(host, pos, _st); };
        }
        return;
    }

    static DRILL_BREAKING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if DRILL_BREAKING.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let Some(held) = hc::acting_held(host) else {
        return;
    };
    let held_id = hc::item_id(host, &held).as_str().to_owned();
    let Some((_, rad_half, base_cost)) = rules::drill_tier(&held_id) else {
        return;
    };
    let eff_mod = has_item_module(&held, "eff");
    let smelt_mod = has_item_module(&held, "smelt");
    let fortune_mod = has_item_module(&held, "fortune");
    let silk_mod = has_item_module(&held, "silk");
    let drill_area = is_drill_3x3(&held);
    let cost = if eff_mod { (base_cost / 2).max(5) } else { base_cost };
    if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
        return;
    }
    if !drill_area {
        return;
    }
    let (u_axes, v_axes): ([i32; 3], [i32; 3]) = if let Some(who) = host.players().acting_player() {
        let rot = who.entity().rotation(host).unwrap_or([0.0, 0.0]);
        if rot[1].abs() > 45.0 {
            ([1, 0, 0], [0, 0, 1])
        } else {
            let yaw = rot[0].rem_euclid(360.0);
            if !(45.0..135.0).contains(&yaw) && !(225.0..315.0).contains(&yaw) {
                ([1, 0, 0], [0, 1, 0])
            } else {
                ([0, 0, 1], [0, 1, 0])
            }
        }
    } else {
        ([1, 0, 0], [0, 1, 0])
    };
    let dim = host.dimension();
    
    DRILL_BREAKING.store(true, std::sync::atomic::Ordering::Relaxed);
    let [cx, cy, cz] = pos;
    'outer: for u in -rad_half..=rad_half {
        for v in -rad_half..=rad_half {
            if u == 0 && v == 0 {
                continue;
            }
            let p = [
                cx + u_axes[0] * u + v_axes[0] * v,
                cy + u_axes[1] * u + v_axes[1] * v,
                cz + u_axes[2] * u + v_axes[2] * v,
            ];
            let b = hc::block_id_at(host, p);
            if !b.is_empty() && b != "minecraft:air" && b != "minecraft:bedrock" && !b.starts_with("minecraft:barrier") {
                if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
                    break 'outer;
                }
                hc::set_block_by_id(host, p, "minecraft:air");
                if !hc::acting_creative(host) {
                    if silk_mod {
                        { let _st = hc::make_stack(host, &b, 1); hc::pop_resource(host, p, _st); };
                    } else if let Some((drop_id, base_cnt)) = rules::quarry_drop_for_block(&b) {
                        let final_id = if smelt_mod {
                            match drop_id {
                                "minecraft:raw_iron" => "minecraft:iron_ingot",
                                "minecraft:raw_copper" => "minecraft:copper_ingot",
                                "minecraft:raw_gold" => "minecraft:gold_ingot",
                                "minecraft:cobblestone" => "minecraft:stone",
                                _ => drop_id,
                            }
                        } else {
                            drop_id
                        };
                        let cnt = if fortune_mod { base_cnt + 1 } else { base_cnt };
                        { let _st = hc::make_stack(host, final_id, cnt); hc::pop_resource(host, p, _st); };
                    }
                }
            }
        }
    }
    DRILL_BREAKING.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// Handles right-clicking tools in the air (not targeting a block): Drill mode toggle, Jetpacks, Magnet, Batteries.
pub(crate) fn use_item(host: &mut dyn ServerHost, stack: Option<&ItemStack>, _player_uuid: u128) -> bool {
    let Some(stack) = stack else {
        return false;
    };
    let held_id_str = hc::item_id(host, stack);
    let held_id = held_id_str.as_str();
    let mut claimed = false;
    let is_crouching = host
        .players()
        .acting_player()
        .is_some_and(|p| hc::is_crouching(host, p.entity()));

    if let Some((tier, rad_half, _)) = rules::drill_tier(held_id) {
        if is_crouching {
            claimed = true;
            let area_label = if rad_half >= 2 { "5x5" } else { "3x3" };
            let mut mode_now = false;
            hc::acting_held_mut(host, |slot| if let Some(held) = slot.as_mut() {
                mode_now = !is_drill_3x3(&held);
                set_drill_3x3(held, mode_now);
            });
            if let Some(who) = host.players().acting_player() {
                let msg = if mode_now {
                    format!("⚡ [Taladro MK{tier}] Modo: {area_label} Excavación Amplia")
                } else {
                    format!("⚡ [Taladro MK{tier}] Modo: 1x1 Precisión")
                };
                let _ = who.action_bar(host, Text::literal(msg));
            }
        }
        return claimed;
    }

    if held_id == rules::ITEM_JETPACK_MK2 {
        claimed = true;
        if is_crouching {
            let mut now_hover = false;
            hc::acting_held_mut(host, |slot| if let Some(held) = slot.as_mut() {
                now_hover = !is_jetpack_hover(held);
                set_jetpack_hover(held, now_hover);
            });
            if let Some(who) = host.players().acting_player() {
                let m = if now_hover { "ACTIVADO" } else { "DESACTIVADO" };
                let _ = who.action_bar(host, Text::literal(format!("🚀 [Jetpack MK2] Modo Hover {m}")));
            }
        } else {
            hc::run_command(host, "playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.4 1.15");
            hc::run_command(host, "effect give @s minecraft:levitation 1 16 true");
            hc::run_command(host, "effect give @s minecraft:slow_falling 8 0 true");
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("🚀 [Jetpack MK2] Propulsión Avanzada"));
            }
        }
        return claimed;
    }

    if held_id == rules::ITEM_JETPACK {
        claimed = true;
        hc::run_command(host, "playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.5 1.0");
        hc::run_command(host, "effect give @s minecraft:levitation 1 15 true");
        hc::run_command(host, "effect give @s minecraft:slow_falling 6 0 true");
        hc::run_command(host, "particle minecraft:flame ~ ~ ~ 0.4 0.1 0.4 0.08 30");
        hc::run_command(host, "particle minecraft:campfire_cosy_smoke ~ ~ ~ 0.3 0.1 0.3 0.03 15");
        if let Some(who) = host.players().acting_player() {
            let _ = who.action_bar(host, Text::literal("🚀 [Jetpack Industrial] ¡Propulsión Aérea Máxima Activada!"));
        }
        return claimed;
    }

    if held_id == rules::ITEM_JET_BOOTS {
        claimed = true;
        hc::run_command(host, "playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.2 1.1");
        hc::run_command(host, "effect give @s minecraft:levitation 1 10 true");
        hc::run_command(host, "effect give @s minecraft:slow_falling 5 0 true");
        hc::run_command(host, "particle minecraft:flame ~ ~ ~ 0.3 0.1 0.3 0.05 20");
        hc::run_command(host, "particle minecraft:smoke ~ ~ ~ 0.2 0.1 0.2 0.02 15");
        if let Some(who) = host.players().acting_player() {
            let _ = who.action_bar(host, Text::literal("🚀 [Botas Cohete] ¡Propulsión Cohete Activada!"));
        }
        return claimed;
    }

    if let Some((tier, radius, cost)) = rules::magnet_tier(held_id) {
        claimed = true;
        if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("⚠ [Imán] Sin energía"));
            }
            return claimed;
        }
        hc::run_command(host, "playsound minecraft:hc::item_id(host, &entity).pickup player @s ~ ~ ~ 1.0 1.5");
        hc::run_command(host, &format!("tp @e[type=item,distance=..{radius}] @s"));
        hc::run_command(host, &format!("tp @e[type=experience_orb,distance=..{radius}] @s"));
        hc::run_command(host, "particle minecraft:portal ~ ~1 ~ 0.5 0.5 0.5 0.1 25");
        if let Some(who) = host.players().acting_player() {
            let _ = who.action_bar(host, Text::literal(format!("🧲 [Imán MK{tier}] ¡Ítems y XP en {radius} blq atraídos!")));
        }
        return claimed;
    }

    if held_id == rules::ITEM_WIRELESS_SILO_TABLET {
        claimed = true;
        let Some(bound) = get_bound_pos(stack) else {
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("📡 [Tablet Silo] Shift + Clic en un Controlador de Silos para vincular."));
            }
            return claimed;
        };
        if !hc::acting_creative(host) && !consume_player_energy(host, 50) {
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("⚠ [Tablet Silo] Sin energía (50 FE)"));
            }
            return claimed;
        }
        if let Some(off_stack) = hc::acting_offhand(host) {
            let sent_name = hc::item_id(host, &off_stack).split(':').nth(1).unwrap_or(&hc::item_id(host, &off_stack)).to_owned();
            let sent_cnt = off_stack.count();
            let rest = insert_into_silo_controller(host, bound, off_stack);
            hc::set_acting_offhand(host, rest);
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal(format!("📡 [Tablet Silo] Enviado {sent_name} x{sent_cnt} a la red!")));
            }
            return claimed;
        }
        let (list, silos_n, _) = sorted_silo_network(host, bound, false, false);
        if is_crouching {
            if let Some((top_id, _, _)) = list.first() {
                let mut taken = 0u32;
                for _ in 0..64 {
                    if pull_one_from_silo_network(host, bound, top_id) {
                        taken += 1;
                    } else {
                        break;
                    }
                }
                if taken > 0 {
                    if let Some(who) = host.players().acting_player() {
                        let p = { let pp = who.entity().position(host).unwrap_or([0.0; 3]); [pp[0].floor() as i32, pp[1].floor() as i32, pp[2].floor() as i32] };
                        { let _st = hc::make_stack(host, top_id, taken); hc::pop_resource(host, p, _st); };
                        let _ = who.action_bar(host, Text::literal(format!("📡 [Tablet Silo] Retirado x{taken} ({top_id})")));
                    }
                }
            }
        } else if let Some(who) = host.players().acting_player() {
            let preview: Vec<String> = list.iter().take(3).map(|(id, c, _)| {
                let s = id.split(':').nth(1).unwrap_or(id);
                format!("{s}:{c}")
            }).collect();
            let summary = if preview.is_empty() { "Vacío".to_owned() } else { preview.join(" | ") };
            let _ = who.action_bar(host, Text::literal(format!("📡 [Tablet Silo] ({silos_n} silos): {summary}")));
        }
        return claimed;
    }

    if held_id == rules::ITEM_QUANTUM_BACKPACK {
        claimed = true;
        let mut now_on = true;
        hc::acting_held_mut(host, |slot| if let Some(held) = slot.as_mut() {
                now_on = !is_qpack_on(held);
            set_qpack_on(held, now_on);
            });
        if let Some(who) = host.players().acting_player() {
            let lbl = if now_on { "ACTIVADO" } else { "DESACTIVADO" };
            let _ = who.action_bar(host, Text::literal(format!("🎒 [Mochila Cuántica] Auto-envío a Silos: {lbl}")));
        }
        return claimed;
    }

    if let Some((tier, range, base_cost)) = rules::laser_tier(held_id) {
        claimed = true;
        let eff_mod = has_item_module(stack, "eff");
        let smelt_mod = tier >= 3 || has_item_module(stack, "smelt");
        let fortune_mod = has_item_module(stack, "fortune");
        let cost = if eff_mod { (base_cost / 2).max(5) } else { base_cost };
        if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("⚠ [Láser] Sin energía"));
            }
            return claimed;
        }
        let dim = host.dimension();
        {
            if let Some(who) = host.players().acting_player() {
                let rot = who.entity().rotation(host).unwrap_or([0.0, 0.0]);
                let yaw = rot[0].to_radians() as f64;
                let pitch = rot[1].to_radians() as f64;
                let dx = -yaw.sin() * pitch.cos();
                let dy = -pitch.sin();
                let dz = yaw.cos() * pitch.cos();
                let [px, py, pz] = who.entity().position(host).unwrap_or([0.0; 3]);
                for step in 1..=range {
                    let bx = (px + dx * step as f64).floor() as i32;
                    let by = (py + 1.5 + dy * step as f64).floor() as i32;
                    let bz = (pz + dz * step as f64).floor() as i32;
                    let p = [bx, by, bz];
                    let b = hc::block_id_at(host, p);
                    if !b.is_empty() && b != "minecraft:air" && b != "minecraft:bedrock" && !b.starts_with("minecraft:barrier") {
                        hc::set_block_by_id(host, p, "minecraft:air");
                        if !hc::acting_creative(host) {
                            if let Some((drop_id, base_cnt)) = rules::quarry_drop_for_block(&b) {
                                let final_id = if smelt_mod {
                                    match drop_id {
                                        "minecraft:raw_iron" => "minecraft:iron_ingot",
                                        "minecraft:raw_copper" => "minecraft:copper_ingot",
                                        "minecraft:raw_gold" => "minecraft:gold_ingot",
                                        "minecraft:cobblestone" => "minecraft:stone",
                                        _ => drop_id,
                                    }
                                } else {
                                    drop_id
                                };
                                let cnt = if fortune_mod { base_cnt + 1 } else { base_cnt };
                                { let _st = hc::make_stack(host, final_id, cnt); hc::pop_resource(host, p, _st); };
                            }
                        }
                    }
                }
                hc::run_command(host, "playsound minecraft:entity.guardian.attack player @s ~ ~ ~ 1.2 1.6");
                let _ = who.action_bar(
                    host,
                    Text::literal(format!("⚡ [Láser Minero MK{tier}] Haz de {range} bloques disparado (-{cost} FE)")),
                );
            }
        }
        return claimed;
    }

    if let Some((tier, dmg, cost)) = rules::sword_tier(held_id) {
        claimed = true;
        if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("⚠ [Sable de Plasma] Sin energía"));
            }
            return claimed;
        }
        if tier >= 3 && is_crouching {
            if let Some(who) = host.players().acting_player() {
                let rot = who.entity().rotation(host).unwrap_or([0.0, 0.0]);
                let ppos = who.entity().position(host).unwrap_or([0.0; 3]);
                let yaw = rot[0].to_radians() as f64;
                let pitch = rot[1].to_radians() as f64;
                let tx = (ppos[0] - yaw.sin() * pitch.cos() * 7.0).round() as i32;
                let ty = (ppos[1] - pitch.sin() * 5.0).round() as i32;
                let tz = (ppos[2] + yaw.cos() * pitch.cos() * 7.0).round() as i32;
                hc::run_command(host, &format!("tp @s {tx} {ty} {tz}"));
                hc::run_command(host, "playsound minecraft:entity.enderman.teleport player @s ~ ~ ~ 1.0 1.2");
                let _ = who.action_bar(host, Text::literal("⚡ [Sable Cuántico] ¡Blink Cuántico!"));
            }
        } else {
            let rad = if tier >= 2 { 5 } else { 3 };
            hc::run_command(host, &format!("damage @e[type=!player,distance=..{rad}] {dmg} minecraft:lightning_bolt by @s"));
            hc::run_command(host, "playsound minecraft:entity.player.attack.sweep player @s ~ ~ ~ 1.2 1.4");
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal(format!("⚔ [Sable de Plasma MK{tier}] Descarga de Plasma ({dmg} daño)")));
            }
        }
        return claimed;
    }

    if rules::is_battery_item(held_id) {
        claimed = true;
        let cap = rules::battery_capacity(held_id).unwrap_or(50_000);
        let cur = get_item_energy(stack);
        if let Some(who) = host.players().acting_player() {
            let _ = who.action_bar(host, Text::literal(format!("🔋 [Batería] Carga: {cur} / {cap} FE (recarga tu equipo automáticamente)")));
        }
    }
    claimed
}

/// Inspects networks, harvests sap from Rubber Tree Sap Cavities, grows Rubber Saplings, interacts with Conveyor Belts (no GUI), configures Item Splitter filter frames, rotates/dismantles machines with Wrench, or uses tools.
pub(crate) fn right_click(host: &mut dyn ServerHost, click: &BlockClick<'_>) -> bool {
    let pos = click.pos;
    let face = click.hit.face;
    let mut canceled = false;
    let block_id = hc::block_id_at(host, pos);
    let held_stack_opt = hc::acting_held(host);
    let held_info = held_stack_opt
        .as_ref()
        .map(|s| (hc::item_id(host, s), s.count(), 0i32));
    let is_crouching = host
        .players()
        .acting_player()
        .is_some_and(|p| hc::is_crouching(host, p.entity()));

    // Portable Energy Scanner: Inspect live telemetry of any Rusty Factory block
    if held_info.as_ref().is_some_and(|(id, _, _)| id == rules::ITEM_ENERGY_SCANNER)
        && block_id.starts_with("rusty_factory:")
    {
        canceled = true;
        let mut msg = format!("📡 [{block_id}]");
        hc::with_mod_block_entity(host, pos, |entity| {
            if let Some(st) = load_state(entity) {
                if let Some(kind) = MachineKind::from_block_id(&block_id) {
                    if kind.max_energy() > 0 {
                        msg.push_str(&format!(" | ⚡ {}/{} FE", st.energy, kind.max_energy()));
                    }
                }
                if block_id == rules::BLOCK_INDUSTRIAL_SILO {
                    let cap = st.silo_capacity.max(2_048);
                    let item = if st.belt_item.is_empty() { "Vacío" } else { &st.belt_item };
                    msg.push_str(&format!(" | 📦 {item}: {} / {cap}", st.belt_count));
                } else if !st.fluid_kind.is_empty() || st.fluid_amount > 0 {
                    msg.push_str(&format!(" | 🚰 {}: {} mB", st.fluid_kind, st.fluid_amount));
                } else if st.belt_filter_attached {
                    let mode = if st.belt_filter_blacklist { "Lista Negra" } else { "Lista Blanca" };
                    let f = if st.splitter_filters[0].is_empty() { "Sin ítem" } else { &st.splitter_filters[0] };
                    msg.push_str(&format!(" | 🔍 {mode}: {f}"));
                } else if !st.belt_item.is_empty() {
                    msg.push_str(&format!(" | 📦 {} x{}", st.belt_item, st.belt_count));
                }
            }
        });
        if let Some(who) = host.players().acting_player() {
            let _ = who.action_bar(host, Text::literal(msg));
        }
        return canceled;
    }

    // Industrial Silo: Front screen = deposit / Shift-lock item / withdraw; Side faces = place blocks or open 1-slot GUI
    if block_id == rules::BLOCK_INDUSTRIAL_SILO {
        let facing = hc::block_property_at(host, pos, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let (front_off, _) = rules::conveyor_offsets(&facing);
        let is_front = face == front_off;

        if let Some((ref held_id, held_count, _)) = held_info {
            if let Some(bonus) = rules::silo_upgrade_bonus(held_id) {
                canceled = true;
                let mut new_cap = 0;
                hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::IndustrialSilo));
                    st.silo_capacity = st.silo_capacity.max(2_048).saturating_add(bonus);
                    new_cap = st.silo_capacity;
                    store_state(entity, &st);
                });
                if !hc::acting_creative(host) {
                    hc::acting_consume_held(host, 1);
                }
                host.block_entity_sync(pos);
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("📦 [Silo Industrial] Capacidad mejorada a {new_cap} ítems")));
                }
                return canceled;
            }
            if is_front {
                canceled = true;
                let item_id = held_id.clone();
                let mut deposited = 0u32;
                let mut total_now = 0u32;
                let mut cap_now = 2_048u32;
                let mut mismatch = false;
                hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::IndustrialSilo));
                    cap_now = st.silo_capacity.max(2_048);
                    if is_crouching && st.belt_count == 0 {
                        // Shift + Right-Click on front screen locks/configures the Silo to this item
                        st.belt_item = item_id.clone();
                        store_state(entity, &st);
                    }
                    if st.belt_item.is_empty() || st.belt_item == item_id {
                        let space = cap_now.saturating_sub(st.belt_count);
                        deposited = held_count.min(space);
                        if deposited > 0 {
                            st.belt_item = item_id.clone();
                            st.belt_count += deposited;
                            store_state(entity, &st);
                        }
                    } else {
                        mismatch = true;
                    }
                    total_now = st.belt_count;
                });
                if deposited > 0 {
                    if !hc::acting_creative(host) {
                        hc::acting_consume_held(host, deposited);
                    }
                    host.block_entity_sync(pos);
                } else if is_crouching {
                    host.block_entity_sync(pos);
                }
                if let Some(who) = host.players().acting_player() {
                    let msg = if mismatch {
                        format!("⚠ [Silo] Bloqueado a otro ítem ({total_now} / {cap_now})")
                    } else {
                        format!("📦 [Silo] {total_now} / {cap_now} ({item_id})")
                    };
                    let _ = who.action_bar(host, Text::literal(msg));
                }
                return canceled;
            } else if held_id != rules::ITEM_WRENCH {
                // Side/top/back face with a block in hand: place that block at `pos + face`!
                if true {
                    canceled = true;
                    let target = [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]];
                    let target_blk = hc::block_id_at(host, target);
                    if matches!(
                        target_blk.as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water"
                    ) {
                        let dim = host.dimension();
                        if hc::set_block_by_id(host, target, held_id) {
                            let forward_facing = cardinal_facing_from_yaw(hc::acting_yaw_pitch(host).0);
                            let raw_facing = opposite_facing(forward_facing);
                            let _ = host.set_block_property(target, "facing", raw_facing);
                            place_block(host, target);
                            if !hc::acting_creative(host) {
                                hc::acting_consume_held(host, 1);
                            }
                        }
                    }
                    return canceled;
                }
            }
        } else if is_front {
            // Empty hand on front screen: withdraw 1 stack (up to 64)
            canceled = true;
            let mut taken: Option<(String, u32)> = None;
            let mut total_now = 0u32;
            let mut cap_now = 2_048u32;
            hc::with_mod_block_entity(host, pos, |entity| {
                if let Some(mut st) = load_state(entity) {
                    cap_now = st.silo_capacity.max(2_048);
                    if !st.belt_item.is_empty() && st.belt_count > 0 {
                        let amt = st.belt_count.min(64);
                        taken = Some((st.belt_item.clone(), amt));
                        st.belt_count -= amt;
                        if st.belt_count == 0 {
                            st.belt_item.clear();
                        }
                        store_state(entity, &st);
                    }
                    total_now = st.belt_count;
                }
            });
            if let Some((iid, amt)) = taken {
                hc::give_items(host, &iid, amt);
                host.block_entity_sync(pos);
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("📦 [Silo] Retirado x{amt} | Restante: {total_now} / {cap_now}")));
                }
            }
            return canceled;
        }
        // Empty hand on non-front face falls through so the 1-slot Silo GUI opens!
    }

    // Equipment Upgrade Station: load a module_* or install loaded module onto held tech tool/armor
    if block_id == rules::BLOCK_UPGRADE_STATION {
        canceled = true;
        if let Some((ref held_id, _, _)) = held_info {
            let mod_code = match held_id.as_str() {
                rules::ITEM_MODULE_NIGHT_VISION => Some("nv"),
                rules::ITEM_MODULE_MAGNET => Some("mag"),
                rules::ITEM_MODULE_EFFICIENCY => Some("eff"),
                rules::ITEM_MODULE_AUTO_SMELT => Some("smelt"),
                rules::ITEM_MODULE_FORTUNE => Some("fortune"),
                rules::ITEM_MODULE_SILK_TOUCH => Some("silk"),
                rules::ITEM_MODULE_SPEED => Some("speed"),
                _ => None,
            };
            if let Some(code) = mod_code {
                hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::UpgradeStation));
                    st.belt_item = code.to_owned();
                    store_state(entity, &st);
                });
                if !hc::acting_creative(host) {
                    hc::acting_consume_held(host, 1);
                }
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("🔧 [Estación de Mejora] Módulo '{code}' cargado. Clic con armadura o taladro para instalar.")));
                }
                return canceled;
            }
            if rules::battery_capacity(held_id).is_some() {
                let mut loaded_mod = String::new();
                hc::with_mod_block_entity(host, pos, |entity| {
                    if let Some(mut st) = load_state(entity) {
                        if !st.belt_item.is_empty() {
                            loaded_mod = st.belt_item.clone();
                            st.belt_item.clear();
                            store_state(entity, &st);
                        }
                    }
                });
                if !loaded_mod.is_empty() {
                    hc::acting_held_mut(host, |slot| if let Some(held) = slot.as_mut() {
                add_item_module(held, &loaded_mod);
            });
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(format!("✅ [Estación de Mejora] Módulo '{loaded_mod}' instalado en tu equipo")));
                    }
                } else if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("ℹ [Estación de Mejora] Primero cargá un Módulo haciendo clic con él."));
                }
                return canceled;
            }
        }
        return canceled;
    }

    // Geothermal Generator: direct lava_bucket feed
    if block_id == rules::BLOCK_GEOTHERMAL
        && held_info.as_ref().is_some_and(|(id, _, _)| id == "minecraft:lava_bucket")
    {
        canceled = true;
        let mut ok = false;
        hc::with_mod_block_entity(host, pos, |entity| {
            let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::GeothermalGenerator));
            if st.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB {
                st.fluid_kind = "lava".to_owned();
                st.fluid_amount += 1000;
                store_state(entity, &st);
                ok = true;
            }
        });
        if ok && !hc::acting_creative(host) {
            let b = hc::make_stack(host, "minecraft:bucket", 1);
            hc::acting_held_mut(host, |slot| *slot = b);
            host.block_entity_sync(pos);
        }
        return canceled;
    }

    // Create-style per-face filter configuration on Item Splitter (no GUI!)
    if block_id == rules::BLOCK_ITEM_SPLITTER {
        let is_wrench = held_info.as_ref().is_some_and(|(id, _, _)| id == rules::ITEM_WRENCH);
        if !is_wrench {
            let facing = hc::block_property_at(host, pos, "facing")
                .unwrap_or_else(|| "north".to_owned());
            if let Some(side) = rules::splitter_side_for_face(&facing, face) {
                if !is_crouching {
                    canceled = true;
                    let side_name = match side {
                        0 => "Izquierda",
                        1 => "Frente",
                        _ => "Derecha",
                    };
                    if let Some((ref held_id, _, _)) = held_info {
                        let filter_item = held_id.clone();
                        hc::with_mod_block_entity(host, pos, |entity| {
                            let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ItemSplitter));
                            st.splitter_filters[side] = filter_item.clone();
                            store_state(entity, &st);
                        });
                        host.block_entity_sync(pos);
                        if let Some(who) = host.players().acting_player() {
                            let _ = who.action_bar(host, Text::literal(format!("🔍 [Divisor] Filtro {side_name}: {filter_item}")));
                        }
                    } else {
                        hc::with_mod_block_entity(host, pos, |entity| {
                            let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ItemSplitter));
                            st.splitter_filters[side].clear();
                            store_state(entity, &st);
                        });
                        host.block_entity_sync(pos);
                        if let Some(who) = host.players().acting_player() {
                            let _ = who.action_bar(host, Text::literal(format!("🧹 [Divisor] Filtro {side_name} limpiado")));
                        }
                    }
                    return canceled;
                }
            }
        }
    }

    // Direct hand interaction on Conveyor Belts (NO GUI!)
    if let Some((_, max_batch)) = rules::conveyor_tier(&block_id) {
        if let Some((ref held_id, _, _)) = held_info {
            if held_id == rules::ITEM_BELT_FILTER || held_id == rules::ITEM_BELT_FILTER_ADV {
                canceled = true;
                let is_adv = held_id == rules::ITEM_BELT_FILTER_ADV;
                let changed = hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ConveyorBelt));
                    if !st.belt_filter_attached || (is_adv && !st.belt_filter_blacklist) {
                        st.belt_filter_attached = true;
                        if is_adv {
                            st.belt_filter_blacklist = true;
                        }
                        store_state(entity, &st);
                        true
                    } else {
                        false
                    }
                });
                if changed == Some(true) {
                    if !hc::acting_creative(host) {
                        hc::acting_consume_held(host, 1);
                    }
                    host.block_entity_sync(pos);
                    if let Some(who) = host.players().acting_player() {
                        let msg = if is_adv {
                            "🔍 [Cinta] Filtro Avanzado acoplado (Shift+Mano Vacía alterna Lista Blanca/Negra)"
                        } else {
                            "🔍 [Cinta] Filtro acoplado. Clic con un ítem para configurarlo."
                        };
                        let _ = who.action_bar(host, Text::literal(msg));
                    }
                }
                return canceled;
            }
            if held_id == rules::ITEM_WRENCH
            || (block_id.starts_with("rusty_factory:")
                && matches!(held_id.as_str(), rules::ITEM_OMNI_TOOL_MK2 | rules::ITEM_OMNI_TOOL_MK3))
        {
                canceled = true;
                if is_crouching {
                    let mut had_filter = false;
                    hc::with_mod_block_entity(host, pos, |entity| {
                        if let Some(mut st) = load_state(entity) {
                            if st.belt_filter_attached {
                                st.belt_filter_attached = false;
                                st.splitter_filters[0].clear();
                                store_state(entity, &st);
                                had_filter = true;
                            }
                        }
                    });
                    if had_filter {
                        hc::give_items(host, rules::ITEM_BELT_FILTER, 1);
                        host.block_entity_sync(pos);
                        if let Some(who) = host.players().acting_player() {
                            let _ = who.action_bar(host, Text::literal("🔧 [Cinta] Filtro retirado"));
                        }
                        return canceled;
                    }
                    destroy_block(host, pos);
                    hc::give_items(host, &block_id, 1);
                    hc::run_command(host, &format!(
                        "setblock {} {} {} minecraft:air destroy",
                        pos[0], pos[1], pos[2]
                    ));
                } else {
                    let current = hc::block_property_at(host, pos, "shape")
                        .unwrap_or_else(|| "straight".to_owned());
                    let next = match current.as_str() {
                        "straight" => "turn_left",
                        "turn_left" => "turn_right",
                        "turn_right" => "up",
                        "up" => "down",
                        _ => "straight",
                    };
                    host.set_block_property(pos, "shape", next);
                    if let Some(who) = host.players().acting_player() {
                        let label = match next {
                            "turn_left" => "Curva Izquierda",
                            "turn_right" => "Curva Derecha",
                            "up" => "Subida (Rampa)",
                            "down" => "Bajada (Rampa)",
                            _ => "Recta",
                        };
                        let _ = who.action_bar(host, Text::literal(format!("🔧 [Cinta] Forma: {label}")));
                    }
                }
                return canceled;
            }
            // Allow placing filters (Item Splitter), other belts, pipes, or extractors directly against belts!
            let is_placeable_logistics = held_id == rules::BLOCK_ITEM_SPLITTER
                || rules::is_conveyor_block(held_id)
                || rules::is_item_pipe(held_id)
                || held_id == rules::BLOCK_EXTRACTOR;
            if is_placeable_logistics {
                canceled = true;
                let target = [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]];
                let target_blk = hc::block_id_at(host, target);
                if matches!(
                    target_blk.as_str(),
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water"
                ) {
                    if hc::set_block_by_id(host, target, held_id) {
        {
                            let forward_facing = cardinal_facing_from_yaw(hc::acting_yaw_pitch(host).0);
                            let raw_facing = opposite_facing(forward_facing);
                            let _ = host.set_block_property(target, "facing", raw_facing);
                            place_block(host, target);
                            if !hc::acting_creative(host) {
                                hc::acting_consume_held(host, 1);
                            }
                            hc::run_command(host, &format!(
                                "playsound minecraft:block.metal.place block @a {} {} {} 0.8 1.1",
                                target[0] as f64 + 0.5,
                                target[1] as f64 + 0.5,
                                target[2] as f64 + 0.5
                            ));
                        }
                    }
                }
                return canceled;
            }
            // If this belt has an inline filter attached and needs configuration (empty filter or crouching with item), set filter!
            let mut filter_set = false;
            let item_to_filter = held_id.clone();
            let filter_changed = hc::with_mod_block_entity(host, pos, |entity| {
                if let Some(mut st) = load_state(entity) {
                    if st.belt_filter_attached && (st.splitter_filters[0].is_empty() || is_crouching) {
                        st.splitter_filters[0] = item_to_filter.clone();
                        store_state(entity, &st);
                        filter_set = true;
                        return true;
                    }
                }
                false
            });
            if filter_changed == Some(true) && filter_set {
                canceled = true;
                host.block_entity_sync(pos);
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("🔍 [Cinta] Filtrando: {item_to_filter}")));
                }
                return canceled;
            }
            // Place 1 item from player's hand onto the conveyor belt (unless sneaking to place blocks!)
            if !is_crouching {
                canceled = true;
                let item_to_put = held_id.clone();
                let mut placed = false;
                let changed = hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ConveyorBelt));
                    if st.belt_item.is_empty() {
                        st.belt_item = item_to_put.clone();
                        st.belt_count = 1;
                        st.progress = 0;
                        store_state(entity, &st);
                        placed = true;
                        true
                    } else if st.belt_item == item_to_put && st.belt_count < max_batch {
                        st.belt_count += 1;
                        store_state(entity, &st);
                        placed = true;
                        true
                    } else {
                        false
                    }
                });
                if changed == Some(true) && placed {
                    if !hc::acting_creative(host) {
                        hc::acting_consume_held(host, 1);
                    }
                    host.block_entity_sync(pos);
                }
                return canceled;
            }
        } else {
            // Empty hand right-click: if crouching on a filtered belt, toggle Whitelist/Blacklist!
            if is_crouching {
                let mut toggled_mode: Option<bool> = None;
                let toggled = hc::with_mod_block_entity(host, pos, |entity| {
                    if let Some(mut st) = load_state(entity) {
                        if st.belt_filter_attached {
                            st.belt_filter_blacklist = !st.belt_filter_blacklist;
                            toggled_mode = Some(st.belt_filter_blacklist);
                            store_state(entity, &st);
                            return true;
                        }
                    }
                    false
                });
                if toggled == Some(true) {
                    canceled = true;
                    host.block_entity_sync(pos);
                    if let (Some(who), Some(bl)) = (host.players().acting_player(), toggled_mode) {
                        let label = if bl { "Lista Negra (bloquea filtro)" } else { "Lista Blanca (solo filtro)" };
                        let _ = who.action_bar(host, Text::literal(format!("🔍 [Cinta] Modo: {label}")));
                    }
                    return canceled;
                }
            }
            // Empty hand right-click: collect carried item from the conveyor belt!
            canceled = true;
            let mut taken: Option<(String, u32)> = None;
            let changed = hc::with_mod_block_entity(host, pos, |entity| {
                let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ConveyorBelt));
                if !st.belt_item.is_empty() {
                    taken = Some((st.belt_item.clone(), st.belt_count.max(1)));
                    st.belt_item.clear();
                    st.belt_count = 0;
                    st.progress = 0;
                    store_state(entity, &st);
                    true
                } else {
                    false
                }
            });
            if changed == Some(true) {
                if let Some((item_id, count)) = taken {
                    hc::give_items(host, &item_id, count);
                }
                host.block_entity_sync(pos);
            }
            return canceled;
        }
    }

    if let Some((ref held_id, _, _)) = held_info {
        // 1. Wrench: Rotate machine facing (normal click) or dismantle into inventory (crouch + click)
        if held_id == rules::ITEM_WRENCH
            || (block_id.starts_with("rusty_factory:")
                && matches!(held_id.as_str(), rules::ITEM_OMNI_TOOL_MK2 | rules::ITEM_OMNI_TOOL_MK3))
        {
            if rules::is_item_pipe(&block_id) {
                canceled = true;
                if is_crouching {
                    destroy_block(host, pos);
                    hc::give_items(host, &block_id, 1);
                    hc::run_command(host, &format!(
                        "setblock {} {} {} minecraft:air destroy",
                        pos[0], pos[1], pos[2]
                    ));
                } else {
                    let targets = rules::find_connected_item_targets(
                        pos,
                        |p| rules::is_item_pipe(&hc::block_id_at(host, p)),
                        |p| is_item_container_block(&hc::block_id_at(host, p)) || rules::is_conveyor_block(&hc::block_id_at(host, p)),
                    );
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(format!("📦 [Tubería de Ítems] Red: {} destinos conectados", targets.len())));
                    }
                }
                return canceled;
            }
            if let Some(kind) = MachineKind::from_block_id(&block_id) {
                canceled = true;
                if is_crouching {
                    hc::give_items(host, &block_id, 1);
                    hc::run_command(host, &format!(
                        "setblock {} {} {} minecraft:air destroy",
                        pos[0], pos[1], pos[2]
                    ));
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(format!("🔧 [Llave Inglesa] {} desmontada", hc::lang_get(kind.title_key()))));
                    }
                } else {
                    let current = hc::block_property_at(host, pos, "facing")
                        .unwrap_or_else(|| "north".to_owned());
                    let next = match current.as_str() {
                        "north" => "east",
                        "east" => "south",
                        "south" => "west",
                        _ => "north",
                    };
                    host.set_block_property(pos, "facing", next);
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(format!("🔧 [Llave Inglesa] Rotada hacia: {next}")));
                    }
                }
                return canceled;
            }

            if rules::is_cable(&block_id) {
                canceled = true;
                let limit = rules::cable_transfer_limit(&block_id);
                let consumers = rules::find_connected_consumers(
                    pos,
                    |p| rules::is_cable(&hc::block_id_at(host, p)),
                    |p| MachineKind::from_block_id(&hc::block_id_at(host, p)).is_some(),
                );
                let cable_name = match block_id.as_str() {
                    rules::BLOCK_CABLE_WHITE => "Cable Blanco [Tier 1]",
                    rules::BLOCK_CABLE_GOLD => "Cable Amarillo [Tier 3]",
                    rules::BLOCK_CABLE_RED => "Cable Rojo [Tier 4]",
                    _ => "Cable Naranja / Cobre [Tier 2]",
                };
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("⚡ [{cable_name}] Capacidad: {limit} FE/t | {} máquinas conectadas", consumers.len())));
                }
                return canceled;
            }

            if rules::is_item_pipe(&block_id) {
                canceled = true;
                let targets = rules::find_connected_item_targets(
                    pos,
                    |p| rules::is_item_pipe(&hc::block_id_at(host, p)),
                    |p| is_item_container_block(&hc::block_id_at(host, p)),
                );
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("📦 [Tubería de Ítems] Red: {} inventarios conectados", targets.len())));
                }
                return canceled;
            }

            if rules::is_fluid_pipe(&block_id) {
                canceled = true;
                let targets = rules::find_connected_item_targets(
                    pos,
                    |p| rules::is_fluid_pipe(&hc::block_id_at(host, p)),
                    |p| matches!(hc::block_id_at(host, p).as_str(), rules::BLOCK_FLUID_TANK | rules::BLOCK_STEAM_BOILER | rules::BLOCK_FLUID_PUMP),
                );
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("🚰 [Tubería de Fluidos] Red hidráulica: {} tanques conectados", targets.len())));
                }
                return canceled;
            }
        }

        // 2. Treetap: Harvest extra sap & saplings from Rubber Sap Log with durability consumption
        if held_id == rules::ITEM_TREETAP && block_id == rules::BLOCK_RUBBER_SAP_LOG {
            canceled = true;
            let has_sap = hc::block_property_at(host, pos, "lit").as_deref() == Some("true");
            if has_sap {
                host.set_block_property(pos, "lit", "false");
                hc::give_items(host, rules::ITEM_RAW_SAP, 3);
                hc::give_items(host, rules::BLOCK_RUBBER_SAPLING, 1);
                hc::damage_held(host, 1);
                hc::run_command(host, "playsound minecraft:item.honey_bottle.drink player @s ~ ~ ~ 1.0 1.2");
                hc::run_command(host, "particle minecraft:dripping_honey ~ ~0.5 ~ 0.2 0.2 0.2 0.05 12");
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("💧 [Espita de Madera] ¡Savia recolectada! (-1 durabilidad)"));
                }
            } else if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("⏳ El hueco de savia se está regenerando..."));
            }
            return canceled;
        }

        // 3. Mining Drill (MK1 3x3, MK2 3x3, MK3 5x5): Crouch + Right-Click toggles area/1x1 (Left-Click mines)
        if let Some((tier, rad_half, _)) = rules::drill_tier(held_id) {
            let area_label = if rad_half >= 2 { "5x5" } else { "3x3" };
            if is_crouching {
                canceled = true;
                let mut mode_now = false;
                hc::acting_held_mut(host, |slot| if let Some(held) = slot.as_mut() {
                mode_now = !is_drill_3x3(&held);
                    set_drill_3x3(held, mode_now);
            });
                if let Some(who) = host.players().acting_player() {
                    let msg = if mode_now {
                        format!("⚡ [Taladro MK{tier}] Modo: {area_label} Excavación Amplia")
                    } else {
                        format!("⚡ [Taladro MK{tier}] Modo: 1x1 Precisión")
                    };
                    let _ = who.action_bar(host, Text::literal(msg));
                }
                return canceled;
            }
        }

        // 3a. Electric Chainsaw (MK1 64, MK2 128+hojas, MK3 256+hojas) & Omni-Tool on logs
        if let Some((tier, max_logs, cost)) = rules::chainsaw_tier(held_id) {
            let blk = hc::block_id_at(host, pos);
            if blk.ends_with("_log") || blk.ends_with("_stem") || blk == rules::BLOCK_RUBBER_LOG || blk == rules::BLOCK_RUBBER_SAP_LOG {
                canceled = true;
                if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(format!("⚠ [Motosierra MK{tier}] Sin energía ({cost} FE)")));
                    }
                    return canceled;
                }
                let mut visited = std::collections::HashSet::new();
                let mut q = std::collections::VecDeque::new();
                q.push_back(pos);
                visited.insert(pos);
                let mut felled = 0usize;
                while let Some(cur) = q.pop_front() {
                    if felled >= max_logs {
                        break;
                    }
                    let cb = hc::block_id_at(host, cur);
                    let is_log = cb.ends_with("_log") || cb.ends_with("_stem") || cb == rules::BLOCK_RUBBER_LOG || cb == rules::BLOCK_RUBBER_SAP_LOG;
                    let is_leaf = tier >= 2 && (cb.ends_with("_leaves") || cb == rules::BLOCK_RUBBER_LEAVES);
                    if is_log || is_leaf {
                        hc::run_command(host, &format!("setblock {} {} {} minecraft:air destroy", cur[0], cur[1], cur[2]));
                        if is_log {
                            felled += 1;
                        }
                        for dx in -1..=1 {
                            for dy in 0..=1 {
                                for dz in -1..=1 {
                                    let nb = [cur[0] + dx, cur[1] + dy, cur[2] + dz];
                                    if visited.insert(nb) {
                                        q.push_back(nb);
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("🪓 [Motosierra MK{tier}] Árbol talado ({felled} troncos, -{cost} FE)")));
                }
                return canceled;
            }
        }

        // 3b. Portable Batteries & Dorsal Battery Pack: Inspect charge in hand
        if rules::is_battery_item(held_id) {
            canceled = true;
            let cap = rules::battery_capacity(held_id).unwrap_or(50_000);
            let cur = hc::acting_held(host).map_or(0, |s| get_item_energy(&s));
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal(format!("🔋 [Batería] Carga: {cur} / {cap} FE (recarga tu equipo automáticamente)")));
            }
            return canceled;
        }

        // 3b2. Jetpack MK2 (Hover): Shift toggles Hover mode, Click propels
        if held_id == rules::ITEM_JETPACK_MK2 {
            canceled = true;
            if is_crouching {
                let mut now_hover = false;
                hc::acting_held_mut(host, |slot| if let Some(held) = slot.as_mut() {
                now_hover = !is_jetpack_hover(held);
                    set_jetpack_hover(held, now_hover);
            });
                if let Some(who) = host.players().acting_player() {
                    let m = if now_hover { "ACTIVADO" } else { "DESACTIVADO" };
                    let _ = who.action_bar(host, Text::literal(format!("🚀 [Jetpack MK2] Modo Hover {m}")));
                }
            } else {
                hc::run_command(host, "playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.4 1.15");
                hc::run_command(host, "effect give @s minecraft:levitation 1 16 true");
                hc::run_command(host, "effect give @s minecraft:slow_falling 8 0 true");
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("🚀 [Jetpack MK2] Propulsión Avanzada"));
                }
            }
            return canceled;
        }

        // 4. Jet Boots: Rocket propulsion boost with flame & smoke exhaust, slow falling to prevent fall damage!
        if held_id == rules::ITEM_JET_BOOTS {
            canceled = true;
            hc::run_command(host, "playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.2 1.1");
            hc::run_command(host, "effect give @s minecraft:levitation 1 10 true");
            hc::run_command(host, "effect give @s minecraft:slow_falling 5 0 true");
            hc::run_command(host, "particle minecraft:flame ~ ~ ~ 0.3 0.1 0.3 0.05 20");
            hc::run_command(host, "particle minecraft:smoke ~ ~ ~ 0.2 0.1 0.2 0.02 15");
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("🚀 [Botas Cohete] ¡Propulsión Cohete Activada!"));
            }
            return canceled;
        }

        // 5. Jetpack: Sustained High-Altitude Flight
        if held_id == rules::ITEM_JETPACK {
            canceled = true;
            hc::run_command(host, "playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.5 1.0");
            hc::run_command(host, "effect give @s minecraft:levitation 1 15 true");
            hc::run_command(host, "effect give @s minecraft:slow_falling 6 0 true");
            hc::run_command(host, "particle minecraft:flame ~ ~ ~ 0.4 0.1 0.4 0.08 30");
            hc::run_command(host, "particle minecraft:campfire_cosy_smoke ~ ~ ~ 0.3 0.1 0.3 0.03 15");
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal("🚀 [Jetpack Industrial] ¡Propulsión Aérea Máxima Activada!"));
            }
            return canceled;
        }

        // 6. Item Magnet (MK1-MK3): Attracts nearby dropped items and XP
        if let Some((tier, radius, cost)) = rules::magnet_tier(held_id) {
            canceled = true;
            if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("⚠ [Imán] Sin energía"));
                }
                return canceled;
            }
            hc::run_command(host, "playsound minecraft:hc::item_id(host, &entity).pickup player @s ~ ~ ~ 1.0 1.5");
            hc::run_command(host, &format!("tp @e[type=item,distance=..{radius}] @s"));
            hc::run_command(host, &format!("tp @e[type=experience_orb,distance=..{radius}] @s"));
            hc::run_command(host, "particle minecraft:portal ~ ~1 ~ 0.5 0.5 0.5 0.1 25");
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal(format!("🧲 [Imán MK{tier}] ¡Ítems y XP en {radius} blq atraídos!")));
            }
            return canceled;
        }

        // 7a. Wireless Silo Tablet & Quantum Backpack: Shift+Click on Silo/Controller binds; Click uses
        if matches!(held_id.as_str(), rules::ITEM_WIRELESS_SILO_TABLET | rules::ITEM_QUANTUM_BACKPACK) {
            canceled = true;
            if is_crouching
                && matches!(
                    block_id.as_str(),
                    rules::BLOCK_SILO_CONTROLLER | rules::BLOCK_SILO_CRAFTING_TERMINAL | rules::BLOCK_INDUSTRIAL_SILO
                )
            {
                hc::acting_held_mut(host, |slot| if let Some(held) = slot.as_mut() {
                set_bound_pos(held, pos);
            });
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("🔗 Vinculado a la red de Silos en [{}, {}, {}]", pos[0], pos[1], pos[2])));
                }
                return canceled;
            }
            if held_id == rules::ITEM_QUANTUM_BACKPACK {
                let mut now_on = true;
                hc::acting_held_mut(host, |slot| if let Some(held) = slot.as_mut() {
                now_on = !is_qpack_on(held);
                    set_qpack_on(held, now_on);
            });
                if let Some(who) = host.players().acting_player() {
                    let lbl = if now_on { "ACTIVADO" } else { "DESACTIVADO" };
                    let _ = who.action_bar(host, Text::literal(format!("🎒 [Mochila Cuántica] Auto-envío a Silos: {lbl}")));
                }
                return canceled;
            }
            let bound_opt = hc::acting_held(host).and_then(|s| get_bound_pos(&s));
            let Some(bound) = bound_opt else {
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("📡 [Tablet Silo] Shift + Clic en un Controlador de Silos para vincular."));
                }
                return canceled;
            };
            if !hc::acting_creative(host) && !consume_player_energy(host, 50) {
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("⚠ [Tablet Silo] Sin energía (50 FE)"));
                }
                return canceled;
            }
            if let Some(off_stack) = hc::acting_offhand(host) {
                let sent_name = hc::item_id(host, &off_stack).split(':').nth(1).unwrap_or(&hc::item_id(host, &off_stack)).to_owned();
                let sent_cnt = off_stack.count();
                let rem = insert_into_silo_controller(host, bound, off_stack);
                hc::set_acting_offhand(host, rem);
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("📡 [Tablet Silo] Enviado {sent_name} x{sent_cnt} a la red!")));
                }
            } else if let Some(who) = host.players().acting_player() {
                let (list, silos_n, _) = sorted_silo_network(host, bound, false, false);
                let preview: Vec<String> = list.iter().take(3).map(|(id, c, _)| {
                    let s = id.split(':').nth(1).unwrap_or(id);
                    format!("{s}:{c}")
                }).collect();
                let summary = if preview.is_empty() { "Vacío".to_owned() } else { preview.join(" | ") };
                let _ = who.action_bar(host, Text::literal(format!("📡 [Tablet Silo] ({silos_n} silos): {summary}")));
            }
            return canceled;
        }

        // 7. Mining Laser (MK1-MK3): High-energy straight plasma beam mining along look direction
        if let Some((tier, range, base_cost)) = rules::laser_tier(held_id) {
            canceled = true;
            let eff_mod = hc::acting_held(host).is_some_and(|h| has_item_module(&h, "eff"));
            let smelt_mod = tier >= 3 || hc::acting_held(host).is_some_and(|h| has_item_module(&h, "smelt"));
            let fortune_mod = hc::acting_held(host).is_some_and(|h| has_item_module(&h, "fortune"));
            let cost = if eff_mod { (base_cost / 2).max(5) } else { base_cost };
            if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("⚠ [Láser] Sin energía"));
                }
                return canceled;
            }
            hc::run_command(host, "playsound minecraft:entity.guardian.attack player @s ~ ~ ~ 1.2 1.6");
            let [cx, cy, cz] = pos;
            let dir = [-face[0], -face[1], -face[2]];
            let dim = host.dimension();
            {
                for dist in 0..range {
                    let p = [cx + dir[0] * dist, cy + dir[1] * dist, cz + dir[2] * dist];
                    let b = hc::block_id_at(host, p);
                    if !b.is_empty() && b != "minecraft:air" && b != "minecraft:bedrock" && !b.starts_with("minecraft:barrier") {
                        hc::set_block_by_id(host, p, "minecraft:air");
                        if !hc::acting_creative(host) {
                            if let Some((drop_id, base_cnt)) = rules::quarry_drop_for_block(&b) {
                                let final_id = if smelt_mod {
                                    match drop_id {
                                        "minecraft:raw_iron" => "minecraft:iron_ingot",
                                        "minecraft:raw_copper" => "minecraft:copper_ingot",
                                        "minecraft:raw_gold" => "minecraft:gold_ingot",
                                        "minecraft:cobblestone" => "minecraft:stone",
                                        _ => drop_id,
                                    }
                                } else {
                                    drop_id
                                };
                                let cnt = if fortune_mod { base_cnt + 1 } else { base_cnt };
                                { let _st = hc::make_stack(host, final_id, cnt); hc::pop_resource(host, p, _st); };
                            }
                        }
                    }
                }
            }
            hc::run_command(host, &format!("particle minecraft:sonic_boom {} {} {} 0 0 0 0 1", cx, cy, cz));
            if let Some(who) = host.players().acting_player() {
                let _ = who.action_bar(host, Text::literal(format!("⚡ [Láser Minero MK{tier}] Haz de {range} bloques disparado (-{cost} FE)")));
            }
            return canceled;
        }

        // 8. Plasma Sword (MK1-MK3)
        if let Some((tier, dmg, cost)) = rules::sword_tier(held_id) {
            canceled = true;
            if !hc::acting_creative(host) && !consume_player_energy(host, cost) {
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("⚠ [Sable de Plasma] Sin energía"));
                }
                return canceled;
            }
            if tier >= 3 && is_crouching {
                let [cx, cy, cz] = pos;
                hc::run_command(host, &format!("tp @s {} {} {}", cx, cy + 1, cz));
                hc::run_command(host, "playsound minecraft:entity.enderman.teleport player @s ~ ~ ~ 1.0 1.2");
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal("⚡ [Sable Cuántico] ¡Blink Cuántico!"));
                }
            } else {
                let rad = if tier >= 2 { 5 } else { 3 };
                hc::run_command(host, &format!("damage @e[type=!player,distance=..{rad}] {dmg} minecraft:lightning_bolt by @s"));
                hc::run_command(host, "playsound minecraft:entity.player.attack.sweep player @s ~ ~ ~ 1.2 1.4");
                if let Some(who) = host.players().acting_player() {
                    let _ = who.action_bar(host, Text::literal(format!("⚔ [Sable de Plasma MK{tier}] Descarga de Plasma ({dmg} daño)")));
                }
            }
            return canceled;
        }

    }

    if is_crouching {
        return canceled;
    }

    if block_id == rules::BLOCK_RUBBER_SAPLING {
        // Require bone meal in hand to accelerate rubber tree growth (50% chance per bone meal)
        if let Some((held_id, _, _)) = held_info {
            if held_id == "minecraft:bone_meal" {
                canceled = true;
                hc::take_items(host, "minecraft:bone_meal", 1);
                if (hc::game_time(host) & 1) == 0 {
                    grow_rubber_tree(host, pos);
                }
            }
        }
        return canceled;
    }

    if block_id == rules::BLOCK_RUBBER_SAP_LOG {
        if held_info.is_none() {
            let has_sap = hc::block_property_at(host, pos, "lit").as_deref() == Some("true");
            if has_sap {
                canceled = true;
                host.set_block_property(pos, "lit", "false");
                hc::give_items(host, rules::ITEM_RAW_SAP, 3);
                if (hc::game_time(host) & 3) == 0 {
                    hc::give_items(host, rules::BLOCK_RUBBER_SAPLING, 1);
                }
            }
        }
        return canceled;
    }

    if block_id == rules::BLOCK_FLUID_TANK {
        if let Some((ref held_id, _, _)) = held_info {
            if held_id == "minecraft:water_bucket" {
                canceled = true;
                let mut msg = None;
                hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::FluidTank));
                    if (st.fluid_kind.is_empty() || st.fluid_kind == "water")
                        && st.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB
                    {
                        st.fluid_kind = "water".to_owned();
                        st.fluid_amount += 1000;
                        store_state(entity, &st);
                        msg = Some(format!("💧 [Tanque] Agua: {} / {} mB", st.fluid_amount, rules::FLUID_TANK_MAX_MB));
                    } else {
                        msg = Some("⚠️ [Tanque] Lleno o contiene otro fluido".to_owned());
                    }
                });
                if let Some(m) = msg {
                    if m.starts_with('💧') {
                        hc::take_items(host, "minecraft:water_bucket", 1);
                        hc::give_items(host, "minecraft:bucket", 1);
                        host.set_block_property(pos, "lit", "true");
                    }
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(m));
                    }
                }
                return canceled;
            } else if held_id == "minecraft:lava_bucket" {
                canceled = true;
                let mut msg = None;
                hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::FluidTank));
                    if (st.fluid_kind.is_empty() || st.fluid_kind == "lava")
                        && st.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB
                    {
                        st.fluid_kind = "lava".to_owned();
                        st.fluid_amount += 1000;
                        store_state(entity, &st);
                        msg = Some(format!("🔥 [Tanque] Lava: {} / {} mB", st.fluid_amount, rules::FLUID_TANK_MAX_MB));
                    } else {
                        msg = Some("⚠️ [Tanque] Lleno o contiene otro fluido".to_owned());
                    }
                });
                if let Some(m) = msg {
                    if m.starts_with('🔥') {
                        hc::take_items(host, "minecraft:lava_bucket", 1);
                        hc::give_items(host, "minecraft:bucket", 1);
                        host.set_block_property(pos, "lit", "true");
                    }
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(m));
                    }
                }
                return canceled;
            } else if held_id == rules::ITEM_RAW_SAP {
                canceled = true;
                let mut msg = None;
                hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::FluidTank));
                    if (st.fluid_kind.is_empty() || st.fluid_kind == "sap")
                        && st.fluid_amount + 250 <= rules::FLUID_TANK_MAX_MB
                    {
                        st.fluid_kind = "sap".to_owned();
                        st.fluid_amount += 250;
                        store_state(entity, &st);
                        msg = Some(format!("🍯 [Tanque] Savia: {} / {} mB", st.fluid_amount, rules::FLUID_TANK_MAX_MB));
                    } else {
                        msg = Some("⚠️ [Tanque] Lleno o contiene otro fluido".to_owned());
                    }
                });
                if let Some(m) = msg {
                    if m.starts_with('🍯') {
                        hc::take_items(host, rules::ITEM_RAW_SAP, 1);
                        host.set_block_property(pos, "lit", "true");
                    }
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(m));
                    }
                }
                return canceled;
            } else if held_id == "minecraft:bucket" {
                canceled = true;
                let mut extracted: Option<&'static str> = None;
                let mut remain = 0;
                hc::with_mod_block_entity(host, pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::FluidTank));
                    if st.fluid_amount >= 1000 && matches!(st.fluid_kind.as_str(), "water" | "lava") {
                        extracted = if st.fluid_kind == "lava" {
                            Some("minecraft:lava_bucket")
                        } else {
                            Some("minecraft:water_bucket")
                        };
                        st.fluid_amount -= 1000;
                        if st.fluid_amount == 0 {
                            st.fluid_kind.clear();
                        }
                        remain = st.fluid_amount;
                        store_state(entity, &st);
                    }
                });
                if let Some(bucket_out) = extracted {
                    hc::take_items(host, "minecraft:bucket", 1);
                    hc::give_items(host, bucket_out, 1);
                    if remain == 0 {
                        host.set_block_property(pos, "lit", "false");
                    }
                    if let Some(who) = host.players().acting_player() {
                        let _ = who.action_bar(host, Text::literal(format!("🪣 [Tanque] Extraído 1,000 mB (Restan: {remain} mB)")));
                    }
                }
                return canceled;
            }
        }
        return canceled;
    }

    if block_id == rules::BLOCK_ENERGY_MONITOR {
        if held_info.is_some() {
            return canceled;
        }
        canceled = true;
        let consumers = rules::find_connected_consumers(
            pos,
            |p| rules::is_cable(&hc::block_id_at(host, p)),
            |p| MachineKind::from_block_id(&hc::block_id_at(host, p)).is_some(),
        );
        let mut total_energy = 0;
        let mut total_capacity = 0;
        for cpos in &consumers {
            hc::with_mod_block_entity(host, *cpos, |entity| {
                if let Some(st) = load_state(entity) {
                    total_energy += st.energy;
                    total_capacity += st.kind.max_energy();
                }
            });
        }
        if let Some(who) = host.players().acting_player() {
            let _ = who.action_bar(host, Text::literal(format!(
                    "⚡ [Monitor de Red] Red Eléctrica: {} / {} FE ({} máquinas conectadas)",
                    total_energy, total_capacity, consumers.len()
                )));
        }
        return canceled;
    }

    if block_id == rules::BLOCK_ENERGY_RELAY {
        if held_info.is_some() {
            return false;
        }
        let total_relays = hc::SERVER_ACTIVE_BE
            .lock()
            .map(|m| m.values().filter(|id| id.as_str() == rules::BLOCK_ENERGY_RELAY).count())
            .unwrap_or(1)
            .max(1);
        if let Some(who) = host.players().acting_player() {
            let _ = who.action_bar(host,
                Text::literal(format!(
                    "📡 [Relé Inalámbrico] Red de Energía Inalámbrica: {} relés activos",
                    total_relays
                )),
            );
        }
        return true;
    }

    if rules::is_cable(&block_id) {
        return canceled;
    }

    if rules::is_item_pipe(&block_id) {
        let dim = host.dimension();
        let targets = rules::find_connected_item_targets(
            pos,
            |p| rules::is_item_pipe(&hc::block_id_at(host, p)),
            |p| is_item_container_block(&hc::block_id_at(host, p)),
        );
        // Silently pull up to 4 items from adjacent container and route through the pipe network
        for [dx, dy, dz] in rules::NEIGHBORS_6 {
            let adj = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let adj_id = hc::block_id_at(host, adj);
            if matches!(
                adj_id.as_str(),
                "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:barrel" | "minecraft:hopper"
            ) {
                for &dest in &targets {
                    if dest != adj {
                        hc::move_items(host,
                            adj,
                            Direction::Down,
                            dest,
                            Direction::Up,
                            None,
                            4,
                        );
                    }
                }
            }
        }
        return canceled;
    }
    canceled
}

pub(crate) fn menu_button(entity: &mut ModBlockEntityView<'_>, id: i32) {
    if let Some(kind) = MachineKind::from_block_id(&entity.id()) {
        let mut state = load_state(entity).unwrap_or_else(|| MachineState::new(kind));
        match id {
            BTN_TOGGLE_POWER => {
                state.enabled = !state.enabled;
            }
            BTN_MODE_OR_SIZE => {
                if matches!(kind, MachineKind::Quarry | MachineKind::AutoHarvester) {
                    // Cycle Quarry/Harvester area: 5x5 (2) -> 7x7 (3) -> 9x9 (4) -> 5x5 (2)
                    state.quarry_radius = match state.quarry_radius {
                        2 => 3,
                        3 => 4,
                        _ => 2,
                    };
                    state.quarry_cursor = 0;
                } else {
                    state.auto_io = !state.auto_io;
                }
            }
            BTN_QUARRY_RESET if kind == MachineKind::Quarry => {
                state.quarry_y_offset = 0;
                state.quarry_cursor = 0;
            }
            20 if matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) => {
                state.quarry_cursor = (state.quarry_cursor - 1).max(0);
            }
            21 if matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) => {
                state.quarry_cursor += 1;
            }
            22 if matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) => {
                state.enabled = !state.enabled;
            }
            23 if matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) => {
                state.auto_io = !state.auto_io;
            }
            _ => {}
        }
        store_state(entity, &state);
    }
}

pub(crate) fn menu_button_v2(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    entity: &mut ModBlockEntityView<'_>,
    id: i32,
    carried: &mut Option<ItemStack>,
) {
    let Some(kind) = MachineKind::from_block_id(&entity.id()) else {
        return;
    };
    if !matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) {
        return;
    }
    if id == 26 {
        if let Ok(mut s) = TERMINAL_SEARCH.lock() {
            s.clear();
        }
        return;
    }
    if !(30..=56).contains(&id) {
        return;
    }
    // 1. If holding an item on cursor -> deposit straight into Silo/Chest network!
    if let Some(cursor_stack) = carried.take() {
        if let Some(rest) = insert_into_silo_controller(host, pos, cursor_stack) {
            *carried = Some(rest);
        }
        host.block_entity_sync(pos);
        return;
    }
    // 2. If cursor is empty -> withdraw up to 64 of clicked item straight onto cursor!
    let state = load_state(entity).unwrap_or_else(|| MachineState::new(kind));
    let search = get_terminal_search();
    let (list, _, _) = filtered_silo_network(host, pos, state.auto_io, state.enabled, &search);
    let page = state.quarry_cursor.max(0) as usize;
    let slot_idx = (id - 30) as usize;
    let Some((target_item, _, _)) = list.get(page * 27 + slot_idx) else {
        return;
    };
    let (silos, chests) = collect_silo_network(host, pos);
    let mut need = 64u32;
    let mut taken = 0u32;
    for &sp in &silos {
        if need == 0 {
            break;
        }
        let mut took = 0u32;
        hc::with_mod_block_entity(host, sp, |sent| {
            if let Some(mut sst) = load_state(sent) {
                if sst.belt_item == *target_item && sst.belt_count > 0 {
                    took = sst.belt_count.min(need);
                    sst.belt_count -= took;
                    if sst.belt_count == 0 {
                        sst.belt_item.clear();
                    }
                    store_state(sent, &sst);
                }
            }
        });
        if took > 0 {
            host.block_entity_sync(sp);
            need -= took;
            taken += took;
        }
    }
    for &cp in &chests {
        if need == 0 {
            break;
        }
        let mut took = 0u32;
        while need > 0 && hc::take_one_matching_from_container(host, cp, &target_item) {
            took += 1;
            need -= 1;
        }
        taken += took;
    }
    if taken > 0 {
        *carried = hc::make_stack(host, target_item, taken);
        host.block_entity_sync(pos);
    }
}

pub(crate) fn screen(block: &str) -> Option<Screen> {
    if let Some(kind) = MachineKind::from_block_id(block) {
        let slots = match kind {
            MachineKind::CoalGenerator
            | MachineKind::BioGenerator
            | MachineKind::SteamBoiler
            | MachineKind::FluidTank => {
                vec![Some([52.0, 34.0])] // Left-center fuel / bucket slot
            }
            MachineKind::SolarGenerator | MachineKind::EnergyBank => {
                vec![Some([124.0, 34.0])] // Right-side recharge / battery slot
            }
            MachineKind::ChargingPad
            | MachineKind::VoidTrashCan
            | MachineKind::IndustrialSilo
            | MachineKind::GeothermalGenerator
            | MachineKind::SteamTurbine
            | MachineKind::NuclearReactor
            | MachineKind::WindTurbine => {
                vec![Some([80.0, 34.0])] // Centered 1-slot
            }
            MachineKind::SiloController => {
                vec![Some([134.0, 88.0]), Some([134.0, 114.0])] // 0: Depositar, 1: Retirar
            }
            MachineKind::SiloCraftingTerminal => {
                vec![
                    Some([18.0, 82.0]), Some([36.0, 82.0]), Some([54.0, 82.0]),
                    Some([18.0, 100.0]), Some([36.0, 100.0]), Some([54.0, 100.0]),
                    Some([18.0, 118.0]), Some([36.0, 118.0]), Some([54.0, 118.0]),
                    Some([134.0, 100.0]),
                ]
            }
            MachineKind::FluidPump => {
                vec![Some([143.0, 35.0])] // Right-side speed upgrade slot
            }
            MachineKind::AlloySmelter => {
                vec![
                    Some([32.0, 35.0]),  // Slot 0: Primary Input
                    Some([54.0, 35.0]),  // Slot 1: Secondary Input / Flux
                    Some([112.0, 35.0]), // Slot 2: Alloy Output
                    Some([143.0, 35.0]), // Slot 3: Speed Upgrade
                ]
            }
            MachineKind::Quarry | MachineKind::AutoHarvester => {
                vec![
                    Some([32.0, 35.0]),  // Slot 0: Buffer
                    None,
                    Some([112.0, 35.0]), // Slot 2: Output
                    Some([143.0, 35.0]), // Slot 3: Speed Upgrade
                ]
            }
            _ => {
                vec![
                    Some([40.0, 35.0]),  // Slot 0: Input / Belt Item
                    None,                // Slot 1: Hidden
                    Some([104.0, 35.0]), // Slot 2: Output
                    Some([143.0, 35.0]), // Slot 3: Speed Upgrade
                ]
            }
        };

        let (height, inventory_y) = if matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) {
            (222.0, 140.0)
        } else {
            (166.0, 84.0)
        };
        return Some(Screen {
            width: 176.0,
            height,
            slots,
            inventory_y,
            inventory_x: 8.0,
            own_labels: true,
            own_background: true,
            ..Screen::default()
        });
    }
    None
}

fn fill_template(key: &str, args: &[String]) -> String {
    let mut line = hc::lang_get(key);
    for arg in args {
        line = line.replacen("%s", arg, 1);
    }
    line
}

fn draw_button(
    host: &dyn Host,
    draw: &mut dyn DrawContext,
    origin: [f32; 2],
    rect: [f32; 4],
    label: &str,
    bg_color: u32,
) {
    let [ox, oy] = origin;
    let [bx, by, bw, bh] = rect;
    draw.fill( [ox + bx, oy + by, bw, bh], 0xFF22_252A);
    draw.fill(
        [ox + bx + 1.0, oy + by + 1.0, bw - 2.0, bh - 2.0],
        bg_color,
    );
    let tw = draw.text_width(label);
    draw.text(
        label,
        [ox + bx + (bw - tw) / 2.0, oy + by + 3.0],
        0xFF_FFFF,
        false,
    );
}

pub(crate) fn screen_draw(
    host: &dyn ClientHost,
    block: &str,
    data: &[i32],
    draw: &mut dyn DrawContext,
    origin: [f32; 2],
) {
    let Some(kind) = MachineKind::from_block_id(block) else {
        return;
    };
    let [x, y] = origin;
    let get = |i: usize| data.get(i).copied().unwrap_or(0);

    if matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) {
        // AE2 / Terminal-style Silo Controller GUI (176 x 222)
        // Outer Minecraft-style light gray panel with bevel
        draw.fill( [x, y, 176.0, 222.0], 0xFFC6_C6C6);
        draw.fill( [x, y, 176.0, 2.0], 0xFFFF_FFFF);
        draw.fill( [x, y, 2.0, 222.0], 0xFFFF_FFFF);
        draw.fill( [x, y + 220.0, 176.0, 2.0], 0xFF55_5555);
        draw.fill( [x + 174.0, y, 2.0, 222.0], 0xFF55_5555);

        // Top 9x3 Dark Storage Grid [7, 7, 162, 54]
        draw.fill( [x + 6.0, y + 6.0, 164.0, 56.0], 0xFF1E_1E1E);
        draw.fill( [x + 7.0, y + 7.0, 162.0, 54.0], 0xFF37_3737);

        TERMINAL_OPEN.store(true, std::sync::atomic::Ordering::Relaxed);
        let sort_by_name = get(4) != 0;
        let ascending = get(5) != 0;
        let ctrl_pos = [get(7), get(8), get(9)];
        let page = get(6).max(0) as usize;
        let search = get_terminal_search();
        let (list, _silos_n, _chests_n) = filtered_silo_network(host, ctrl_pos, sort_by_name, ascending, &search);
        let total_pages = ((list.len().saturating_sub(1)) / 27) + 1;
        let cur_page = page.min(total_pages - 1);

        for row in 0..3 {
            for col in 0..9 {
                let idx = cur_page * 27 + row * 9 + col;
                let sx = x + 7.0 + col as f32 * 18.0;
                let sy = y + 7.0 + row as f32 * 18.0;
                if let Some((item_id, cnt, is_silo)) = list.get(idx) {
                    draw.fill( [sx + 0.5, sy + 0.5, 17.0, 17.0], 0xFF42_4242);
                    hc::draw_item_id(host, draw, item_id, [sx + 1.0, sy + 1.0], 16.0);
                    let cnt_str = if *cnt >= 1_000_000 {
                        format!("{}M", cnt / 1_000_000)
                    } else if *cnt >= 1_000 {
                        format!("{}K", cnt / 1_000)
                    } else {
                        cnt.to_string()
                    };
                    let col_txt = if *is_silo { 0xFF88_FFAA } else { 0xFFFF_FFFF };
                    let tw = (cnt_str.len() as f32 * 5.0).min(16.0);
                    draw.text( &cnt_str, [sx + 17.0 - tw, sy + 10.0], col_txt, true);
                }
            }
        }

        // Middle Control Bar (y + 64)
        let dir_lbl = if ascending { "↑" } else { "↓" };
        let sort_lbl = if sort_by_name { "A" } else { "#" };
        draw_button(host, draw, origin, [7.0, 64.0, 13.0, 13.0], dir_lbl, 0xFF5A_5A5A);
        draw_button(host, draw, origin, [22.0, 64.0, 13.0, 13.0], sort_lbl, 0xFF5A_5A5A);
        draw_button(host, draw, origin, [37.0, 64.0, 13.0, 13.0], "◀", 0xFF5A_5A5A);
        draw_button(host, draw, origin, [52.0, 64.0, 13.0, 13.0], "▶", 0xFF5A_5A5A);

        // Dark search bar [68, 64, 88, 13] + [X] clear button [157, 64, 12, 13]
        draw.fill( [x + 68.0, y + 64.0, 88.0, 13.0], 0xFF14_1414);
        if search.is_empty() {
            draw.text( "Buscar...", [x + 71.0, y + 66.5], 0xFF70_7888, false);
        } else {
            draw.text( &format!("{search}_"), [x + 71.0, y + 66.5], 0xFFFF_FFFF, false);
        }
        draw_button(host, draw, origin, [157.0, 64.0, 12.0, 13.0], "X", 0xFF5A_5A5A);

        // Lower 3x3 Visual Grid + Arrow + Deposit Slot
        for r in 0..3 {
            for c in 0..3 {
                let gx = x + 18.0 + c as f32 * 18.0;
                let gy = y + 82.0 + r as f32 * 18.0;
                draw.fill( [gx, gy, 17.0, 17.0], 0xFF37_3737);
                draw.fill( [gx + 1.0, gy + 1.0, 16.0, 16.0], 0xFF8B_8B8B);
            }
        }
        draw.text( "→", [x + 88.0, y + 104.0], 0xFF55_5555, false);
        if kind == MachineKind::SiloCraftingTerminal {
            draw.text( "Salida", [x + 126.0, y + 88.0], 0xFF3A_3A3A, false);
            draw.fill( [x + 133.0, y + 99.0, 18.0, 18.0], 0xFF37_3737);
            draw.fill( [x + 134.0, y + 100.0, 17.0, 17.0], 0xFF8B_8B8B);
        } else {
            draw.text( "Depositar", [x + 84.0, y + 93.0], 0xFF3A_3A3A, false);
            draw.fill( [x + 133.0, y + 87.0, 18.0, 18.0], 0xFF37_3737);
            draw.fill( [x + 134.0, y + 88.0, 17.0, 17.0], 0xFF8B_8B8B);
            draw.text( "Retirar", [x + 92.0, y + 119.0], 0xFF3A_3A3A, false);
            draw.fill( [x + 133.0, y + 113.0, 18.0, 18.0], 0xFF37_3737);
            draw.fill( [x + 134.0, y + 114.0, 17.0, 17.0], 0xFF8B_8B8B);
        }
        // Player Inventory 3x9 + 1x9 hotbar slot frames (inventory_x=8, inventory_y=140)
        for row in 0..3 {
            for col in 0..9 {
                let ix = x + 7.0 + col as f32 * 18.0;
                let iy = y + 139.0 + row as f32 * 18.0;
                draw.fill( [ix, iy, 18.0, 18.0], 0xFF37_3737);
                draw.fill( [ix + 1.0, iy + 1.0, 17.0, 17.0], 0xFFFF_FFFF);
                draw.fill( [ix + 1.0, iy + 1.0, 16.0, 16.0], 0xFF8B_8B8B);
            }
        }
        for col in 0..9 {
            let ix = x + 7.0 + col as f32 * 18.0;
            let iy = y + 197.0;
            draw.fill( [ix, iy, 18.0, 18.0], 0xFF37_3737);
            draw.fill( [ix + 1.0, iy + 1.0, 17.0, 17.0], 0xFFFF_FFFF);
            draw.fill( [ix + 1.0, iy + 1.0, 16.0, 16.0], 0xFF8B_8B8B);
        }
        return;
    }

    draw.sprite(kind.gui_sprite(), [0.0, 0.0, 176.0, 166.0], [x, y, 176.0, 166.0], None);
    let status_code = get(0);
    let energy = get(1);
    let max_energy = get(2).max(1);
    let progress_pct = get(3).clamp(0, 100);
    let rate = get(4);
    let enabled = get(5) != 0;
    let auto_io = get(6) != 0;
    let quarry_radius = get(7).clamp(2, 4);
    let quarry_y = get(8);
    let quarry_mined = get(9);

    // 1. Header Title (white text on dark steel header bar)
    let tier_tag = match kind.tier() {
        MachineTier::Tier1 => "[Tier 1]",
        MachineTier::Tier2 => "[Tier 2]",
        MachineTier::Storage => "[Almacén]",
    };
    let raw_title = hc::lang_get(kind.title_key());
    let title = format!("{raw_title} {tier_tag}");
    draw.text( &title, [x + 7.0, y + 5.0], 0xF0_F4F8, true);

    // 2. Status line below header
    let status_text = hc::lang_get(&format!("rusty_factory.status.{status_code}"));
    let status_color = if status_code == StepStatus::Working as i32 {
        0x1B_6E28
    } else if status_code == StepStatus::Paused as i32 {
        0x66_6666
    } else {
        0x9E_2A2B
    };
    draw.text( &status_text, [x + 8.0, y + 20.0], status_color, false);

    // 3. Machine-Specific Center Modules
    if matches!(kind, MachineKind::Quarry | MachineKind::AutoHarvester) {
        // Radar monitor display inside [54, 26]..[106, 54]
        let side = quarry_radius * 2 + 1;
        let area_txt = format!("{side}x{side}");
        let y_txt = if kind == MachineKind::Quarry {
            format!("Y:{quarry_y}")
        } else {
            format!("#{quarry_mined}")
        };
        draw.text( &area_txt, [x + 60.0, y + 30.0], 0x55_FFAA, false);
        draw.text( &y_txt, [x + 60.0, y + 42.0], 0x38_F2FF, false);
    } else if matches!(kind, MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler) {
        let burn_left = get(8).max(0);
        let burn_secs = burn_left / 20;
        let (accent_col, bolt_col) = match kind {
            MachineKind::CoalGenerator => (0xFFFF_7B25, 0xFFFF_D166),
            MachineKind::BioGenerator => (0xFF55_DD44, 0xFF99_FF66),
            _ => (0xFF38_D6FF, 0xFFAA_F0FF),
        };
        // Module A: Lightning Bolt + Tick Duration Bar [74, 28..104, 54]
        draw.text( "⚡", [x + 76.0, y + 29.0], bolt_col, false);
        let tick_txt = if burn_left > 0 {
            format!("{burn_left}t")
        } else {
            "0t".to_string()
        };
        draw.text( &tick_txt, [x + 85.0, y + 30.0], 0xFF_FFFF, false);
        draw.fill( [x + 76.0, y + 41.0, 26.0, 5.0], 0xFF22_262E);
        let bar_w = (26.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 26.0);
        if bar_w > 0.0 {
            draw.fill( [x + 76.0, y + 41.0, bar_w, 5.0], accent_col);
        }
        let sec_txt = format!("{burn_secs}s");
        draw.text( &sec_txt, [x + 79.0, y + 48.0], 0xAA_B8C8, false);

        // Module B: Stored & Generated Energy Readout [108, 28..168, 54]
        draw.text( &format!("Alm:{energy}"), [x + 111.0, y + 31.0], 0x70_E8FF, false);
        draw.text( &format!("Gen:+{rate}/t"), [x + 111.0, y + 43.0], bolt_col, false);
    } else if kind == MachineKind::SolarGenerator {
        let is_3x3 = get(7) != 0;
        let tag = if is_3x3 { "☀ MATRIZ 3x3 (x2)" } else { "☀ PANEL 1x1" };
        let tag_col = if is_3x3 { 0x55_FFAA } else { 0xFF_D166 };
        draw.text( tag, [x + 30.0, y + 30.0], tag_col, false);
        draw.text( &format!("Gen: +{rate} FE/t"), [x + 30.0, y + 40.0], 0xFF_E066, false);
        draw.text( &format!("Alm: {energy}/{max_energy}"), [x + 30.0, y + 49.0], 0x70_E8FF, false);
    } else if kind == MachineKind::EnergyBank {
        let cells = get(7).max(1);
        draw.fill( [x + 30.0, y + 34.0, 84.0, 9.0], 0xFF20_2832);
        let bar_w = (84.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 84.0);
        if bar_w > 0.0 {
            draw.fill( [x + 30.0, y + 34.0, bar_w, 9.0], 0xFF00_E5FF);
        }
        draw.text( &format!("{progress_pct}% ({cells} cel)"), [x + 32.0, y + 25.0], 0x70_E8FF, false);
        draw.text( &format!("{energy}/{max_energy} FE"), [x + 30.0, y + 46.0], 0xFF_FFFF, false);
    } else if kind == MachineKind::ChargingPad {
        draw.text( "⚡", [x + 56.0, y + 39.0], 0xFF00_FFAA, false);
        draw.text( "⚡", [x + 112.0, y + 39.0], 0xFF00_FFAA, false);
        draw.text( "INDUCCIÓN", [x + 62.0, y + 23.0], 0xFF00_FFAA, false);
    } else if kind == MachineKind::VoidTrashCan {
        draw.text( "VACÍO", [x + 74.0, y + 23.0], 0xDD_88FF, false);
    } else if kind == MachineKind::FluidTank {
        let fcode = get(7);
        let famount = get(8);
        let fmax = get(9).max(1);
        let fname = rules::fluid_name_from_code(fcode);
        let fcol = match fcode {
            1 => 0xFF25_85E8,
            2 => 0xFFFF_5511,
            3 => 0xFFEE_9B15,
            4 => 0xFFD0_F4FF,
            _ => 0xFF3A_404A,
        };
        // Tall vertical fluid column at [12, 26, 14, 30]
        draw.fill( [x + 12.0, y + 26.0, 14.0, 30.0], 0xFF1A_1E26);
        let fh = (30.0 * (famount as f32 / fmax as f32)).clamp(0.0, 30.0);
        if fh > 0.0 {
            draw.fill( [x + 12.0, y + 26.0 + (30.0 - fh), 14.0, fh], fcol);
        }
        draw.text( &format!("Fluido: {fname}"), [x + 78.0, y + 31.0], fcol, false);
        draw.text( &format!("{famount}/{fmax} mB"), [x + 78.0, y + 43.0], 0xFF_FFFF, false);
    } else if kind == MachineKind::FluidPump {
        let fcode = get(7);
        let rate_mb = get(8);
        let fname = if fcode == 0 { "Sin fuente" } else { rules::fluid_name_from_code(fcode) };
        draw.text( &format!("Fuente: {fname}"), [x + 30.0, y + 31.0], 0x38_D6FF, false);
        draw.text( &format!("Bombeo: {rate_mb} mB/t"), [x + 30.0, y + 43.0], 0x55_FFAA, false);
    } else if kind == MachineKind::AlloySmelter {
        draw.fill( [x + 76.0, y + 39.0, 30.0, 8.0], 0xFF3A_3F48);
        let bar_w = (30.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 30.0);
        if bar_w > 0.0 {
            draw.fill( [x + 76.0, y + 39.0, bar_w, 8.0], 0xFFFF_7722);
        }
        draw.text( "➔", [x + 87.0, y + 37.0], 0xFF_FFFF, false);
    } else {
        draw.fill( [x + 64.0, y + 39.0, 34.0, 8.0], 0xFF3A_3F48);
        let bar_w = (34.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 34.0);
        if bar_w > 0.0 {
            let bar_col = if matches!(kind, MachineKind::ConveyorBelt | MachineKind::ItemExtractor) {
                0xFFFF_B820
            } else {
                0xFF38_D6FF
            };
            draw.fill( [x + 64.0, y + 39.0, bar_w, 8.0], bar_col);
        }
    }

    // 4. Vertical Energy Bar on left edge `[12, 26, 8, 30]`
    if !matches!(
        kind,
        MachineKind::ConveyorBelt | MachineKind::ItemExtractor | MachineKind::FluidTank | MachineKind::VoidTrashCan
    ) {
        draw.fill( [x + 12.0, y + 26.0, 8.0, 30.0], 0xFF20_242A);
        let e_ratio = (energy as f32 / max_energy as f32).clamp(0.0, 1.0);
        let e_h = 30.0 * e_ratio;
        if e_h > 0.0 {
            draw.fill(
                [x + 12.0, y + 26.0 + (30.0 - e_h), 8.0, e_h],
                0xFF00_E5FF,
            );
        }
    }

    // 5. Dark Telemetry Strip Readout (`y = 60..79`)
    if kind == MachineKind::Quarry {
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Minados: {quarry_mined} blq | Prof: Y={quarry_y}");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFF_D166, false);
    } else if kind == MachineKind::AutoHarvester {
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Cosechados: {quarry_mined} cultivos/troncos");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0x66_FF88, false);
    } else if kind == MachineKind::CoalGenerator {
        let burn_left = get(8).max(0);
        let line1 = format!("Almacenada: {energy}/{max_energy} FE | +{rate} FE/t");
        let line2 = format!("Combustión: {burn_left} ticks ({progress_pct}%)");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFF_9E00, false);
    } else if kind == MachineKind::BioGenerator {
        let burn_left = get(8).max(0);
        let line1 = format!("Almacenada: {energy}/{max_energy} FE | +{rate} FE/t");
        let line2 = format!("Biomasa: {burn_left} ticks ({progress_pct}%)");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0x70_E000, false);
    } else if kind == MachineKind::SteamBoiler {
        let burn_left = get(8).max(0);
        let line1 = format!("Almacenada: {energy}/{max_energy} FE | +{rate} FE/t");
        let line2 = format!("Presión Vapor: {burn_left} ticks ({progress_pct}%)");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0x38_D6FF, false);
    } else if kind == MachineKind::FluidTank {
        let fcode = get(7);
        let famount = get(8);
        let fmax = get(9).max(1);
        let fname = rules::fluid_name_from_code(fcode);
        let line1 = format!("Especificación: {fname} ({progress_pct}%)");
        let line2 = format!("Capacidad: {famount} / {fmax} mB (Usa Balde)");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x38_D6FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xA8_DADC, false);
    } else if kind == MachineKind::FluidPump {
        let fcode = get(7);
        let rate_mb = get(8);
        let fname = if fcode == 0 { "Sin fluido" } else { rules::fluid_name_from_code(fcode) };
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Bombeando {fname}: {rate_mb} mB/t");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0x55_FFAA, false);
    } else if kind == MachineKind::VoidTrashCan {
        let line1 = "Papelera de Vacío: Incineración Instantánea";
        let line2 = "Coloca cualquier ítem para destruirlo";
        draw.text( line1, [x + 10.0, y + 60.0], 0xDD_88FF, false);
        draw.text( line2, [x + 10.0, y + 70.0], 0xAA_B8C8, false);
    } else if kind == MachineKind::SolarGenerator {
        let is_3x3 = get(7) != 0;
        let line1 = format!("FE: {energy}/{max_energy} | Genera: +{rate} FE/t");
        let line2 = if is_3x3 {
            "Matriz 3x3 ACTIVA (+100% doble captación)".to_string()
        } else if rate > 0 {
            "Generando energía solar (Día despejado)".to_string()
        } else {
            "Sin radiación solar (Noche o bajo techo)".to_string()
        };
        let col2 = if is_3x3 { 0x55_FFAA } else if rate > 0 { 0xFF_D166 } else { 0xAA_AA_AA };
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], col2, false);
    } else if kind == MachineKind::ChargingPad {
        let line1 = format!("FE: {energy}/{max_energy} | -20 FE/t en carga");
        let line2 = "Inducción activa: Carga armaduras y slot".to_string();
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFF00_FFAA, false);
    } else if kind == MachineKind::AlloySmelter {
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Fundición de Aleaciones: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFFFF_7722, false);
    } else if kind == MachineKind::EnergyBank {
        let cells = get(7).max(1);
        let line1 = format!("FE Total: {energy}/{max_energy}");
        let line2 = if cells > 1 {
            format!("Red Multibloque: {cells} celdas unificadas")
        } else {
            "Batería Modular (10,000 FE por celda)".to_string()
        };
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0x38_D6FF, false);
    } else if kind == MachineKind::ConveyorBelt {
        let line1 = format!("Velocidad Cinta: {}%", 100 + progress_pct / 2);
        let line2 = if auto_io {
            "Auto-Extraer (Atrás): ACTIVO"
        } else {
            "Auto-Extraer (Atrás): MANUAL"
        };
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF_D166, false);
        draw.text( line2, [x + 10.0, y + 70.0], 0x90_E0EF, false);
    } else if kind == MachineKind::ItemExtractor {
        let line1 = format!("Extraídos a Tubería: {quarry_mined} ítems");
        let line2 = "Slot 0 = Filtro (Vacío = Todo)";
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF_D166, false);
        draw.text( line2, [x + 10.0, y + 70.0], 0x90_E0EF, false);
    } else if kind == MachineKind::SapExtractor {
        let line1 = format!("FE: {energy}/{max_energy} | Extracción de Savia");
        let line2 = format!("Progreso extracción: {progress_pct}% (Tronco de caucho)");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFF_B820, false);
    } else if kind == MachineKind::Vulcanizer {
        let line1 = format!("FE: {energy}/{max_energy} | Horno Vulcanizador");
        let line2 = format!("Vulcanizando caucho puro: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xEE_82EE, false);
    } else if kind == MachineKind::MechanicalPress {
        let line1 = format!("FE: {energy}/{max_energy} | Prensa Hidráulica");
        let line2 = format!("Estampado de placas y engranajes: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0x55_FFAA, false);
    } else if kind == MachineKind::ElectricCrusher {
        let line1 = format!("FE: {energy}/{max_energy} | Triturador Cinético");
        let line2 = format!("Molienda (x2 polvo mineral): {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFF_D166, false);
    } else if kind == MachineKind::ElectricFurnace {
        let line1 = format!("FE: {energy}/{max_energy} | Horno Eléctrico");
        let line2 = format!("Fundición de alta temperatura: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFF_8844, false);
    } else if kind == MachineKind::SiloController {
        let ctrl_pos = [get(7), get(8), get(9)];
        let page = get(6).max(0) as usize;
        let (list, silos_n, chests_n) = aggregate_silo_network(host, ctrl_pos);
        let total_pages = ((list.len().saturating_sub(1)) / 4) + 1;
        draw.text(
            &format!("Silos: {silos_n} | Cofres: {chests_n} | Pág {}/{total_pages}", (page + 1).min(total_pages)),
            [x + 8.0, y + 18.0],
            0xFF38_F2FF,
            false,
        );
        for row in 0..4 {
            let ry = y + 34.0 + row as f32 * 24.0;
            draw.fill( [x + 8.0, ry, 160.0, 21.0], 0xFF1A_202A);
            if let Some((item_id, cnt, is_silo)) = list.get(page * 4 + row) {
                let short = item_id.split(':').nth(1).unwrap_or(item_id);
                let tag = if *is_silo { "[Silo]" } else { "[Cofre]" };
                let col = if *is_silo { 0xFF55_FF88 } else { 0xFFFF_C800 };
                draw.text( &format!("{short}"), [x + 12.0, ry + 3.0], 0xFFE8_EEF8, false);
                draw.text( &format!("x{cnt} {tag} (Clic: -64)"), [x + 12.0, ry + 12.0], col, false);
            } else {
                draw.text( "---", [x + 12.0, ry + 7.0], 0xFF55_6070, false);
            }
        }
        draw_button(host, draw, [x, y], [8.0, 134.0, 48.0, 14.0], "◀ Ant", 0xFF28_5066);
        draw_button(host, draw, [x, y], [120.0, 134.0, 48.0, 14.0], "Sig ▶", 0xFF28_5066);
        return;
    } else if kind == MachineKind::GeothermalGenerator {
        let line1 = format!("FE: {energy}/{max_energy} (+{rate}/t)");
        let line2 = format!("Lava interna: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFFFF_8833, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFFFF_B820, false);
    } else if kind == MachineKind::SteamTurbine {
        let line1 = format!("FE: {energy}/{max_energy} (+{rate}/t)");
        let line2 = format!("Vapor interno: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF38_F2FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFFD0_F4FF, false);
    } else if kind == MachineKind::NuclearReactor {
        let line1 = format!("FE: {energy}/{max_energy} (+{rate}/t)");
        let line2 = format!("Núcleo Nuclear: {progress_pct}% (Diamante/Nether)");
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF4A_DE80, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFFBB_F7D0, false);
    } else if kind == MachineKind::WindTurbine {
        let line1 = format!("FE: {energy}/{max_energy} (+{rate}/t)");
        let line2 = format!("Viento por Altura: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF38_F2FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFFE2_E8F0, false);
    } else if kind == MachineKind::HydroponicChamber {
        let line1 = format!("FE: {energy}/{max_energy} | Cultivo Hidropónico");
        let line2 = format!("Crecimiento: {progress_pct}% (Slot 0 = Semilla)");
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF4A_DE80, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFF86_EFAC, false);
    } else if kind == MachineKind::LaserTurret {
        let line1 = format!("FE: {energy}/{max_energy} | Radio: 16 blq");
        let line2 = format!("Torreta Láser: {progress_pct}% (10 daño)");
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFFF8_7171, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFFFCA5A5, false);
    } else if kind == MachineKind::WirelessCharger {
        let line1 = format!("FE: {energy}/{max_energy} | Radio: 32 blq");
        let line2 = "Carga inalámbrica de inventario y armadura".to_string();
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF38_F2FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFF88_E5FF, false);
    } else if kind == MachineKind::TeleporterPad {
        let line1 = format!("FE: {energy}/{max_energy} | Enlaces: {rate}");
        let line2 = "Agachate encima para teletransportarte (-1,000 FE)".to_string();
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF2D_D4BF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFFC0_84FC, false);
    } else if kind == MachineKind::Recycler {
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Reciclaje de Chatarra: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF55_FF88, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0xFFAA_EEAA, false);
    } else if kind == MachineKind::Tesseract {
        let line1 = format!("FE: {energy}/{max_energy} (Inalámbrico)");
        let line2 = "Transmite FE e ítems entre Tesseracts";
        draw.text( &line1, [x + 10.0, y + 60.0], 0xFF38_F2FF, false);
        draw.text( line2, [x + 10.0, y + 70.0], 0xFFAA_88FF, false);
    } else if kind == MachineKind::IndustrialSilo {
        let bar_col = if progress_pct >= 90 {
            0xFFE6_3232
        } else if progress_pct >= 65 {
            0xFFE6_B822
        } else {
            0xFF38_D96B
        };
        let line1 = format!("Almacenado: {energy} / {max_energy} ({progress_pct}%)");
        let line2 = "Slot 0 = Depositar (Frente = Bloquear/Retirar)";
        draw.text( &line1, [x + 10.0, y + 58.0], bar_col, false);
        draw.text( line2, [x + 10.0, y + 68.0], 0xFFAA_B4C4, false);
        draw.fill( [x + 10.0, y + 78.0, 156.0, 5.0], 0xFF22_2834);
        let fw = (156.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 156.0);
        if fw > 0.5 {
            draw.fill( [x + 10.0, y + 78.0, fw, 5.0], bar_col);
        }
    } else if kind == MachineKind::IndustrialAssembler {
        let line1 = format!("FE: {energy}/{max_energy} | Ensamblador Robótico");
        let line2 = format!("Ensamblaje automatizado: {progress_pct}%");
        draw.text( &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &line2, [x + 10.0, y + 70.0], 0x38_F2FF, false);
    } else {
        let energy_line = fill_template(
            "rusty_factory.screen.energy",
            &[energy.to_string(), max_energy.to_string()],
        );
        let rate_line = fill_template("rusty_factory.screen.rate", &[rate.to_string()]);
        draw.text( &energy_line, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        draw.text( &rate_line, [x + 10.0, y + 70.0], 0xA8_DADC, false);
    }

    // 6. Interactive Buttons
    if !matches!(kind, MachineKind::FluidTank | MachineKind::VoidTrashCan) {
        let pwr_label = hc::lang_get(if enabled {
            "rusty_factory.screen.start"
        } else {
            "rusty_factory.screen.stop"
        });
        let pwr_bg = if enabled { 0xFF2B_8A3E } else { 0xFFA6_1E22 };
        draw_button(host, draw, origin, POWER_BTN_RECT, &pwr_label, pwr_bg);
    }

    if matches!(kind, MachineKind::Quarry | MachineKind::AutoHarvester) {
        let side = quarry_radius * 2 + 1;
        let size_label = format!("{side}x{side}");
        draw_button(host, draw, origin, MODE_BTN_RECT, &size_label, 0xFF3B_5B92);
        if kind == MachineKind::Quarry {
            draw_button(host, draw, origin, RESET_BTN_RECT, "TOP", 0xFF6C_584C);
        }
    } else if !matches!(
        kind,
        MachineKind::CoalGenerator
            | MachineKind::BioGenerator
            | MachineKind::SteamBoiler
            | MachineKind::SolarGenerator
            | MachineKind::ChargingPad
            | MachineKind::EnergyBank
            | MachineKind::VoidTrashCan
            | MachineKind::FluidTank
            | MachineKind::FluidPump
    ) {
        let auto_label = hc::lang_get(if auto_io {
            "rusty_factory.screen.auto_on"
        } else {
            "rusty_factory.screen.auto_off"
        });
        let auto_bg = if auto_io { 0xFF19_71C2 } else { 0xFF5C_677D };
        draw_button(host, draw, origin, MODE_BTN_RECT, &auto_label, auto_bg);
    }
}

fn in_rect(at: [f32; 2], rect: [f32; 4]) -> bool {
    let [x, y, w, h] = rect;
    (x..x + w).contains(&at[0]) && (y..y + h).contains(&at[1])
}

pub(crate) fn screen_click(
    block: &str,
    at: [f32; 2],
    right: bool,
) -> Option<i32> {
    let mut out_btn: Option<i32> = None;
    let button = &mut out_btn;
    let Some(kind) = MachineKind::from_block_id(block) else {
        return None;
    };
    if right {
        return out_btn;
    }
    if matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) {
        if in_rect(at, [7.0, 64.0, 13.0, 13.0]) {
            *button = Some(22);
        } else if in_rect(at, [22.0, 64.0, 13.0, 13.0]) {
            *button = Some(23);
        } else if in_rect(at, [37.0, 64.0, 13.0, 13.0]) {
            *button = Some(20);
        } else if in_rect(at, [52.0, 64.0, 13.0, 13.0]) {
            *button = Some(21);
        } else if in_rect(at, [157.0, 64.0, 12.0, 13.0]) {
            *button = Some(26);
        } else {
            for row in 0..3 {
                for col in 0..9 {
                    let sx = 7.0 + col as f32 * 18.0;
                    let sy = 7.0 + row as f32 * 18.0;
                    if in_rect(at, [sx, sy, 18.0, 18.0]) {
                        *button = Some(30 + (row * 9 + col) as i32);
                        return out_btn;
                    }
                }
            }
        }
        return out_btn;
    }
    if in_rect(at, POWER_BTN_RECT)
        && !matches!(kind, MachineKind::FluidTank | MachineKind::VoidTrashCan)
    {
        *button = Some(BTN_TOGGLE_POWER);
    } else if in_rect(at, MODE_BTN_RECT)
        && !matches!(
            kind,
            MachineKind::CoalGenerator
                | MachineKind::BioGenerator
                | MachineKind::SteamBoiler
                | MachineKind::SolarGenerator
                | MachineKind::ChargingPad
                | MachineKind::EnergyBank
                | MachineKind::VoidTrashCan
                | MachineKind::FluidTank
                | MachineKind::FluidPump
        )
    {
        *button = Some(BTN_MODE_OR_SIZE);
    } else if in_rect(at, RESET_BTN_RECT) && kind == MachineKind::Quarry {
        *button = Some(BTN_QUARRY_RESET);
    }
    out_btn
}

pub(crate) fn screen_tooltip(
    block: &str,
    data: &[i32],
    at: [f32; 2],
    lines: &mut Vec<String>,
) {
    let Some(kind) = MachineKind::from_block_id(block) else {
        return;
    };
    let get = |i: usize| data.get(i).copied().unwrap_or(0);
    let energy = get(1);
    let max_energy = get(2).max(1);
    let progress = get(3);
    let auto_io = get(6) != 0;
    let quarry_radius = get(7).clamp(2, 4);

    if matches!(kind, MachineKind::SiloController | MachineKind::SiloCraftingTerminal) {
        if in_rect(at, [7.0, 7.0, 162.0, 54.0]) {
            lines.push("§aGrilla de Red de Silos".into());
            lines.push("§7• Clic con ítem en cursor: Depositar en silos".into());
            lines.push("§7• Clic con cursor vacío: Retirar x64".into());
        } else if in_rect(at, [7.0, 64.0, 13.0, 13.0]) {
            lines.push("§bAlternar orden Ascendente / Descendente".into());
        } else if in_rect(at, [22.0, 64.0, 13.0, 13.0]) {
            lines.push("§bOrdenar por Cantidad (#) o Nombre (A)".into());
        } else if in_rect(at, [37.0, 64.0, 13.0, 13.0]) {
            lines.push("§bPágina Anterior".into());
        } else if in_rect(at, [52.0, 64.0, 13.0, 13.0]) {
            lines.push("§bPágina Siguiente".into());
        } else if in_rect(at, [68.0, 64.0, 88.0, 13.0]) {
            lines.push("§eBuscador: escribí letras para filtrar por nombre".into());
        } else if in_rect(at, [157.0, 64.0, 12.0, 13.0]) {
            lines.push("§cLimpiar búsqueda".into());
        }
        return;
    }

    if kind == MachineKind::FluidTank && in_rect(at, [10.0, 24.0, 18.0, 34.0]) {
        let fcode = get(7);
        let famount = get(8);
        let fmax = get(9).max(1);
        let fname = rules::fluid_name_from_code(fcode);
        lines.push(format!("§bFluido: {fname}"));
        lines.push(format!("§7Cantidad: {famount} / {fmax} mB ({progress}%)"));
    } else if in_rect(at, [10.0, 24.0, 12.0, 34.0])
        && !matches!(
            kind,
            MachineKind::ConveyorBelt | MachineKind::ItemExtractor | MachineKind::FluidTank | MachineKind::VoidTrashCan
        )
    {
        lines.push(format!("§bAlmacenamiento: {energy} / {max_energy} FE"));
        lines.push("§7Conecta Cables o usa una Batería".into());
    } else if in_rect(at, [51.0, 33.0, 18.0, 18.0])
        && matches!(
            kind,
            MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler | MachineKind::FluidTank
        )
    {
        match kind {
            MachineKind::CoalGenerator => {
                lines.push("§6Ranura de Combustible".into());
                lines.push("§7Coloca Carbón, Carbón Vegetal o Bloques de Carbón".into());
            }
            MachineKind::BioGenerator => {
                lines.push("§aRanura de Biomasa".into());
                lines.push("§7Coloca Savia, Savia Refinada, Hojas o Cultivos".into());
            }
            MachineKind::SteamBoiler => {
                lines.push("§bRanura de Caldera".into());
                lines.push("§7Coloca Carbón o Combustible de Alta Energía".into());
            }
            MachineKind::FluidTank => {
                lines.push("§bRanura de Balde / Fluido".into());
                lines.push("§7Coloca Balde de Agua, Lava, Savia o Balde Vacío".into());
            }
            _ => {}
        }
    } else if in_rect(at, [123.0, 33.0, 18.0, 18.0])
        && matches!(kind, MachineKind::SolarGenerator | MachineKind::EnergyBank)
    {
        if kind == MachineKind::SolarGenerator {
            lines.push("§eRanura de Recarga Solar".into());
            lines.push("§7Coloca Baterías, Jetpack, Taladro o Casco".into());
        } else {
            lines.push("§bRanura de Batería".into());
            lines.push("§7Coloca Baterías Portátiles para cargar/descargar".into());
        }
    } else if in_rect(at, [79.0, 33.0, 18.0, 18.0])
        && matches!(kind, MachineKind::ChargingPad | MachineKind::VoidTrashCan)
    {
        if kind == MachineKind::ChargingPad {
            lines.push("§bRanura de Carga por Inducción".into());
            lines.push("§7Coloca cualquier ítem eléctrico o párate encima".into());
        } else {
            lines.push("§dRanura de Destrucción".into());
            lines.push("§7Cualquier ítem colocado aquí es incinerado al instante".into());
        }
    } else if in_rect(at, [64.0, 28.0, 38.0, 26.0]) {
        if matches!(kind, MachineKind::Quarry | MachineKind::AutoHarvester) {
            let side = quarry_radius * 2 + 1;
            lines.push(format!("§eÁrea 3D activa: {side}x{side}"));
            lines.push(format!("§7Progreso del ciclo actual: {progress}%"));
        } else if matches!(kind, MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler) {
            let burn_left = get(8).max(0);
            lines.push(format!("§eCombustión restante: {burn_left} ticks ({progress}%)"));
        } else {
            lines.push(format!("§aProgreso: {progress}%"));
        }
    } else if in_rect(at, POWER_BTN_RECT)
        && !matches!(kind, MachineKind::FluidTank | MachineKind::VoidTrashCan)
    {
        lines.push("§fAlternar Encendido / Pausa de la máquina".into());
    } else if in_rect(at, MODE_BTN_RECT)
        && !matches!(
            kind,
            MachineKind::CoalGenerator
                | MachineKind::BioGenerator
                | MachineKind::SteamBoiler
                | MachineKind::SolarGenerator
                | MachineKind::ChargingPad
                | MachineKind::EnergyBank
                | MachineKind::VoidTrashCan
                | MachineKind::FluidTank
                | MachineKind::FluidPump
        )
    {
        if matches!(kind, MachineKind::Quarry | MachineKind::AutoHarvester) {
            let side = quarry_radius * 2 + 1;
            lines.push(format!("§eTamaño de Área: {side}x{side}"));
            lines.push("§7Clic para cambiar entre 5x5, 7x7 y 9x9".into());
        } else if kind == MachineKind::ConveyorBelt {
            lines.push(if auto_io {
                "§bAuto-Extraer: §aACTIVO §7(toma ítems del cofre/máquina detrás)".into()
            } else {
                "§bAuto-Extraer: §cMANUAL §7(solo transporta lo que recibe)".into()
            });
        } else {
            lines.push(if auto_io {
                "§bAuto-Expulsar: §aACTIVO §7(envía salida a Tuberías/Cintas/Cofres)".into()
            } else {
                "§bAuto-Expulsar: §cMANUAL §7(mantiene salida en la ranura)".into()
            });
        }
    } else if in_rect(at, RESET_BTN_RECT) && kind == MachineKind::Quarry {
        lines.push("§eReiniciar profundidad de la Quarry a la capa superior".into());
    }
}

pub(crate) fn commands(commands: &mut Vec<Command>) {
    commands.push(Command::op("rustyfactory kit", 2));
}

pub(crate) fn perform(
    host: &mut dyn ServerHost,
    command: &CommandInvocation,
    _source: &CommandSource,
) -> Flow {
    if command.literals != ["rustyfactory", "kit"] {
        return Flow::Continue;
    }
    for item in [
        "rusty_factory:wrench 1",
        "rusty_factory:treetap 1",
        "rusty_factory:mining_drill 1",
        "rusty_factory:jet_boots 1",
        "rusty_factory:rubber_sapling 8",
        "rusty_factory:rubber_sap_log 4",
        "rusty_factory:raw_sap 16",
        "rusty_factory:rubber_bar 16",
        "rusty_factory:sap_extractor 1",
        "rusty_factory:vulcanizer 1",
        "rusty_factory:bio_generator 1",
        "rusty_factory:quarry 1",
        "rusty_factory:auto_harvester 1",
        "rusty_factory:coal_generator 2",
        "rusty_factory:solar_generator 9",
        "rusty_factory:energy_bank 4",
        "rusty_factory:cable_white 16",
        "rusty_factory:copper_cable 16",
        "rusty_factory:cable_gold 16",
        "rusty_factory:cable_red 16",
        "rusty_factory:item_pipe 32",
        "rusty_factory:item_extractor 4",
        "rusty_factory:conveyor_belt 32",
        "rusty_factory:electric_crusher 1",
        "rusty_factory:electric_furnace 1",
        "rusty_factory:mechanical_press 1",
        "rusty_factory:industrial_assembler 1",
        "rusty_factory:block_breaker 2",
        "rusty_factory:block_placer 2",
        "rusty_factory:speed_upgrade 8",
        "rusty_factory:fortune_upgrade 8",
        "rusty_factory:energy_battery 4",
        "rusty_factory:energy_monitor 1",
        "rusty_factory:fluid_tank 2",
        "rusty_factory:steam_boiler 1",
        "rusty_factory:fluid_pump 1",
        "rusty_factory:void_trash_can 1",
        "rusty_factory:energy_relay 2",
        "rusty_factory:jetpack 1",
        "rusty_factory:item_magnet 1",
        "rusty_factory:mining_laser 1",
        "rusty_factory:charging_pad 2",
        "rusty_factory:fluid_pipe 32",
        "rusty_factory:alloy_smelter 1",
        "rusty_factory:solar_helmet 1",
    ] {
        hc::run_command(host, &format!("give @s {item}"));
    }
    hc::acting_feedback(
        host,
        "[Rusty Factory v1.0.0] ¡Kit industrial avanzado entregado (Mega Panel 3x3, Plataforma de Carga Inductiva, Tubería de Fluidos, Fundición de Aleaciones, Casco Solar y más)!",
    );
    Flow::Cancel
}

pub fn get_item_energy(stack: &ItemStack) -> i32 {
    let key = "rf_energy".into();
    hc::get_custom_data(stack)
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            Tag::Int(v) => Some(*v),
            Tag::Long(v) => Some(*v as i32),
            _ => None,
        })
        .unwrap_or(0)
        .max(0)
}

pub fn set_item_energy(stack: &mut ItemStack, energy: i32) {
    let cap = 10_000_000;
    let clamped = energy.clamp(0, cap);
    let mut map = hc::get_custom_data(stack).cloned().unwrap_or_default();
    map.insert(
        "rf_energy".into(),
        Tag::Int(clamped),
    );
    hc::set_custom_data(stack, map);
}

pub fn is_drill_3x3(stack: &ItemStack) -> bool {
    let key = "drill_3x3".into();
    hc::get_custom_data(stack)
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            Tag::Byte(v) => Some(*v != 0),
            _ => None,
        })
        .unwrap_or(true)
}

pub fn set_drill_3x3(stack: &mut ItemStack, enabled: bool) {
    let mut map = hc::get_custom_data(stack).cloned().unwrap_or_default();
    map.insert(
        "drill_3x3".into(),
        Tag::Byte(if enabled { 1 } else { 0 }),
    );
    hc::set_custom_data(stack, map);
}

pub fn consume_player_energy(host: &mut dyn ServerHost, cost: i32) -> bool {
    let mut needed = cost;
    hc::acting_held_mut(host, |slot| {
        if let Some(held) = slot.as_mut() {
            let cur = get_item_energy(held);
            if cur > 0 {
                let take = cur.min(needed);
                set_item_energy(held, cur - take);
                needed -= take;
            }
        }
    });
    if needed > 0 {
        if let Some(p) = host.players().acting_player() {
            if let Some(slots) = p.inventory_slots(host) {
                for (idx, mut st) in slots {
                    let iid = hc::item_id(host, &st);
                    if rules::is_battery_item(&iid) {
                        let cur = get_item_energy(&st);
                        if cur > 0 {
                            let take = cur.min(needed);
                            set_item_energy(&mut st, cur - take);
                            let _ = p.set_inventory_slot(host, idx, Some(st));
                            needed -= take;
                            if needed == 0 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    needed == 0
}

fn tick_player_equipment(host: &mut dyn ServerHost) {
    static LAST_EQUIP_TICK: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(-1);
    let now = hc::game_time(host);
    let prev = LAST_EQUIP_TICK.load(std::sync::atomic::Ordering::Relaxed);
    if now == prev || now % 10 != 0 {
        return;
    }
    LAST_EQUIP_TICK.store(now, std::sync::atomic::Ordering::Relaxed);

    let Some(player) = host.players().acting_player() else {
        return;
    };
    let living = LivingRef::new(player.entity());
    let eq_slots = [EquipmentSlot::Head, EquipmentSlot::Chest, EquipmentSlot::Legs, EquipmentSlot::Feet];

    // 1. Solar / Quantum Helmet passive daytime charge into worn armor, batteries & tools
    let has_solar_helmet = living.equipment(host, EquipmentSlot::Head).is_some_and(|st| {
        let id = hc::item_id(host, &st);
        id == rules::ITEM_SOLAR_HELMET || id == rules::ITEM_QUANTUM_HELMET
    });
    let is_day = (hc::day_time(host).rem_euclid(24_000)) < 12_000;
    if has_solar_helmet && is_day {
        let mut charged = false;
        for eq in eq_slots {
            if let Some(mut st) = living.equipment(host, eq) {
                let id = hc::item_id(host, &st);
                if let Some(cap) = rules::battery_capacity(&id) {
                    let cur = get_item_energy(&st);
                    if cur < cap {
                        set_item_energy(&mut st, cur + 50);
                        let _ = living.set_equipment(host, eq, Some(st));
                        charged = true;
                        break;
                    }
                }
            }
        }
        if !charged {
            if let Some(slots) = player.inventory_slots(host) {
                for (idx, mut st) in slots {
                    let id = hc::item_id(host, &st);
                    if let Some(cap) = rules::battery_capacity(&id) {
                        let cur = get_item_energy(&st);
                        if cur < cap {
                            set_item_energy(&mut st, cur + 50);
                            let _ = player.set_inventory_slot(host, idx, Some(st));
                            break;
                        }
                    }
                }
            }
        }
    }

    // 2. Portable Batteries in inventory automatically recharge worn armor & tools
    let inv_slots = player.inventory_slots(host).unwrap_or_default();
    let mut available = 0;
    for (_, st) in &inv_slots {
        let id = hc::item_id(host, st);
        if rules::is_battery_item(&id) {
            available += get_item_energy(st);
        }
    }
    if available > 0 {
        let mut used = 0;
        let budget = available.min(400);
        for eq in eq_slots {
            if used >= budget { break; }
            if let Some(mut st) = living.equipment(host, eq) {
                let id = hc::item_id(host, &st);
                if !rules::is_battery_item(&id) {
                    if let Some(cap) = rules::battery_capacity(&id) {
                        let cur = get_item_energy(&st);
                        let add = (budget - used).min(cap - cur);
                        if add > 0 {
                            set_item_energy(&mut st, cur + add);
                            let _ = living.set_equipment(host, eq, Some(st));
                            used += add;
                        }
                    }
                }
            }
        }
        for (idx, mut st) in inv_slots.clone() {
            if used >= budget { break; }
            let id = hc::item_id(host, &st);
            if !rules::is_battery_item(&id) {
                if let Some(cap) = rules::battery_capacity(&id) {
                    let cur = get_item_energy(&st);
                    let add = (budget - used).min(cap - cur);
                    if add > 0 {
                        set_item_energy(&mut st, cur + add);
                        let _ = player.set_inventory_slot(host, idx, Some(st));
                        used += add;
                    }
                }
            }
        }
        if used > 0 {
            let mut left = used;
            for (idx, mut st) in inv_slots.clone() {
                if left == 0 { break; }
                let id = hc::item_id(host, &st);
                if rules::is_battery_item(&id) {
                    let cur = get_item_energy(&st);
                    let take = cur.min(left);
                    if take > 0 {
                        set_item_energy(&mut st, cur - take);
                        let _ = player.set_inventory_slot(host, idx, Some(st));
                        left -= take;
                    }
                }
            }
        }
    }

    // 3. Worn Armor Tiers (Steel / Industrial / Quantum) + Installed Modules + Jetpack MK2 Hover
    let mut jump_lvl: i32 = -1;
    let mut speed_lvl: i32 = -1;
    let mut res_lvl: i32 = -1;
    let mut has_nv = false;
    let mut has_mag = false;
    let mut has_hover = false;
    let mut has_fire_res = false;
    for eq in eq_slots {
        if let Some(st) = living.equipment(host, eq) {
            let id = hc::item_id(host, &st);
            match id.as_str() {
                rules::ITEM_INDUSTRIAL_BOOTS => jump_lvl = jump_lvl.max(0),
                rules::ITEM_INDUSTRIAL_LEGGINGS => speed_lvl = speed_lvl.max(0),
                rules::ITEM_INDUSTRIAL_CHESTPLATE => res_lvl = res_lvl.max(0),
                rules::ITEM_INDUSTRIAL_HELMET => has_nv = true,
                rules::ITEM_QUANTUM_BOOTS => {
                    jump_lvl = jump_lvl.max(1);
                    has_hover = true;
                }
                rules::ITEM_QUANTUM_LEGGINGS => speed_lvl = speed_lvl.max(1),
                rules::ITEM_QUANTUM_CHESTPLATE => {
                    res_lvl = res_lvl.max(1);
                    has_fire_res = true;
                }
                rules::ITEM_QUANTUM_HELMET => has_nv = true,
                rules::ITEM_JETPACK_MK2 if is_jetpack_hover(&st) => has_hover = true,
                _ => {}
            }
            if has_item_module(&st, "nv") {
                has_nv = true;
            }
            if has_item_module(&st, "mag") {
                has_mag = true;
            }
            if has_item_module(&st, "speed") {
                speed_lvl = (speed_lvl + 1).max(0);
            }
        }
    }
    let has_haste = hc::acting_held(host).is_some_and(|h| has_item_module(&h, "speed"));
    let mut inv_mag_radius = 0i32;
    let inv_slots = player.inventory_slots(host).unwrap_or_default();
    for (_, st) in &inv_slots {
        let id = hc::item_id(host, st);
        if matches!(id.as_str(), rules::ITEM_ITEM_MAGNET_MK2 | rules::ITEM_ITEM_MAGNET_MK3)
            && (hc::acting_creative(host) || get_item_energy(st) > 0)
        {
            let r = if id == rules::ITEM_ITEM_MAGNET_MK3 { 28 } else { 16 };
            inv_mag_radius = inv_mag_radius.max(r);
        }
    }
    if has_hover {
        hc::run_command(host, "effect give @s minecraft:slow_falling 2 0 true");
    }
    if now % 10 == 0 && inv_mag_radius > 0 {
        hc::run_command(host, &format!("tp @e[type=item,distance=..{inv_mag_radius}] @s"));
        hc::run_command(host, &format!("tp @e[type=experience_orb,distance=..{inv_mag_radius}] @s"));
    }
    if now % 20 == 0 {
        let mut qpack_bound: Option<[i32; 3]> = None;
        for (_, st) in &inv_slots {
            let id = hc::item_id(host, st);
            if id == rules::ITEM_QUANTUM_BACKPACK
                && is_qpack_on(st)
                && (hc::acting_creative(host) || get_item_energy(st) >= 10)
            {
                if let Some(b) = get_bound_pos(st) {
                    qpack_bound = Some(b);
                    break;
                }
            }
        }
        if let Some(bound) = qpack_bound {
            for (idx, st) in inv_slots {
                if idx < 9 { continue; }
                let id = hc::item_id(host, &st);
                let is_bulk = st.count() >= 64
                    && matches!(
                        id.as_str(),
                        "minecraft:cobblestone"
                            | "minecraft:cobbled_deepslate"
                            | "minecraft:dirt"
                            | "minecraft:gravel"
                            | "minecraft:netherrack"
                            | "minecraft:andesite"
                            | "minecraft:diorite"
                            | "minecraft:granite"
                    );
                if is_bulk {
                    let rem = insert_into_silo_controller(host, bound, st);
                    let _ = player.set_inventory_slot(host, idx, rem);
                    break;
                }
            }
        }
    }
    if now % 40 == 0 {
        if jump_lvl >= 0 {
            hc::run_command(host, &format!("effect give @s minecraft:jump_boost 3 {jump_lvl} true"));
        }
        if speed_lvl >= 0 {
            hc::run_command(host, &format!("effect give @s minecraft:speed 3 {speed_lvl} true"));
        }
        if has_haste {
            hc::run_command(host, "effect give @s minecraft:haste 3 1 true");
        }
        if res_lvl >= 0 {
            hc::run_command(host, &format!("effect give @s minecraft:resistance 3 {res_lvl} true"));
        }
        if has_fire_res {
            hc::run_command(host, "effect give @s minecraft:fire_resistance 3 0 true");
        }
        if has_nv {
            hc::run_command(host, "effect give @s minecraft:night_vision 12 0 true");
        }
        if has_mag {
            hc::run_command(host, "tp @e[type=item,distance=..8] @s");
        }
    }
}

pub(crate) fn tooltip(
    host: &dyn ClientHost,
    stack: &ItemStack,
    lines: &mut Vec<(Text, u32)>,
) {
    let id_str = hc::item_id(host, stack);
    let id = id_str.as_str();
    if !id.starts_with("rusty_factory:") {
        return;
    }

    let desc: &[&str] = match id {
        rules::BLOCK_QUARRY => &[
            "§7Excava automáticamente un área de 5x5 hasta el fondo.",
            "§e• Aplana la superficie y mina con taladro 3D",
        ],
        rules::BLOCK_QUARRY_MK2 => &[
            "§7Quarry Avanzado: excava un área de 13x13 a 2x velocidad.",
            "§b• Fortuna I integrada y andamio 3D reforzado",
        ],
        rules::BLOCK_QUARRY_MK3 => &[
            "§7Quarry Cuántico: excava un área de 21x21 a 4x velocidad.",
            "§d• Fortuna II integrada y máxima eficiencia",
        ],
        rules::BLOCK_GENERATOR => &["§7Quema carbón y combustibles para generar energía (FE)."],
        rules::BLOCK_BIO_GENERATOR => &["§7Genera energía (FE) usando biocombustible y cultivos."],
        rules::BLOCK_SOLAR => &["§7Genera energía solar gratuita durante el día."],
        rules::BLOCK_STEAM_BOILER => &["§7Calienta agua para producir vapor y energía térmica."],
        rules::BLOCK_ENERGY_BANK => &["§7Almacena grandes reservas de FE y recarga baterías."],
        rules::BLOCK_ENERGY_RELAY => &["§7Transmite energía de forma inalámbrica a máquinas cercanas."],
        rules::BLOCK_ENERGY_MONITOR => &["§7Muestra el consumo y almacenamiento total de la red."],
        rules::BLOCK_CHARGING_PAD => &["§7Recarga herramientas, armaduras y baterías por inducción."],
        rules::BLOCK_CRUSHER => &["§7Tritura menas en polvo duplicando el rendimiento."],
        rules::BLOCK_FURNACE => &["§7Funde ítems rápidamente usando electricidad (FE)."],
        rules::BLOCK_ALLOY_SMELTER => &["§7Combina metales para forjar acero y aleaciones."],
        rules::BLOCK_PRESS => &["§7Prensa lingotes para fabricar placas y engranajes."],
        rules::BLOCK_VULCANIZER => &["§7Procesa savia cruda para obtener caucho industrial."],
        rules::BLOCK_ASSEMBLER => &["§7Ensambla componentes electrónicos y piezas avanzadas."],
        rules::BLOCK_HARVESTER => &["§7Cosecha y replanta cultivos automáticamente en área."],
        rules::BLOCK_BREAKER => &["§7Rompe el bloque que tiene enfrente usando energía."],
        rules::BLOCK_PLACER => &["§7Coloca bloques automáticamente frente a la máquina."],
        rules::BLOCK_SAP_EXTRACTOR => &["§7Extrae savia de troncos de caucho automáticamente."],
        rules::BLOCK_FLUID_PUMP => &["§7Bombea agua o lava hacia tuberías y tanques."],
        rules::BLOCK_FLUID_TANK => &["§7Almacena fluidos (agua, lava, savia, vapor)."],
        rules::BLOCK_TRASH_CAN => &["§7Destruye instantáneamente cualquier ítem o fluido sobrante."],
        rules::BLOCK_CONVEYOR => &["§7Transporta ítems (Tier 1: 1 ítem/s). Shift al colocar invierte."],
        rules::BLOCK_CONVEYOR_MK2 => &["§7Cinta rápida (Tier 2: lotes de 4 ítems a 2x velocidad)."],
        rules::BLOCK_CONVEYOR_MK3 => &["§7Cinta ultra rápida (Tier 3: lotes de 16 ítems a 4x velocidad)."],
        rules::ITEM_BELT_FILTER => &[
            "§7Acopla un filtro directo sobre cualquier cinta transportadora.",
            "§b• Clic derecho en cinta para acoplar y elegir ítem",
        ],
        rules::BLOCK_ITEM_SPLITTER => &[
            "§7Divide y filtra ítems en 3 salidas (Izq / Frente / Der).",
            "§b• Clic en cada cara para asignar su filtro",
        ],
        rules::BLOCK_ITEM_PIPE => &["§7Tubería de vidrio para transportar ítems (4 ítems, velocidad base)."],
        rules::BLOCK_ITEM_PIPE_MK2 => &["§eTubería de Ítems MK2: transporta hasta 16 ítems por pulso (2.5x más rápida)."],
        rules::BLOCK_ITEM_PIPE_MK3 => &["§bTubería de Ítems MK3: transporta stacks completos (64 ítems) al instante."],
        rules::BLOCK_FLUID_PIPE => &["§7Tubería de fluidos (250 mB/t, capacidad 1,000 mB)."],
        rules::BLOCK_FLUID_PIPE_MK2 => &["§eTubería de Fluidos MK2 (1,000 mB/t, capacidad 4,000 mB)."],
        rules::BLOCK_FLUID_PIPE_MK3 => &["§bTubería de Fluidos MK3 (4,000 mB/t, capacidad 16,000 mB)."],
        rules::BLOCK_EXTRACTOR => &["§7Extrae ítems de cofres/máquinas hacia tubos o cintas."],
        rules::BLOCK_CABLE_WHITE => &["§7Cable básico de energía (hasta 120 FE/t)."],
        rules::BLOCK_CABLE_COPPER => &["§7Cable de cobre (hasta 360 FE/t)."],
        rules::BLOCK_CABLE_GOLD => &["§7Cable de oro de alta tensión (hasta 1,200 FE/t)."],
        rules::BLOCK_CABLE_RED => &["§7Cable superconductor (hasta 10,000 FE/t)."],
        rules::ITEM_MINING_DRILL => &[
            "§7Taladro eléctrico capaz de excavar planos 3x3.",
            "§b• Shift + Clic Derecho: alterna modo 1x1 / 3x3",
        ],
        rules::ITEM_MINING_LASER => &["§7Dispara un haz de plasma minero que perfora bloques."],
        rules::ITEM_BATTERY => &["§7Batería portátil (50,000 FE). Recarga tu equipo en el inventario."],
        rules::ITEM_BATTERY_MK2 => &["§7Batería de alta capacidad (250,000 FE). Recarga tu equipo."],
        rules::ITEM_SOLAR_HELMET => &["§7Casco solar: recarga baterías y armadura durante el día."],
        rules::ITEM_JETPACK => &["§7Propulsión aérea sostenida para volar."],
        rules::ITEM_JET_BOOTS => &["§7Botas cohete con impulso vertical y caída lenta."],
        rules::ITEM_INDUSTRIAL_HELMET => &["§7Casco blindado con visión nocturna integrada."],
        rules::ITEM_INDUSTRIAL_CHESTPLATE => &["§7Pechera blindada con resistencia industrial."],
        rules::ITEM_INDUSTRIAL_LEGGINGS => &["§7Pantalones motorizados con aumento de velocidad."],
        rules::ITEM_INDUSTRIAL_BOOTS => &["§7Botas hidráulicas con impulso de salto."],
        rules::ITEM_STEEL_HELMET | rules::ITEM_STEEL_CHESTPLATE | rules::ITEM_STEEL_LEGGINGS | rules::ITEM_STEEL_BOOTS => &[
            "§7Armadura de Acero (Tier 1): protección robusta con 25,000 FE.",
        ],
        rules::ITEM_QUANTUM_HELMET => &["§dCasco Cuántico (Tier 3): Visión Nocturna + carga solar."],
        rules::ITEM_QUANTUM_CHESTPLATE => &["§dPechera Cuántica (Tier 3): Resistencia II + Ignífugo."],
        rules::ITEM_QUANTUM_LEGGINGS => &["§dPantalones Cuánticos (Tier 3): Velocidad II motorizada."],
        rules::ITEM_QUANTUM_BOOTS => &["§dBotas Cuánticas (Tier 3): Salto II + caída lenta."],
        rules::ITEM_ITEM_MAGNET => &["§7Atrae ítems y experiencia cercanos hacia vos."],
        rules::ITEM_WRENCH => &["§7Rota máquinas/cintas o desmantela con Shift + Clic Derecho."],
        rules::ITEM_TREETAP => &["§7Extrae savia manualmente de árboles de caucho."],
        rules::ITEM_SPEED_UPGRADE => &["§7Mejora de velocidad para máquinas (+velocidad)."],
        rules::ITEM_FORTUNE_UPGRADE => &["§7Mejora de fortuna para Quarry (+drops de menas)."],
        rules::BLOCK_UPGRADE_STATION => &["§7Instala módulos en tu armadura y taladros."],
        rules::BLOCK_GEOTHERMAL => &["§7Reactor Geotérmico: genera +120 FE/t consumiendo lava."],
        rules::BLOCK_STEAM_TURBINE => &["§7Turbina de Vapor: genera +160 FE/t con vapor de la caldera."],
        rules::BLOCK_TESSERACT => &["§7Tesseract: transmite energía e ítems a distancia sin cables."],
        rules::BLOCK_INDUSTRIAL_SILO => &["§7Silo de Alta Capacidad (2K base, ampliable hasta 500K+)."],
        rules::BLOCK_SILO_CONTROLLER => &["§7Controlador de Silos: muestra todos los ítems de silos y cofres conectados."],
        rules::BLOCK_SILO_CRAFTING_TERMINAL => &["§bTerminal de Crafteo de Silos: grilla 3x3 conectada a toda tu red de silos y cofres."],
        rules::BLOCK_NUCLEAR_REACTOR => &["§aReactor Nuclear (Tier 3): genera +1,000 FE/t con combustible nuclear."],
        rules::BLOCK_WIND_TURBINE => &["§eTurbina Eólica (Tier 2): genera más FE cuanto más alto se coloca (hasta +120 FE/t)."],
        rules::BLOCK_HYDROPONIC_CHAMBER => &["§aCámara Hidropónica (Tier 2): cultiva semillas y caucho en 1 bloque con FE."],
        rules::BLOCK_LASER_TURRET => &["§cTorreta Láser Defensiva (Tier 3): ataca mobs cercanos en 16 blq (10 daño)."],
        rules::BLOCK_WIRELESS_CHARGER => &["§bCargador Inalámbrico de Área (Tier 3): recarga tu equipo en 32 blq."],
        rules::BLOCK_TELEPORTER_PAD => &["§dPlataforma Teletransportadora (Tier 3): agachate encima para viajar a otro pad."],
        rules::ITEM_WIRELESS_SILO_TABLET => &[
            "§bControl Remoto de Silos: vincula con Shift+Clic en un Controlador.",
            "§7• Clic: ver stock o enviar mano secundaria | Shift+Clic: retirar top x64",
        ],
        rules::ITEM_QUANTUM_BACKPACK => &[
            "§dMochila Cuántica Dimensional: vincula con Shift+Clic en tu red de Silos.",
            "§7• Clic: activa/desactiva auto-envío de stacks minados a tus Silos",
        ],
        rules::BLOCK_RECYCLER => &["§7Recicla bloques sobrantes en polvo metálico y materiales."],
        rules::ITEM_MINING_DRILL_MK2 => &["§e[Tier 2] §7Taladro MK2: excava planos 3x3 con menor consumo (25 FE)."],
        rules::ITEM_MINING_DRILL_MK3 => &["§b[Tier 3] §7Taladro MK3: excava planos 5x5 a gran velocidad (15 FE)."],
        rules::ITEM_CHAINSAW => &["§7[Tier 1] Motosierra Eléctrica: tala árboles completos (64 troncos)."],
        rules::ITEM_CHAINSAW_MK2 => &["§e[Tier 2] §7Motosierra MK2: tala hasta 128 troncos y limpia hojas."],
        rules::ITEM_CHAINSAW_MK3 => &["§b[Tier 3] §7Motosierra MK3: tala hasta 256 troncos y hojas con gasto mínimo."],
        rules::ITEM_MINING_LASER_MK2 => &["§e[Tier 2] §7Láser Minero MK2: perfora 8 bloques en línea."],
        rules::ITEM_MINING_LASER_MK3 => &["§b[Tier 3] §7Láser Minero MK3: perfora 16 bloques al instante."],
        rules::ITEM_PLASMA_SWORD_MK1 => &["§7[Tier 1] Sable de Acero Energizado: descarga de 9 daño eléctrico."],
        rules::ITEM_PLASMA_SWORD_MK2 => &["§e[Tier 2] §7Sable de Plasma MK2: descarga en área de 12 daño."],
        rules::ITEM_PLASMA_SWORD_MK3 => &["§b[Tier 3] §7Sable Cuántico MK3: 16 daño + Blink (Shift + Clic Derecho)."],
        rules::ITEM_OMNI_TOOL_MK2 => &["§e[Tier 2] §7Omni-Herramienta MK2: Taladro 3x3 + Motosierra + Llave Inglesa."],
        rules::ITEM_OMNI_TOOL_MK3 => &["§b[Tier 3] §7Omni-Herramienta MK3: Taladro 5x5 + Motosierra + Llave Inglesa."],
        rules::ITEM_ITEM_MAGNET_MK2 => &["§e[Tier 2] §7Imán MK2: atrae ítems y XP en 24 bloques."],
        rules::ITEM_ITEM_MAGNET_MK3 => &["§b[Tier 3] §7Imán Cuántico MK3: atrae ítems y XP en 40 bloques."],
        rules::ITEM_BATTERY_PACK => &["§e[Tier 2] §7Mochila Dorsal (1,000,000 FE): recarga todo tu equipo."],
        rules::ITEM_QUANTUM_BATTERY => &["§b[Tier 3] §7Batería Núcleo Cuántico (5,000,000 FE): reserva máxima."],
        rules::ITEM_JETPACK_MK2 => &["§b[Tier 3] §7Jetpack MK2 con Modo Hover (Shift + Clic para flotar)."],
        rules::ITEM_ENERGY_SCANNER => &["§7Escáner portátil: clic en máquinas/tubos para ver su estado."],
        rules::ITEM_BELT_FILTER_ADV => &["§7Filtro avanzado de cinta con Lista Blanca / Lista Negra."],
        rules::ITEM_MODULE_NIGHT_VISION => &["§7Módulo: añade Visión Nocturna a tu casco."],
        rules::ITEM_MODULE_MAGNET => &["§7Módulo: atrae ítems cercanos hacia vos."],
        rules::ITEM_MODULE_EFFICIENCY => &["§7Módulo: reduce a la mitad el gasto de FE de tu taladro."],
        rules::ITEM_MODULE_AUTO_SMELT => &["§7Módulo: funde menas automáticamente al minar."],
        rules::ITEM_MODULE_FORTUNE => &["§7Módulo: aplica Fortuna eléctrica al minar menas."],
        rules::ITEM_MODULE_SILK_TOUCH => &["§7Módulo: activa Toque de Seda eléctrico en herramientas."],
        rules::ITEM_MODULE_SPEED => &["§7Módulo: overclock de velocidad en herramientas y botas."],
        rules::ITEM_NETHERITE_SEED_CORE => &["§7Núcleo precursor: fúndelo en el Horno de Aleación para obtener Netherite Scrap."],
        rules::ITEM_SILO_UPGRADE_5K => &["§7Aumenta la capacidad de un Silo Industrial en +5,000 ítems."],
        rules::ITEM_SILO_UPGRADE_10K => &["§7Aumenta la capacidad de un Silo Industrial en +10,000 ítems."],
        rules::ITEM_SILO_UPGRADE_100K => &["§7Aumenta la capacidad de un Silo Industrial en +100,000 ítems."],
        rules::ITEM_SILO_UPGRADE_500K => &["§7Mejora suprema: +500,000 ítems de capacidad en el Silo."],
        _ => &["§7Componente industrial de Rusty Factory."],
    };

    for line in desc {
        lines.push((Text::literal(*line), 0xFFAA_B4C4));
    }

    if let Some(cap) = rules::battery_capacity(id) {
        let cur = get_item_energy(stack);
        let pct = ((cur as f32 / cap.max(1) as f32) * 100.0).round() as i32;
        lines.push((
            Text::literal(format!("§b⚡ Energía: §f{cur} / {cap} FE ({pct}%)")),
            0xFF38_F2FF,
        ));
    }
    if matches!(id, rules::ITEM_MINING_DRILL | rules::ITEM_MINING_DRILL_MK2 | rules::ITEM_OMNI_TOOL_MK2) {
        let mode = if is_drill_3x3(stack) { "3x3" } else { "1x1" };
        lines.push((
            Text::literal(format!("§e⛏ Modo actual: §f{mode}")),
            0xFFFF_D038,
        ));
    } else if matches!(id, rules::ITEM_MINING_DRILL_MK3 | rules::ITEM_OMNI_TOOL_MK3) {
        let mode = if is_drill_3x3(stack) { "5x5" } else { "1x1" };
        lines.push((
            Text::literal(format!("§e⛏ Modo actual: §f{mode}")),
            0xFFFF_D038,
        ));
    } else if id == rules::ITEM_JETPACK_MK2 {
        let hover = if is_jetpack_hover(stack) { "§aON" } else { "§7OFF" };
        lines.push((
            Text::literal(format!("§b🚀 Modo Hover: {hover}")),
            0xFF38_F2FF,
        ));
    } else if matches!(id, rules::ITEM_WIRELESS_SILO_TABLET | rules::ITEM_QUANTUM_BACKPACK) {
        if let Some(b) = get_bound_pos(stack) {
            lines.push((
                Text::literal(format!("§a🔗 Vinculado a: §f[{}, {}, {}]", b[0], b[1], b[2])),
                0xFF4A_DE80,
            ));
        } else {
            lines.push((
                Text::literal("§7Sin vincular (Shift + Clic en Silo/Controlador)"),
                0xFFAA_B4C4,
            ));
        }
        if id == rules::ITEM_QUANTUM_BACKPACK {
            let st_txt = if is_qpack_on(stack) { "§aON" } else { "§7OFF" };
            lines.push((
                Text::literal(format!("§d🎒 Auto-Envío: {st_txt}")),
                0xFFD0_88FF,
            ));
        }
    }
    let mut mods = Vec::new();
    if has_item_module(stack, "nv") {
        mods.push("Visión Nocturna");
    }
    if has_item_module(stack, "mag") {
        mods.push("Imán");
    }
    if has_item_module(stack, "eff") {
        mods.push("Eficiencia");
    }
    if has_item_module(stack, "smelt") {
        mods.push("Auto-Fundición");
    }
    if has_item_module(stack, "fortune") {
        mods.push("Fortuna");
    }
    if has_item_module(stack, "silk") {
        mods.push("Toque de Seda");
    }
    if has_item_module(stack, "speed") {
        mods.push("Velocidad");
    }
    if !mods.is_empty() {
        lines.push((
            Text::literal(format!("§a✦ Módulos: §f{}", mods.join(", "))),
            0xFF4A_DE80,
        ));
    }
}

pub(crate) fn hud(
    host: &dyn ClientHost,
    draw: &mut dyn DrawContext,
    corners: &mut Corners,
) {
    

    let battery_color = |pct: f32| -> u32 {
        if pct > 50.0 {
            0xFF38_D96B // Verde (>50%)
        } else if pct >= 35.0 {
            0xFFE6_B822 // Amarillo (35%-50%)
        } else if pct >= 15.0 {
            0xFFE8_7A22 // Naranja (15%-35%)
        } else {
            0xFFE6_3232 // Rojo (<15%)
        }
    };

    // 1. Collect held tech item (tool or portable battery)
    let held_entry: Option<(String, i32, i32)> = hc::local_held(host).and_then(|st| {
        let iid = hc::item_id(host, &st);
        let cap = rules::battery_capacity(&iid)?;
        let cur = get_item_energy(&st);
        let label = match iid.as_str() {
            rules::ITEM_MINING_DRILL => {
                let m = if is_drill_3x3(&st) { "3x3" } else { "1x1" };
                format!("Taladro ({m})")
            }
            rules::ITEM_MINING_DRILL_MK2 => {
                let m = if is_drill_3x3(&st) { "3x3" } else { "1x1" };
                format!("Taladro MK2 ({m})")
            }
            rules::ITEM_MINING_DRILL_MK3 => {
                let m = if is_drill_3x3(&st) { "5x5" } else { "1x1" };
                format!("Taladro MK3 ({m})")
            }
            rules::ITEM_CHAINSAW => "Motosierra".to_owned(),
            rules::ITEM_CHAINSAW_MK2 => "Motosierra MK2".to_owned(),
            rules::ITEM_CHAINSAW_MK3 => "Motosierra MK3".to_owned(),
            rules::ITEM_MINING_LASER => "Láser Minero".to_owned(),
            rules::ITEM_MINING_LASER_MK2 => "Láser MK2".to_owned(),
            rules::ITEM_MINING_LASER_MK3 => "Láser MK3".to_owned(),
            rules::ITEM_PLASMA_SWORD_MK1 => "Sable MK1".to_owned(),
            rules::ITEM_PLASMA_SWORD_MK2 => "Sable MK2".to_owned(),
            rules::ITEM_PLASMA_SWORD_MK3 => "Sable MK3".to_owned(),
            rules::ITEM_OMNI_TOOL_MK2 => {
                let m = if is_drill_3x3(&st) { "3x3" } else { "1x1" };
                format!("Omni MK2 ({m})")
            }
            rules::ITEM_OMNI_TOOL_MK3 => {
                let m = if is_drill_3x3(&st) { "5x5" } else { "1x1" };
                format!("Omni MK3 ({m})")
            }
            rules::ITEM_ITEM_MAGNET_MK2 => "Imán MK2".to_owned(),
            rules::ITEM_ITEM_MAGNET_MK3 => "Imán MK3".to_owned(),
            rules::ITEM_BATTERY => "Batería MK1".to_owned(),
            rules::ITEM_BATTERY_MK2 => "Batería MK2".to_owned(),
            rules::ITEM_BATTERY_PACK => "Mochila Bat.".to_owned(),
            rules::ITEM_QUANTUM_BATTERY => "Núcleo Cuántico".to_owned(),
            rules::ITEM_WIRELESS_SILO_TABLET => "Tablet Silo".to_owned(),
            rules::ITEM_QUANTUM_BACKPACK => "Mochila Cuántica".to_owned(),
            rules::ITEM_JETPACK => "Jetpack".to_owned(),
            rules::ITEM_JETPACK_MK2 => "Jetpack MK2".to_owned(),
            rules::ITEM_JET_BOOTS => "Botas Cohete".to_owned(),
            rules::ITEM_SOLAR_HELMET => "Casco Solar".to_owned(),
            rules::ITEM_STEEL_HELMET => "Casco Acero".to_owned(),
            rules::ITEM_STEEL_CHESTPLATE => "Pechera Acero".to_owned(),
            rules::ITEM_STEEL_LEGGINGS => "Pantalón Acero".to_owned(),
            rules::ITEM_STEEL_BOOTS => "Botas Acero".to_owned(),
            rules::ITEM_INDUSTRIAL_HELMET => "Casco Ind.".to_owned(),
            rules::ITEM_INDUSTRIAL_CHESTPLATE => "Pechera Ind.".to_owned(),
            rules::ITEM_INDUSTRIAL_LEGGINGS => "Pantalón Ind.".to_owned(),
            rules::ITEM_INDUSTRIAL_BOOTS => "Botas Ind.".to_owned(),
            rules::ITEM_QUANTUM_HELMET => "Casco Cuántico".to_owned(),
            rules::ITEM_QUANTUM_CHESTPLATE => "Pechera Cuántica".to_owned(),
            rules::ITEM_QUANTUM_LEGGINGS => "Pantalón Cuántico".to_owned(),
            rules::ITEM_QUANTUM_BOOTS => "Botas Cuánticas".to_owned(),
            _ => "Equipo Tech".to_owned(),
        };
        Some((label, cur, cap))
    });

    // 2. Collect per-piece equipped tech armor from `local_armor_slot` (0=Head, 1=Chest, 2=Legs, 3=Feet)
    let read_armor_slot = |slot_idx: usize| -> Option<(&'static str, i32, i32)> {
        let st = hc::local_armor_slot(host, slot_idx)?;
        let iid = hc::item_id(host, &st);
        let cap = rules::battery_capacity(&iid)?;
        let cur = get_item_energy(&st);
        let short_name = match iid.as_str() {
            rules::ITEM_SOLAR_HELMET => "Solar",
            rules::ITEM_STEEL_HELMET => "Acero",
            rules::ITEM_INDUSTRIAL_HELMET => "Casco",
            rules::ITEM_QUANTUM_HELMET => "Cuánt.",
            rules::ITEM_JETPACK => "Jetpack",
            rules::ITEM_JETPACK_MK2 => "Jet MK2",
            rules::ITEM_STEEL_CHESTPLATE => "Acero",
            rules::ITEM_INDUSTRIAL_CHESTPLATE => "Pecho",
            rules::ITEM_QUANTUM_CHESTPLATE => "Cuánt.",
            rules::ITEM_STEEL_LEGGINGS => "Acero",
            rules::ITEM_INDUSTRIAL_LEGGINGS => "Piernas",
            rules::ITEM_QUANTUM_LEGGINGS => "Cuánt.",
            rules::ITEM_JET_BOOTS => "Cohete",
            rules::ITEM_STEEL_BOOTS => "Acero",
            rules::ITEM_INDUSTRIAL_BOOTS => "Botas",
            rules::ITEM_QUANTUM_BOOTS => "Cuánt.",
            _ => "Tech",
        };
        Some((short_name, cur, cap))
    };

    let head = read_armor_slot(0);
    let chest = read_armor_slot(1);
    let legs = read_armor_slot(2);
    let feet = read_armor_slot(3);
    let has_any_armor = head.is_some() || chest.is_some() || legs.is_some() || feet.is_some();

    // 3. Collect reserve portable batteries in inventory
    let mut bat_cur: i32 = 0;
    let mut bat_cap: i32 = 0;
    for st in hc::local_inventory(host).into_iter().take(36) {
        let iid = hc::item_id(host, &st);
        if rules::is_battery_item(&iid) {
            bat_cur += get_item_energy(&st);
            bat_cap += rules::battery_capacity(&iid).unwrap_or(50_000);
        }
    }

    // 0. Look-At Block Info Card (Silo capacity bar Green/Yellow/Red, Geothermal/Turbine/Tesseract/UpgradeStation info)
    if let Some(player) = host.local_player() {
        let (yaw, pitch) = hc::local_yaw_pitch(host);
        let yaw_rad = (yaw as f64).to_radians();
        let pitch_rad = (pitch as f64).to_radians();
        let dir = [
            -yaw_rad.sin() * pitch_rad.cos(),
            -pitch_rad.sin(),
            yaw_rad.cos() * pitch_rad.cos(),
        ];
        let ppos = player.entity().position(host).unwrap_or([0.0, 0.0, 0.0]);
        let eye = [ppos[0], ppos[1] + 1.62, ppos[2]];
        let mut hit_pos: Option<[i32; 3]> = None;
        for step in 1..=100 {
            let t = step as f64 * 0.05;
            let cell = [
                (eye[0] + dir[0] * t).floor() as i32,
                (eye[1] + dir[1] * t).floor() as i32,
                (eye[2] + dir[2] * t).floor() as i32,
            ];
            let blk = hc::block_id_at(host, cell);
            if !matches!(blk.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") {
                if blk.starts_with("rusty_factory:") {
                    hit_pos = Some(cell);
                }
                break;
            }
        }
        if let Some(hp) = hit_pos {
            let hblk = hc::block_id_at(host, hp);
            let mut info_title = String::new();
            let mut info_sub = String::new();
            let mut bar_ratio: Option<(f32, u32)> = None;

            if let Some(st) = hc::read_machine_state(host, hp) {
                {
                    if hblk == rules::BLOCK_INDUSTRIAL_SILO {
                        let cap = st.silo_capacity.max(2_048);
                        let fmt_k = |n: u32| if n >= 1000 { format!("{:.1}K", n as f32 / 1000.0).replace(".0K", "K") } else { format!("{n}") };
                        let item_short = if st.belt_item.is_empty() {
                            "Vacío".to_owned()
                        } else {
                            st.belt_item.split(':').nth(1).unwrap_or(&st.belt_item).to_owned()
                        };
                        let r = (st.belt_count as f32 / cap as f32).clamp(0.0, 1.0);
                        let bcol = if r >= 0.90 {
                            0xFFE6_3232 // Rojo: lleno
                        } else if r >= 0.65 {
                            0xFFE6_B822 // Amarillo: llenándose
                        } else {
                            0xFF38_D96B // Verde: libre
                        };
                        info_title = format!("📦 Silo: {item_short}");
                        info_sub = format!("{} / {}", fmt_k(st.belt_count), fmt_k(cap));
                        bar_ratio = Some((r, bcol));
                    } else if hblk == rules::BLOCK_UPGRADE_STATION {
                        info_title = "🔧 Estación de Mejora".to_owned();
                        info_sub = if st.belt_item.is_empty() {
                            "Clic con Módulo para cargar".to_owned()
                        } else {
                            format!("Módulo listo: {}", st.belt_item)
                        };
                    } else if let Some(kind) = MachineKind::from_block_id(&hblk) {
                        if kind.max_energy() > 0 {
                            let r = (st.energy as f32 / kind.max_energy() as f32).clamp(0.0, 1.0);
                            info_title = hblk.split(':').nth(1).unwrap_or(&hblk).replace('_', " ");
                            info_sub = format!("⚡ {} / {} FE", st.energy, kind.max_energy());
                            bar_ratio = Some((r, 0xFF38_F2FF));
                        }
                    }
                }
            }

            if !info_title.is_empty() {
                let iw = 152.0_f32;
                let ih = if bar_ratio.is_some() { 38.0 } else { 30.0 };
                let (ix, iy) = corners.place(Corner::BottomRight, iw, ih);
                let border_c = bar_ratio.map_or(0xFF38_F2FF, |(_, c)| c);
                draw.fill( [ix, iy, iw, ih], 0xBE10_141C);
                draw.fill( [ix, iy, iw, 1.5], border_c);
                draw.fill( [ix, iy + ih - 1.5, iw, 1.5], 0xFF28_5066);
                draw.fill( [ix, iy, 1.5, ih], border_c);
                draw.fill( [ix + iw - 1.5, iy, 1.5, ih], 0xFF28_5066);
                draw.text( &info_title, [ix + 6.0, iy + 5.0], 0xFFE8_EEF8, true);
                draw.text( &info_sub, [ix + 6.0, iy + 17.0], border_c, true);
                if let Some((r, bcol)) = bar_ratio {
                    draw.fill( [ix + 6.0, iy + 29.0, iw - 12.0, 4.0], 0xFF22_2834);
                    let fw = ((iw - 12.0) * r).clamp(0.0, iw - 12.0);
                    if fw > 0.5 {
                        draw.fill( [ix + 6.0, iy + 29.0, fw, 4.0], bcol);
                    }
                }
            }
        }
    }

    if held_entry.is_none() && !has_any_armor {
        return;
    }

    let mut extra_rows = 0usize;
    if held_entry.is_some() {
        extra_rows += 1;
    }
    if bat_cap > 0 {
        extra_rows += 1;
    }

    let armor_block_h = if has_any_armor { 58.0 } else { 0.0 };
    let w = 152.0_f32;
    let h = 22.0 + armor_block_h + extra_rows as f32 * 18.0;
    let (x, y) = corners.place(Corner::BottomRight, w, h);

    // Dark industrial background + cyan border
    draw.fill( [x, y, w, h], 0xBE10_141C);
    draw.fill( [x, y, w, 1.5], 0xFF38_F2FF);
    draw.fill( [x, y + h - 1.5, w, 1.5], 0xFF28_5066);
    draw.fill( [x, y, 1.5, h], 0xFF38_F2FF);
    draw.fill( [x + w - 1.5, y, 1.5, h], 0xFF28_5066);

    draw.text( "⚡ ESTADO TECH", [x + 6.0, y + 5.0], 0xFF38_F2FF, true);

    let mut ry = y + 19.0;

    // Draw 2D Character Silhouette (thick black outer frame + colored armor segments + % readout)
    if has_any_armor {
        let fx = x + 8.0;
        let fy = ry;
        let tx = x + 46.0;
        let empty_col = 0xFF1A_1E24;
        let sil_black = 0xFF0A_0C10;

        let piece_style = |info: Option<(&'static str, i32, i32)>| -> (u32, String) {
            if let Some((label, cur, cap)) = info {
                let pct = ((cur as f32 / cap.max(1) as f32) * 100.0).clamp(0.0, 100.0);
                (battery_color(pct), format!("{label}: {}%", pct as i32))
            } else {
                (empty_col, "---".to_owned())
            }
        };

        let (head_col, head_txt) = piece_style(head);
        let (chest_col, chest_txt) = piece_style(chest);
        let (legs_col, legs_txt) = piece_style(legs);
        let (feet_col, feet_txt) = piece_style(feet);

        // Thick black outer silhouette backing (like reference figure)
        draw.fill( [fx + 7.0, fy, 16.0, 12.0], sil_black);       // Head silhouette
        draw.fill( [fx, fy + 11.0, 30.0, 18.0], sil_black);     // Torso + Arms silhouette
        draw.fill( [fx + 6.0, fy + 29.0, 18.0, 23.0], sil_black); // Legs + Boots silhouette

        let draw_seg = |b: &mut dyn DrawContext, px: f32, py: f32, pw: f32, ph: f32, col: u32, lit: bool| {
            b.fill( [px, py, pw, ph], col);
            if lit {
                b.fill( [px, py, pw, 1.0], 0x44FF_FFFF);
            }
        };

        // Head
        draw_seg(draw, fx + 9.0, fy + 2.0, 12.0, 9.0, head_col, head.is_some());
        draw.text( &head_txt, [tx, fy + 2.0], if head.is_some() { head_col } else { 0xFF55_6070 }, true);

        // Chest + Arms
        draw_seg(draw, fx + 2.0, fy + 13.0, 4.0, 14.0, chest_col, chest.is_some());
        draw_seg(draw, fx + 8.0, fy + 13.0, 14.0, 14.0, chest_col, chest.is_some());
        draw_seg(draw, fx + 24.0, fy + 13.0, 4.0, 14.0, chest_col, chest.is_some());
        draw.text( &chest_txt, [tx, fy + 16.0], if chest.is_some() { chest_col } else { 0xFF55_6070 }, true);

        // Legs
        draw_seg(draw, fx + 8.0, fy + 29.0, 6.0, 13.0, legs_col, legs.is_some());
        draw_seg(draw, fx + 16.0, fy + 29.0, 6.0, 13.0, legs_col, legs.is_some());
        draw.text( &legs_txt, [tx, fy + 31.0], if legs.is_some() { legs_col } else { 0xFF55_6070 }, true);

        // Boots
        draw_seg(draw, fx + 8.0, fy + 44.0, 6.0, 6.0, feet_col, feet.is_some());
        draw_seg(draw, fx + 16.0, fy + 44.0, 6.0, 6.0, feet_col, feet.is_some());
        draw.text( &feet_txt, [tx, fy + 43.0], if feet.is_some() { feet_col } else { 0xFF55_6070 }, true);

        ry += armor_block_h;
    }

    let draw_bar = |b: &mut dyn DrawContext, label: &str, cur: i32, cap: i32, row_y: f32| {
        let pct = ((cur as f32 / cap.max(1) as f32) * 100.0).clamp(0.0, 100.0);
        let col = battery_color(pct);
        b.text(
            &format!("{label}: {}%", pct as i32),
            [x + 6.0, row_y],
            col,
            true,
        );
        b.fill( [x + 6.0, row_y + 10.0, w - 12.0, 4.0], 0xFF22_2834);
        let fill_w = ((w - 12.0) * (pct / 100.0)).clamp(0.0, w - 12.0);
        if fill_w > 0.5 {
            b.fill( [x + 6.0, row_y + 10.0, fill_w, 4.0], col);
        }
    };

    if let Some((ref label, cur, cap)) = held_entry {
        draw_bar(draw, label, cur, cap, ry);
        ry += 18.0;
    }
    if bat_cap > 0 {
        draw_bar(draw, "Baterías", bat_cur, bat_cap, ry);
    }
}

fn belt_filter_matches(filter: &str, item: &str, blacklist: bool) -> bool {
    let in_list = filter
        .split(',')
        .any(|f| !f.trim().is_empty() && f.trim() == item);
    if blacklist {
        !in_list
    } else {
        in_list
    }
}

fn has_item_module(stack: &ItemStack, code: &str) -> bool {
    let key = "rf_modules".into();
    hc::get_custom_data(stack)
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            Tag::String(s) => s.to_string().ok(),
            _ => None,
        })
        .is_some_and(|s| s.split(',').any(|m| m == code))
}

fn add_item_module(stack: &mut ItemStack, code: &str) {
    let key = "rf_modules".into();
    let mut map = hc::get_custom_data(stack).cloned().unwrap_or_default();
    let existing = map
        .get(&key)
        .and_then(|t| match t {
            Tag::String(s) => s.to_string().ok(),
            _ => None,
        })
        .unwrap_or_default();
    if !existing.split(',').any(|m| m == code) {
        let next = if existing.is_empty() {
            code.to_owned()
        } else {
            format!("{existing},{code}")
        };
        map.insert(
            key,
            Tag::String(next.as_str().into()),
        );
        hc::set_custom_data(stack, map);
    }
}

fn is_jetpack_hover(stack: &ItemStack) -> bool {
    let key = "hover".into();
    hc::get_custom_data(stack)
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            Tag::Byte(v) => Some(*v != 0),
            _ => None,
        })
        .unwrap_or(false)
}

fn set_jetpack_hover(stack: &mut ItemStack, hover: bool) {
    let mut map = hc::get_custom_data(stack).cloned().unwrap_or_default();
    map.insert(
        "hover".into(),
        Tag::Byte(if hover { 1 } else { 0 }),
    );
    hc::set_custom_data(stack, map);
}

fn get_bound_pos(stack: &ItemStack) -> Option<[i32; 3]> {
    let c = hc::get_custom_data(stack)?;
    let read_i = |k: &str| match c.get(&k.into()) {
        Some(Tag::Int(v)) => Some(*v),
        _ => None,
    };
    Some([read_i("bind_x")?, read_i("bind_y")?, read_i("bind_z")?])
}

fn set_bound_pos(stack: &mut ItemStack, pos: [i32; 3]) {
    let mut map = hc::get_custom_data(stack).cloned().unwrap_or_default();
    map.insert("bind_x".into(), Tag::Int(pos[0]));
    map.insert("bind_y".into(), Tag::Int(pos[1]));
    map.insert("bind_z".into(), Tag::Int(pos[2]));
    hc::set_custom_data(stack, map);
}

fn is_qpack_on(stack: &ItemStack) -> bool {
    let key = "qpack_on".into();
    hc::get_custom_data(stack)
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            Tag::Byte(v) => Some(*v != 0),
            _ => None,
        })
        .unwrap_or(true)
}

fn set_qpack_on(stack: &mut ItemStack, on: bool) {
    let mut map = hc::get_custom_data(stack).cloned().unwrap_or_default();
    map.insert(
        "qpack_on".into(),
        Tag::Byte(if on { 1 } else { 0 }),
    );
    hc::set_custom_data(stack, map);
}
