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
            MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler => {
                *container = Some(ModContainer {
                    slots: 1,
                    faces: std::array::from_fn(|_| vec![0]),
                    inputs: vec![0],
                    outputs: vec![],
                    menu: Some(rules::MENU.to_owned()),
                });
            }
            MachineKind::SolarGenerator | MachineKind::EnergyMonitor | MachineKind::FluidTank => {
                *container = None;
            }
            MachineKind::EnergyBank => {
                *container = Some(ModContainer {
                    slots: 1,
                    faces: std::array::from_fn(|_| vec![0]),
                    inputs: vec![0],
                    outputs: vec![0],
                    menu: Some(rules::MENU.to_owned()),
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
            | rules::BLOCK_CONVEYOR
            | rules::BLOCK_GENERATOR
            | rules::BLOCK_BIO_GENERATOR
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

/// Routes `stack` from `source_pos` into connected `rusty_factory:item_pipe` networks or adjacent containers.
/// Returns the remaining stack (`None` if all items were inserted).
fn route_item_from(game: &mut Game<'_>, source_pos: [i32; 3], mut stack: ItemStack) -> Option<ItemStack> {
    let targets = rules::find_connected_item_targets(
        source_pos,
        |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
        |p| is_item_container_block(&game.native_block(p)),
    );

    for target_pos in targets {
        if stack.count == 0 {
            return None;
        }
        // Try inserting through Up first (inputs for furnaces/machines), then North
        match game.native_insert_item(target_pos, Direction::Up, stack) {
            None => return None,
            Some(rest) => {
                stack = match game.native_insert_item(target_pos, Direction::North, rest) {
                    None => return None,
                    Some(r2) => r2,
                };
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
        |p| game.native_block(p) == rules::BLOCK_CABLE,
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

/// Simulates the Conveyor Belt:
/// 1. Picks up dropped items in the world lying on top of the belt.
/// 2. Optionally pulls items from a container behind the belt (`auto_io`).
/// 3. Moves carried items forward into the block in front (next belt, pipe network, chest, or machine).
fn tick_conveyor(
    game: &mut Game<'_>,
    pos: [i32; 3],
    entity: &mut ModBlockEntity,
    state: &mut MachineState,
) {
    let dim = game.native_dimension();
    let facing = game
        .native_block_property(pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let (front_off, back_off) = rules::conveyor_offsets(&facing);
    let front_pos = [
        pos[0] + front_off[0],
        pos[1] + front_off[1],
        pos[2] + front_off[2],
    ];
    let back_pos = [
        pos[0] + back_off[0],
        pos[1] + back_off[1],
        pos[2] + back_off[2],
    ];

    // 1. Pick up dropped item entities lying on the belt if SLOT_INPUT is empty
    if entity.items[rules::SLOT_INPUT].is_none() {
        let dropped = game.native_item_entities(
            dim,
            [pos[0] as f64, pos[1] as f64, pos[2] as f64],
            [pos[0] as f64 + 1.0, pos[1] as f64 + 1.15, pos[2] as f64 + 1.0],
        );
        if let Some(first) = dropped.first() {
            entity.items[rules::SLOT_INPUT] = Some(first.stack.clone());
            let mut empty = first.stack.clone();
            empty.count = 0;
            game.native_set_item_entity(dim, first.id, empty);
        }
    }

    // 2. If Auto-Pull (`state.auto_io`) is enabled and belt is empty, pull 1 item from container behind
    if state.auto_io && entity.items[rules::SLOT_INPUT].is_none() {
        let back_block = game.native_block(back_pos);
        if back_block != rules::BLOCK_CONVEYOR && is_item_container_block(&back_block) {
            if let Some(slots) = game.native_container_slots(dim, back_pos, Direction::Down) {
                for s in slots {
                    if s.can_take && s.stack.is_some() {
                        if let Some(pulled) =
                            game.native_extract_item(dim, back_pos, Direction::Down, s.slot, 1)
                        {
                            entity.items[rules::SLOT_INPUT] = Some(pulled);
                            break;
                        }
                    }
                }
            }
        }
    }

    // 3. When progress reaches 0, push 1 item forward into the block in front (or pipe network)
    if state.progress == 0 {
        for slot in [rules::SLOT_INPUT, rules::SLOT_OUTPUT, rules::SLOT_FUEL] {
            if let Some(stack) = entity.items.get(slot).cloned().flatten() {
                let mut one = stack.clone();
                one.count = 1;
                let front_block = game.native_block(front_pos);
                let leftover = if front_block == rules::BLOCK_ITEM_PIPE {
                    route_item_from(game, pos, one)
                } else {
                    match game.native_insert_item(front_pos, Direction::Up, one) {
                        None => None,
                        Some(r) => game.native_insert_item(front_pos, Direction::North, r),
                    }
                };
                if leftover.is_none() {
                    if stack.count > 1 {
                        if let Some(Some(s)) = entity.items.get_mut(slot) {
                            s.count -= 1;
                        }
                    } else {
                        entity.items[slot] = None;
                    }
                    break;
                }
            }
        }
    }

    // Sync the visible belt item ID for 3D rendering on the belt
    let current_item = entity
        .items
        .iter()
        .find_map(|s| s.as_ref().map(|st| st.item.to_string()))
        .unwrap_or_default();
    if state.belt_item != current_item {
        state.belt_item = current_item;
        store_state(entity, state);
        game.native_block_entity_sync(pos);
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

    let cost = rules::effective_energy_cost(MachineKind::Quarry, speed_upgrades);
    let needed = rules::effective_work_ticks(MachineKind::Quarry, speed_upgrades);

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

        // Scan up to 12 positions to skip over air/water/bedrock quickly
        for _ in 0..12 {
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
                    fortune_upgrades.min(4)
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
            if matches!(
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
        MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::EnergyBank => 1,
        MachineKind::SolarGenerator => 0,
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

    if kind == MachineKind::ConveyorBelt {
        let outcome = rules::step_machine(&mut state, None, None, None, speed_upgrades);
        if state.enabled {
            tick_conveyor(game, pos, entity, &mut state);
        }
        let period = rules::effective_work_ticks(kind, speed_upgrades).max(1);
        let pct = ((state.progress * 100) / period).clamp(0, 100);
        entity.data = vec![
            outcome.status as i32,
            0,
            1,
            pct,
            0,
            i32::from(state.enabled),
            i32::from(state.auto_io),
            0,
            0,
            0,
        ];
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

    if matches!(kind, MachineKind::FluidTank | MachineKind::EnergyMonitor) {
        store_state(entity, &state);
        return;
    }

    let is_gen = matches!(kind, MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler);
    let fuel_ticks = if is_gen {
        entity.items.first().and_then(|s| s.as_ref()).and_then(fuel_burn_ticks)
    } else {
        entity.items.get(rules::SLOT_FUEL).and_then(|s| s.as_ref()).and_then(fuel_burn_ticks)
    };

    if kind == MachineKind::SolarGenerator {
        let is_array_3x3 = rules::is_solar_array_3x3(pos, |p| game.native_block(p) == rules::BLOCK_SOLAR);
        let max_e = if is_array_3x3 { 4_000 } else { 2_000 };
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
                let sent = distribute_energy(game, pos, state.energy.min(if is_array_3x3 { 320 } else { 160 }), false);
                state.energy -= sent;
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
/// Renders items smoothly gliding along the top of the 3D Conveyor Belt!
pub(in crate::game) fn submit(
    game: &Game<'_>,
    pos: [i32; 3],
    block: &str,
    state: BlockStateId,
    entity: &ModBlockEntity,
    draw: &mut BlockEntityDraw,
) {
    if !game.native_on(rules::ID) || block != rules::BLOCK_CONVEYOR {
        return;
    }
    let Some(st) = load_state(entity) else {
        return;
    };
    if st.belt_item.is_empty() {
        return;
    }
    let facing = game
        .native_block_state_property(state, "facing")
        .unwrap_or("north");
    // Smooth belt motion from back (0.18) to front (0.82) along the facing axis
    let t = if st.enabled {
        ((draw.game_time as f32 + draw.partial) * 0.08).rem_euclid(1.0)
    } else {
        0.5
    };
    let travel = 0.18 + t * 0.64;
    let (ix, iz, yaw) = match facing {
        "north" => (0.5, 1.0 - travel, 0.0),
        "south" => (0.5, travel, 180.0),
        "east" => (travel, 0.5, 270.0),
        "west" => (1.0 - travel, 0.5, 90.0),
        _ => (0.5, 1.0 - travel, 0.0),
    };

    let stack = ItemStack::new(&st.belt_item, 1);
    draw.push();
    draw.translate([ix, 0.52, iz]);
    draw.rotate_about([0.0, 1.0, 0.0], yaw, [0.0, 0.0, 0.0]);
    draw.scale([0.52, 0.52, 0.52]);
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

        if entity.id == rules::BLOCK_ENERGY_BANK {
            let st = load_state(entity)
                .unwrap_or_else(|| MachineState::new(MachineKind::EnergyBank));
            let cluster_len = rules::find_battery_cluster([x, y, z], |p| {
                game.native_block(p) == rules::BLOCK_ENERGY_BANK
            }).len().max(1);
            let cluster_max = (cluster_len as i32) * 10_000;
            let pct = (st.energy * 100) / cluster_max.max(1);
            let label = if cluster_len > 1 {
                format!("BANCO [{cluster_len}x]: {} / {}k ({pct}%)", st.energy, cluster_max / 1000)
            } else {
                format!("BANCO FE: {} / 10k ({pct}%)", st.energy)
            };
            primitives
                .text(
                    [x as f64 + 0.5, y as f64 + 1.35, z as f64 + 0.5],
                    label,
                    0xFF38_F2FF,
                    0.8,
                )
                .background();
            continue;
        }

        if entity.id == rules::BLOCK_SOLAR {
            let is_3x3 = rules::is_solar_array_3x3([x, y, z], |p| game.native_block(p) == rules::BLOCK_SOLAR);
            if is_3x3 {
                primitives
                    .text(
                        [x as f64 + 0.5, y as f64 + 1.15, z as f64 + 0.5],
                        "☀️ MATRIZ 3x3 (2x)",
                        0xFF55_FFAA,
                        0.7,
                    )
                    .background();
            }
            continue;
        }

        if entity.id == rules::BLOCK_ENERGY_MONITOR {
            primitives
                .text(
                    [x as f64 + 0.5, y as f64 + 1.25, z as f64 + 0.5],
                    "⚡ MONITOR DE RED",
                    0xFF38_F2FF,
                    0.75,
                )
                .background();
            continue;
        }

        if entity.id == rules::BLOCK_FLUID_TANK {
            let is_filled = game.native_block_property([x, y, z], "lit").as_deref() == Some("true");
            if is_filled {
                primitives.filled_box(
                    [x as f64 + 0.12, y as f64 + 0.08, z as f64 + 0.12],
                    [x as f64 + 0.88, y as f64 + 0.88, z as f64 + 0.88],
                    0xAA25_85E8,
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

            // Green agricultural perimeter frame
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
            let side = st.quarry_side_len();
            let label = format!(
                "COSECHADORA {side}x{side} | {} FE | Cosechados: {}",
                st.energy, st.quarry_mined
            );
            primitives
                .text(
                    [x as f64 + 0.5, y as f64 + 1.4, z as f64 + 0.5],
                    label,
                    0xFF66_FF88,
                    0.8,
                )
                .background();
            continue;
        }

        if entity.id != rules::BLOCK_QUARRY {
            continue;
        }

        let st = load_state(entity).unwrap_or_else(|| MachineState::new(MachineKind::Quarry));
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
        let top_y = y as f64 + 3.0;

        // 1. Outer Industrial Quarry Frame (Yellow/Amber hazard structural outline)
        primitives.box_outline([x0, base_y, z0], [x1, top_y, z1], 0xFFFF_B81C, 3.0);
        // Top Gantry Rail Ring
        primitives.box_outline(
            [x0, top_y - 0.22, z0],
            [x1, top_y, z1],
            0xFFFF_7A18,
            2.5,
        );

        // 2. Compute current target block & Gantry position
        let target = if st.last_target != [0, 0, 0] {
            st.last_target
        } else {
            rules::quarry_target_pos(pos, &facing, &st)
        };
        let tx = (target[0] as f64 + 0.5).clamp(x0 + 0.5, x1 - 0.5);
        let tz = (target[2] as f64 + 0.5).clamp(z0 + 0.5, z1 - 0.5);
        let ty = (target[1] as f64 + 0.5).min(top_y - 0.5);

        // Moving X-Bridge bar across the Quarry frame at `tz`
        primitives.filled_box(
            [x0, top_y - 0.20, tz - 0.12],
            [x1, top_y - 0.04, tz + 0.12],
            0xD8FF_A818,
        );

        // Drill Head Carriage at `[tx, top_y, tz]`
        primitives.filled_box(
            [tx - 0.28, top_y - 0.35, tz - 0.28],
            [tx + 0.28, top_y + 0.12, tz + 0.28],
            0xFFFF_D038,
        );

        // 3. Vertical Drill Pipe & Active Mining Laser Beam down to `[tx, ty, tz]`
        if st.enabled {
            let pulse = ((game.world.game_time as f32 + partial) * 0.25).sin() * 0.5 + 0.5;
            let laser_color = if pulse > 0.5 {
                0xFF38_F2FF
            } else {
                0xFFFF_9E22
            };
            // Vertical steel drill column
            primitives.line(
                [tx, top_y - 0.35, tz],
                [tx, ty + 0.3, tz],
                0xFF45_4E5C,
                4.0,
            );
            // Glowing mining laser core
            primitives.line([tx, top_y - 0.35, tz], [tx, ty, tz], laser_color, 2.5);
            // Drill tip box
            primitives.filled_box(
                [tx - 0.16, ty, tz - 0.16],
                [tx + 0.16, ty + 0.45, tz + 0.16],
                0xFF38_F2FF,
            );
            // Highlight outline on the block currently being excavated
            primitives.box_outline(
                [target[0] as f64, target[1] as f64, target[2] as f64],
                [
                    target[0] as f64 + 1.0,
                    target[1] as f64 + 1.0,
                    target[2] as f64 + 1.0,
                ],
                0xFF38_F2FF,
                2.0,
            );
        }

        // 4. Floating 3D Quarry Telemetry Label above the machine
        let side = st.quarry_side_len();
        let label = format!(
            "QUARRY {side}x{side} [Y={}] | {} FE | Minados: {}",
            target[1], st.energy, st.quarry_mined
        );
        primitives
            .text(
                [x as f64 + 0.5, y as f64 + 1.45, z as f64 + 0.5],
                label,
                if st.enabled { 0xFF55_FF99 } else { 0xFFFF_6666 },
                0.85,
            )
            .background();
    }
}

/// Inspects networks, harvests sap from Rubber Tree Sap Cavities, grows Rubber Saplings, rotates/dismantles machines with Wrench, or uses tools.
pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let block_id = game.native_block(pos);
    let held_info = game.native_held();
    let is_crouching = game
        .player
        .as_ref()
        .is_some_and(|p| p.pose == horizon_client::pose::Pose::Crouching);

    if let Some((ref held_id, _, _)) = held_info {
        // 1. Wrench: Rotate machine facing (normal click) or dismantle into inventory (crouch + click)
        if held_id == rules::ITEM_WRENCH {
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

            if block_id == rules::BLOCK_CABLE {
                *canceled = true;
                let consumers = rules::find_connected_consumers(
                    pos,
                    |p| game.native_block(p) == rules::BLOCK_CABLE,
                    |p| MachineKind::from_block_id(&game.native_block(p)).is_some(),
                );
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(
                        who.uuid,
                        Text::literal(format!("⚡ [Cable de Cobre] Red: {} máquinas conectadas", consumers.len())),
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
        }

        // 2. Treetap: Harvest extra sap & saplings from Rubber Sap Log
        if held_id == rules::ITEM_TREETAP && block_id == rules::BLOCK_RUBBER_SAP_LOG {
            *canceled = true;
            let has_sap = game.native_block_property(pos, "lit").as_deref() == Some("true");
            if has_sap {
                game.native_set_block_property(pos, "lit", "false");
                game.native_give_items(rules::ITEM_RAW_SAP, 4);
                game.native_give_items(rules::BLOCK_RUBBER_SAPLING, 1);
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(
                        who.uuid,
                        Text::literal("💧 [Espita de Madera] ¡4x Savia recolectada con éxito!"),
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

        // 3. Mining Drill: Crouch to toggle 1x1 / 3x3 mode; Click to instantly drill a 3x3 rock slice
        if held_id == rules::ITEM_MINING_DRILL {
            static DRILL_MODE_3X3: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
            if is_crouching {
                *canceled = true;
                let prev = DRILL_MODE_3X3.fetch_xor(true, std::sync::atomic::Ordering::Relaxed);
                let mode_now = !prev;
                if let Some(who) = game.native_acting_player() {
                    let msg = if mode_now {
                        "⚡ [Taladro Minero] Modo: 3x3 Excavación Amplia"
                    } else {
                        "⚡ [Taladro Minero] Modo: 1x1 Precisión"
                    };
                    game.native_action_bar(who.uuid, Text::literal(msg));
                }
                return;
            } else if DRILL_MODE_3X3.load(std::sync::atomic::Ordering::Relaxed) {
                *canceled = true;
                let [cx, cy, cz] = pos;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        for dz in -1..=1 {
                            if (dx as i32).abs() + (dz as i32).abs() <= 2 && (dy as i32).abs() <= 1 {
                                let p = [cx + dx, cy + dy, cz + dz];
                                let b = game.native_block(p);
                                if !b.is_empty() && b != "minecraft:air" && b != "minecraft:bedrock" && !b.starts_with("minecraft:barrier") {
                                    game.native_run_command(&format!("setblock {} {} {} minecraft:air destroy", p[0], p[1], p[2]));
                                }
                            }
                        }
                    }
                }
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal("⚡ [Taladro Minero] 3x3 Excavado"));
                }
                return;
            }
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
    }

    if block_id == rules::BLOCK_RUBBER_SAPLING {
        *canceled = true;
        // Require bone meal in hand to accelerate rubber tree growth (50% chance per bone meal)
        if let Some((held_id, _, _)) = held_info {
            if held_id == "minecraft:bone_meal" {
                game.native_take_items("minecraft:bone_meal", 1);
                if (game.world.game_time & 1) == 0 {
                    grow_rubber_tree(game, pos);
                }
            }
        }
        return;
    }

    if block_id == rules::BLOCK_RUBBER_SAP_LOG {
        *canceled = true;
        let has_sap = game.native_block_property(pos, "lit").as_deref() == Some("true");
        if has_sap {
            game.native_set_block_property(pos, "lit", "false");
            game.native_give_items(rules::ITEM_RAW_SAP, 3);
            if (game.world.game_time & 3) == 0 {
                game.native_give_items(rules::BLOCK_RUBBER_SAPLING, 1);
            }
        }
        return;
    }

    if block_id == rules::BLOCK_FLUID_TANK {
        *canceled = true;
        if let Some((ref held_id, _, _)) = held_info {
            if held_id == "minecraft:water_bucket" {
                game.native_take_items("minecraft:water_bucket", 1);
                game.native_give_items("minecraft:bucket", 1);
                game.native_set_block_property(pos, "lit", "true");
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal("💧 [Tanque de Fluidos] Lleno con Agua (1000 mB)"));
                }
                return;
            } else if held_id == "minecraft:lava_bucket" {
                game.native_take_items("minecraft:lava_bucket", 1);
                game.native_give_items("minecraft:bucket", 1);
                game.native_set_block_property(pos, "lit", "true");
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal("🔥 [Tanque de Fluidos] Lleno con Lava (1000 mB)"));
                }
                return;
            } else if held_id == rules::ITEM_RAW_SAP {
                game.native_take_items(rules::ITEM_RAW_SAP, 1);
                game.native_set_block_property(pos, "lit", "true");
                if let Some(who) = game.native_acting_player() {
                    game.native_action_bar(who.uuid, Text::literal("🍯 [Tanque de Fluidos] Almacenando Savia de Caucho"));
                }
                return;
            } else if held_id == "minecraft:bucket" {
                let is_filled = game.native_block_property(pos, "lit").as_deref() == Some("true");
                if is_filled {
                    game.native_take_items("minecraft:bucket", 1);
                    game.native_give_items("minecraft:water_bucket", 1);
                    game.native_set_block_property(pos, "lit", "false");
                    if let Some(who) = game.native_acting_player() {
                        game.native_action_bar(who.uuid, Text::literal("🪣 [Tanque de Fluidos] Vaciado en Balde"));
                    }
                    return;
                }
            }
        }
        let is_filled = game.native_block_property(pos, "lit").as_deref() == Some("true");
        if let Some(who) = game.native_acting_player() {
            let msg = if is_filled {
                "💧 [Tanque de Fluidos] Nivel: 1,000 / 1,000 mB (Lleno)"
            } else {
                "💧 [Tanque de Fluidos] Nivel: 0 / 1,000 mB (Vacío - Usa un balde)"
            };
            game.native_action_bar(who.uuid, Text::literal(msg));
        }
        return;
    }

    if block_id == rules::BLOCK_ENERGY_MONITOR {
        *canceled = true;
        let consumers = rules::find_connected_consumers(
            pos,
            |p| game.native_block(p) == rules::BLOCK_CABLE,
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

    if block_id == rules::BLOCK_CABLE {
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
            _ => {}
        }
        store_state(entity, &state);
    }
}

pub(in crate::game) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    if let Some(kind) = MachineKind::from_block_id(block) {
        let slots = match kind {
            MachineKind::CoalGenerator | MachineKind::BioGenerator => {
                vec![Some([80.0, 36.0])] // Single centered fuel slot
            }
            MachineKind::SolarGenerator => {
                vec![] // Pure telemetry screen
            }
            MachineKind::EnergyBank => {
                vec![Some([80.0, 36.0])] // Handheld battery recharge slot
            }
            _ => {
                vec![
                    Some([44.0, 35.0]),  // Slot 0: Input / Belt Item
                    None,                // Slot 1: Hidden for pure cable-powered machines
                    Some([106.0, 35.0]), // Slot 2: Output
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

    // 3. Progress Bar / Radar Monitor / Combustion Gauge
    if matches!(kind, MachineKind::Quarry | MachineKind::AutoHarvester) {
        // Radar monitor display inside [64, 28]..[102, 54]
        let side = quarry_radius * 2 + 1;
        let area_txt = format!("{side}x{side}");
        let y_txt = if kind == MachineKind::Quarry {
            format!("Y:{quarry_y}")
        } else {
            format!("#{quarry_mined}")
        };
        game.native_text(batch, &area_txt, [x + 68.0, y + 31.0], 0x55_FFAA, false);
        game.native_text(batch, &y_txt, [x + 68.0, y + 42.0], 0x38_F2FF, false);
    } else if matches!(kind, MachineKind::CoalGenerator | MachineKind::BioGenerator) {
        // Combustion gauge directly above centered fuel slot [80, 36]
        game.native_fill(batch, [x + 74.0, y + 25.0, 28.0, 5.0], 0xFF3A_3F48);
        let bar_w = (28.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 28.0);
        if bar_w > 0.0 {
            let col = if kind == MachineKind::CoalGenerator { 0xFFFF_6B35 } else { 0xFF70_E000 };
            game.native_fill(batch, [x + 74.0, y + 25.0, bar_w, 5.0], col);
        }
    } else if kind == MachineKind::SolarGenerator {
        // Monitor banner covering slot area
        let is_3x3 = get(7) != 0;
        game.native_fill(batch, [x + 36.0, y + 26.0, 104.0, 24.0], 0xFF18_2028);
        if is_3x3 {
            game.native_text(batch, "[Matriz 3x3: 2x ACTIVA]", [x + 40.0, y + 34.0], 0x55_FFAA, false);
        } else {
            game.native_text(batch, "[Panel Solar Indiv.]", [x + 44.0, y + 34.0], 0xFF_D166, false);
        }
    } else if kind == MachineKind::EnergyBank {
        // Modular storage gauge above center battery slot
        game.native_fill(batch, [x + 60.0, y + 25.0, 56.0, 5.0], 0xFF2A_3038);
        let bar_w = (56.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 56.0);
        if bar_w > 0.0 {
            game.native_fill(batch, [x + 60.0, y + 25.0, bar_w, 5.0], 0xFF00_E5FF);
        }
    } else {
        game.native_fill(batch, [x + 66.0, y + 39.0, 34.0, 8.0], 0xFF3A_3F48);
        let bar_w = (34.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 34.0);
        if bar_w > 0.0 {
            let bar_col = if matches!(kind, MachineKind::ConveyorBelt | MachineKind::ItemExtractor) {
                0xFFFF_B820
            } else {
                0xFF38_D6FF
            };
            game.native_fill(batch, [x + 66.0, y + 39.0, bar_w, 8.0], bar_col);
        }
    }

    // 4. Vertical Energy Bar on left edge `[8, 30, 6, 24]`
    if !matches!(kind, MachineKind::ConveyorBelt | MachineKind::ItemExtractor) {
        game.native_fill(batch, [x + 8.0, y + 31.0, 6.0, 24.0], 0xFF26_2A30);
        let e_ratio = (energy as f32 / max_energy as f32).clamp(0.0, 1.0);
        let e_h = 24.0 * e_ratio;
        if e_h > 0.0 {
            game.native_fill(
                batch,
                [x + 8.0, y + 31.0 + (24.0 - e_h), 6.0, e_h],
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
        let line1 = format!("FE: {energy}/{max_energy} (+20 FE/t)");
        let line2 = format!("Combustión Carbón: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0xFF_9E00, false);
    } else if kind == MachineKind::BioGenerator {
        let line1 = format!("FE: {energy}/{max_energy} (+24 FE/t)");
        let line2 = format!("Biomasa / Savia: {progress_pct}%");
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, &line2, [x + 10.0, y + 70.0], 0x70_E000, false);
    } else if kind == MachineKind::SolarGenerator {
        let is_3x3 = get(7) != 0;
        let line1 = format!("FE: {energy}/{max_energy} (+{rate} FE/t)");
        let line2 = if is_3x3 {
            "Matriz Solar 3x3: +100% Doble Captación"
        } else {
            "Tip: Coloca 3x3 paneles juntos para 2x!"
        };
        let col2 = if is_3x3 { 0x55_FFAA } else { 0xFF_D166 };
        game.native_text(batch, &line1, [x + 10.0, y + 60.0], 0x70_E8FF, false);
        game.native_text(batch, line2, [x + 10.0, y + 70.0], col2, false);
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
    let pwr_label = lang::get(if enabled {
        "rusty_factory.screen.start"
    } else {
        "rusty_factory.screen.stop"
    });
    let pwr_bg = if enabled { 0xFF2B_8A3E } else { 0xFFA6_1E22 };
    draw_button(game, batch, origin, POWER_BTN_RECT, &pwr_label, pwr_bg);

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
            | MachineKind::SolarGenerator
            | MachineKind::EnergyBank
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
    if in_rect(at, POWER_BTN_RECT) {
        *button = Some(BTN_TOGGLE_POWER);
    } else if in_rect(at, MODE_BTN_RECT)
        && !matches!(
            kind,
            MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SolarGenerator
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

    if in_rect(at, [6.0, 29.0, 10.0, 28.0])
        && !matches!(kind, MachineKind::ConveyorBelt | MachineKind::ItemExtractor)
    {
        lines.push(format!("§bAlmacenamiento: {energy} / {max_energy} FE"));
        lines.push("§7Conecta Cables de Cobre o usa una Batería".into());
    } else if in_rect(at, [64.0, 28.0, 38.0, 26.0]) {
        if matches!(kind, MachineKind::Quarry | MachineKind::AutoHarvester) {
            let side = quarry_radius * 2 + 1;
            lines.push(format!("§eÁrea 3D activa: {side}x{side}"));
            lines.push(format!("§7Progreso del ciclo actual: {progress}%"));
        } else {
            lines.push(format!("§aProgreso: {progress}%"));
        }
    } else if in_rect(at, POWER_BTN_RECT) {
        lines.push("§fAlternar Encendido / Pausa de la máquina".into());
    } else if in_rect(at, MODE_BTN_RECT)
        && !matches!(
            kind,
            MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SolarGenerator
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
        "rusty_factory:solar_generator 4",
        "rusty_factory:energy_bank 1",
        "rusty_factory:copper_cable 32",
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
    ] {
        game.native_run_command(&format!("give @s {item}"));
    }
    game.native_feedback(Text::literal(
        "[Rusty Factory v0.4.0] ¡Kit industrial completo entregado (Árbol de Caucho con Savia, Espita, Extractor de Savia, Vulcanizadora de Goma, Bio-Generador, Quarry 3D, Cosechadora 3D, Paneles Solares y más)!",
    ));
}
