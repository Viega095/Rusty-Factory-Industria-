// Horizon engine glue for Rusty Factory (`crate::game::rusty_factory_glue`).
// Connects Horizon's native block entity hooks, 3D rendering, world primitives,
// container menus, screens, and player interactions to `runtime.rs`.

use crate::game::{
    Game,
    native_block_entity_draw::BlockEntityDraw,
    rusty_factory as rules,
};
use horizon_assets::item_model::DisplayContext;
use horizon_client::{
    containers::{BlockEntity, ModBlockEntity, ModContainer},
    hud::HudBatch,
    items::ItemStack,
    lang,
    mod_api::{NativeCommand, NativeMod, NativeScreen},
    text::Text,
    world_primitives::WorldPrimitives,
};
use horizon_core::geometry::Direction;
use horizon_world::{
    nbt::{Compound, JavaString, Tag},
    registry::BlockStateId,
};
use rules::{MachineKind, MachineState, MachineTier, StepStatus};

const STATE_TAG: &str = "rusty_machine";
const BTN_TOGGLE_POWER: i32 = 0;
const BTN_MODE_OR_SIZE: i32 = 1;
const BTN_QUARRY_RESET: i32 = 2;

const POWER_BTN_RECT: [f32; 4] = [136.0, 18.0, 34.0, 14.0];
const MODE_BTN_RECT: [f32; 4] = [136.0, 62.0, 34.0, 14.0];
const RESET_BTN_RECT: [f32; 4] = [98.0, 18.0, 34.0, 14.0];

fn state_key() -> JavaString {
    JavaString::from(STATE_TAG)
}

pub(in crate::game) fn load_state(entity: &ModBlockEntity) -> Option<MachineState> {
    match entity.nbt.get(&state_key()) {
        Some(Tag::ByteArray(bytes)) => serde_json::from_slice(bytes).ok(),
        _ => None,
    }
}

pub(in crate::game) fn store_state(entity: &mut ModBlockEntity, state: &MachineState) {
    if let Ok(bytes) = serde_json::to_vec(state) {
        entity.nbt.insert(state_key(), Tag::ByteArray(bytes));
    }
}

pub(in crate::game) fn listed(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: rules::ID.into(),
        title: format!("Rusty Factory {}", rules::VERSION),
        description: lang::get_or(
            "rusty_factory.mod.description",
            "Industrial automation: 3D Quarry, Item Pipes, Copper Energy Cables, 3D Animated Conveyor Belts, Crusher, Furnace & Press.",
        ),
        keys: Vec::new(),
    });
}

pub(in crate::game) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    let is_tree_tick_block = matches!(block, rules::BLOCK_RUBBER_SAP_LOG | rules::BLOCK_RUBBER_SAPLING);
    if (MachineKind::from_block_id(block).is_some() || is_tree_tick_block) && entity.is_none() {
        let mut nbt = Compound::default();
        if let Some(kind) = MachineKind::from_block_id(block) {
            if let Ok(bytes) = serde_json::to_vec(&MachineState::new(kind)) {
                nbt.insert(state_key(), Tag::ByteArray(bytes));
            }
        }
        *entity = Some((block.to_owned(), nbt));
    }
}

pub(in crate::game) fn block_entity_ticks(block: &str, ticks: &mut bool) {
    if MachineKind::from_block_id(block).is_some()
        || matches!(block, rules::BLOCK_RUBBER_SAP_LOG | rules::BLOCK_RUBBER_SAPLING)
    {
        *ticks = true;
    }
}

pub(in crate::game) fn update_tag(
    _pos: [i32; 3],
    entity: &ModBlockEntity,
    tag: &mut Option<Compound>,
) {
    if MachineKind::from_block_id(&entity.id).is_some() {
        *tag = Some(entity.nbt.clone());
    }
}

pub(in crate::game) fn container(block: &str, container: &mut Option<ModContainer>) {
    if let Some(kind) = MachineKind::from_block_id(block) {
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
            | MachineKind::SiloController => {
                *container = Some(ModContainer {
                    slots: 1,
                    faces: std::array::from_fn(|_| vec![0]),
                    inputs: vec![0],
                    outputs: vec![0],
                    menu: Some(rules::MENU_SINGLE.to_owned()),
                });
            }
            MachineKind::EnergyMonitor
            | MachineKind::EnergyRelay
            | MachineKind::ConveyorBelt
            | MachineKind::ItemPipe
            | MachineKind::FluidPipe
            | MachineKind::ItemSplitter
            | MachineKind::UpgradeStation => {
                *container = None;
            }
            MachineKind::EnergyBank => {
                *container = Some(ModContainer {
                    slots: 1,
                    faces: std::array::from_fn(|_| vec![0]),
                    inputs: vec![0],
                    outputs: vec![0],
                    menu: Some(rules::MENU_SINGLE.to_owned()),
                });
            }
            _ => {
                *container = Some(ModContainer {
                    slots: rules::SLOTS,
                    faces: std::array::from_fn(|_| {
                        vec![rules::SLOT_INPUT, rules::SLOT_FUEL, rules::SLOT_OUTPUT]
                    }),
                    inputs: vec![rules::SLOT_INPUT, rules::SLOT_FUEL],
                    outputs: vec![rules::SLOT_OUTPUT],
                    menu: Some(rules::MENU.to_owned()),
                });
            }
        }
    }
}

