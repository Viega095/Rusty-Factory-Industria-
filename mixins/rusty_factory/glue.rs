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
    text::{self, Style, Text},
    world_primitives::WorldPrimitives,
};
use horizon_core::geometry::Direction;
use horizon_world::{
    nbt::{Compound, JavaString, Tag},
    registry::BlockStateId,
};
use rules::{MachineKind, MachineState, StepStatus};

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
    if MachineKind::from_block_id(block).is_some() && entity.is_none() {
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
    if MachineKind::from_block_id(block).is_some() {
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
    if MachineKind::from_block_id(block).is_some() {
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

fn fuel_burn_ticks(stack: &ItemStack) -> Option<i32> {
    match &*stack.item {
        rules::ITEM_BATTERY => Some(-1),
        "minecraft:coal" | "minecraft:charcoal" => Some(1600),
        "minecraft:coal_block" => Some(16000),
        "minecraft:blaze_rod" => Some(2400),
        "minecraft:lava_bucket" => Some(20000),
        "minecraft:dried_kelp_block" => Some(4000),
        "minecraft:oak_planks"
        | "minecraft:spruce_planks"
        | "minecraft:birch_planks"
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
            | rules::BLOCK_CONVEYOR
            | rules::BLOCK_GENERATOR
    )
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

/// Distributes generated FE from a Coal Generator across connected copper cables and adjacent machines.
fn distribute_energy(game: &mut Game<'_>, gen_pos: [i32; 3], available: i32) -> i32 {
    if available <= 0 {
        return 0;
    }
    let targets = rules::find_connected_consumers(
        gen_pos,
        |p| game.native_block(p) == rules::BLOCK_CABLE,
        |p| {
            matches!(
                MachineKind::from_block_id(&game.native_block(p)),
                Some(
                    MachineKind::ElectricCrusher
                        | MachineKind::ElectricFurnace
                        | MachineKind::MechanicalPress
                        | MachineKind::Quarry
                )
            )
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
            let send = remaining.min(space).min(200);
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
            if let Some((drop_item, drop_count)) = rules::quarry_drop_for_block(&block_id) {
                if let Some(air_state) = game.native_default_state("minecraft:air") {
                    game.native_set_block(dim, target_pos, air_state, 3);
                }
                state.quarry_mined += 1;
                state.last_target = target_pos;

                let produced = ItemStack {
                    item: drop_item.into(),
                    count: drop_count,
                    damage: 0,
                    extra: None,
                };

                // Put into output slot or route directly into pipes/chests
                let mut rest = Some(produced);
                if state.auto_io {
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

fn consume_one_fuel(entity: &mut ModBlockEntity) {
    if let Some(fuel_stack) = entity.items[rules::SLOT_FUEL].as_mut() {
        let is_lava = &*fuel_stack.item == "minecraft:lava_bucket";
        if fuel_stack.count > 1 {
            fuel_stack.count -= 1;
        } else {
            entity.items[rules::SLOT_FUEL] = is_lava.then(|| ItemStack {
                item: "minecraft:bucket".into(),
                count: 1,
                damage: 0,
                extra: None,
            });
        }
    }
}

pub(in crate::game) fn tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let Some(kind) = MachineKind::from_block_id(&entity.id) else {
        return;
    };
    if !game.native_on(rules::ID) {
        return;
    }
    if entity.items.len() < rules::SLOTS {
        entity.items.resize(rules::SLOTS, None);
    }

    let mut state = load_state(entity).unwrap_or_else(|| MachineState::new(kind));
    let speed_upgrades = entity.items[rules::SLOT_UPGRADE]
        .as_ref()
        .filter(|s| &*s.item == rules::ITEM_SPEED_UPGRADE)
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

    let fuel_ticks = entity.items[rules::SLOT_FUEL]
        .as_ref()
        .and_then(fuel_burn_ticks);

    if kind == MachineKind::Quarry {
        let (status, rate) = tick_quarry(game, pos, entity, &mut state, fuel_ticks, speed_upgrades);
        update_lit_state(game, pos, status == StepStatus::Working);
        let needed = rules::effective_work_ticks(kind, speed_upgrades).max(1);
        let pct = ((state.progress * 100) / needed).clamp(0, 100);
        let cur_y = (pos[1] - 1 - state.quarry_y_offset).max(-60);
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
        match entity.items[rules::SLOT_OUTPUT].as_mut() {
            Some(existing) if &*existing.item == prod.item => {
                existing.count += prod.count;
            }
            None => {
                entity.items[rules::SLOT_OUTPUT] = Some(ItemStack {
                    item: prod.item.into(),
                    count: prod.count,
                    damage: 0,
                    extra: None,
                });
            }
            _ => {}
        }
    }

    if state.auto_io && kind != MachineKind::CoalGenerator && entity.items[rules::SLOT_OUTPUT].is_some() {
        try_auto_eject_output(game, pos, entity, 4);
    }

    if kind == MachineKind::CoalGenerator && state.energy > 0 {
        let sent = distribute_energy(game, pos, state.energy.min(240));
        state.energy -= sent;
    }

    update_lit_state(game, pos, outcome.status == StepStatus::Working);

    let needed = rules::effective_work_ticks(kind, speed_upgrades).max(1);
    let pct = if kind == MachineKind::CoalGenerator {
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
        if entity.id != rules::BLOCK_QUARRY {
            continue;
        }
        let dx = x as f64 + 0.5 - camera[0];
        let dy = y as f64 + 0.5 - camera[1];
        let dz = z as f64 + 0.5 - camera[2];
        if dx * dx + dy * dy + dz * dz > 64.0 * 64.0 {
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

/// Inspects networks or triggers manual pipe extraction when right-clicking cables/pipes/machines.
pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let Some(who) = game.native_acting_player() else {
        return;
    };
    let block_id = game.native_block(pos);

    if block_id == rules::BLOCK_CABLE {
        let consumers = rules::find_connected_consumers(
            pos,
            |p| game.native_block(p) == rules::BLOCK_CABLE,
            |p| MachineKind::from_block_id(&game.native_block(p)).is_some(),
        );
        let style = Style {
            color: Some(text::AQUA),
            ..Style::default()
        };
        game.native_tell(
            who.uuid,
            Text::styled(
                format!(
                    "[Rusty Factory] Cable de Energía | Máquinas conectadas en red: {}",
                    consumers.len()
                ),
                style,
            ),
        );
        return;
    }

    if block_id == rules::BLOCK_ITEM_PIPE {
        let dim = game.native_dimension();
        let targets = rules::find_connected_item_targets(
            pos,
            |p| game.native_block(p) == rules::BLOCK_ITEM_PIPE,
            |p| is_item_container_block(&game.native_block(p)),
        );
        // Also pull up to 4 items from any adjacent chest/furnace and route them through the pipe network!
        let mut moved_total = 0u32;
        for [dx, dy, dz] in rules::NEIGHBORS_6 {
            let adj = [pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let adj_id = game.native_block(adj);
            if matches!(
                adj_id.as_str(),
                "minecraft:chest" | "minecraft:trapped_chest" | "minecraft:barrel" | "minecraft:hopper"
            ) {
                for &dest in &targets {
                    if dest != adj {
                        moved_total += game.native_move_items(
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

        let style = Style {
            color: Some(text::GOLD),
            ..Style::default()
        };
        let msg = if moved_total > 0 {
            format!(
                "[Rusty Factory] Tubería de Ítems | Contenedores en red: {} | ¡{} ítems transportados!",
                targets.len(),
                moved_total
            )
        } else {
            format!(
                "[Rusty Factory] Tubería de Ítems | Contenedores/Máquinas conectados: {}",
                targets.len()
            )
        };
        game.native_tell(who.uuid, Text::styled(msg, style));
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
                if kind == MachineKind::Quarry {
                    // Cycle Quarry mining area: 5x5 (2) -> 7x7 (3) -> 9x9 (4) -> 5x5 (2)
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
    if MachineKind::from_block_id(block).is_some() {
        *screen = Some(NativeScreen {
            width: 176.0,
            height: 166.0,
            slots: vec![
                Some([44.0, 35.0]),  // Slot 0: Input / Belt Item
                Some([17.0, 35.0]),  // Slot 1: Fuel / Battery
                Some([106.0, 35.0]), // Slot 2: Output
                Some([143.0, 35.0]), // Slot 3: Speed Upgrade
            ],
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
    let title = lang::get(kind.title_key());
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

    // 3. Progress Bar / Radar Monitor between slots [44, 35] and [106, 35]
    if kind == MachineKind::Quarry {
        // Radar monitor display inside [64, 28]..[102, 54]
        let side = quarry_radius * 2 + 1;
        let area_txt = format!("{side}x{side}");
        let y_txt = format!("Y:{quarry_y}");
        game.native_text(batch, &area_txt, [x + 68.0, y + 31.0], 0x55_FFAA, false);
        game.native_text(batch, &y_txt, [x + 68.0, y + 42.0], 0x38_F2FF, false);
    } else {
        game.native_fill(batch, [x + 66.0, y + 39.0, 34.0, 8.0], 0xFF3A_3F48);
        let bar_w = (34.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 34.0);
        if bar_w > 0.0 {
            let bar_col = if kind == MachineKind::ConveyorBelt {
                0xFFFF_B820
            } else {
                0xFF38_D6FF
            };
            game.native_fill(batch, [x + 66.0, y + 39.0, bar_w, 8.0], bar_col);
        }
    }

    // 4. Vertical Energy Bar on left edge `[8, 30, 6, 24]`
    if kind != MachineKind::ConveyorBelt {
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
    } else if kind == MachineKind::ConveyorBelt {
        let line1 = format!("Velocidad Cinta: {}%", 100 + progress_pct / 2);
        let line2 = if auto_io {
            "Auto-Extraer (Atrás): ACTIVO"
        } else {
            "Auto-Extraer (Atrás): MANUAL"
        };
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

    if kind == MachineKind::Quarry {
        let side = quarry_radius * 2 + 1;
        let size_label = format!("{side}x{side}");
        draw_button(game, batch, origin, MODE_BTN_RECT, &size_label, 0xFF3B_5B92);
        draw_button(game, batch, origin, RESET_BTN_RECT, "TOP", 0xFF6C_584C);
    } else if kind != MachineKind::CoalGenerator {
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
    }
    if in_rect(at, POWER_BTN_RECT) {
        *button = Some(BTN_TOGGLE_POWER);
    } else if in_rect(at, MODE_BTN_RECT) && kind != MachineKind::CoalGenerator {
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

    if in_rect(at, [6.0, 29.0, 10.0, 28.0]) && kind != MachineKind::ConveyorBelt {
        lines.push(format!("§bAlmacenamiento: {energy} / {max_energy} FE"));
        lines.push("§7Conecta Cables de Cobre o usa una Batería".into());
    } else if in_rect(at, [64.0, 28.0, 38.0, 26.0]) {
        if kind == MachineKind::Quarry {
            let side = quarry_radius * 2 + 1;
            lines.push(format!("§eExcavación 3D activa: área {side}x{side}"));
            lines.push(format!("§7Progreso del bloque actual: {progress}%"));
        } else {
            lines.push(format!("§aProgreso: {progress}%"));
        }
    } else if in_rect(at, POWER_BTN_RECT) {
        lines.push("§fAlternar Encendido / Pausa de la máquina".into());
    } else if in_rect(at, MODE_BTN_RECT) && kind != MachineKind::CoalGenerator {
        if kind == MachineKind::Quarry {
            let side = quarry_radius * 2 + 1;
            lines.push(format!("§eTamaño de Quarry: {side}x{side}"));
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
        "rusty_factory:quarry 1",
        "rusty_factory:coal_generator 2",
        "rusty_factory:copper_cable 32",
        "rusty_factory:item_pipe 32",
        "rusty_factory:conveyor_belt 32",
        "rusty_factory:electric_crusher 1",
        "rusty_factory:electric_furnace 1",
        "rusty_factory:mechanical_press 1",
        "rusty_factory:speed_upgrade 8",
        "rusty_factory:energy_battery 4",
    ] {
        game.native_run_command(&format!("give @s {item}"));
    }
    game.native_feedback(Text::literal(
        "[Rusty Factory] ¡Kit industrial completo entregado (Quarry 3D, Tuberías, Cintas, Generadores y Máquinas)!",
    ));
}