fn fuel_burn_ticks(stack: &ItemStack) -> Option<i32> {
    match &*stack.item {
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
fn grow_rubber_tree(game: &mut Game<'_>, pos: [i32; 3]) {
    let dim = game.native_dimension();
    let seed = game.world.game_time ^ ((pos[0] as i64).wrapping_mul(31) ^ (pos[2] as i64));
    for (p, block_id, opt_prop) in rules::rubber_tree_blocks(pos, seed) {
        let cur = game.native_block(p);
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
        if let Some(st) = game.native_default_state(block_id) {
            game.native_set_block(dim, p, st, 3);
            if let Some((k, v)) = opt_prop {
                game.native_set_block_property(p, k, v);
            }
            if block_id == rules::BLOCK_RUBBER_SAP_LOG {
                game.native_set_block_property(p, "lit", "true");
            }
        }
    }
}

/// Scans all connected `industrial_silo`s / `silo_controller`s and adjacent chests/barrels.
fn collect_silo_network(game: &Game<'_>, start: [i32; 3]) -> (Vec<[i32; 3]>, Vec<[i32; 3]>) {
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
            let b = game.native_block(nb);
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
fn aggregate_silo_network(game: &Game<'_>, start: [i32; 3]) -> (Vec<(String, u32, bool)>, usize, usize) {
    let (silos, chests) = collect_silo_network(game, start);
    let mut map: std::collections::BTreeMap<String, (u32, bool)> = std::collections::BTreeMap::new();

    for sp in &silos {
        if let Some(BlockEntity::Mod(ent)) = game.world.block_entities.get(&(sp[0], sp[1], sp[2])) {
            if let Some(st) = load_state(ent) {
                if !st.belt_item.is_empty() && st.belt_count > 0 {
                    let e = map.entry(st.belt_item).or_insert((0, true));
                    e.0 = e.0.saturating_add(st.belt_count);
                    e.1 = true;
                }
            }
        }
    }
    for cp in &chests {
        if let Some(BlockEntity::Chest(slots)) = game.world.block_entities.get(&(cp[0], cp[1], cp[2])) {
            for slot in slots.iter().flatten() {
                let e = map.entry(slot.item.to_string()).or_insert((0, false));
                e.0 = e.0.saturating_add(slot.count);
            }
        }
    }
    let mut list: Vec<(String, u32, bool)> = map.into_iter().map(|(k, (v, s))| (k, v, s)).collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    (list, silos.len(), chests.len())
}

/// Routes `stack` through a `silo_controller` into matching Silos, then empty Silos, then connected Chests.
fn insert_into_silo_controller(game: &mut Game<'_>, pos: [i32; 3], mut stack: ItemStack) -> Option<ItemStack> {
    let (silos, chests) = collect_silo_network(game, pos);
    // 1. Silos already holding this exact item
    for &sp in &silos {
        let matches_item = game
            .world
            .block_entities
            .get(&(sp[0], sp[1], sp[2]))
            .and_then(|be| if let BlockEntity::Mod(e) = be { load_state(e) } else { None })
            .is_some_and(|st| st.belt_item == stack.item.as_ref());
        if matches_item {
            match insert_into_silo(game, sp, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        }
    }
    // 2. Empty Silos
    for &sp in &silos {
        let is_empty = game
            .world
            .block_entities
            .get(&(sp[0], sp[1], sp[2]))
            .and_then(|be| if let BlockEntity::Mod(e) = be { load_state(e) } else { None })
            .is_some_and(|st| st.belt_item.is_empty());
        if is_empty {
            match insert_into_silo(game, sp, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        }
    }
    // 3. Connected Chests
    for &cp in &chests {
        match game.native_insert_item(cp, Direction::Up, stack) {
            None => return None,
            Some(rest) => stack = rest,
        }
    }
    Some(stack)
}

/// Inserts `stack` into an `industrial_silo` ONLY if it is empty or already holds that exact item type.
fn insert_into_silo(game: &mut Game<'_>, pos: [i32; 3], stack: ItemStack) -> Option<ItemStack> {
    if game.native_block(pos) != rules::BLOCK_INDUSTRIAL_SILO {
        return Some(stack);
    }
    let item_id = stack.item.to_string();
    let mut rem = stack.count;
    let changed = game.native_with_mod_block_entity(pos, |entity| {
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
        game.native_block_entity_sync(pos);
    }
    if rem == 0 {
        None
    } else {
        Some(ItemStack::new(&stack.item, rem))
    }
}

/// Inserts `stack` onto a conveyor belt's `MachineState` if it is empty or can batch more of the same item.
/// Returns `None` if all items were accepted, or `Some(leftover)`.
fn insert_into_conveyor(game: &mut Game<'_>, pos: [i32; 3], stack: ItemStack) -> Option<ItemStack> {
    let block = game.native_block(pos);
    let Some((_, max_batch)) = rules::conveyor_tier(&block) else {
        return Some(stack);
    };
    let mut accepted = 0u32;
    let changed = game.native_with_mod_block_entity(pos, |entity| {
        let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ConveyorBelt));
        if !st.enabled {
            return false;
        }
        let item_id = stack.item.to_string();
        if st.belt_item.is_empty() {
            let take = stack.count.min(max_batch);
            st.belt_item = item_id;
            st.belt_count = take;
            st.progress = 0;
            accepted = take;
            store_state(entity, &st);
            true
        } else if st.belt_item == item_id && st.belt_count < max_batch {
            let space = max_batch - st.belt_count;
            let take = stack.count.min(space);
            st.belt_count += take;
            accepted = take;
            store_state(entity, &st);
            true
        } else {
            false
        }
    });
    if changed == Some(true) && accepted > 0 {
        game.native_block_entity_sync(pos);
        if stack.count > accepted {
            let mut rest = stack;
            rest.count -= accepted;
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
fn insert_into_splitter(game: &mut Game<'_>, pos: [i32; 3], stack: ItemStack) -> Option<ItemStack> {
    if game.native_block(pos) != rules::BLOCK_ITEM_SPLITTER {
        return Some(stack);
    }
    let facing = game
        .native_block_property(pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let out_offsets = [
        rules::conveyor_exit_offset(&facing, "turn_left"),
        rules::conveyor_exit_offset(&facing, "straight"),
        rules::conveyor_exit_offset(&facing, "turn_right"),
    ];
    let mut start_idx = 0usize;
    let mut filters = [String::new(), String::new(), String::new()];
    game.native_with_mod_block_entity(pos, |entity| {
        if let Some(st) = load_state(entity) {
            start_idx = (st.progress as usize) % 3;
            filters = st.splitter_filters;
        }
    });

    let item_id = stack.item.to_string();
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
        let tb = game.native_block(target);
        let is_air = matches!(tb.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air");
        let leftover = if rules::is_conveyor_block(&tb) {
            insert_into_conveyor(game, target, stack.clone())
        } else if tb == rules::BLOCK_ITEM_PIPE {
            insert_into_pipe(game, pos, target, stack.clone(), None)
        } else if tb == rules::BLOCK_INDUSTRIAL_SILO {
            insert_into_silo(game, target, stack.clone())
        } else if tb == rules::BLOCK_SILO_CONTROLLER {
            insert_into_silo_controller(game, target, stack.clone())
        } else if is_item_container_block(&tb) {
            match game.native_insert_item(target, Direction::Up, stack.clone()) {
                None => None,
                Some(r) => game.native_insert_item(target, Direction::North, r),
            }
        } else if is_air {
            let target_below = [target[0], target[1] - 1, target[2]];
            if rules::is_conveyor_block(&game.native_block(target_below)) {
                insert_into_conveyor(game, target_below, stack.clone())
            } else if has_matching_filter {
                let drop_pos = [
                    pos[0] as f64 + 0.5 + off[0] as f64 * 0.65,
                    pos[1] as f64 + 0.30,
                    pos[2] as f64 + 0.5 + off[2] as f64 * 0.65,
                ];
                let vel = [off[0] as f64 * 0.12, 0.06, off[2] as f64 * 0.12];
                game.spawn_item(stack.clone(), drop_pos, vel, 10);
                None
            } else {
                Some(stack.clone())
            }
        } else {
            Some(stack.clone())
        };
        if leftover.as_ref().map_or(0, |r| r.count) < stack.count {
            let next_idx = ((idx + 1) % 3) as i32;
            game.native_with_mod_block_entity(pos, |entity| {
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
    game: &mut Game<'_>,
    from_pos: [i32; 3],
    pipe_pos: [i32; 3],
    stack: ItemStack,
    target_pos: Option<[i32; 3]>,
) -> Option<ItemStack> {
    if game.native_block(pipe_pos) != rules::BLOCK_ITEM_PIPE {
        return Some(stack);
    }
    let from_off = [from_pos[0] - pipe_pos[0], from_pos[1] - pipe_pos[1], from_pos[2] - pipe_pos[2]];
    let from_dir = rules::dir_index_from_offset(from_off);
    let next_step = rules::find_next_pipe_step(
        pipe_pos,
        target_pos,
        |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE && p != from_pos,
        |p| p != from_pos && (is_item_container_block(&game.native_block(p)) || rules::is_conveyor_block(&game.native_block(p))),
    );
    let (to_dir, final_target) = if let Some((step, tgt)) = next_step {
        let to_off = [step[0] - pipe_pos[0], step[1] - pipe_pos[1], step[2] - pipe_pos[2]];
        (rules::dir_index_from_offset(to_off), Some(tgt))
    } else {
        (rules::opposite_dir_index(from_dir), target_pos)
    };

    let inserted = game.native_with_mod_block_entity(pipe_pos, |entity| {
        let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ItemPipe));
        if !st.belt_item.is_empty() {
            return false;
        }
        st.belt_item = stack.item.to_string();
        st.belt_count = stack.count;
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
        game.native_block_entity_sync(pipe_pos);
        None
    } else {
        Some(stack)
    }
}

/// Routes `stack` from `source_pos` into connected `rusty_factory:item_pipe` networks (step-by-step visible travel)
/// or directly into adjacent containers / conveyor belts.
/// Returns the remaining stack (`None` if all items were inserted).
fn route_item_from(game: &mut Game<'_>, source_pos: [i32; 3], mut stack: ItemStack) -> Option<ItemStack> {
    // 1. Try direct adjacent containers first (except conveyors)
    for d in rules::DIRS {
        let adj = [source_pos[0] + d[0], source_pos[1] + d[1], source_pos[2] + d[2]];
        let blk = game.native_block(adj);
        if blk == rules::BLOCK_INDUSTRIAL_SILO {
            match insert_into_silo(game, adj, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        } else if blk == rules::BLOCK_SILO_CONTROLLER {
            match insert_into_silo_controller(game, adj, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        } else if !rules::is_conveyor_block(&blk) && is_item_container_block(&blk) {
            match game.native_insert_item(adj, Direction::Up, stack) {
                None => return None,
                Some(rest) => {
                    stack = match game.native_insert_item(adj, Direction::North, rest) {
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
        |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
        |p| p != source_pos && (is_item_container_block(&game.native_block(p)) || rules::is_conveyor_block(&game.native_block(p)) || game.native_block(p) == rules::BLOCK_ITEM_SPLITTER),
    );
    if !targets.is_empty() {
        for d in rules::DIRS {
            let adj = [source_pos[0] + d[0], source_pos[1] + d[1], source_pos[2] + d[2]];
            if game.native_block(adj) == rules::BLOCK_ITEM_PIPE {
                if insert_into_pipe(game, source_pos, adj, stack.clone(), None).is_none() {
                    return None;
                }
            }
        }
    }

    // 3. Try adjacent conveyor belts
    for d in rules::DIRS {
        let adj = [source_pos[0] + d[0], source_pos[1] + d[1], source_pos[2] + d[2]];
        if rules::is_conveyor_block(&game.native_block(adj)) {
            match insert_into_conveyor(game, adj, stack) {
                None => return None,
                Some(rest) => stack = rest,
            }
        }
    }

    Some(stack)
}

/// Auto-ejects up to `max_count` items from `entity.items[SLOT_OUTPUT]` into connected pipes/belts/chests.
fn try_auto_eject_output(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    max_count: u32,
) {
    let Some(out_stack) = entity.items.get(rules::SLOT_OUTPUT).cloned().flatten() else {
        return;
    };
    let send_count = out_stack.count.min(max_count).max(1);
    let to_send = ItemStack {
        count: send_count,
        ..out_stack.clone()
    };
    let leftover = route_item_from(game, pos, to_send);
    let sent = send_count - leftover.as_ref().map_or(0, |r| r.count);
    if sent > 0 {
        if let Some(Some(slot_ref)) = entity.items.get_mut(rules::SLOT_OUTPUT) {
            if slot_ref.count > sent {
                slot_ref.count -= sent;
            } else {
                entity.items[rules::SLOT_OUTPUT] = None;
            }
        }
    }
}

/// Distributes FE from a Generator, Solar Panel, or Energy Bank across connected copper cables and adjacent machines.
/// If `source_is_bank` is true, it will not send energy into another Energy Bank (preventing loops).
fn distribute_energy(
    game: &mut Game<'_>,
    gen_pos: [i32; 3],
    available: i32,
    source_is_bank: bool,
) -> i32 {
    if available <= 0 {
        return 0;
    }
    let targets = rules::find_connected_consumers(
        gen_pos,
        |p| rules::is_cable(&game.native_block(p)),
        |p| {
            let Some(k) = MachineKind::from_block_id(&game.native_block(p)) else {
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
        let Some(kind) = MachineKind::from_block_id(&game.native_block(target_pos)) else {
            continue;
        };
        let mut transferred = 0;
        game.native_with_mod_block_entity(target_pos, |target_entity| {
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
fn update_lit_state(game: &mut Game<'_>, pos: [i32; 3], working: bool) {
    let want = if working { "true" } else { "false" };
    if game.native_block_property(pos, "lit").as_deref() != Some(want) {
        game.native_set_block_property(pos, "lit", want);
    }
}

/// Simulates the Pneumatic Item Pipe (`rusty_factory:item_pipe`):
/// Advances the travelling item smoothly from `pipe_from` to `pipe_to`,
/// then hands it off to the next pipe, conveyor belt, or destination container.
fn tick_item_pipe(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
) {
    if state.belt_item.is_empty() {
        return;
    }
    const PIPE_TICKS: i32 = 10;
    if state.progress < PIPE_TICKS {
        state.progress += 1;
        store_state(entity, state);
        game.native_block_entity_sync(pos);
        if state.progress < PIPE_TICKS {
            return;
        }
    }

    let from_off = rules::offset_from_dir_index(state.pipe_from);
    let from_pos = [pos[0] + from_off[0], pos[1] + from_off[1], pos[2] + from_off[2]];
    let preferred = if state.pipe_has_target { Some(state.pipe_target) } else { None };
    let is_pipe_target = |p: [i32; 3]| {
        p != from_pos
            && (is_item_container_block(&game.native_block(p))
                || rules::is_conveyor_block(&game.native_block(p))
                || game.native_block(p) == rules::BLOCK_ITEM_SPLITTER)
    };
    let next_step = rules::find_next_pipe_step(
        pos,
        preferred,
        |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE && p != from_pos,
        is_pipe_target,
    );

    let stack = ItemStack::new(&state.belt_item, state.belt_count.max(1));

    // If there is no target at all in front of this pipe, check if this is an open pipe end (air) and drop cleanly
    let Some((step_pos, final_target)) = next_step else {
        let opp = rules::offset_from_dir_index(rules::opposite_dir_index(state.pipe_from));
        let open_pos = [pos[0] + opp[0], pos[1] + opp[1], pos[2] + opp[2]];
        if matches!(
            game.native_block(open_pos).as_str(),
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        ) {
            let drop_pos = [
                pos[0] as f64 + 0.5 + opp[0] as f64 * 0.6,
                pos[1] as f64 + 0.35 + opp[1] as f64 * 0.4,
                pos[2] as f64 + 0.5 + opp[2] as f64 * 0.6,
            ];
            let vel = [opp[0] as f64 * 0.12, 0.06, opp[2] as f64 * 0.12];
            game.spawn_item(stack, drop_pos, vel, 10);
            state.belt_item.clear();
            state.belt_count = 0;
            state.progress = 0;
            state.pipe_has_target = false;
            store_state(entity, state);
            game.native_block_entity_sync(pos);
        }
        return;
    };

    let step_block = game.native_block(step_pos);
    let leftover = if step_block == rules::BLOCK_ITEM_PIPE {
        insert_into_pipe(game, pos, step_pos, stack.clone(), Some(final_target))
    } else if step_block == rules::BLOCK_ITEM_SPLITTER {
        insert_into_splitter(game, step_pos, stack.clone())
    } else if step_block == rules::BLOCK_INDUSTRIAL_SILO {
        insert_into_silo(game, step_pos, stack.clone())
    } else if step_block == rules::BLOCK_SILO_CONTROLLER {
        insert_into_silo_controller(game, step_pos, stack.clone())
    } else if rules::is_conveyor_block(&step_block) {
        insert_into_conveyor(game, step_pos, stack.clone())
    } else {
        match game.native_insert_item(step_pos, Direction::Up, stack.clone()) {
            None => None,
            Some(rest) => game.native_insert_item(step_pos, Direction::North, rest),
        }
    };

    match leftover {
        None => {
            state.belt_item.clear();
            state.belt_count = 0;
            state.progress = 0;
            state.pipe_has_target = false;
            store_state(entity, state);
            game.native_block_entity_sync(pos);
        }
        Some(rest) if rest.count < state.belt_count => {
            state.belt_count = rest.count;
            store_state(entity, state);
            game.native_block_entity_sync(pos);
        }
        Some(_) => {
            // Destination chest/target is full! Try any other reachable target in the network (including branching back!)
            state.pipe_has_target = false;
            let all_targets = rules::find_connected_item_targets(
                pos,
                |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
                |p| {
                    p != final_target
                        && p != step_pos
                        && (is_item_container_block(&game.native_block(p))
                            || rules::is_conveyor_block(&game.native_block(p))
                            || game.native_block(p) == rules::BLOCK_ITEM_SPLITTER)
                },
            );
            let mut rerouted = false;
            for alt in all_targets {
                if let Some((alt_step, _)) = rules::find_next_pipe_step(
                    pos,
                    Some(alt),
                    |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
                    |p| p == alt,
                ) {
                    let alt_blk = game.native_block(alt_step);
                    let rem = if alt_blk == rules::BLOCK_ITEM_PIPE {
                        insert_into_pipe(game, pos, alt_step, stack.clone(), Some(alt))
                    } else if alt_blk == rules::BLOCK_ITEM_SPLITTER {
                        insert_into_splitter(game, alt_step, stack.clone())
                    } else if alt_blk == rules::BLOCK_INDUSTRIAL_SILO {
                        insert_into_silo(game, alt_step, stack.clone())
                    } else if alt_blk == rules::BLOCK_SILO_CONTROLLER {
                        insert_into_silo_controller(game, alt_step, stack.clone())
                    } else if rules::is_conveyor_block(&alt_blk) {
                        insert_into_conveyor(game, alt_step, stack.clone())
                    } else {
                        match game.native_insert_item(alt_step, Direction::Up, stack.clone()) {
                            None => None,
                            Some(r) => game.native_insert_item(alt_step, Direction::North, r),
                        }
                    };
                    if rem.is_none() {
                        state.belt_item.clear();
                        state.belt_count = 0;
                        state.progress = 0;
                        state.pipe_has_target = false;
                        store_state(entity, state);
                        game.native_block_entity_sync(pos);
                        rerouted = true;
                        break;
                    } else if let Some(r) = rem {
                        if r.count < state.belt_count {
                            state.belt_count = r.count;
                            store_state(entity, state);
                            game.native_block_entity_sync(pos);
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
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
) {
    let dim = game.native_dimension();
    let (max_ticks, max_batch) = rules::conveyor_tier(&entity.id).unwrap_or((20, 1));
    let facing = game
        .native_block_property(pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let shape = game
        .native_block_property(pos, "shape")
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
        let dropped = game.native_item_entities(
            dim,
            [pos[0] as f64 + 0.1, pos[1] as f64, pos[2] as f64 + 0.1],
            [pos[0] as f64 + 0.9, pos[1] as f64 + 1.15, pos[2] as f64 + 0.9],
        );
        if let Some(first) = dropped.first() {
            let take = first.stack.count.min(max_batch);
            state.belt_item = first.stack.item.to_string();
            state.belt_count = take;
            state.progress = 0;
            let mut rem = first.stack.clone();
            rem.count = first.stack.count.saturating_sub(take);
            game.native_set_item_entity(dim, first.id, rem);
            store_state(entity, state);
            game.native_block_entity_sync(pos);
            return;
        }
    }

    // 2. If Auto-Pull (`state.auto_io`) is enabled and belt is empty, pull up to `max_batch` from container behind
    if state.auto_io && state.belt_item.is_empty() {
        let back_block = game.native_block(back_pos);
        if !rules::is_conveyor_block(&back_block) && is_item_container_block(&back_block) {
            if let Some(slots) = game.native_container_slots(dim, back_pos, Direction::Down) {
                for s in slots {
                    if s.can_take && s.stack.is_some() {
                        if let Some(pulled) =
                            game.native_extract_item(dim, back_pos, Direction::Down, s.slot, max_batch)
                        {
                            state.belt_item = pulled.item.to_string();
                            state.belt_count = pulled.count;
                            state.progress = 0;
                            store_state(entity, state);
                            game.native_block_entity_sync(pos);
                            return;
                        }
                    }
                }
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
        game.native_block_entity_sync(pos);
        // If an inline Belt Filter is attached and item does NOT match filter (whitelist/blacklist), deflect to the side!
        if state.progress >= (max_ticks / 2).max(1)
            && state.belt_filter_attached
            && !state.splitter_filters[0].is_empty()
            && !belt_filter_matches(&state.splitter_filters[0], &state.belt_item, state.belt_filter_blacklist)
        {
            let stack = ItemStack::new(&state.belt_item, state.belt_count.max(1));
            let side_offs = [
                rules::conveyor_exit_offset(&facing, "turn_right"),
                rules::conveyor_exit_offset(&facing, "turn_left"),
            ];
            let mut deflected = false;
            for soff in side_offs {
                let sp = [pos[0] + soff[0], pos[1], pos[2] + soff[2]];
                let sb = game.native_block(sp);
                if rules::is_conveyor_block(&sb) {
                    if insert_into_conveyor(game, sp, stack.clone()).is_none() {
                        deflected = true;
                        break;
                    }
                } else if is_item_container_block(&sb) {
                    if game.native_insert_item(sp, Direction::Up, stack.clone()).is_none() {
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
                    game.spawn_item(stack.clone(), drop_pos, vel, 10);
                    deflected = true;
                    break;
                }
            }
            if deflected {
                state.belt_item.clear();
                state.belt_count = 0;
                state.progress = 0;
                store_state(entity, state);
                game.native_block_entity_sync(pos);
                return;
            }
        }
        // If an Item Splitter sits directly on top of this belt, feed it once item reaches center
        if state.progress >= (max_ticks / 2).max(1) {
            let above_pos = [pos[0], pos[1] + 1, pos[2]];
            if game.native_block(above_pos) == rules::BLOCK_ITEM_SPLITTER {
                let stack = ItemStack::new(&state.belt_item, state.belt_count.max(1));
                let leftover = insert_into_splitter(game, above_pos, stack);
                if leftover.is_none() {
                    state.belt_item.clear();
                    state.belt_count = 0;
                    state.progress = 0;
                    store_state(entity, state);
                    game.native_block_entity_sync(pos);
                    return;
                } else if let Some(rest) = leftover {
                    if rest.count < state.belt_count {
                        state.belt_count = rest.count;
                        store_state(entity, state);
                        game.native_block_entity_sync(pos);
                    }
                }
            }
        }
        if state.progress < max_ticks {
            return;
        }
    }

    // 4. Animation finished: push forward into next belt/pipe/container, OR drop into world at the end!
    let stack = ItemStack::new(&state.belt_item, state.belt_count.max(1));
    let front_block = game.native_block(front_pos);
    let mut dropped_at_end = false;
    let leftover = if rules::is_conveyor_block(&front_block) {
        insert_into_conveyor(game, front_pos, stack)
    } else if front_block == rules::BLOCK_ITEM_SPLITTER {
        insert_into_splitter(game, front_pos, stack)
    } else if front_block == rules::BLOCK_ITEM_PIPE {
        insert_into_pipe(game, pos, front_pos, stack, None)
    } else if front_block == rules::BLOCK_INDUSTRIAL_SILO {
        insert_into_silo(game, front_pos, stack)
    } else if front_block == rules::BLOCK_SILO_CONTROLLER {
        insert_into_silo_controller(game, front_pos, stack)
    } else if is_item_container_block(&front_block) {
        match game.native_insert_item(front_pos, Direction::Up, stack) {
            None => None,
            Some(r) => game.native_insert_item(front_pos, Direction::North, r),
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
        game.spawn_item(stack, drop_pos, vel, 10);
        dropped_at_end = true;
        None
    } else {
        // Blocked by a solid non-container block at the end: check if there's a belt 1 block below (step-down)
        let below_front = [front_pos[0], front_pos[1] - 1, front_pos[2]];
        if rules::is_conveyor_block(&game.native_block(below_front)) {
            insert_into_conveyor(game, below_front, stack)
        } else {
            Some(stack)
        }
    };

    if dropped_at_end || leftover.is_none() {
        state.belt_item.clear();
        state.belt_count = 0;
        state.progress = 0;
        store_state(entity, state);
        game.native_block_entity_sync(pos);
    } else if let Some(rest) = leftover {
        if rest.count < state.belt_count {
            state.belt_count = rest.count;
            store_state(entity, state);
            game.native_block_entity_sync(pos);
        }
    }
}

/// Simulates the Automated Industrial Quarry (`rusty_factory:quarry`):
/// Excavates blocks in a `5x5`, `7x7`, or `9x9` area behind the Quarry down to Bedrock,
/// moves the 3D gantry & laser to the mined block, and outputs drops into its buffer or pipes/chests!
fn tick_quarry(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
    fortune_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    let (tier_radius, tier_speed, tier_fortune) = rules::quarry_tier(&entity.id);
    if state.quarry_radius < tier_radius {
        state.quarry_radius = tier_radius;
    }
    let cost = rules::effective_energy_cost(MachineKind::Quarry, speed_upgrades);
    let base_needed = rules::effective_work_ticks(MachineKind::Quarry, speed_upgrades);
    let needed = (base_needed / tier_speed as i32).max(1);

    // Check if output slot is completely full before mining
    if let Some(existing) = entity.items[rules::SLOT_OUTPUT].as_ref() {
        let max_s = game.native_max_stack_size(&existing.item).unwrap_or(64);
        if existing.count >= max_s {
            if state.auto_io {
                try_auto_eject_output(game, pos, entity, 8);
            }
            if entity.items[rules::SLOT_OUTPUT]
                .as_ref()
                .is_some_and(|s| s.count >= max_s)
            {
                return (StepStatus::OutputFull, 0);
            }
        }
    }

    state.energy -= cost;
    state.progress += 1;

    if state.progress >= needed {
        state.progress = 0;
        let facing = game
            .native_block_property(pos, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let side = state.quarry_side_len();
        let layer_area = side * side;
        let dim = game.native_dimension();
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
                        let b = game.native_block(sp);
                        if let Some((drop_item, base_count)) = rules::quarry_drop_for_block(&b) {
                            if let Some(air_state) = game.native_default_state("minecraft:air") {
                                game.native_set_block(dim, sp, air_state, 3);
                            }
                            state.quarry_mined += 1;
                            state.last_target = sp;
                            let produced = ItemStack {
                                item: drop_item.into(),
                                count: base_count,
                                damage: 0,
                                extra: None,
                            };
                            store_or_route_output(game, pos, entity, state.auto_io, produced);
                            store_state(entity, state);
                            game.native_block_entity_sync(pos);
                            cleared_surface = true;
                            break 'surface;
                        }
                    }
                }
            }
        }

        // Scan up to 12 positions to skip over air/water/bedrock quickly
        for _ in 0..if cleared_surface { 0 } else { 12 } {
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

            let block_id = game.native_block(target_pos);
            if let Some((drop_item, base_count)) = rules::quarry_drop_for_block(&block_id) {
                if let Some(air_state) = game.native_default_state("minecraft:air") {
                    game.native_set_block(dim, target_pos, air_state, 3);
                }
                state.quarry_mined += 1;
                state.last_target = target_pos;

                let bonus = if block_id.ends_with("_ore") || block_id == "minecraft:ancient_debris" {
                    (fortune_upgrades + tier_fortune).min(6)
                } else {
                    0
                };
                let produced = ItemStack {
                    item: drop_item.into(),
                    count: base_count + bonus,
                    damage: 0,
                    extra: None,
                };

                store_or_route_output(game, pos, entity, state.auto_io, produced);
                store_state(entity, state);
                game.native_block_entity_sync(pos);
                break;
            }
        }
    }

    if state.auto_io && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(game, pos, entity, 4);
    }

    (StepStatus::Working, -cost)
}

/// Inserts `produced` into connected pipes/containers (if `auto_io`) or `SLOT_OUTPUT`.
fn store_or_route_output(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    auto_io: bool,
    produced: ItemStack,
) {
    let mut rest = Some(produced);
    if auto_io {
        rest = route_item_from(game, pos, rest.take().unwrap());
    }
    if let Some(leftover) = rest {
        match entity.items[rules::SLOT_OUTPUT].as_mut() {
            Some(existing) if &*existing.item == &*leftover.item => {
                let max_s = game.native_max_stack_size(&existing.item).unwrap_or(64);
                existing.count = (existing.count + leftover.count).min(max_s);
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
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
    fortune_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(entity);
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
        let facing = game
            .native_block_property(pos, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let side = state.quarry_side_len();
        let layer_area = side * side;
        let dim = game.native_dimension();

        for _ in 0..8 {
            let target_pos = rules::harvester_target_pos(pos, &facing, state);
            state.quarry_cursor += 1;
            if state.quarry_cursor >= layer_area {
                state.quarry_cursor = 0;
                state.quarry_y_offset = (state.quarry_y_offset + 1) % 2;
            }

            let block_id = game.native_block(target_pos);
            let age_prop = game.native_block_property(target_pos, "age");
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
                    if game.native_block(above) == block_id {
                        if let Some(air_state) = game.native_default_state("minecraft:air") {
                            game.native_set_block(dim, above, air_state, 3);
                        }
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
                    game.native_set_block_property(target_pos, "age", reset_age);
                    state.last_target = target_pos;
                } else {
                    if let Some(air_state) = game.native_default_state("minecraft:air") {
                        game.native_set_block(dim, target_pos, air_state, 3);
                    }
                    state.last_target = target_pos;
                }

                state.quarry_mined += 1;
                let produced = ItemStack {
                    item: drop_item.into(),
                    count: base_count + fortune_upgrades.min(4),
                    damage: 0,
                    extra: None,
                };
                store_or_route_output(game, pos, entity, state.auto_io, produced);
                store_state(entity, state);
                game.native_block_entity_sync(pos);
                break;
            }
        }
    }

    if state.auto_io && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(game, pos, entity, 4);
    }

    (StepStatus::Working, -cost)
}

/// Simulates the Automated Pneumatic Pipe Extractor (`rusty_factory:item_extractor`):
/// Automatically pulls items from adjacent Chests/Barrels/Furnaces/Hoppers and routes them
/// through connected `rusty_factory:item_pipe` networks or `conveyor_belt`s!
/// If `SLOT_INPUT` contains an item, it acts as a Smart Filter (only extracting matching items).
fn tick_extractor(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
    speed_upgrades: u32,
) -> StepStatus {
    let outcome = rules::step_machine(state, None, None, None, speed_upgrades);
    if outcome.status != StepStatus::Working {
        return outcome.status;
    }

    if state.progress == 0 {
        let dim = game.native_dimension();
        let filter_item = entity.items[rules::SLOT_INPUT]
            .as_ref()
            .map(|s| s.item.to_string());
        let batch_size = (2 + speed_upgrades.min(8) * 2).min(16);

        let targets = rules::find_connected_item_targets(
            pos,
            |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
            |p| is_item_container_block(&game.native_block(p)),
        );

        for [dx, dy, dz] in rules::NEIGHBORS_6 {
            let adj = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let adj_block = game.native_block(adj);
            if adj_block == rules::BLOCK_INDUSTRIAL_SILO {
                let mut extracted: Option<ItemStack> = None;
                game.native_with_mod_block_entity(adj, |sent| {
                    if let Some(mut sst) = load_state(sent) {
                        let matches_filter = filter_item
                            .as_deref()
                            .is_none_or(|f| f == sst.belt_item);
                        if matches_filter && !sst.belt_item.is_empty() && sst.belt_count > 0 {
                            let take = sst.belt_count.min(batch_size);
                            extracted = Some(ItemStack::new(&sst.belt_item, take));
                            sst.belt_count -= take;
                            if sst.belt_count == 0 {
                                sst.belt_item.clear();
                            }
                            store_state(sent, &sst);
                        }
                    }
                });
                if let Some(st_out) = extracted {
                    game.native_block_entity_sync(adj);
                    if let Some(rest) = route_item_from(game, pos, st_out) {
                        let _ = insert_into_silo(game, adj, rest);
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
                    let moved = game.native_move_items(
                        dim,
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
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
    fortune_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    let facing = game
        .native_block_property(pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let (front_off, _) = rules::conveyor_offsets(&facing);
    let target_pos = [
        pos[0] + front_off[0],
        pos[1] + front_off[1],
        pos[2] + front_off[2],
    ];
    let block_id = game.native_block(target_pos);
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
        let dim = game.native_dimension();
        if let Some(air_state) = game.native_default_state("minecraft:air") {
            game.native_set_block(dim, target_pos, air_state, 3);
        }
        state.quarry_mined += 1;
        let produced = ItemStack {
            item: drop_item.into(),
            count: base_count + fortune_upgrades.min(4),
            damage: 0,
            extra: None,
        };
        store_or_route_output(game, pos, entity, state.auto_io, produced);
    }

    if state.auto_io && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(game, pos, entity, 4);
    }

    (StepStatus::Working, -cost)
}

/// Simulates the Automated Block Placer (`rusty_factory:block_placer`):
/// Places block items from `SLOT_INPUT` into the world directly in front of its `facing` face when air.
fn tick_placer(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    let Some(in_stack) = entity.items[rules::SLOT_INPUT].clone() else {
        state.progress = 0;
        return (StepStatus::Idle, 0);
    };
    let Some(place_state) = game.native_default_state(&in_stack.item) else {
        state.progress = 0;
        return (StepStatus::Idle, 0);
    };

    let facing = game
        .native_block_property(pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let (front_off, _) = rules::conveyor_offsets(&facing);
    let target_pos = [
        pos[0] + front_off[0],
        pos[1] + front_off[1],
        pos[2] + front_off[2],
    ];
    let current_block = game.native_block(target_pos);
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
        let dim = game.native_dimension();
        if game.native_set_block(dim, target_pos, place_state, 3) {
            state.quarry_mined += 1;
            if let Some(s) = entity.items[rules::SLOT_INPUT].as_mut() {
                if s.count > 1 {
                    s.count -= 1;
                } else {
                    entity.items[rules::SLOT_INPUT] = None;
                }
            }
        }
    }

    (StepStatus::Working, -cost)
}

/// Simulates the Automated Sap Extractor (`rusty_factory:sap_extractor`):
/// Scans adjacent blocks (and vertical trunk columns around it) for `rubber_sap_log` or `rubber_log`,
/// extracts `raw_sap` without destroying the tree, and refills sap cavities over time!
fn tick_sap_extractor(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
    fuel_ticks: Option<i32>,
    speed_upgrades: u32,
    fortune_upgrades: u32,
) -> (StepStatus, i32) {
    let outcome = rules::step_machine(state, None, fuel_ticks, None, speed_upgrades);
    if outcome.consume_fuel {
        consume_one_fuel(entity);
    }
    if outcome.status != StepStatus::Working {
        return (outcome.status, 0);
    }

    // Look for any Rubber Tree trunk block around the extractor (horizontal neighbors + up to 4 blocks up)
    let mut found_trunk: Option<([i32; 3], bool)> = None;
    'search: for [dx, _, dz] in [[-1, 0, 0], [1, 0, 0], [0, 0, -1], [0, 0, 1]] {
        for dy in 0..=4 {
            let p = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let bid = game.native_block(p);
            if bid == rules::BLOCK_RUBBER_SAP_LOG {
                let has_sap = game.native_block_property(p, "lit").as_deref() == Some("true");
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
            game.native_set_block_property(trunk_pos, "lit", "false");
        }
        let base_sap = if is_active_cavity { 3 } else { 1 };
        let sap_count = base_sap + fortune_upgrades.min(4);
        state.quarry_mined += sap_count as i32;
        state.last_target = trunk_pos;

        let produced = ItemStack {
            item: rules::ITEM_RAW_SAP.into(),
            count: sap_count,
            damage: 0,
            extra: None,
        };
        store_or_route_output(game, pos, entity, state.auto_io, produced);
        store_state(entity, state);
        game.native_block_entity_sync(pos);
    }

    if state.auto_io && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(game, pos, entity, 4);
    }

    (StepStatus::Working, -cost)
}

fn consume_one_fuel(entity: &mut ModBlockEntity) {
    let slot_idx = if entity.items.len() == 1 { 0 } else { rules::SLOT_FUEL };
    if let Some(Some(fuel_stack)) = entity.items.get_mut(slot_idx) {
        let is_lava = &*fuel_stack.item == "minecraft:lava_bucket";
        if fuel_stack.count > 1 {
            fuel_stack.count -= 1;
        } else {
            entity.items[slot_idx] = is_lava.then(|| ItemStack {
                item: "minecraft:bucket".into(),
                count: 1,
                damage: 0,
                extra: None,
            });
        }
    }
}

pub(in crate::game) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if !game.native_on(rules::ID) {
        return;
    }
    tick_player_equipment(game);

    // 1. Rubber Sapling automatic growth tick (grows into a full Rubber Tree after ~8 seconds)
    if entity.id == rules::BLOCK_RUBBER_SAPLING {
        let t = entity.data.first().copied().unwrap_or(0) + 1;
        if t >= 160 {
            grow_rubber_tree(game, pos);
        } else {
            entity.data = vec![t];
        }
        return;
    }

    // 2. Rubber Sap Log Cavity natural sap regeneration tick (refills golden resin after ~10 seconds)
    if entity.id == rules::BLOCK_RUBBER_SAP_LOG {
        let is_full = game.native_block_property(pos, "lit").as_deref() == Some("true");
        if !is_full {
            let t = entity.data.first().copied().unwrap_or(0) + 1;
            if t >= 200 {
                game.native_set_block_property(pos, "lit", "true");
                entity.data = vec![0];
            } else {
                entity.data = vec![t];
            }
        }
        return;
    }

    let Some(kind) = MachineKind::from_block_id(&entity.id) else {
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
        .filter(|s| &*s.item == rules::ITEM_SPEED_UPGRADE)
        .map_or(0, |s| s.count);
    let fortune_upgrades = entity.items.get(rules::SLOT_UPGRADE)
        .and_then(|s| s.as_ref())
        .filter(|s| &*s.item == rules::ITEM_FORTUNE_UPGRADE)
        .map_or(0, |s| s.count);

    if kind == MachineKind::FluidPipe {
        if state.fluid_amount > 0 {
            state.fluid_amount = (state.fluid_amount - 5).max(0);
            if state.fluid_amount == 0 {
                state.fluid_kind.clear();
            }
            store_state(entity, &state);
            game.native_block_entity_sync(pos);
        }
        return;
    }

    if kind == MachineKind::ItemSplitter {
        if state.enabled && state.auto_io && (game.world.game_time % 5 == 0) {
            let dim = game.native_dimension();
            let facing = game
                .native_block_property(pos, "facing")
                .unwrap_or_else(|| "north".to_owned());
            let (_, back_off) = rules::conveyor_offsets(&facing);
            let back_pos = [pos[0] + back_off[0], pos[1] + back_off[1], pos[2] + back_off[2]];
            let back_block = game.native_block(back_pos);
            if !rules::is_conveyor_block(&back_block) && is_item_container_block(&back_block) {
                if let Some(slots) = game.native_container_slots(dim, back_pos, Direction::Down) {
                    for s in slots {
                        if s.can_take && s.stack.is_some() {
                            if let Some(pulled) = game.native_extract_item(dim, back_pos, Direction::Down, s.slot, 1) {
                                if let Some(rest) = insert_into_splitter(game, pos, pulled) {
                                    let _ = game.native_insert_item(back_pos, Direction::Up, rest);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        return;
    }

    if kind == MachineKind::ItemPipe {
        tick_item_pipe(game, pos, entity, &mut state);
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::ConveyorBelt {
        if state.enabled {
            tick_conveyor(game, pos, entity, &mut state);
        }
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::ItemExtractor {
        let status = tick_extractor(game, pos, entity, &mut state, speed_upgrades);
        update_lit_state(game, pos, status == StepStatus::Working);
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
            update_lit_state(game, pos, true);
        } else {
            update_lit_state(game, pos, false);
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
            let b = game.native_block(check_p);
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
                    for [dx, dy, dz] in rules::NEIGHBORS_6 {
                        let target = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
                        let tb = game.native_block(target);
                        if tb == rules::BLOCK_FLUID_TANK {
                            tank_targets.push(target);
                        } else if tb == rules::BLOCK_FLUID_PIPE {
                            game.native_with_mod_block_entity(target, |p_ent| {
                                let mut p_st = load_state(p_ent).unwrap_or_else(|| MachineState::new(MachineKind::FluidPipe));
                                p_st.fluid_kind = fk.to_owned();
                                p_st.fluid_amount = 100;
                                store_state(p_ent, &p_st);
                            });
                            game.native_block_entity_sync(target);
                            let tanks = rules::find_connected_item_targets(
                                target,
                                |p| game.native_block(p) == rules::BLOCK_FLUID_PIPE,
                                |p| game.native_block(p) == rules::BLOCK_FLUID_TANK,
                            );
                            tank_targets.extend(tanks);
                        }
                    }
                    for tank_pos in tank_targets {
                        let mut filled_now = false;
                        game.native_with_mod_block_entity(tank_pos, |t_ent| {
                            let mut t_st = load_state(t_ent).unwrap_or_else(|| MachineState::new(MachineKind::FluidTank));
                            if (t_st.fluid_kind.is_empty() || t_st.fluid_kind == fk)
                                && t_st.fluid_amount < rules::FLUID_TANK_MAX_MB
                            {
                                t_st.fluid_kind = fk.to_owned();
                                t_st.fluid_amount = (t_st.fluid_amount + pump_mb).min(rules::FLUID_TANK_MAX_MB);
                                store_state(t_ent, &t_st);
                                filled_now = true;
                            }
                        });
                        if filled_now {
                            game.native_set_block_property(tank_pos, "lit", "true");
                            break;
                        }
                    }
                } else {
                    status = StepStatus::NoPower;
                }
            }
        }
        update_lit_state(game, pos, status == StepStatus::Working);
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
            if slot0.item.as_ref() == "minecraft:lava_bucket" && state.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB {
                state.fluid_kind = "lava".to_owned();
                state.fluid_amount += 1000;
                *slot0 = ItemStack::new("minecraft:bucket", 1);
            }
        }
        if state.fluid_amount < 500 {
            for d in rules::DIRS {
                let adj = [pos[0] + d[0], pos[1] + d[1], pos[2] + d[2]];
                if matches!(game.native_block(adj).as_str(), rules::BLOCK_FLUID_TANK | rules::BLOCK_FLUID_PIPE) {
                    let mut pulled = 0;
                    game.native_with_mod_block_entity(adj, |other| {
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
            let sent = distribute_energy(game, pos, state.energy.min(240), false);
            state.energy -= sent;
        }
        update_lit_state(game, pos, net_rate > 0);
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
                if matches!(game.native_block(adj).as_str(), rules::BLOCK_STEAM_BOILER | rules::BLOCK_FLUID_TANK | rules::BLOCK_FLUID_PIPE) {
                    let mut pulled = 0;
                    game.native_with_mod_block_entity(adj, |other| {
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
            let sent = distribute_energy(game, pos, state.energy.min(320), false);
            state.energy -= sent;
        }
        update_lit_state(game, pos, net_rate > 0);
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
                    if in_stack.count > 1 {
                        in_stack.count -= 1;
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
                let prod = ItemStack::new(pick, 1);
                store_or_route_output(game, pos, entity, state.auto_io, prod);
            }
        }
        update_lit_state(game, pos, working);
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
            let in_id = slot0.item.to_string();
            if state.belt_item.is_empty() || state.belt_item == in_id {
                let space = cap.saturating_sub(state.belt_count);
                let add = slot0.count.min(space);
                if add > 0 {
                    state.belt_item = in_id;
                    state.belt_count += add;
                    if slot0.count > add {
                        slot0.count -= add;
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
            if let Some(rest) = insert_into_silo_controller(game, pos, in_stack) {
                entity.items[0] = Some(rest);
            }
        }
        let (list, silos_n, chests_n) = aggregate_silo_network(game, pos);
        let max_page = (list.len().saturating_sub(1) / 4) as i32;
        if state.quarry_cursor > max_page {
            state.quarry_cursor = max_page;
        }
        let page = state.quarry_cursor.max(0) as usize;

        // Handle pending row click withdrawal (`state.quarry_y_offset == 1..=4`)
        if state.quarry_y_offset > 0 {
            let row_idx = (state.quarry_y_offset - 1) as usize;
            state.quarry_y_offset = 0;
            if let Some((target_item, _, _)) = list.get(page * 4 + row_idx) {
                let (silos, chests) = collect_silo_network(game, pos);
                let mut need = 64u32;
                let mut taken = 0u32;
                for &sp in &silos {
                    if need == 0 {
                        break;
                    }
                    let mut took = 0u32;
                    game.native_with_mod_block_entity(sp, |sent| {
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
                        game.native_block_entity_sync(sp);
                        need -= took;
                        taken += took;
                    }
                }
                for &cp in &chests {
                    if need == 0 {
                        break;
                    }
                    let mut took = 0u32;
                    if let Some(BlockEntity::Chest(slots)) = game.world.block_entities.get_mut(&(cp[0], cp[1], cp[2])) {
                        for slot in slots.iter_mut() {
                            if need == 0 {
                                break;
                            }
                            if let Some(st) = slot.as_mut() {
                                if st.item.as_ref() == target_item.as_str() {
                                    let amt = st.count.min(need);
                                    took += amt;
                                    need -= amt;
                                    if st.count > amt {
                                        st.count -= amt;
                                    } else {
                                        *slot = None;
                                    }
                                }
                            }
                        }
                    }
                    taken += took;
                }
                if taken > 0 {
                    game.native_give_items(target_item, taken);
                }
            }
        }

        update_lit_state(game, pos, silos_n + chests_n > 0);
        entity.data = vec![
            if silos_n + chests_n > 0 { StepStatus::Working as i32 } else { StepStatus::Idle as i32 },
            silos_n as i32,
            chests_n as i32,
            list.len() as i32,
            0,
            1,
            page as i32,
            pos[0],
            pos[1],
            pos[2],
        ];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::Tesseract {
        // 1. Wireless cross-world energy sharing with other Tesseracts
        let mut peers: Vec<[i32; 3]> = Vec::new();
        for (&(px, py, pz), be) in &game.world.block_entities {
            if let BlockEntity::Mod(ent) = be {
                if ent.id == rules::BLOCK_TESSERACT && [px, py, pz] != pos {
                    peers.push([px, py, pz]);
                }
            }
        }
        if state.energy > 0 {
            let sent = distribute_energy(game, pos, state.energy.min(500), false);
            state.energy -= sent;
        }
        if !peers.is_empty() && state.energy > 100 {
            let share = (state.energy / (peers.len() as i32 + 1)).min(1_000);
            if share > 0 {
                for peer in &peers {
                    let mut accepted = 0;
                    game.native_with_mod_block_entity(*peer, |pent| {
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
                    game.native_with_mod_block_entity(*peer, |pent| {
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
                if let Some(rest) = route_item_from(game, pos, out) {
                    entity.items[rules::SLOT_OUTPUT] = Some(rest);
                }
            }
        }
        update_lit_state(game, pos, !peers.is_empty() && state.energy > 0);
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
            let sent = distribute_energy(game, pos, state.energy.min(500), false);
            state.energy -= sent;
        }
        update_lit_state(game, pos, state.enabled && state.energy > 0);
        let pct = (state.energy * 100) / kind.max_energy().max(1);
        entity.data = vec![1, state.energy, kind.max_energy(), pct, 0, i32::from(state.enabled), 0, 0, 0, 0];
        store_state(entity, &state);
        return;
    }

    if kind == MachineKind::FluidTank {
        // Process bucket / fluid item in slot 0 inside Tank GUI
        if let Some(Some(slot0)) = entity.items.first_mut() {
            let item_id = slot0.item.as_ref();
            if item_id == "minecraft:water_bucket"
                && (state.fluid_kind.is_empty() || state.fluid_kind == "water")
                && state.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB
            {
                state.fluid_kind = "water".to_owned();
                state.fluid_amount += 1000;
                *slot0 = ItemStack::new("minecraft:bucket", 1);
            } else if item_id == "minecraft:lava_bucket"
                && (state.fluid_kind.is_empty() || state.fluid_kind == "lava")
                && state.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB
            {
                state.fluid_kind = "lava".to_owned();
                state.fluid_amount += 1000;
                *slot0 = ItemStack::new("minecraft:bucket", 1);
            } else if item_id == rules::ITEM_RAW_SAP
                && (state.fluid_kind.is_empty() || state.fluid_kind == "sap")
                && state.fluid_amount + 250 <= rules::FLUID_TANK_MAX_MB
            {
                state.fluid_kind = "sap".to_owned();
                state.fluid_amount += 250;
                if slot0.count > 1 {
                    slot0.count -= 1;
                } else {
                    entity.items[0] = None;
                }
            } else if item_id == "minecraft:bucket" && slot0.count == 1 && state.fluid_amount >= 1000 {
                if state.fluid_kind == "water" {
                    state.fluid_amount -= 1000;
                    *slot0 = ItemStack::new("minecraft:water_bucket", 1);
                    if state.fluid_amount == 0 {
                        state.fluid_kind.clear();
                    }
                } else if state.fluid_kind == "lava" {
                    state.fluid_amount -= 1000;
                    *slot0 = ItemStack::new("minecraft:lava_bucket", 1);
                    if state.fluid_amount == 0 {
                        state.fluid_kind.clear();
                    }
                }
            }
        }
        update_lit_state(game, pos, state.fluid_amount > 0);
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
        entity.items.first().and_then(|s| s.as_ref()).and_then(fuel_burn_ticks)
    } else {
        entity.items.get(rules::SLOT_FUEL).and_then(|s| s.as_ref()).and_then(fuel_burn_ticks)
    };

    if is_gen {
        let max_e = kind.max_energy();
        let outcome = rules::step_machine(&mut state, None, fuel_ticks, None, speed_upgrades);
        if outcome.consume_fuel {
            consume_one_fuel(entity);
        }
        let gen_rate = if outcome.status == StepStatus::Working {
            kind.energy_per_tick() + (speed_upgrades.min(8) as i32) * 10
        } else {
            0
        };
        if state.energy > 0 {
            let sent = distribute_energy(game, pos, state.energy.min(kind.energy_per_tick() * 4), false);
            state.energy -= sent;
        }
        update_lit_state(game, pos, outcome.status == StepStatus::Working);
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
        let is_array_3x3 = rules::is_solar_array_3x3(pos, |p| game.native_block(p) == rules::BLOCK_SOLAR);
        let max_e = if is_array_3x3 { 20_000 } else { 2_000 };
        let mut net_rate = 0;
        let status = if !state.enabled {
            StepStatus::Paused
        } else {
            let (_block_light, sky_light) = game.native_light([pos[0], pos[1] + 1, pos[2]]);
            let day_time = game.native_day_time();
            let (rain, _thunder) = game.native_weather();
            let solar_gen = rules::solar_generation_rate(day_time, sky_light, rain, speed_upgrades, is_array_3x3);
            if solar_gen > 0 {
                state.energy = (state.energy + solar_gen).min(max_e);
                net_rate = solar_gen;
            }
            if state.energy > 0 {
                let sent = distribute_energy(game, pos, state.energy.min(if is_array_3x3 { 600 } else { 200 }), false);
                state.energy -= sent;
            }
            // Recharging item placed in single centered slot 0 (Battery, Jetpack, Drill, Jet Boots, Mining Laser, Solar Helmet)
            if let Some(Some(item_stack)) = entity.items.first_mut() {
                let charge_cost = 40;
                if state.energy >= charge_cost {
                    if matches!(
                        item_stack.item.as_ref(),
                        rules::ITEM_BATTERY
                            | rules::ITEM_JETPACK
                            | rules::ITEM_JET_BOOTS
                            | rules::ITEM_MINING_DRILL
                            | rules::ITEM_MINING_LASER
                            | rules::ITEM_SOLAR_HELMET
                    ) {
                        if item_stack.damage > 0 {
                            item_stack.damage = item_stack.damage.saturating_sub(1);
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
        update_lit_state(game, pos, status == StepStatus::Working);
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
                if let Some(cap) = rules::battery_capacity(item_stack.item.as_ref()) {
                    let cur = get_item_energy(item_stack);
                    if cur < cap {
                        let add = 500.min(cap - cur).min(state.energy);
                        if add > 0 {
                            set_item_energy(item_stack, cur + add);
                            state.energy -= add;
                        }
                    }
                    if item_stack.damage > 0 {
                        item_stack.damage = item_stack.damage.saturating_sub(2);
                        state.energy = (state.energy - charge_cost).max(0);
                    }
                }
            }
        }
        // 2. Standing induction wireless charging for player & inventory batteries/tools:
        let mut player_on_pad = false;
        let pad_center = [pos[0] as f64 + 0.5, pos[1] as f64 + 0.2, pos[2] as f64 + 0.5];
        if let Some(player) = &game.player {
            let dist_sq = (player.position.x - pad_center[0]).powi(2)
                + (player.position.y - pad_center[1]).powi(2)
                + (player.position.z - pad_center[2]).powi(2);
            if dist_sq <= 1.4 * 1.4 {
                player_on_pad = true;
                if state.energy >= 20 {
                    state.energy -= 20;
                    if (game.world.game_time + pos[0] as i64) % 15 == 0 {
                        game.native_run_command(&format!(
                            "particle minecraft:electric_spark {} {} {} 0.3 0.1 0.3 0.05 12",
                            pad_center[0], pad_center[1], pad_center[2]
                        ));
                    }
                    if (game.world.game_time + pos[0] as i64) % 30 == 0 {
                        game.native_run_command("playsound minecraft:block.beacon.ambient player @s ~ ~ ~ 0.35 1.8");
                        if let Some(who) = game.native_acting_player() {
                            game.native_action_bar(
                                who.uuid,
                                Text::literal("⚡ [Plataforma de Carga] Recarga por Inducción activa (500 FE/s)"),
                            );
                        }
                    }
                }
            }
        }
        if player_on_pad && state.energy > 0 {
            // (is_battery, pct, is_armor, idx, cur, cap)
            let mut candidates: Vec<(bool, u32, bool, usize, i32, i32)> = Vec::new();
            for (idx, slot) in game.armor.iter().enumerate() {
                if let Some(st) = slot.as_ref() {
                    if let Some(cap) = rules::battery_capacity(st.item.as_ref()) {
                        let cur = get_item_energy(st);
                        if cur < cap {
                            let pct = ((cur as i64 * 1000) / cap.max(1) as i64) as u32;
                            candidates.push((false, pct, true, idx, cur, cap));
                        }
                    }
                }
            }
            for (idx, slot) in game.inventory.iter().enumerate() {
                if let Some(st) = slot.as_ref() {
                    if let Some(cap) = rules::battery_capacity(st.item.as_ref()) {
                        let cur = get_item_energy(st);
                        if cur < cap {
                            let is_battery = rules::is_battery_item(st.item.as_ref());
                            let pct = ((cur as i64 * 1000) / cap.max(1) as i64) as u32;
                            candidates.push((is_battery, pct, false, idx, cur, cap));
                        }
                    }
                }
            }
            candidates.sort_by_key(|c| (c.0, c.1));
            let mut transferred = 0;
            for (_, _, is_armor, idx, cur, cap) in candidates {
                if transferred >= state.energy {
                    break;
                }
                let slot_opt = if is_armor {
                    game.armor.get_mut(idx)
                } else {
                    game.inventory.get_mut(idx)
                };
                if let Some(Some(st)) = slot_opt {
                    let add = 500.min(cap - cur).min(state.energy - transferred);
                    if add > 0 {
                        set_item_energy(st, cur + add);
                        transferred += add;
                    }
                }
            }
            state.energy -= transferred;
        }
        update_lit_state(game, pos, player_on_pad || entity.items.first().is_some_and(|s| s.is_some()));
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
        let cluster = rules::find_battery_cluster(pos, |p| game.native_block(p) == rules::BLOCK_ENERGY_BANK);
        let cluster_len = cluster.len() as i32;
        let cluster_max = cluster_len * 10_000;

        let prev_energy = state.energy;
        let outcome = rules::step_machine(&mut state, None, fuel_ticks, None, speed_upgrades);
        if outcome.consume_fuel {
            consume_one_fuel(entity);
        }
        let mut net_rate = state.energy - prev_energy;
        if state.enabled && state.auto_io && state.energy > 0 {
            let sent = distribute_energy(game, pos, state.energy.min(400), true);
            state.energy -= sent;
            if sent > 0 {
                net_rate = -sent;
            }
        }
        update_lit_state(game, pos, state.enabled && state.energy > 0);
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
                game,
                pos,
                entity,
                &mut state,
                fuel_ticks,
                speed_upgrades,
                fortune_upgrades,
            ),
            MachineKind::AutoHarvester => tick_harvester(
                game,
                pos,
                entity,
                &mut state,
                fuel_ticks,
                speed_upgrades,
                fortune_upgrades,
            ),
            MachineKind::BlockBreaker => tick_breaker(
                game,
                pos,
                entity,
                &mut state,
                fuel_ticks,
                speed_upgrades,
                fortune_upgrades,
            ),
            MachineKind::BlockPlacer => {
                tick_placer(game, pos, entity, &mut state, fuel_ticks, speed_upgrades)
            }
            MachineKind::SapExtractor => tick_sap_extractor(
                game,
                pos,
                entity,
                &mut state,
                fuel_ticks,
                speed_upgrades,
                fortune_upgrades,
            ),
            _ => unreachable!(),
        };
        update_lit_state(game, pos, status == StepStatus::Working);
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
        .map(|s| s.item.to_string());
    let output_info = entity.items[rules::SLOT_OUTPUT].as_ref().map(|s| {
        let max_stack = game.native_max_stack_size(&s.item).unwrap_or(64);
        (s.item.to_string(), s.count, max_stack)
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
        consume_one_fuel(entity);
    }

    if outcome.consume_input {
        if let Some(in_stack) = entity.items[rules::SLOT_INPUT].as_mut() {
            if in_stack.count > 1 {
                in_stack.count -= 1;
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
            Some(existing) if &*existing.item == prod.item => {
                let max_s = game.native_max_stack_size(&existing.item).unwrap_or(64);
                existing.count = (existing.count + total_count).min(max_s);
            }
            None => {
                entity.items[rules::SLOT_OUTPUT] = Some(ItemStack {
                    item: prod.item.into(),
                    count: total_count,
                    damage: 0,
                    extra: None,
                });
            }
            _ => {}
        }
    }

    let is_gen = matches!(kind, MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler);
    if state.auto_io && !is_gen && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(game, pos, entity, 4);
    }

    if is_gen && state.energy > 0 {
        let max_send = if kind == MachineKind::SteamBoiler { 600 } else { 280 };
        let sent = distribute_energy(game, pos, state.energy.min(max_send), false);
        state.energy -= sent;
    }


    update_lit_state(game, pos, outcome.status == StepStatus::Working);

    // Dynamic machine sound & particle effects
    if outcome.status == StepStatus::Working && ((game.world.game_time + (pos[0] as i64 * 11)) % 15 == 0) {
        let px = pos[0] as f64 + 0.5;
        let py = pos[1] as f64 + 1.05;
        let pz = pos[2] as f64 + 0.5;
        match kind {
            MachineKind::CoalGenerator | MachineKind::BioGenerator => {
                game.native_run_command(&format!("particle minecraft:smoke {px} {py} {pz} 0.15 0.1 0.15 0.01 2"));
            }
            MachineKind::SteamBoiler => {
                game.native_run_command(&format!("particle minecraft:cloud {px} {py} {pz} 0.15 0.15 0.15 0.02 2"));
            }
            MachineKind::ElectricCrusher | MachineKind::MechanicalPress => {
                game.native_run_command(&format!("particle minecraft:crit {px} {py} {pz} 0.2 0.1 0.2 0.04 3"));
            }
            MachineKind::ElectricFurnace | MachineKind::AlloySmelter => {
                game.native_run_command(&format!("particle minecraft:flame {px} {py} {pz} 0.15 0.08 0.15 0.01 2"));
            }
            _ => {}
        }
    }
    if outcome.status == StepStatus::Working && ((game.world.game_time + (pos[0] as i64 * 11)) % 40 == 0) {
        match kind {
            MachineKind::SteamBoiler => {
                game.native_run_command(&format!("playsound minecraft:block.fire.extinguish block @a {} {} {} 0.3 1.2", pos[0], pos[1], pos[2]));
            }
            MachineKind::ElectricCrusher => {
                game.native_run_command(&format!("playsound minecraft:block.grindstone.use block @a {} {} {} 0.35 0.9", pos[0], pos[1], pos[2]));
            }
            MachineKind::MechanicalPress => {
                game.native_run_command(&format!("playsound minecraft:block.piston.extend block @a {} {} {} 0.4 0.8", pos[0], pos[1], pos[2]));
            }
            MachineKind::AlloySmelter => {
                game.native_run_command(&format!("playsound minecraft:block.blastfurnace.fire_crackle block @a {} {} {} 0.35 1.1", pos[0], pos[1], pos[2]));
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
pub(in crate::game) fn submit(
    game: &Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    if !game.native_on(rules::ID) {
        return;
    }

    if block == rules::BLOCK_INDUSTRIAL_SILO {
        let Some(st) = load_state(entity) else {
            return;
        };
        if !st.belt_item.is_empty() && st.belt_count > 0 {
            let facing = game
                .native_block_state_property(state, "facing")
                .unwrap_or("north");
            let (front_off, _) = rules::conveyor_offsets(facing);
            let yaw = match facing {
                "north" => 0.0,
                "south" => 180.0,
                "east" => 270.0,
                "west" => 90.0,
                _ => 0.0,
            };
            let stack = ItemStack::new(&st.belt_item, 1);
            draw.push();
            draw.translate([
                0.5 + front_off[0] as f32 * 0.52,
                0.60,
                0.5 + front_off[2] as f32 * 0.52,
            ]);
            draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
            draw.scale([0.46, 0.46, 0.46]);
            let _ = game.native_draw_item(
                draw,
                &stack,
                DisplayContext::Fixed,
                None,
                pos[0].wrapping_mul(31) ^ pos[2],
            );
            draw.pop();
        }
        return;
    }

    if block == rules::BLOCK_ITEM_SPLITTER {
        let Some(st) = load_state(entity) else {
            return;
        };
        let facing = game
            .native_block_state_property(state, "facing")
            .unwrap_or("north");
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
            let stack = ItemStack::new(filter_id, 1);
            draw.push();
            draw.translate([
                0.5 + off[0] as f32 * 0.46,
                0.48,
                0.5 + off[2] as f32 * 0.46,
            ]);
            draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
            draw.scale([0.34, 0.34, 0.34]);
            let _ = game.native_draw_item(
                draw,
                &stack,
                DisplayContext::Fixed,
                None,
                pos[0].wrapping_mul(31) ^ pos[2] ^ i as i32,
            );
            draw.pop();
        }
        return;
    }

    if block == rules::BLOCK_ITEM_PIPE {
        let Some(st) = load_state(entity) else {
            return;
        };
        if st.belt_item.is_empty() {
            return;
        }
        let t = ((st.progress as f32 + if st.progress < 10 { draw.partial } else { 0.0 }) / 10.0).clamp(0.0, 1.0);
        let from_off = rules::offset_from_dir_index(st.pipe_from);
        let to_off = rules::offset_from_dir_index(st.pipe_to);
        let (ix, iy, iz) = if t < 0.5 {
            let u = t * 2.0;
            (
                0.5 + from_off[0] as f32 * 0.5 * (1.0 - u),
                0.5 + from_off[1] as f32 * 0.5 * (1.0 - u),
                0.5 + from_off[2] as f32 * 0.5 * (1.0 - u),
            )
        } else {
            let u = (t - 0.5) * 2.0;
            (
                0.5 + to_off[0] as f32 * 0.5 * u,
                0.5 + to_off[1] as f32 * 0.5 * u,
                0.5 + to_off[2] as f32 * 0.5 * u,
            )
        };
        let stack = ItemStack::new(&st.belt_item, 1);
        draw.push();
        draw.translate([ix, iy, iz]);
        draw.rotate_about([0.0, 1.0, 0.0], (draw.game_time as f32 + draw.partial) * 4.0, [0.0, 0.0, 0.0]);
        draw.scale([0.30, 0.30, 0.30]);
        let _ = game.native_draw_item(
            draw,
            &stack,
            DisplayContext::Fixed,
            None,
            pos[0].wrapping_mul(31) ^ pos[2],
        );
        draw.pop();
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
            let fstack = ItemStack::new(first_f, 1);
            draw.push();
            draw.translate([0.5, 0.72, 0.5]);
            draw.scale([0.36, 0.36, 0.36]);
            let _ = game.native_draw_item(
                draw,
                &fstack,
                DisplayContext::Fixed,
                None,
                pos[0].wrapping_mul(17) ^ pos[2],
            );
            draw.pop();
        }
    }
    if st.belt_item.is_empty() {
        return;
    }
    let facing = game
        .native_block_state_property(state, "facing")
        .unwrap_or("north");
    let shape = game
        .native_block_state_property(state, "shape")
        .unwrap_or("straight");

    let t = ((st.progress as f32 + if st.enabled && st.progress < max_ticks { draw.partial } else { 0.0 })
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

    let stack = ItemStack::new(&st.belt_item, 1);
    draw.push();
    draw.translate([ix, iy, iz]);
    draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
    draw.scale([0.46, 0.46, 0.46]);
    let _ = game.native_draw_item(
        draw,
        &stack,
        DisplayContext::Fixed,
        None,
        pos[0].wrapping_mul(31) ^ pos[2],
    );
    draw.pop();
}

/// 3D World Primitives (`native_world_primitives`):
/// Renders the BuildCraft-style 3D Quarry Frame, Moving X/Z Gantry Bridge, Drill Head,
/// glowing Mining Laser beam, and floating Quarry Telemetry Hologram!
pub(in crate::game) fn world_primitives(
    game: &Game<'_>,
    partial: f32,
    camera: [f64; 3],
    primitives: &mut WorldPrimitives,
) {
    if !game.native_on(rules::ID) {
        return;
    }

    for (&(x, y, z), be) in &game.world.block_entities {
        let BlockEntity::Mod(entity) = be else {
            continue;
        };
        let dx = x as f64 + 0.5 - camera[0];
        let dy = y as f64 + 0.5 - camera[1];
        let dz = z as f64 + 0.5 - camera[2];
        if dx * dx + dy * dy + dz * dz > 64.0 * 64.0 {
            continue;
        }

    if entity.id == rules::BLOCK_INDUSTRIAL_SILO {
            let st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::IndustrialSilo));
            let pos = [x, y, z];
            let facing = game
                .native_block_property(pos, "facing")
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

        if entity.id == rules::BLOCK_TESSERACT {
            let pulse = ((game.world.game_time as f64 + partial as f64) * 0.15).sin() * 0.04;
            let r = 0.22 + pulse;
            primitives.filled_box(
                [x as f64 + 0.5 - r, y as f64 + 0.5 - r, z as f64 + 0.5 - r],
                [x as f64 + 0.5 + r, y as f64 + 0.5 + r, z as f64 + 0.5 + r],
                0xCC38_E0FF,
            );
            primitives.box_outline([x as f64 + 0.12, y as f64 + 0.12, z as f64 + 0.12], [x as f64 + 0.88, y as f64 + 0.88, z as f64 + 0.88], 0xFF78_66FF, 1.8);
            continue;
        }

        if entity.id == rules::BLOCK_GEOTHERMAL {
            let st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::GeothermalGenerator));
            if st.fluid_amount > 0 {
                primitives.filled_box(
                    [x as f64 + 0.24, y as f64 + 0.96, z as f64 + 0.24],
                    [x as f64 + 0.76, y as f64 + 1.03, z as f64 + 0.76],
                    0xEEFF_6618,
                );
            }
            continue;
        }

        if entity.id == rules::BLOCK_STEAM_TURBINE {
            let st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::SteamTurbine));
            if st.fluid_amount > 0 {
                let spin = ((game.world.game_time / 2) % 2) == 0;
                let (rx, rz) = if spin { (0.32, 0.08) } else { (0.08, 0.32) };
                primitives.filled_box(
                    [x as f64 + 0.5 - rx, y as f64 + 0.98, z as f64 + 0.5 - rz],
                    [x as f64 + 0.5 + rx, y as f64 + 1.06, z as f64 + 0.5 + rz],
                    0xFFD0_F4FF,
                );
            }
            continue;
        }

        if entity.id == rules::BLOCK_FLUID_PIPE {
            let st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::FluidPipe));
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

        if entity.id == rules::BLOCK_FLUID_TANK {
            let st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::FluidTank));
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

        if entity.id == rules::BLOCK_HARVESTER {
            let st = load_state(entity)
                .unwrap_or_else(|| MachineState::new(MachineKind::AutoHarvester));
            let pos = [x, y, z];
            let facing = game
                .native_block_property(pos, "facing")
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

        if rules::is_conveyor_block(&entity.id) {
            if let Some(st) = load_state(entity) {
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

        if !rules::is_quarry_block(&entity.id) {
            continue;
        }

        let (tier_radius, _, _) = rules::quarry_tier(&entity.id);
        let st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::Quarry));
        let eff_radius = st.quarry_radius.max(tier_radius);
        let pos = [x, y, z];
        let facing = game
            .native_block_property(pos, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let (min_x, max_x, min_z, max_z) =
            rules::quarry_area_bounds(pos, &facing, eff_radius);

        let x0 = min_x as f64;
        let x1 = (max_x + 1) as f64;
        let z0 = min_z as f64;
        let z1 = (max_z + 1) as f64;
        let base_y = y as f64;
        let top_y = y as f64 + 3.25;
        let (beam_col_a, beam_col_b, accent_col) = match entity.id.as_str() {
            rules::BLOCK_QUARRY_MK2 => (0xFF4A_76A4, 0xFFB8_8E2C, 0xFF5A_8EC0),
            rules::BLOCK_QUARRY_MK3 => (0xFF3E_8292, 0xFF1C_2026, 0xFF72_669E),
            _ => (0xFFC8_9826, 0xFF20_242B, 0xFFD0_8228),
        };
        let steel_dark = 0xFF26_2A32;
        let steel_mid = 0xFF46_4E5C;
        let steel_light = 0xFF6E_7888;

        // 1. 3D Industrial Construction Scaffolding (non-overlapping segmented beams)
        let draw_hazard_beam = |prims: &mut WorldPrimitives, a: [f64; 3], b: [f64; 3], r: f64| {
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

        let anim_t = game.world.game_time as f64 + partial as f64;
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
    if let (Some((held_id, _, _)), Some(player)) = (game.native_held(), &game.player) {
        let is_belt = rules::is_conveyor_block(&held_id);
        let is_splitter = held_id == rules::BLOCK_ITEM_SPLITTER;
        if is_belt || is_splitter {
            let yaw_rad = (game.yaw as f64).to_radians();
            let pitch_rad = (game.pitch as f64).to_radians();
            let dir = [
                -yaw_rad.sin() * pitch_rad.cos(),
                -pitch_rad.sin(),
                yaw_rad.cos() * pitch_rad.cos(),
            ];
            let eye = [
                player.position.x,
                player.position.y + f64::from(player.dimensions.eye_height),
                player.position.z,
            ];
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
                    let blk = game.native_block(cell);
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
                let is_crouching = player.pose == horizon_client::pose::Pose::Crouching;
                let forward_facing = cardinal_facing_from_yaw(game.yaw);
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

fn compute_belt_shape(game: &Game<'_>, pos: [i32; 3], facing: &str) -> &'static str {
    let (front_off, back_off) = rules::conveyor_offsets(facing);
    let front_up = [pos[0] + front_off[0], pos[1] + 1, pos[2] + front_off[2]];
    let back_up = [pos[0] + back_off[0], pos[1] + 1, pos[2] + back_off[2]];
    let front_pos = [pos[0] + front_off[0], pos[1], pos[2] + front_off[2]];

    if rules::is_conveyor_block(&game.native_block(front_up)) {
        return "up";
    }
    if rules::is_conveyor_block(&game.native_block(back_up)) {
        return "down";
    }

    let is_linkable_output = |p: [i32; 3], into_dir: [i32; 3]| -> bool {
        let b = game.native_block(p);
        if rules::is_conveyor_block(&b) {
            let target_facing = game
                .native_block_property(p, "facing")
                .unwrap_or_else(|| "north".to_owned());
            let (target_front, _) = rules::conveyor_offsets(&target_facing);
            // Do not turn into a belt that is pointing directly into us
            target_front != [-into_dir[0], 0, -into_dir[2]]
        } else {
            b == rules::BLOCK_ITEM_PIPE || b == rules::BLOCK_ITEM_SPLITTER || is_item_container_block(&b)
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
pub(in crate::game) fn place_block(game: &mut Game<'_>, pos: [i32; 3]) {
    if !game.native_on(rules::ID) {
        return;
    }
    let block_id = game.native_block(pos);
    if rules::is_quarry_block(&block_id) {
        let (tier_radius, _, _) = rules::quarry_tier(&block_id);
        game.native_with_mod_block_entity(pos, |entity| {
            let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::Quarry));
            st.quarry_radius = tier_radius;
            store_state(entity, &st);
        });
        game.native_block_entity_sync(pos);
        return;
    }
    let is_belt = rules::is_conveyor_block(&block_id);
    let is_splitter = block_id == rules::BLOCK_ITEM_SPLITTER;
    if !is_belt && !is_splitter {
        return;
    }

    let is_crouching = game
        .player
        .as_ref()
        .is_some_and(|p| p.pose == horizon_client::pose::Pose::Crouching);

    // `minecraft:furnace` template faces towards the player; flip so normal placement goes FORWARD (away from player),
    // and Shift (crouching) reverses direction!
    let raw_facing = game
        .native_block_property(pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let facing = if is_crouching {
        raw_facing.clone()
    } else {
        opposite_facing(&raw_facing).to_owned()
    };
    if facing != raw_facing {
        game.native_set_block_property(pos, "facing", &facing);
    }

    if !is_belt {
        return;
    }

    let desired_shape = compute_belt_shape(game, pos, &facing);
    if desired_shape != "straight" {
        game.native_set_block_property(pos, "shape", desired_shape);
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
        if !rules::is_conveyor_block(&game.native_block(nb)) {
            continue;
        }
        let nb_facing = game
            .native_block_property(nb, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let nb_shape = game
            .native_block_property(nb, "shape")
            .unwrap_or_else(|| "straight".to_owned());
        if nb_shape == "straight" {
            let updated = compute_belt_shape(game, nb, &nb_facing);
            if updated != nb_shape {
                game.native_set_block_property(nb, "shape", updated);
            }
        }
    }
}

/// Drops any item currently travelling inside an `item_pipe` or riding on a `conveyor_belt` when broken.
pub(in crate::game) fn destroy_block(game: &mut Game<'_>, pos: [i32; 3]) {
    if !game.native_on(rules::ID) {
        return;
    }
    let block_id = game.native_block(pos);
    if block_id != rules::BLOCK_ITEM_PIPE && !rules::is_conveyor_block(&block_id) {
        return;
    }
    let mut dropped_item: Option<(String, u32)> = None;
    game.native_with_mod_block_entity(pos, |entity| {
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
        game.pop_resource(pos, ItemStack::new(&item_id, count));
    }
}

/// Inspects networks, harvests sap from Rubber Tree Sap Cavities, grows Rubber Saplings, interacts with Conveyor Belts (no GUI), configures Item Splitter filter frames, rotates/dismantles machines with Wrench, or uses tools.
pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: [i32; 3], face: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let block_id = game.native_block(pos);
    let held_info = game.native_held();
    let is_crouching = game
        .player
        .as_ref()
        .is_some_and(|p| p.pose == horizon_client::pose::Pose::Crouching);

    // Portable Energy Scanner: Inspect live telemetry of any Rusty Factory block
    if held_info.as_ref().is_some_and(|(id, _, _)| id == rules::ITEM_ENERGY_SCANNER)
        && block_id.starts_with("rusty_factory:")
    {
        *canceled = true;
        let mut msg = format!("📡 [{block_id}]");
        game.native_with_mod_block_entity(pos, |entity| {
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
        if let Some(who) = game.native_acting_player() {
            game.native_action_bar(who.uuid, Text::literal(msg));
        }
        return;
    }

    // Industrial Silo: Front screen = deposit / Shift-lock item / withdraw; Side faces = place blocks or open 1-slot GUI
    if block_id == rules::BLOCK_INDUSTRIAL_SILO {
        let facing = game
            .native_block_property(pos, "facing")
            .unwrap_or_else(|| "north".to_owned());
        let (front_off, _) = rules::conveyor_offsets(&facing);
        let is_front = face == front_off;

        if let Some((ref held_id, held_count, _)) = held_info {
            if let Some(bonus) = rules::silo_upgrade_bonus(held_id) {
                *canceled = true;
                let mut new_cap = 0;
                game.native_with_mod_block_entity(pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::IndustrialSilo));
                    st.silo_capacity = st.silo_capacity.max(2_048).saturating_add(bonus);
                    new_cap = st.silo_capacity;
                    store_state(entity, &st);
                });
                if !game.creative {
                    if let Some(held) = game.held_mut() {
                        if held.count > 1 { held.count -= 1; } else { *game.held_mut() = None; }
                    }
                }
                game.native_block_entity_sync(pos);
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal(format!("📦 [Silo Industrial] Capacidad mejorada a {new_cap} ítems")));
                }
                return;
            }
            if is_front {
                *canceled = true;
                let item_id = held_id.clone();
                let mut deposited = 0u32;
                let mut total_now = 0u32;
                let mut cap_now = 2_048u32;
                let mut mismatch = false;
                game.native_with_mod_block_entity(pos, |entity| {
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
                    if !game.creative {
                        if let Some(held) = game.held_mut() {
                            if held.count > deposited { held.count -= deposited; } else { *game.held_mut() = None; }
                        }
                    }
                    game.native_block_entity_sync(pos);
                } else if is_crouching {
                    game.native_block_entity_sync(pos);
                }
                if let Some(who) = game.native_acting_player() {
                    let msg = if mismatch {
                        format!("⚠ [Silo] Bloqueado a otro ítem ({total_now} / {cap_now})")
                    } else {
                        format!("📦 [Silo] {total_now} / {cap_now} ({item_id})")
                    };
                    game.native_action_bar(who.uuid, Text::literal(msg));
                }
                return;
            } else if held_id != rules::ITEM_WRENCH {
                // Side/top/back face with a block in hand: place that block at `pos + face`!
                if let Some(place_state) = game.native_default_state(held_id) {
                    *canceled = true;
                    let target = [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]];
                    let target_blk = game.native_block(target);
                    if matches!(
                        target_blk.as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water"
                    ) {
                        let dim = game.native_dimension();
                        if game.native_set_block(dim, target, place_state, 3) {
                            let forward_facing = cardinal_facing_from_yaw(game.yaw);
                            let raw_facing = opposite_facing(forward_facing);
                            let _ = game.native_set_block_property(target, "facing", raw_facing);
                            place_block(game, target);
                            if !game.creative {
                                if let Some(held) = game.held_mut() {
                                    if held.count > 1 { held.count -= 1; } else { *game.held_mut() = None; }
                                }
                            }
                        }
                    }
                    return;
                }
            }
        } else if is_front {
            // Empty hand on front screen: withdraw 1 stack (up to 64)
            *canceled = true;
            let mut taken: Option<(String, u32)> = None;
            let mut total_now = 0u32;
            let mut cap_now = 2_048u32;
            game.native_with_mod_block_entity(pos, |entity| {
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
                game.native_give_items(&iid, amt);
                game.native_block_entity_sync(pos);
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal(format!("📦 [Silo] Retirado x{amt} | Restante: {total_now} / {cap_now}")));
                }
            }
            return;
        }
        // Empty hand on non-front face falls through so the 1-slot Silo GUI opens!
    }

    // Equipment Upgrade Station: load a module_* or install loaded module onto held tech tool/armor
    if block_id == rules::BLOCK_UPGRADE_STATION {
        *canceled = true;
        if let Some((ref held_id, _, _)) = held_info {
            let mod_code = match held_id.as_str() {
                rules::ITEM_MODULE_NIGHT_VISION => Some("nv"),
                rules::ITEM_MODULE_MAGNET => Some("mag"),
                rules::ITEM_MODULE_EFFICIENCY => Some("eff"),
                _ => None,
            };
            if let Some(code) = mod_code {
                game.native_with_mod_block_entity(pos, |entity| {
                    let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::UpgradeStation));
                    st.belt_item = code.to_owned();
                    store_state(entity, &st);
                });
                if !game.creative {
                    if let Some(held) = game.held_mut() {
                        if held.count > 1 { held.count -= 1; } else { *game.held_mut() = None; }
                    }
                }
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal(format!("🔧 [Estación de Mejora] Módulo '{code}' cargado. Clic con armadura o taladro para instalar.")));
                }
                return;
            }
            if rules::battery_capacity(held_id).is_some() {
                let mut loaded_mod = String::new();
                game.native_with_mod_block_entity(pos, |entity| {
                    if let Some(mut st) = load_state(entity) {
                        if !st.belt_item.is_empty() {
                            loaded_mod = st.belt_item.clone();
                            st.belt_item.clear();
                            store_state(entity, &st);
                        }
                    }
                });
                if !loaded_mod.is_empty() {
                    if let Some(held) = game.held_mut() {
                        add_item_module(held, &loaded_mod);
                    }
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(who.uuid, Text::literal(format!("✅ [Estación de Mejora] Módulo '{loaded_mod}' instalado en tu equipo")));
                    }
                } else if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal("ℹ [Estación de Mejora] Primero cargá un Módulo haciendo clic con él."));
                }
                return;
            }
        }
        return;
    }

    // Geothermal Generator: direct lava_bucket feed
    if block_id == rules::BLOCK_GEOTHERMAL
        && held_info.as_ref().is_some_and(|(id, _, _)| id == "minecraft:lava_bucket")
    {
        *canceled = true;
        let mut ok = false;
        game.native_with_mod_block_entity(pos, |entity| {
            let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::GeothermalGenerator));
            if st.fluid_amount + 1000 <= rules::FLUID_TANK_MAX_MB {
                st.fluid_kind = "lava".to_owned();
                st.fluid_amount += 1000;
                store_state(entity, &st);
                ok = true;
            }
        });
        if ok && !game.creative {
            if let Some(held) = game.held_mut() {
                *held = ItemStack::new("minecraft:bucket", 1);
            }
            game.native_block_entity_sync(pos);
        }
        return;
    }

    // Create-style per-face filter configuration on Item Splitter (no GUI!)
    if block_id == rules::BLOCK_ITEM_SPLITTER {
        let is_wrench = held_info.as_ref().is_some_and(|(id, _, _)| id == rules::ITEM_WRENCH);
        if !is_wrench {
            let facing = game
                .native_block_property(pos, "facing")
                .unwrap_or_else(|| "north".to_owned());
            if let Some(side) = rules::splitter_side_for_face(&facing, face) {
                if !is_crouching {
                    *canceled = true;
                    let side_name = match side {
                        0 => "Izquierda",
                        1 => "Frente",
                        _ => "Derecha",
                    };
                    if let Some((ref held_id, _, _)) = held_info {
                        let filter_item = held_id.clone();
                        game.native_with_mod_block_entity(pos, |entity| {
                            let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ItemSplitter));
                            st.splitter_filters[side] = filter_item.clone();
                            store_state(entity, &st);
                        });
                        game.native_block_entity_sync(pos);
                        if let Some(who) = game.native_acting_player() {
                            game.native_action_bar(
                                who.uuid,
                                Text::literal(format!("🔍 [Divisor] Filtro {side_name}: {filter_item}")),
                            );
                        }
                    } else {
                        game.native_with_mod_block_entity(pos, |entity| {
                            let mut st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::ItemSplitter));
                            st.splitter_filters[side].clear();
                            store_state(entity, &st);
                        });
                        game.native_block_entity_sync(pos);
                        if let Some(who) = game.native_acting_player() {
                            game.native_action_bar(
                                who.uuid,
                                Text::literal(format!("🧹 [Divisor] Filtro {side_name} limpiado")),
                            );
                        }
                    }
                    return;
                }
            }
        }
    }

    // Direct hand interaction on Conveyor Belts (NO GUI!)
    if let Some((_, max_batch)) = rules::conveyor_tier(&block_id) {
        if let Some((ref held_id, _, _)) = held_info {
            if held_id == rules::ITEM_BELT_FILTER || held_id == rules::ITEM_BELT_FILTER_ADV {
                *canceled = true;
                let is_adv = held_id == rules::ITEM_BELT_FILTER_ADV;
                let changed = game.native_with_mod_block_entity(pos, |entity| {
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
                    if !game.creative {
                        if let Some(held) = game.held_mut() {
                            if held.count > 1 {
                                held.count -= 1;
                            } else {
                                *game.held_mut() = None;
                            }
                        }
                    }
                    game.native_block_entity_sync(pos);
                    if let Some(who) = game.native_acting_player() {
                        let msg = if is_adv {
                            "🔍 [Cinta] Filtro Avanzado acoplado (Shift+Mano Vacía alterna Lista Blanca/Negra)"
                        } else {
                            "🔍 [Cinta] Filtro acoplado. Clic con un ítem para configurarlo."
                        };
                        game.native_action_bar(who.uuid, Text::literal(msg));
                    }
                }
                return;
            }
            if held_id == rules::ITEM_WRENCH {
                *canceled = true;
                if is_crouching {
                    let mut had_filter = false;
                    game.native_with_mod_block_entity(pos, |entity| {
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
                        game.native_give_items(rules::ITEM_BELT_FILTER, 1);
                        game.native_block_entity_sync(pos);
                        if let Some(who) = game.native_acting_player() {
                            game.native_action_bar(who.uuid, Text::literal("🔧 [Cinta] Filtro retirado"));
                        }
                        return;
                    }
                    destroy_block(game, pos);
                    game.native_give_items(&block_id, 1);
                    game.native_run_command(&format!(
                        "setblock {} {} {} minecraft:air destroy",
                        pos[0], pos[1], pos[2]
                    ));
                } else {
                    let current = game
                        .native_block_property(pos, "shape")
                        .unwrap_or_else(|| "straight".to_owned());
                    let next = match current.as_str() {
                        "straight" => "turn_left",
                        "turn_left" => "turn_right",
                        "turn_right" => "up",
                        "up" => "down",
                        _ => "straight",
                    };
                    game.native_set_block_property(pos, "shape", next);
                    if let Some(who) = game.native_acting_player() {
                        let label = match next {
                            "turn_left" => "Curva Izquierda",
                            "turn_right" => "Curva Derecha",
                            "up" => "Subida (Rampa)",
                            "down" => "Bajada (Rampa)",
                            _ => "Recta",
                        };
                        game.native_action_bar(
                            who.uuid,
                            Text::literal(format!("🔧 [Cinta] Forma: {label}")),
                        );
                    }
                }
                return;
            }
            // Allow placing filters (Item Splitter), other belts, pipes, or extractors directly against belts!
            let is_placeable_logistics = held_id == rules::BLOCK_ITEM_SPLITTER
                || rules::is_conveyor_block(held_id)
                || held_id == rules::BLOCK_ITEM_PIPE
                || held_id == rules::BLOCK_EXTRACTOR;
            if is_placeable_logistics {
                *canceled = true;
                let target = [pos[0] + face[0], pos[1] + face[1], pos[2] + face[2]];
                let target_blk = game.native_block(target);
                if matches!(
                    target_blk.as_str(),
                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" | "minecraft:water"
                ) {
                    if let Some(place_state) = game.native_default_state(held_id) {
                        let dim = game.native_dimension();
                        if game.native_set_block(dim, target, place_state, 3) {
                            let forward_facing = cardinal_facing_from_yaw(game.yaw);
                            let raw_facing = opposite_facing(forward_facing);
                            let _ = game.native_set_block_property(target, "facing", raw_facing);
                            place_block(game, target);
                            if !game.creative {
                                if let Some(held) = game.held_mut() {
                                    if held.count > 1 {
                                        held.count -= 1;
                                    } else {
                                        *game.held_mut() = None;
                                    }
                                }
                            }
                            game.native_run_command(&format!(
                                "playsound minecraft:block.metal.place block @a {} {} {} 0.8 1.1",
                                target[0] as f64 + 0.5,
                                target[1] as f64 + 0.5,
                                target[2] as f64 + 0.5
                            ));
                        }
                    }
                }
                return;
            }
            // If this belt has an inline filter attached and needs configuration (empty filter or crouching with item), set filter!
            let mut filter_set = false;
            let item_to_filter = held_id.clone();
            let filter_changed = game.native_with_mod_block_entity(pos, |entity| {
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
                *canceled = true;
                game.native_block_entity_sync(pos);
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(
                        who.uuid,
                        Text::literal(format!("🔍 [Cinta] Filtrando: {item_to_filter}")),
                    );
                }
                return;
            }
            // Place 1 item from player's hand onto the conveyor belt (unless sneaking to place blocks!)
            if !is_crouching {
                *canceled = true;
                let item_to_put = held_id.clone();
                let mut placed = false;
                let changed = game.native_with_mod_block_entity(pos, |entity| {
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
                    if !game.creative {
                        if let Some(held) = game.held_mut() {
                            if held.count > 1 {
                                held.count -= 1;
                            } else {
                                *game.held_mut() = None;
                            }
                        }
                    }
                    game.native_block_entity_sync(pos);
                }
                return;
            }
        } else {
            // Empty hand right-click: if crouching on a filtered belt, toggle Whitelist/Blacklist!
            if is_crouching {
                let mut toggled_mode: Option<bool> = None;
                let toggled = game.native_with_mod_block_entity(pos, |entity| {
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
                    *canceled = true;
                    game.native_block_entity_sync(pos);
                    if let (Some(who), Some(bl)) = (game.native_acting_player(), toggled_mode) {
                        let label = if bl { "Lista Negra (bloquea filtro)" } else { "Lista Blanca (solo filtro)" };
                        game.native_action_bar(who.uuid, Text::literal(format!("🔍 [Cinta] Modo: {label}")));
                    }
                    return;
                }
            }
            // Empty hand right-click: collect carried item from the conveyor belt!
            *canceled = true;
            let mut taken: Option<(String, u32)> = None;
            let changed = game.native_with_mod_block_entity(pos, |entity| {
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
                    game.native_give_items(&item_id, count);
                }
                game.native_block_entity_sync(pos);
            }
            return;
        }
    }

    if let Some((ref held_id, _, _)) = held_info {
        // 1. Wrench: Rotate machine facing (normal click) or dismantle into inventory (crouch + click)
        if held_id == rules::ITEM_WRENCH {
            if block_id == rules::BLOCK_ITEM_PIPE {
                *canceled = true;
                if is_crouching {
                    destroy_block(game, pos);
                    game.native_give_items(&block_id, 1);
                    game.native_run_command(&format!(
                        "setblock {} {} {} minecraft:air destroy",
                        pos[0], pos[1], pos[2]
                    ));
                } else {
                    let targets = rules::find_connected_item_targets(
                        pos,
                        |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
                        |p| is_item_container_block(&game.native_block(p)) || rules::is_conveyor_block(&game.native_block(p)),
                    );
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(
                            who.uuid,
                            Text::literal(format!("📦 [Tubería de Ítems] Red: {} destinos conectados", targets.len())),
                        );
                    }
                }
                return;
            }
            if let Some(kind) = MachineKind::from_block_id(&block_id) {
                *canceled = true;
                if is_crouching {
                    game.native_give_items(&block_id, 1);
                    game.native_run_command(&format!(
                        "setblock {} {} {} minecraft:air destroy",
                        pos[0], pos[1], pos[2]
                    ));
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(
                            who.uuid,
                            Text::literal(format!("🔧 [Llave Inglesa] {} desmontada", lang::get(kind.title_key()))),
                        );
                    }
                } else {
                    let current = game
                        .native_block_property(pos, "facing")
                        .unwrap_or_else(|| "north".to_owned());
                    let next = match current.as_str() {
                        "north" => "east",
                        "east" => "south",
                        "south" => "west",
                        _ => "north",
                    };
                    game.native_set_block_property(pos, "facing", next);
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(
                            who.uuid,
                            Text::literal(format!("🔧 [Llave Inglesa] Rotada hacia: {next}")),
                        );
                    }
                }
                return;
            }

            if rules::is_cable(&block_id) {
                *canceled = true;
                let limit = rules::cable_transfer_limit(&block_id);
                let consumers = rules::find_connected_consumers(
                    pos,
                    |p| rules::is_cable(&game.native_block(p)),
                    |p| MachineKind::from_block_id(&game.native_block(p)).is_some(),
                );
                let cable_name = match block_id.as_str() {
                    rules::BLOCK_CABLE_WHITE => "Cable Blanco [Tier 1]",
                    rules::BLOCK_CABLE_GOLD => "Cable Amarillo [Tier 3]",
                    rules::BLOCK_CABLE_RED => "Cable Rojo [Tier 4]",
                    _ => "Cable Naranja / Cobre [Tier 2]",
                };
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(
                        who.uuid,
                        Text::literal(format!("⚡ [{cable_name}] Capacidad: {limit} FE/t | {} máquinas conectadas", consumers.len())),
                    );
                }
                return;
            }

            if block_id == rules::BLOCK_ITEM_PIPE {
                *canceled = true;
                let targets = rules::find_connected_item_targets(
                    pos,
                    |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
                    |p| is_item_container_block(&game.native_block(p)),
                );
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(
                        who.uuid,
                        Text::literal(format!("📦 [Tubería de Ítems] Red: {} inventarios conectados", targets.len())),
                    );
                }
                return;
            }

            if block_id == rules::BLOCK_FLUID_PIPE {
                *canceled = true;
                let targets = rules::find_connected_item_targets(
                    pos,
                    |p| game.native_block(p) == rules::BLOCK_FLUID_PIPE,
                    |p| matches!(game.native_block(p).as_str(), rules::BLOCK_FLUID_TANK | rules::BLOCK_STEAM_BOILER | rules::BLOCK_FLUID_PUMP),
                );
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(
                        who.uuid,
                        Text::literal(format!("🚰 [Tubería de Fluidos] Red hidráulica: {} tanques conectados", targets.len())),
                    );
                }
                return;
            }
        }

        // 2. Treetap: Harvest extra sap & saplings from Rubber Sap Log with durability consumption
        if held_id == rules::ITEM_TREETAP && block_id == rules::BLOCK_RUBBER_SAP_LOG {
            *canceled = true;
            let has_sap = game.native_block_property(pos, "lit").as_deref() == Some("true");
            if has_sap {
                game.native_set_block_property(pos, "lit", "false");
                game.native_give_items(rules::ITEM_RAW_SAP, 3);
                game.native_give_items(rules::BLOCK_RUBBER_SAPLING, 1);
                game.damage_held(1);
                game.native_run_command("playsound minecraft:item.honey_bottle.drink player @s ~ ~ ~ 1.0 1.2");
                game.native_run_command("particle minecraft:dripping_honey ~ ~0.5 ~ 0.2 0.2 0.2 0.05 12");
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(
                        who.uuid,
                        Text::literal("💧 [Espita de Madera] ¡Savia recolectada! (-1 durabilidad)"),
                    );
                }
            } else if let Some(who) = game.native_acting_player() {
                game.native_action_bar(
                    who.uuid,
                    Text::literal("⏳ El hueco de savia se está regenerando..."),
                );
            }
            return;
        }

        // 3. Mining Drill (MK1 3x3, MK2 3x3, MK3 5x5): Crouch toggles area/1x1; Click excavates using FE
        if let Some((tier, rad_half, base_cost)) = rules::drill_tier(held_id) {
            static DRILL_MODE_3X3: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
            let area_label = if rad_half >= 2 { "5x5" } else { "3x3" };
            if is_crouching {
                *canceled = true;
                let prev = DRILL_MODE_3X3.fetch_xor(true, std::sync::atomic::Ordering::Relaxed);
                let mode_now = !prev;
                if let Some(held) = game.held_mut() {
                    set_drill_3x3(held, mode_now);
                }
                if let Some(who) = game.native_acting_player() {
                    let msg = if mode_now {
                        format!("⚡ [Taladro MK{tier}] Modo: {area_label} Excavación Amplia")
                    } else {
                        format!("⚡ [Taladro MK{tier}] Modo: 1x1 Precisión")
                    };
                    game.native_action_bar(who.uuid, Text::literal(msg));
                }
                return;
            } else if DRILL_MODE_3X3.load(std::sync::atomic::Ordering::Relaxed) {
                *canceled = true;
                let eff_mod = game.held().is_some_and(|h| has_item_module(h, "eff"));
                let cost = if eff_mod { (base_cost / 2).max(5) } else { base_cost };
                if !game.creative && !consume_player_energy(game, cost) {
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(
                            who.uuid,
                            Text::literal("⚠ [Taladro] Sin energía (recarga o lleva una Batería en el inventario)"),
                        );
                    }
                    return;
                }
                let [cx, cy, cz] = pos;
                let (u_axes, v_axes): ([i32; 3], [i32; 3]) = if face[1] != 0 {
                    ([1, 0, 0], [0, 0, 1])
                } else if face[2] != 0 {
                    ([1, 0, 0], [0, 1, 0])
                } else {
                    ([0, 0, 1], [0, 1, 0])
                };
                for u in -rad_half..=rad_half {
                    for v in -rad_half..=rad_half {
                        let p = [
                            cx + u_axes[0] * u + v_axes[0] * v,
                            cy + u_axes[1] * u + v_axes[1] * v,
                            cz + u_axes[2] * u + v_axes[2] * v,
                        ];
                        let b = game.native_block(p);
                        if !b.is_empty() && b != "minecraft:air" && b != "minecraft:bedrock" && !b.starts_with("minecraft:barrier") {
                            game.native_run_command(&format!("setblock {} {} {} minecraft:air destroy", p[0], p[1], p[2]));
                        }
                    }
                }
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal(format!("⚡ [Taladro MK{tier}] {area_label} Excavado (-{cost} FE)")));
                }
                return;
            }
        }

        // 3a. Electric Chainsaw: Fell entire connected tree (up to 64 logs) using FE
        if held_id == rules::ITEM_CHAINSAW {
            let blk = game.native_block(pos);
            if blk.ends_with("_log") || blk.ends_with("_stem") || blk == rules::BLOCK_RUBBER_LOG || blk == rules::BLOCK_RUBBER_SAP_LOG {
                *canceled = true;
                if !game.creative && !consume_player_energy(game, 30) {
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(who.uuid, Text::literal("⚠ [Motosierra] Sin energía (30 FE)"));
                    }
                    return;
                }
                let mut visited = std::collections::HashSet::new();
                let mut q = std::collections::VecDeque::new();
                q.push_back(pos);
                visited.insert(pos);
                let mut felled = 0;
                while let Some(cur) = q.pop_front() {
                    if felled >= 64 {
                        break;
                    }
                    let cb = game.native_block(cur);
                    if cb.ends_with("_log") || cb.ends_with("_stem") || cb == rules::BLOCK_RUBBER_LOG || cb == rules::BLOCK_RUBBER_SAP_LOG {
                        game.native_run_command(&format!("setblock {} {} {} minecraft:air destroy", cur[0], cur[1], cur[2]));
                        felled += 1;
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
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal(format!("🪓 [Motosierra] Árbol talado ({felled} troncos, -30 FE)")));
                }
                return;
            }
        }

        // 3b. Portable Batteries & Dorsal Battery Pack: Inspect charge in hand
        if rules::is_battery_item(held_id) {
            *canceled = true;
            let cap = rules::battery_capacity(held_id).unwrap_or(50_000);
            let cur = game.held().map_or(0, get_item_energy);
            if let Some(who) = game.native_acting_player() {
                game.native_action_bar(
                    who.uuid,
                    Text::literal(format!("🔋 [Batería] Carga: {cur} / {cap} FE (recarga tu equipo automáticamente)")),
                );
            }
            return;
        }

        // 3b2. Jetpack MK2 (Hover): Shift toggles Hover mode, Click propels
        if held_id == rules::ITEM_JETPACK_MK2 {
            *canceled = true;
            if is_crouching {
                let mut now_hover = false;
                if let Some(held) = game.held_mut() {
                    now_hover = !is_jetpack_hover(held);
                    set_jetpack_hover(held, now_hover);
                }
                if let Some(who) = game.native_acting_player() {
                    let m = if now_hover { "ACTIVADO" } else { "DESACTIVADO" };
                    game.native_action_bar(who.uuid, Text::literal(format!("🚀 [Jetpack MK2] Modo Hover {m}")));
                }
            } else {
                game.native_run_command("playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.4 1.15");
                game.native_run_command("effect give @s minecraft:levitation 1 16 true");
                game.native_run_command("effect give @s minecraft:slow_falling 8 0 true");
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal("🚀 [Jetpack MK2] Propulsión Avanzada"));
                }
            }
            return;
        }

        // 3c. Industrial Armor Set: Equip directly
        if matches!(
            held_id.as_str(),
            rules::ITEM_INDUSTRIAL_HELMET
                | rules::ITEM_INDUSTRIAL_CHESTPLATE
                | rules::ITEM_INDUSTRIAL_LEGGINGS
                | rules::ITEM_INDUSTRIAL_BOOTS
        ) {
            *canceled = true;
            let slot = match held_id.as_str() {
                rules::ITEM_INDUSTRIAL_HELMET => "armor.head",
                rules::ITEM_INDUSTRIAL_CHESTPLATE => "armor.chest",
                rules::ITEM_INDUSTRIAL_LEGGINGS => "armor.legs",
                _ => "armor.feet",
            };
            game.native_run_command(&format!("item replace entity @s {slot} with {held_id}"));
            game.native_run_command("playsound minecraft:item.armor.equip_netherite player @s ~ ~ ~ 1.0 1.0");
            if let Some(who) = game.native_acting_player() {
                game.native_action_bar(
                    who.uuid,
                    Text::literal("🛡 [Armadura Industrial] Equipada. Consume FE de tus baterías para darte mejoras."),
                );
            }
            return;
        }

        // 4. Jet Boots: Rocket propulsion boost with flame & smoke exhaust, slow falling to prevent fall damage!
        if held_id == rules::ITEM_JET_BOOTS {
            *canceled = true;
            game.native_run_command("playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.2 1.1");
            game.native_run_command("effect give @s minecraft:levitation 1 10 true");
            game.native_run_command("effect give @s minecraft:slow_falling 5 0 true");
            game.native_run_command("particle minecraft:flame ~ ~ ~ 0.3 0.1 0.3 0.05 20");
            game.native_run_command("particle minecraft:smoke ~ ~ ~ 0.2 0.1 0.2 0.02 15");
            if let Some(who) = game.native_acting_player() {
                game.native_action_bar(
                    who.uuid,
                    Text::literal("🚀 [Botas Cohete] ¡Propulsión Cohete Activada!"),
                );
            }
            return;
        }

        // 5. Jetpack: Sustained High-Altitude Flight
        if held_id == rules::ITEM_JETPACK {
            *canceled = true;
            game.native_run_command("playsound minecraft:entity.firework_rocket.launch player @s ~ ~ ~ 1.5 1.0");
            game.native_run_command("effect give @s minecraft:levitation 1 15 true");
            game.native_run_command("effect give @s minecraft:slow_falling 6 0 true");
            game.native_run_command("particle minecraft:flame ~ ~ ~ 0.4 0.1 0.4 0.08 30");
            game.native_run_command("particle minecraft:campfire_cosy_smoke ~ ~ ~ 0.3 0.1 0.3 0.03 15");
            if let Some(who) = game.native_acting_player() {
                game.native_action_bar(
                    who.uuid,
                    Text::literal("🚀 [Jetpack Industrial] ¡Propulsión Aérea Máxima Activada!"),
                );
            }
            return;
        }

        // 6. Item Magnet: Attracts nearby dropped items and XP
        if held_id == rules::ITEM_ITEM_MAGNET {
            *canceled = true;
            game.native_run_command("playsound minecraft:entity.item.pickup player @s ~ ~ ~ 1.0 1.5");
            game.native_run_command("tp @e[type=item,distance=..12] @s");
            game.native_run_command("tp @e[type=experience_orb,distance=..12] @s");
            game.native_run_command("particle minecraft:portal ~ ~1 ~ 0.5 0.5 0.5 0.1 25");
            if let Some(who) = game.native_acting_player() {
                game.native_action_bar(
                    who.uuid,
                    Text::literal("🧲 [Imán Industrial] ¡Ítems y experiencia cercanos atraídos hacia ti!"),
                );
            }
            return;
        }

        // 7. Mining Laser: High-energy straight plasma beam mining
        if held_id == rules::ITEM_MINING_LASER {
            *canceled = true;
            game.native_run_command("playsound minecraft:entity.guardian.attack player @s ~ ~ ~ 1.2 1.6");
            let [cx, cy, cz] = pos;
            for dist in 0..4 {
                let p = [cx, cy + dist, cz];
                let b = game.native_block(p);
                if !b.is_empty() && b != "minecraft:air" && b != "minecraft:bedrock" && !b.starts_with("minecraft:barrier") {
                    game.native_run_command(&format!("setblock {} {} {} minecraft:air destroy", p[0], p[1], p[2]));
                }
            }
            game.native_run_command(&format!("particle minecraft:sonic_boom {} {} {} 0 0 0 0 1", cx, cy, cz));
            if let Some(who) = game.native_acting_player() {
                game.native_action_bar(
                    who.uuid,
                    Text::literal("⚡ [Láser Minero] Haz de Plasma Concentrado Disparado"),
                );
            }
            return;
        }

        // 8. Solar Helmet: Equip directly to head armor slot
        if held_id == rules::ITEM_SOLAR_HELMET {
            *canceled = true;
            game.native_run_command("item replace entity @s armor.head with rusty_factory:solar_helmet");
            game.native_run_command("playsound minecraft:item.armor.equip_diamond player @s ~ ~ ~ 1.0 1.0");
            if let Some(who) = game.native_acting_player() {
                game.native_action_bar(
                    who.uuid,
                    Text::literal("☀️ [Casco Solar] Equipado. Genera recarga pasiva de día."),
                );
            }
            return;
        }

    }

    if is_crouching {
        return;
    }

    if block_id == rules::BLOCK_RUBBER_SAPLING {
        // Require bone meal in hand to accelerate rubber tree growth (50% chance per bone meal)
        if let Some((held_id, _, _)) = held_info {
            if held_id == "minecraft:bone_meal" {
                *canceled = true;
                game.native_take_items("minecraft:bone_meal", 1);
                if (game.world.game_time & 1) == 0 {
                    grow_rubber_tree(game, pos);
                }
            }
        }
        return;
    }

    if block_id == rules::BLOCK_RUBBER_SAP_LOG {
        if held_info.is_none() {
            let has_sap = game.native_block_property(pos, "lit").as_deref() == Some("true");
            if has_sap {
                *canceled = true;
                game.native_set_block_property(pos, "lit", "false");
                game.native_give_items(rules::ITEM_RAW_SAP, 3);
                if (game.world.game_time & 3) == 0 {
                    game.native_give_items(rules::BLOCK_RUBBER_SAPLING, 1);
                }
            }
        }
        return;
    }

    if block_id == rules::BLOCK_FLUID_TANK {
        if let Some((ref held_id, _, _)) = held_info {
            if held_id == "minecraft:water_bucket" {
                *canceled = true;
                let mut msg = None;
                game.native_with_mod_block_entity(pos, |entity| {
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
                        game.native_take_items("minecraft:water_bucket", 1);
                        game.native_give_items("minecraft:bucket", 1);
                        game.native_set_block_property(pos, "lit", "true");
                    }
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(who.uuid, Text::literal(m));
                    }
                }
                return;
            } else if held_id == "minecraft:lava_bucket" {
                *canceled = true;
                let mut msg = None;
                game.native_with_mod_block_entity(pos, |entity| {
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
                        game.native_take_items("minecraft:lava_bucket", 1);
                        game.native_give_items("minecraft:bucket", 1);
                        game.native_set_block_property(pos, "lit", "true");
                    }
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(who.uuid, Text::literal(m));
                    }
                }
                return;
            } else if held_id == rules::ITEM_RAW_SAP {
                *canceled = true;
                let mut msg = None;
                game.native_with_mod_block_entity(pos, |entity| {
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
                        game.native_take_items(rules::ITEM_RAW_SAP, 1);
                        game.native_set_block_property(pos, "lit", "true");
                    }
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(who.uuid, Text::literal(m));
                    }
                }
                return;
            } else if held_id == "minecraft:bucket" {
                *canceled = true;
                let mut extracted: Option<&'static str> = None;
                let mut remain = 0;
                game.native_with_mod_block_entity(pos, |entity| {
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
                    game.native_take_items("minecraft:bucket", 1);
                    game.native_give_items(bucket_out, 1);
                    if remain == 0 {
                        game.native_set_block_property(pos, "lit", "false");
                    }
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(who.uuid, Text::literal(format!("🪣 [Tanque] Extraído 1,000 mB (Restan: {remain} mB)")));
                    }
                }
                return;
            }
        }
        return;
    }

    if block_id == rules::BLOCK_ENERGY_MONITOR {
        if held_info.is_some() {
            return;
        }
        *canceled = true;
        let consumers = rules::find_connected_consumers(
            pos,
            |p| rules::is_cable(&game.native_block(p)),
            |p| MachineKind::from_block_id(&game.native_block(p)).is_some(),
        );
        let mut total_energy = 0;
        let mut total_capacity = 0;
        for cpos in &consumers {
            game.native_with_mod_block_entity(*cpos, |entity| {
                if let Some(st) = load_state(entity) {
                    total_energy += st.energy;
                    total_capacity += st.kind.max_energy();
                }
            });
        }
        if let Some(who) = game.native_acting_player() {
            game.native_action_bar(
                who.uuid,
                Text::literal(format!(
                    "⚡ [Monitor de Red] Red Eléctrica: {} / {} FE ({} máquinas conectadas)",
                    total_energy, total_capacity, consumers.len()
                )),
            );
        }
        return;
    }

    if block_id == rules::BLOCK_ENERGY_RELAY {
        if held_info.is_some() {
            return;
        }
        *canceled = true;
        let mut total_relays = 0;
        for (&_p, be) in &game.world.block_entities {
            if let BlockEntity::Mod(entity) = be {
                if entity.id == rules::BLOCK_ENERGY_RELAY {
                    total_relays += 1;
                }
            }
        }
        if let Some(who) = game.native_acting_player() {
            game.native_action_bar(
                who.uuid,
                Text::literal(format!(
                    "📡 [Relé Inalámbrico] Red de Energía Inalámbrica: {} relés activos",
                    total_relays
                )),
            );
        }
        return;
    }

    if rules::is_cable(&block_id) {
        return;
    }

    if block_id == rules::BLOCK_ITEM_PIPE {
        let dim = game.native_dimension();
        let targets = rules::find_connected_item_targets(
            pos,
            |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
            |p| is_item_container_block(&game.native_block(p)),
        );
        // Silently pull up to 4 items from adjacent container and route through the pipe network
        for [dx, dy, dz] in rules::NEIGHBORS_6 {
            let adj = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let adj_id = game.native_block(adj);
            if matches!(
                adj_id.as_str(),
                "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:barrel" | "minecraft:hopper"
            ) {
                for &dest in &targets {
                    if dest != adj {
                        game.native_move_items(
                            dim,
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
        return;
    }
}

pub(in crate::game) fn menu_button(entity: &mut ModBlockEntity, id: i32) {
    if let Some(kind) = MachineKind::from_block_id(&entity.id) {
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
            20 if kind == MachineKind::SiloController => {
                state.quarry_cursor = (state.quarry_cursor - 1).max(0);
            }
            21 if kind == MachineKind::SiloController => {
                state.quarry_cursor += 1;
            }
            30..=33 if kind == MachineKind::SiloController => {
                state.quarry_y_offset = 1 + (id - 30);
            }
            _ => {}
        }
        store_state(entity, &state);
    }
}

pub(in crate::game) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
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
            | MachineKind::SteamTurbine => {
                vec![Some([80.0, 34.0])] // Centered 1-slot
            }
            MachineKind::SiloController => {
                vec![Some([150.0, 14.0])] // Top-right deposit slot
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

        *screen = Some(NativeScreen {
            width: 176.0,
            height: 166.0,
            slots,
            inventory_y: 84.0,
            inventory_x: 8.0,
            own_labels: true,
            own_background: true,
        });
    }
}

fn fill_template(key: &str, args: &[String]) -> String {
    let mut line = lang::get(key);
    for arg in args {
        line = line.replacen("%s", arg, 1);
    }
    line
}

fn draw_button(
    game: &Game<'_>,
    batch: &mut HudBatch,
    origin: [f32; 2],
    rect: [f32; 4],
    label: &str,
    bg_color: u32,
) {
    let [ox, oy] = origin;
    let [bx, by, bw, bh] = rect;
    game.native_fill(batch, [ox + bx, oy + by, bw, bh], 0xFF22_252A);
    game.native_fill(
        batch,
        [ox + bx + 1.0, oy + by + 1.0, bw - 2.0, bh - 2.0],
        bg_color,
    );
    let tw = game.native_text_width(label);
    game.native_text(
        batch,
        label,
        [ox + bx + (bw - tw) / 2.0, oy + by + 3.0],
        0xFF_FFFF,
        false,
    );
}

pub(in crate::game) fn screen_draw(
    game: &Game<'_>,
    block: &str,
    data: &[i32],
    batch: &mut HudBatch,
    origin: [f32; 2],
) {
    let Some(kind) = MachineKind::from_block_id(block) else {
        return;
    };
    let [x, y] = origin;
    game.native_sprite(
        batch,
        kind.gui_sprite(),
        [0.0, 0.0, 176.0, 166.0],
        [x, y, 176.0, 166.0],
    );

    let get = |i: usize| data.get(i).copied().unwrap_or(0);
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
    let raw_title = lang::get(kind.title_key());
    let title = format!("{raw_title} {tier_tag}");
    game.native_text(batch, &title, [x + 7.0, y + 5.0], 0xF0_F4F8, true);

    // 2. Status line below header
    let status_text = lang::get(&format!("rusty_factory.status.{status_code}"));
    let status_color = if status_code == StepStatus::Working as i32 {
        0x1B_6E28
    } else if status_code == StepStatus::Paused as i32 {
        0x66_6666
    } else {
        0x9E_2A2B
    };
    game.native_text(batch, &status_text, [x + 8.0, y + 20.0], status_color, false);

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
        game.native_text(batch, &area_txt, [x + 60.0, y + 30.0], 0x55_FFAA, false);
        game.native_text(batch, &y_txt, [x + 60.0, y + 42.0], 0x38_F2FF, false);
    } else if matches!(kind, MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler) {
        let burn_left = get(8).max(0);
        let burn_secs = burn_left / 20;
        let (accent_col, bolt_col) = match kind {
            MachineKind::CoalGenerator => (0xFFFF_7B25, 0xFFFF_D166),
            MachineKind::BioGenerator => (0xFF55_DD44, 0xFF99_FF66),
            _ => (0xFF38_D6FF, 0xFFAA_F0FF),
        };
        // Module A: Lightning Bolt + Tick Duration Bar [74, 28..104, 54]
        game.native_text(batch, "⚡", [x + 76.0, y + 29.0], bolt_col, false);
        let tick_txt = if burn_left > 0 {
            format!("{burn_left}t")
        } else {
            "0t".to_string()
        };
        game.native_text(batch, &tick_txt, [x + 85.0, y + 30.0], 0xFF_FFFF, false);
        game.native_fill(batch, [x + 76.0, y + 41.0, 26.0, 5.0], 0xFF22_262E);
        let bar_w = (26.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 26.0);
        if bar_w > 0.0 {
            game.native_fill(batch, [x + 76.0, y + 41.0, bar_w, 5.0], accent_col);
        }
        let sec_txt = format!("{burn_secs}s");
        game.native_text(batch, &sec_txt, [x + 79.0, y + 48.0], 0xAA_B8C8, false);

        // Module B: Stored & Generated Energy Readout [108, 28..168, 54]
        game.native_text(batch, &format!("Alm:{energy}"), [x + 111.0, y + 31.0], 0x70_E8FF, false);
        game.native_text(batch, &format!("Gen:+{rate}/t"), [x + 111.0, y + 43.0], bolt_col, false);
    } else if kind == MachineKind::SolarGenerator {
        let is_3x3 = get(7) != 0;
        let tag = if is_3x3 { "☀ MATRIZ 3x3 (x2)" } else { "☀ PANEL 1x1" };
        let tag_col = if is_3x3 { 0x55_FFAA } else { 0xFF_D166 };
        game.native_text(batch, tag, [x + 30.0, y + 30.0], tag_col, false);
        game.native_text(batch, &format!("Gen: +{rate} FE/t"), [x + 30.0, y + 40.0], 0xFF_E066, false);
        game.native_text(batch, &format!("Alm: {energy}/{max_energy}"), [x + 30.0, y + 49.0], 0x70_E8FF, false);
    } else if kind == MachineKind::EnergyBank {
        let cells = get(7).max(1);
        game.native_fill(batch, [x + 30.0, y + 34.0, 84.0, 9.0], 0xFF20_2832);
        let bar_w = (84.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 84.0);
        if bar_w > 0.0 {
            game.native_fill(batch, [x + 30.0, y + 34.0, bar_w, 9.0], 0xFF00_E5FF);
        }
        game.native_text(batch, &format!("{progress_pct}% ({cells} cel)"), [x + 32.0, y + 25.0], 0x70_E8FF, false);
        game.native_text(batch, &format!("{energy}/{max_energy} FE"), [x + 30.0, y + 46.0], 0xFF_FFFF, false);
    } else if kind == MachineKind::ChargingPad {
        game.native_text(batch, "⚡", [x + 56.0, y + 39.0], 0xFF00_FFAA, false);
        game.native_text(batch, "⚡", [x + 112.0, y + 39.0], 0xFF00_FFAA, false);
        game.native_text(batch, "INDUCCIÓN", [x + 62.0, y + 23.0], 0xFF00_FFAA, false);
    } else if kind == MachineKind::VoidTrashCan {
        game.native_text(batch, "VACÍO", [x + 74.0, y + 23.0], 0xDD_88FF, false);
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
        game.native_fill(batch, [x + 12.0, y + 26.0, 14.0, 30.0], 0xFF1A_1E26);
        let fh = (30.0 * (famount as f32 / fmax as f32)).clamp(0.0, 30.0);
        if fh > 0.0 {
            game.native_fill(batch, [x + 12.0, y + 26.0 + (30.0 - fh), 14.0, fh], fcol);
        }
        game.native_text(batch, &format!("Fluido: {fname}"), [x + 78.0, y + 31.0], fcol, false);
        game.native_text(batch, &format!("{famount}/{fmax} mB"), [x + 78.0, y + 43.0], 0xFF_FFFF, false);
    } else if kind == MachineKind::FluidPump {
        let fcode = get(7);
        let rate_mb = get(8);
        let fname = if fcode == 0 { "Sin fuente" } else { rules::fluid_name_from_code(fcode) };
        game.native_text(batch, &format!("Fuente: {fname}"), [x + 30.0, y + 31.0], 0x38_D6FF, false);
        game.native_text(batch, &format!("Bombeo: {rate_mb} mB/t"), [x + 30.0, y + 43.0], 0x55_FFAA, false);
    } else if kind == MachineKind::AlloySmelter {
        game.native_fill(batch, [x + 76.0, y + 39.0, 30.0, 8.0], 0xFF3A_3F48);
        let bar_w = (30.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 30.0);
        if bar_w > 0.0 {
            game.native_fill(batch, [x + 76.0, y + 39.0, bar_w, 8.0], 0xFFFF_7722);
        }
        game.native_text(batch, "➔", [x + 87.0, y + 37.0], 0xFF_FFFF, false);
    } else {
        game.native_fill(batch, [x + 64.0, y + 39.0, 34.0, 8.0], 0xFF3A_3F48);
        let bar_w = (34.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 34.0);
        if bar_w > 0.0 {
            let bar_col = if matches!(kind, MachineKind::ConveyorBelt | MachineKind::ItemExtractor) {
                0xFFFF_B820
            } else {
                0xFF38_D6FF
            };
            game.native_fill(batch, [x + 64.0, y + 39.0, bar_w, 8.0], bar_col);
        }
    }

    // 4. Vertical Energy Bar on left edge `[12, 26, 8, 30]`
    if !matches!(
        kind,
        MachineKind::ConveyorBelt | MachineKind::ItemExtractor | MachineKind::FluidTank | MachineKind::VoidTrashCan
    ) {
        game.native_fill(batch, [x + 12.0, y + 26.0, 8.0, 30.0], 0xFF20_242A);
        let e_ratio = (energy as f32 / max_energy as f32).clamp(0.0, 1.0);
        let e_h = 30.0 * e_ratio;
        if e_h > 0.0 {
            game.native_fill(
                batch,
                [x + 12.0, y + 26.0 + (30.0 - e_h), 8.0, e_h],
                0xFF00_E5FF,
            );
        }
    }

    // 5. Dark Telemetry Strip Readout (`y = 60..79`)
    if kind == MachineKind::Quarry {
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Minados: {quarry_mined} blq | Prof: Y={quarry_y}");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFF_D166, false);
    } else if kind == MachineKind::AutoHarvester {
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Cosechados: {quarry_mined} cultivos/troncos");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0x66_FF88, false);
    } else if kind == MachineKind::CoalGenerator {
        let burn_left = get(8).max(0);
        let line1 = format!("Almacenada: {energy}/{max_energy} FE | +{rate} FE/t");
        let line2 = format!("Combustión: {burn_left} ticks ({progress_pct}%)");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFF_9E00, false);
    } else if kind == MachineKind::BioGenerator {
        let burn_left = get(8).max(0);
        let line1 = format!("Almacenada: {energy}/{max_energy} FE | +{rate} FE/t");
        let line2 = format!("Biomasa: {burn_left} ticks ({progress_pct}%)");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0x70_E000, false);
    } else if kind == MachineKind::SteamBoiler {
        let burn_left = get(8).max(0);
        let line1 = format!("Almacenada: {energy}/{max_energy} FE | +{rate} FE/t");
        let line2 = format!("Presión Vapor: {burn_left} ticks ({progress_pct}%)");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0x38_D6FF, false);
    } else if kind == MachineKind::FluidTank {
        let fcode = get(7);
        let famount = get(8);
        let fmax = get(9).max(1);
        let fname = rules::fluid_name_from_code(fcode);
        let line1 = format!("Especificación: {fname} ({progress_pct}%)");
        let line2 = format!("Capacidad: {famount} / {fmax} mB (Usa Balde)");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x38_D6FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xA8_DADC, false);
    } else if kind == MachineKind::FluidPump {
        let fcode = get(7);
        let rate_mb = get(8);
        let fname = if fcode == 0 { "Sin fluido" } else { rules::fluid_name_from_code(fcode) };
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Bombeando {fname}: {rate_mb} mB/t");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0x55_FFAA, false);
    } else if kind == MachineKind::VoidTrashCan {
        let line1 = "Papelera de Vacío: Incineración Instantánea";
        let line2 = "Coloca cualquier ítem para destruirlo";
        game.native_text(batch, line1, [x + 10.0, y + 60.0], 0xDD_88FF, false);
        game.native_text(batch, line2, [x + 10.0, y + 70.0], 0xAA_B8C8, false);
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
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], col2, false);
    } else if kind == MachineKind::ChargingPad {
        let line1 = format!("FE: {energy}/{max_energy} | -20 FE/t en carga");
        let line2 = "Inducción activa: Carga armaduras y slot".to_string();
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFF00_FFAA, false);
    } else if kind == MachineKind::AlloySmelter {
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Fundición de Aleaciones: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFFFF_7722, false);
    } else if kind == MachineKind::EnergyBank {
        let cells = get(7).max(1);
        let line1 = format!("FE Total: {energy}/{max_energy}");
        let line2 = if cells > 1 {
            format!("Red Multibloque: {cells} celdas unificadas")
        } else {
            "Batería Modular (10,000 FE por celda)".to_string()
        };
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0x38_D6FF, false);
    } else if kind == MachineKind::ConveyorBelt {
        let line1 = format!("Velocidad Cinta: {}%", 100 + progress_pct / 2);
        let line2 = if auto_io {
            "Auto-Extraer (Atrás): ACTIVO"
        } else {
            "Auto-Extraer (Atrás): MANUAL"
        };
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0xFF_D166, false);
        game.native_text(batch, line2, [x + 10.0, y + 70.0], 0x90_E0EF, false);
    } else if kind == MachineKind::ItemExtractor {
        let line1 = format!("Extraídos a Tubería: {quarry_mined} ítems");
        let line2 = "Slot 0 = Filtro (Vacío = Todo)";
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0xFF_D166, false);
        game.native_text(batch, line2, [x + 10.0, y + 70.0], 0x90_E0EF, false);
    } else if kind == MachineKind::SapExtractor {
        let line1 = format!("FE: {energy}/{max_energy} | Extracción de Savia");
        let line2 = format!("Progreso extracción: {progress_pct}% (Tronco de caucho)");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFF_B820, false);
    } else if kind == MachineKind::Vulcanizer {
        let line1 = format!("FE: {energy}/{max_energy} | Horno Vulcanizador");
        let line2 = format!("Vulcanizando caucho puro: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xEE_82EE, false);
    } else if kind == MachineKind::MechanicalPress {
        let line1 = format!("FE: {energy}/{max_energy} | Prensa Hidráulica");
        let line2 = format!("Estampado de placas y engranajes: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0x55_FFAA, false);
    } else if kind == MachineKind::ElectricCrusher {
        let line1 = format!("FE: {energy}/{max_energy} | Triturador Cinético");
        let line2 = format!("Molienda (x2 polvo mineral): {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFF_D166, false);
    } else if kind == MachineKind::ElectricFurnace {
        let line1 = format!("FE: {energy}/{max_energy} | Horno Eléctrico");
        let line2 = format!("Fundición de alta temperatura: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFF_8844, false);
    } else if kind == MachineKind::SiloController {
        let ctrl_pos = [get(7), get(8), get(9)];
        let page = get(6).max(0) as usize;
        let (list, silos_n, chests_n) = aggregate_silo_network(game, ctrl_pos);
        let total_pages = ((list.len().saturating_sub(1)) / 4) + 1;
        game.native_text(
            batch,
            &format!("Silos: {silos_n} | Cofres: {chests_n} | Pág {}/{total_pages}", (page + 1).min(total_pages)),
            [x + 8.0, y + 18.0],
            0xFF38_F2FF,
            false,
        );
        for row in 0..4 {
            let ry = y + 34.0 + row as f32 * 24.0;
            game.native_fill(batch, [x + 8.0, ry, 160.0, 21.0], 0xFF1A_202A);
            if let Some((item_id, cnt, is_silo)) = list.get(page * 4 + row) {
                let short = item_id.split(':').nth(1).unwrap_or(item_id);
                let tag = if *is_silo { "[Silo]" } else { "[Cofre]" };
                let col = if *is_silo { 0xFF55_FF88 } else { 0xFFFF_C800 };
                game.native_text(batch, &format!("{short}"), [x + 12.0, ry + 3.0], 0xFFE8_EEF8, false);
                game.native_text(batch, &format!("x{cnt} {tag} (Clic: -64)"), [x + 12.0, ry + 12.0], col, false);
            } else {
                game.native_text(batch, "---", [x + 12.0, ry + 7.0], 0xFF55_6070, false);
            }
        }
        draw_button(game, batch, [x, y], [8.0, 134.0, 48.0, 14.0], "◀ Ant", 0xFF28_5066);
        draw_button(game, batch, [x, y], [120.0, 134.0, 48.0, 14.0], "Sig ▶", 0xFF28_5066);
        return;
    } else if kind == MachineKind::GeothermalGenerator {
        let line1 = format!("FE: {energy}/{max_energy} (+{rate}/t)");
        let line2 = format!("Lava interna: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0xFFFF_8833, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFFFF_B820, false);
    } else if kind == MachineKind::SteamTurbine {
        let line1 = format!("FE: {energy}/{max_energy} (+{rate}/t)");
        let line2 = format!("Vapor interno: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0xFF38_F2FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFFD0_F4FF, false);
    } else if kind == MachineKind::Recycler {
        let line1 = format!("FE: {energy}/{max_energy} ({rate}/t)");
        let line2 = format!("Reciclaje de Chatarra: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0xFF55_FF88, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFFAA_EEAA, false);
    } else if kind == MachineKind::Tesseract {
        let line1 = format!("FE: {energy}/{max_energy} (Inalámbrico)");
        let line2 = "Transmite FE e ítems entre Tesseracts";
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0xFF38_F2FF, false);
        game.native_text(batch, line2, [x + 10.0, y + 70.0], 0xFFAA_88FF, false);
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
        game.native_text(batch, &line1, [x + 10.0, y + 58.0], bar_col, false);
        game.native_text(batch, line2, [x + 10.0, y + 68.0], 0xFFAA_B4C4, false);
        game.native_fill(batch, [x + 10.0, y + 78.0, 156.0, 5.0], 0xFF22_2834);
        let fw = (156.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 156.0);
        if fw > 0.5 {
            game.native_fill(batch, [x + 10.0, y + 78.0, fw, 5.0], bar_col);
        }
    } else if kind == MachineKind::IndustrialAssembler {
        let line1 = format!("FE: {energy}/{max_energy} | Ensamblador Robótico");
        let line2 = format!("Ensamblaje automatizado: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0x38_F2FF, false);
    } else {
        let energy_line = fill_template(
            "rusty_factory.screen.energy",
            &[energy.to_string(), max_energy.to_string()],
        );
        let rate_line = fill_template("rusty_factory.screen.rate", &[rate.to_string()]);
        game.native_text(batch, &energy_line, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &rate_line, [x + 10.0, y + 70.0], 0xA8_DADC, false);
    }

    // 6. Interactive Buttons
    if !matches!(kind, MachineKind::FluidTank | MachineKind::VoidTrashCan) {
        let pwr_label = lang::get(if enabled {
            "rusty_factory.screen.start"
        } else {
            "rusty_factory.screen.stop"
        });
        let pwr_bg = if enabled { 0xFF2B_8A3E } else { 0xFFA6_1E22 };
        draw_button(game, batch, origin, POWER_BTN_RECT, &pwr_label, pwr_bg);
    }

    if matches!(kind, MachineKind::Quarry | MachineKind::AutoHarvester) {
        let side = quarry_radius * 2 + 1;
        let size_label = format!("{side}x{side}");
        draw_button(game, batch, origin, MODE_BTN_RECT, &size_label, 0xFF3B_5B92);
        if kind == MachineKind::Quarry {
            draw_button(game, batch, origin, RESET_BTN_RECT, "TOP", 0xFF6C_584C);
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
        let auto_label = lang::get(if auto_io {
            "rusty_factory.screen.auto_on"
        } else {
            "rusty_factory.screen.auto_off"
        });
        let auto_bg = if auto_io { 0xFF19_71C2 } else { 0xFF5C_677D };
        draw_button(game, batch, origin, MODE_BTN_RECT, &auto_label, auto_bg);
    }
}

fn in_rect(at: [f32; 2], rect: [f32; 4]) -> bool {
    let [x, y, w, h] = rect;
    (x..x + w).contains(&at[0]) && (y..y + h).contains(&at[1])
}

pub(in crate::game) fn screen_click(
    block: &str,
    at: [f32; 2],
    right: bool,
    button: &mut Option<i32>,
) {
    let Some(kind) = MachineKind::from_block_id(block) else {
        return;
    };
    if right {
        return;
    };
    if kind == MachineKind::SiloController {
        if in_rect(at, [8.0, 134.0, 48.0, 14.0]) {
            *button = Some(20);
        } else if in_rect(at, [120.0, 134.0, 48.0, 14.0]) {
            *button = Some(21);
        } else {
            for row in 0..4 {
                let ry = 34.0 + row as f32 * 24.0;
                if in_rect(at, [8.0, ry, 160.0, 21.0]) {
                    *button = Some(30 + row as i32);
                    break;
                }
            }
        }
        return;
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
}

pub(in crate::game) fn screen_tooltip(
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

pub(in crate::game) fn commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::op("rustyfactory kit", 2));
}

pub(in crate::game) fn perform(game: &mut Game<'_>, line: &str, ran: &mut bool) {
    if *ran || line.trim() != "rustyfactory kit" {
        return;
    }
    *ran = true;
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
        game.native_run_command(&format!("give @s {item}"));
    }
    game.native_feedback(Text::literal(
        "[Rusty Factory v0.5.0] ¡Kit industrial avanzado entregado (Mega Panel 3x3, Plataforma de Carga Inductiva, Tubería de Fluidos, Fundición de Aleaciones, Casco Solar y más)!",
    ));
}

pub fn get_item_energy(stack: &ItemStack) -> i32 {
    let key = horizon_world::nbt::JavaString::from("rf_energy");
    stack
        .custom_data()
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            horizon_world::nbt::Tag::Int(v) => Some(*v),
            horizon_world::nbt::Tag::Long(v) => Some(*v as i32),
            _ => None,
        })
        .unwrap_or(0)
        .max(0)
}

pub fn set_item_energy(stack: &mut ItemStack, energy: i32) {
    let cap = rules::battery_capacity(stack.item.as_ref()).unwrap_or(50_000);
    let clamped = energy.clamp(0, cap);
    let mut map = stack.custom_data().cloned().unwrap_or_default();
    map.insert(
        horizon_world::nbt::JavaString::from("rf_energy"),
        horizon_world::nbt::Tag::Int(clamped),
    );
    stack
        .data_mut()
        .set_other("minecraft:custom_data", Some(horizon_world::nbt::Tag::Compound(map)));
}

pub fn is_drill_3x3(stack: &ItemStack) -> bool {
    let key = horizon_world::nbt::JavaString::from("drill_3x3");
    stack
        .custom_data()
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            horizon_world::nbt::Tag::Byte(v) => Some(*v != 0),
            _ => None,
        })
        .unwrap_or(true)
}

pub fn set_drill_3x3(stack: &mut ItemStack, enabled: bool) {
    let mut map = stack.custom_data().cloned().unwrap_or_default();
    map.insert(
        horizon_world::nbt::JavaString::from("drill_3x3"),
        horizon_world::nbt::Tag::Byte(if enabled { 1 } else { 0 }),
    );
    stack
        .data_mut()
        .set_other("minecraft:custom_data", Some(horizon_world::nbt::Tag::Compound(map)));
}

pub fn consume_player_energy(game: &mut Game<'_>, cost: i32) -> bool {
    let mut needed = cost;
    if let Some(held) = game.held_mut() {
        let cur = get_item_energy(held);
        if cur > 0 {
            let take = cur.min(needed);
            set_item_energy(held, cur - take);
            needed -= take;
        }
    }
    if needed > 0 {
        for slot in game.inventory.iter_mut() {
            let Some(st) = slot.as_mut() else {
                continue;
            };
            if rules::is_battery_item(st.item.as_ref()) {
                let cur = get_item_energy(st);
                if cur > 0 {
                    let take = cur.min(needed);
                    set_item_energy(st, cur - take);
                    needed -= take;
                    if needed == 0 {
                        break;
                    }
                }
            }
        }
    }
    needed == 0
}

fn tick_player_equipment(game: &mut Game<'_>) {
    static LAST_EQUIP_TICK: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(-1);
    let now = game.world.game_time;
    let prev = LAST_EQUIP_TICK.load(std::sync::atomic::Ordering::Relaxed);
    if now == prev || now % 10 != 0 {
        return;
    }
    LAST_EQUIP_TICK.store(now, std::sync::atomic::Ordering::Relaxed);

    // 1. Solar / Quantum Helmet passive daytime charge into worn armor, batteries & tools
    let has_solar_helmet = game.armor[0].as_ref().is_some_and(|st| {
        st.item.as_ref() == rules::ITEM_SOLAR_HELMET || st.item.as_ref() == rules::ITEM_QUANTUM_HELMET
    });
    let is_day = (game.native_day_time().rem_euclid(24_000)) < 12_000;
    if has_solar_helmet && is_day {
        let mut charged = false;
        for slot in game.armor.iter_mut().chain(game.inventory.iter_mut()) {
            if let Some(st) = slot.as_mut() {
                if let Some(cap) = rules::battery_capacity(st.item.as_ref()) {
                    let cur = get_item_energy(st);
                    if cur < cap {
                        set_item_energy(st, cur + 50);
                        charged = true;
                        break;
                    }
                }
            }
        }
        let _ = charged;
    }

    // 2. Portable Batteries in inventory automatically recharge worn armor & tools
    let mut available = 0;
    for slot in game.inventory.iter() {
        if let Some(st) = slot.as_ref() {
            if rules::is_battery_item(st.item.as_ref()) {
                available += get_item_energy(st);
            }
        }
    }
    if available > 0 {
        let mut used = 0;
        let budget = available.min(400);
        for slot in game.armor.iter_mut().chain(game.inventory.iter_mut()) {
            if let Some(st) = slot.as_mut() {
                let id = st.item.as_ref();
                if !rules::is_battery_item(id) {
                    if let Some(cap) = rules::battery_capacity(id) {
                        let cur = get_item_energy(st);
                        if cur < cap {
                            let add = (budget - used).min(cap - cur);
                            if add > 0 {
                                set_item_energy(st, cur + add);
                                used += add;
                            }
                        }
                    }
                }
            }
            if used >= budget {
                break;
            }
        }
        if used > 0 {
            let mut left = used;
            for slot in game.inventory.iter_mut() {
                if let Some(st) = slot.as_mut() {
                    if rules::is_battery_item(st.item.as_ref()) {
                        let cur = get_item_energy(st);
                        let take = cur.min(left);
                        if take > 0 {
                            set_item_energy(st, cur - take);
                            left -= take;
                        }
                    }
                }
                if left == 0 {
                    break;
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
    for slot in game.armor.iter() {
        if let Some(st) = slot.as_ref() {
            match st.item.as_ref() {
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
                rules::ITEM_JETPACK_MK2 if is_jetpack_hover(st) => has_hover = true,
                _ => {}
            }
            if has_item_module(st, "nv") {
                has_nv = true;
            }
            if has_item_module(st, "mag") {
                has_mag = true;
            }
        }
    }
    if has_hover {
        game.native_run_command("effect give @s minecraft:slow_falling 2 0 true");
    }
    if now % 40 == 0 {
        if jump_lvl >= 0 {
            game.native_run_command(&format!("effect give @s minecraft:jump_boost 3 {jump_lvl} true"));
        }
        if speed_lvl >= 0 {
            game.native_run_command(&format!("effect give @s minecraft:speed 3 {speed_lvl} true"));
        }
        if res_lvl >= 0 {
            game.native_run_command(&format!("effect give @s minecraft:resistance 3 {res_lvl} true"));
        }
        if has_fire_res {
            game.native_run_command("effect give @s minecraft:fire_resistance 3 0 true");
        }
        if has_nv {
            game.native_run_command("effect give @s minecraft:night_vision 12 0 true");
        }
        if has_mag {
            game.native_run_command("tp @e[type=item,distance=..8] @s");
        }
    }
}

pub(in crate::game) fn tooltip(
    _game: &Game<'_>,
    stack: &ItemStack,
    lines: &mut Vec<(Text, u32)>,
) {
    let id = stack.item.as_ref();
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
        rules::BLOCK_ITEM_PIPE => &["§7Tubería de vidrio para transportar ítems entre inventarios."],
        rules::BLOCK_FLUID_PIPE => &["§7Tubería para mover fluidos entre tanques y máquinas."],
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
        rules::BLOCK_RECYCLER => &["§7Recicla bloques sobrantes en polvo metálico y materiales."],
        rules::ITEM_MINING_DRILL_MK2 => &["§7Taladro MK2: excava planos 3x3 con menor consumo (25 FE)."],
        rules::ITEM_MINING_DRILL_MK3 => &["§7Taladro MK3: excava planos 5x5 a gran velocidad (15 FE)."],
        rules::ITEM_CHAINSAW => &["§7Motosierra Eléctrica: tala árboles completos de un clic."],
        rules::ITEM_BATTERY_PACK => &["§7Mochila Dorsal (1,000,000 FE): recarga todo tu equipo."],
        rules::ITEM_JETPACK_MK2 => &["§7Jetpack MK2 con Modo Hover (Shift + Clic para flotar)."],
        rules::ITEM_ENERGY_SCANNER => &["§7Escáner portátil: clic en máquinas/tubos para ver su estado."],
        rules::ITEM_BELT_FILTER_ADV => &["§7Filtro avanzado de cinta con Lista Blanca / Lista Negra."],
        rules::ITEM_MODULE_NIGHT_VISION => &["§7Módulo: añade Visión Nocturna a tu casco."],
        rules::ITEM_MODULE_MAGNET => &["§7Módulo: atrae ítems cercanos hacia vos."],
        rules::ITEM_MODULE_EFFICIENCY => &["§7Módulo: reduce a la mitad el gasto de FE de tu taladro."],
        rules::ITEM_SILO_UPGRADE_5K => &["§7Aumenta la capacidad de un Silo Industrial en +5,000 ítems."],
        rules::ITEM_SILO_UPGRADE_10K => &["§7Aumenta la capacidad de un Silo Industrial en +10,000 ítems."],
        rules::ITEM_SILO_UPGRADE_100K => &["§7Aumenta la capacidad de un Silo Industrial en +100,000 ítems."],
        rules::ITEM_SILO_UPGRADE_500K => &["§7Mejora suprema: +500,000 ítems de capacidad en el Silo."],
        _ => &["§7Componente industrial de Rusty Factory."],
    };

    for line in desc {
        lines.push((Text::legacy(line), 0xFFAA_B4C4));
    }

    if let Some(cap) = rules::battery_capacity(id) {
        let cur = get_item_energy(stack);
        lines.push((
            Text::legacy(&format!("§b⚡ Energía: §f{cur} / {cap} FE")),
            0xFF38_F2FF,
        ));
    }
    if id == rules::ITEM_MINING_DRILL {
        let mode = if is_drill_3x3(stack) { "3x3" } else { "1x1" };
        lines.push((
            Text::legacy(&format!("§e⛏ Modo actual: §f{mode}")),
            0xFFFF_D038,
        ));
    }
}

pub(in crate::game) fn hud(
    game: &mut Game<'_>,
    batch: &mut HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    if !game.native_on(rules::ID) {
        return;
    }

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
    let held_entry: Option<(String, i32, i32)> = game.held().and_then(|st| {
        let cap = rules::battery_capacity(st.item.as_ref())?;
        let cur = get_item_energy(st);
        let label = match st.item.as_ref() {
            rules::ITEM_MINING_DRILL => {
                let m = if is_drill_3x3(st) { "3x3" } else { "1x1" };
                format!("Taladro ({m})")
            }
            rules::ITEM_MINING_DRILL_MK2 => {
                let m = if is_drill_3x3(st) { "3x3" } else { "1x1" };
                format!("Taladro MK2 ({m})")
            }
            rules::ITEM_MINING_DRILL_MK3 => {
                let m = if is_drill_3x3(st) { "5x5" } else { "1x1" };
                format!("Taladro MK3 ({m})")
            }
            rules::ITEM_CHAINSAW => "Motosierra".to_owned(),
            rules::ITEM_MINING_LASER => "Láser Minero".to_owned(),
            rules::ITEM_BATTERY => "Batería MK1".to_owned(),
            rules::ITEM_BATTERY_MK2 => "Batería MK2".to_owned(),
            rules::ITEM_BATTERY_PACK => "Mochila Bat.".to_owned(),
            rules::ITEM_JETPACK => "Jetpack".to_owned(),
            rules::ITEM_JETPACK_MK2 => "Jetpack MK2".to_owned(),
            rules::ITEM_JET_BOOTS => "Botas Cohete".to_owned(),
            rules::ITEM_SOLAR_HELMET => "Casco Solar".to_owned(),
            rules::ITEM_INDUSTRIAL_HELMET => "Casco Ind.".to_owned(),
            rules::ITEM_INDUSTRIAL_CHESTPLATE => "Pechera Ind.".to_owned(),
            rules::ITEM_INDUSTRIAL_LEGGINGS => "Pantalón Ind.".to_owned(),
            rules::ITEM_INDUSTRIAL_BOOTS => "Botas Ind.".to_owned(),
            _ => "Equipo Tech".to_owned(),
        };
        Some((label, cur, cap))
    });

    // 2. Collect per-piece equipped tech armor from `game.armor` (0=Head, 1=Chest, 2=Legs, 3=Feet)
    let read_armor_slot = |slot_idx: usize| -> Option<(&'static str, i32, i32)> {
        let st = game.armor.get(slot_idx)?.as_ref()?;
        let cap = rules::battery_capacity(st.item.as_ref())?;
        let cur = get_item_energy(st);
        let short_name = match st.item.as_ref() {
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
    for slot in game.inventory.iter().take(36) {
        if let Some(st) = slot.as_ref() {
            if rules::is_battery_item(st.item.as_ref()) {
                bat_cur += get_item_energy(st);
                bat_cap += rules::battery_capacity(st.item.as_ref()).unwrap_or(50_000);
            }
        }
    }

    // 0. Look-At Block Info Card (Silo capacity bar Green/Yellow/Red, Geothermal/Turbine/Tesseract/UpgradeStation info)
    if let Some(player) = &game.player {
        let yaw_rad = (game.yaw as f64).to_radians();
        let pitch_rad = (game.pitch as f64).to_radians();
        let dir = [
            -yaw_rad.sin() * pitch_rad.cos(),
            -pitch_rad.sin(),
            yaw_rad.cos() * pitch_rad.cos(),
        ];
        let eye = [
            player.position.x,
            player.position.y + f64::from(player.dimensions.eye_height),
            player.position.z,
        ];
        let mut hit_pos: Option<[i32; 3]> = None;
        for step in 1..=100 {
            let t = step as f64 * 0.05;
            let cell = [
                (eye[0] + dir[0] * t).floor() as i32,
                (eye[1] + dir[1] * t).floor() as i32,
                (eye[2] + dir[2] * t).floor() as i32,
            ];
            let blk = game.native_block(cell);
            if !matches!(blk.as_str(), "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air") {
                if blk.starts_with("rusty_factory:") {
                    hit_pos = Some(cell);
                }
                break;
            }
        }
        if let Some(hp) = hit_pos {
            let hblk = game.native_block(hp);
            let mut info_title = String::new();
            let mut info_sub = String::new();
            let mut bar_ratio: Option<(f32, u32)> = None;

            if let Some(BlockEntity::Mod(ent)) = game.world.block_entities.get(&(hp[0], hp[1], hp[2])) {
                if let Some(st) = load_state(ent) {
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
                let (ix, iy) = corners.place(horizon_client::extras::Corner::BottomRight, iw, ih);
                let border_c = bar_ratio.map_or(0xFF38_F2FF, |(_, c)| c);
                game.native_fill(batch, [ix, iy, iw, ih], 0xBE10_141C);
                game.native_fill(batch, [ix, iy, iw, 1.5], border_c);
                game.native_fill(batch, [ix, iy + ih - 1.5, iw, 1.5], 0xFF28_5066);
                game.native_fill(batch, [ix, iy, 1.5, ih], border_c);
                game.native_fill(batch, [ix + iw - 1.5, iy, 1.5, ih], 0xFF28_5066);
                game.native_text(batch, &info_title, [ix + 6.0, iy + 5.0], 0xFFE8_EEF8, true);
                game.native_text(batch, &info_sub, [ix + 6.0, iy + 17.0], border_c, true);
                if let Some((r, bcol)) = bar_ratio {
                    game.native_fill(batch, [ix + 6.0, iy + 29.0, iw - 12.0, 4.0], 0xFF22_2834);
                    let fw = ((iw - 12.0) * r).clamp(0.0, iw - 12.0);
                    if fw > 0.5 {
                        game.native_fill(batch, [ix + 6.0, iy + 29.0, fw, 4.0], bcol);
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
    let (x, y) = corners.place(horizon_client::extras::Corner::BottomRight, w, h);

    // Dark industrial background + cyan border
    game.native_fill(batch, [x, y, w, h], 0xBE10_141C);
    game.native_fill(batch, [x, y, w, 1.5], 0xFF38_F2FF);
    game.native_fill(batch, [x, y + h - 1.5, w, 1.5], 0xFF28_5066);
    game.native_fill(batch, [x, y, 1.5, h], 0xFF38_F2FF);
    game.native_fill(batch, [x + w - 1.5, y, 1.5, h], 0xFF28_5066);

    game.native_text(batch, "⚡ ESTADO TECH", [x + 6.0, y + 5.0], 0xFF38_F2FF, true);

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
        game.native_fill(batch, [fx + 7.0, fy, 16.0, 12.0], sil_black);       // Head silhouette
        game.native_fill(batch, [fx, fy + 11.0, 30.0, 18.0], sil_black);     // Torso + Arms silhouette
        game.native_fill(batch, [fx + 6.0, fy + 29.0, 18.0, 23.0], sil_black); // Legs + Boots silhouette

        let draw_seg = |g: &Game<'_>, b: &mut HudBatch, px: f32, py: f32, pw: f32, ph: f32, col: u32, lit: bool| {
            g.native_fill(b, [px, py, pw, ph], col);
            if lit {
                g.native_fill(b, [px, py, pw, 1.0], 0x44FF_FFFF);
            }
        };

        // Head
        draw_seg(game, batch, fx + 9.0, fy + 2.0, 12.0, 9.0, head_col, head.is_some());
        game.native_text(batch, &head_txt, [tx, fy + 2.0], if head.is_some() { head_col } else { 0xFF55_6070 }, true);

        // Chest + Arms
        draw_seg(game, batch, fx + 2.0, fy + 13.0, 4.0, 14.0, chest_col, chest.is_some());
        draw_seg(game, batch, fx + 8.0, fy + 13.0, 14.0, 14.0, chest_col, chest.is_some());
        draw_seg(game, batch, fx + 24.0, fy + 13.0, 4.0, 14.0, chest_col, chest.is_some());
        game.native_text(batch, &chest_txt, [tx, fy + 16.0], if chest.is_some() { chest_col } else { 0xFF55_6070 }, true);

        // Legs
        draw_seg(game, batch, fx + 8.0, fy + 29.0, 6.0, 13.0, legs_col, legs.is_some());
        draw_seg(game, batch, fx + 16.0, fy + 29.0, 6.0, 13.0, legs_col, legs.is_some());
        game.native_text(batch, &legs_txt, [tx, fy + 31.0], if legs.is_some() { legs_col } else { 0xFF55_6070 }, true);

        // Boots
        draw_seg(game, batch, fx + 8.0, fy + 44.0, 6.0, 6.0, feet_col, feet.is_some());
        draw_seg(game, batch, fx + 16.0, fy + 44.0, 6.0, 6.0, feet_col, feet.is_some());
        game.native_text(batch, &feet_txt, [tx, fy + 43.0], if feet.is_some() { feet_col } else { 0xFF55_6070 }, true);

        ry += armor_block_h;
    }

    let draw_bar = |g: &Game<'_>, b: &mut HudBatch, label: &str, cur: i32, cap: i32, row_y: f32| {
        let pct = ((cur as f32 / cap.max(1) as f32) * 100.0).clamp(0.0, 100.0);
        let col = battery_color(pct);
        g.native_text(
            b,
            &format!("{label}: {}%", pct as i32),
            [x + 6.0, row_y],
            col,
            true,
        );
        g.native_fill(b, [x + 6.0, row_y + 10.0, w - 12.0, 4.0], 0xFF22_2834);
        let fill_w = ((w - 12.0) * (pct / 100.0)).clamp(0.0, w - 12.0);
        if fill_w > 0.5 {
            g.native_fill(b, [x + 6.0, row_y + 10.0, fill_w, 4.0], col);
        }
    };

    if let Some((ref label, cur, cap)) = held_entry {
        draw_bar(game, batch, label, cur, cap, ry);
        ry += 18.0;
    }
    if bat_cap > 0 {
        draw_bar(game, batch, "Baterías", bat_cur, bat_cap, ry);
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
    let key = horizon_world::nbt::JavaString::from("rf_modules");
    stack
        .custom_data()
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            horizon_world::nbt::Tag::String(s) => s.to_string().ok(),
            _ => None,
        })
        .is_some_and(|s| s.split(',').any(|m| m == code))
}

fn add_item_module(stack: &mut ItemStack, code: &str) {
    let key = horizon_world::nbt::JavaString::from("rf_modules");
    let mut map = stack.custom_data().cloned().unwrap_or_default();
    let existing = map
        .get(&key)
        .and_then(|t| match t {
            horizon_world::nbt::Tag::String(s) => s.to_string().ok(),
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
            horizon_world::nbt::Tag::String(horizon_world::nbt::JavaString::from(next.as_str())),
        );
        stack
            .data_mut()
            .set_other("minecraft:custom_data", Some(horizon_world::nbt::Tag::Compound(map)));
    }
}

fn is_jetpack_hover(stack: &ItemStack) -> bool {
    let key = horizon_world::nbt::JavaString::from("hover");
    stack
        .custom_data()
        .and_then(|c| c.get(&key))
        .and_then(|t| match t {
            horizon_world::nbt::Tag::Byte(v) => Some(*v != 0),
            _ => None,
        })
        .unwrap_or(false)
}

fn set_jetpack_hover(stack: &mut ItemStack, hover: bool) {
    let mut map = stack.custom_data().cloned().unwrap_or_default();
    map.insert(
        horizon_world::nbt::JavaString::from("hover"),
        horizon_world::nbt::Tag::Byte(if hover { 1 } else { 0 }),
    );
    stack
        .data_mut()
        .set_other("minecraft:custom_data", Some(horizon_world::nbt::Tag::Compound(map)));
}
