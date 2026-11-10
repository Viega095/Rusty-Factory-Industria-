// Horizon engine glue for Rusty Factory (`crate::game::rusty_factory_glue`).
// Connects Horizon's native block entity hooks, container menus, screens, and
// player interactions to the pure Rust rules in `runtime.rs`.

use crate::game::{Game, rusty_factory as rules};
use horizon_client::{
    containers::{BlockEntity, ModBlockEntity, ModContainer},
    hud::HudBatch,
    items::ItemStack,
    lang,
    mod_api::{NativeCommand, NativeMod, NativeScreen},
    text::{self, Style, Text},
};
use horizon_core::geometry::Direction;
use horizon_world::nbt::{Compound, JavaString, Tag};
use rules::{MachineKind, MachineState, StepStatus};

const STATE_TAG: &str = "rusty_machine";
const TOGGLE_BUTTON: i32 = 0;
const BUTTON_RECT: [f32; 4] = [136.0, 14.0, 32.0, 16.0];

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
            "Industrial automation, energy networks, crusher, electric furnace, press and conveyor belts.",
        ),
        keys: Vec::new(),
    });
}

pub(in crate::game) fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if MachineKind::from_block_id(block).is_some() && entity.is_none() {
        *entity = Some((block.to_owned(), Compound::default()));
    }
}

pub(in crate::game) fn block_entity_ticks(block: &str, ticks: &mut bool) {
    if MachineKind::from_block_id(block).is_some() {
        *ticks = true;
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
            let send = remaining.min(space).min(120);
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

/// Moves items along a Conveyor Belt into the container in front of it.
fn tick_conveyor(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    let facing = game
        .native_block_property(pos, "facing")
        .unwrap_or_else(|| "north".to_owned());
    let (front_off, _) = rules::conveyor_offsets(&facing);
    let front_pos = [
        pos[0] + front_off[0],
        pos[1] + front_off[1],
        pos[2] + front_off[2],
    ];

    for slot in 0..rules::SLOTS {
        if let Some(stack) = entity.items.get(slot).cloned().flatten() {
            let mut one = stack.clone();
            one.count = 1;
            let rest = game.native_insert_item(front_pos, Direction::Up, one);
            if rest.is_none() {
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

    if kind == MachineKind::ConveyorBelt {
        let outcome = rules::step_machine(&mut state, None, None, None, 0);
        if state.progress == 0 {
            tick_conveyor(game, pos, entity);
        }
        entity.data = vec![outcome.status as i32, 0, 0, state.progress * 10, 0, i32::from(state.enabled)];
        store_state(entity, &state);
        return;
    }

    let input_id = entity.items[rules::SLOT_INPUT]
        .as_ref()
        .map(|s| s.item.to_string());
    let fuel_ticks = entity.items[rules::SLOT_FUEL]
        .as_ref()
        .and_then(fuel_burn_ticks);
    let output_info = entity.items[rules::SLOT_OUTPUT].as_ref().map(|s| {
        let max_stack = game.native_max_stack_size(&s.item).unwrap_or(64);
        (s.item.to_string(), s.count, max_stack)
    });
    let speed_upgrades = entity.items[rules::SLOT_UPGRADE]
        .as_ref()
        .filter(|s| &*s.item == rules::ITEM_SPEED_UPGRADE)
        .map_or(0, |s| s.count);

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

    if kind == MachineKind::CoalGenerator && state.energy > 0 {
        let sent = distribute_energy(game, pos, state.energy.min(160));
        state.energy -= sent;
    }

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
    ];

    if entity.data != data {
        entity.data = data;
    }
    store_state(entity, &state);
}

/// Inspects or toggles a machine when right-clicking with the Industrial Wrench.
pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: [i32; 3], canceled: &mut bool) {
    if *canceled {
        return;
    }
    let Some(who) = game.native_acting_player() else {
        return;
    };
    let block_id = game.native_block(pos);

    // Check if the block is a cable or a machine and player is holding the Wrench
    let is_factory_block =
        block_id == rules::BLOCK_CABLE || MachineKind::from_block_id(&block_id).is_some();
    if !is_factory_block {
        return;
    }

    // If it's a cable, report connected machines count in chat
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
                    "[Rusty Factory] Cable de Cobre | Máquinas conectadas en red: {}",
                    consumers.len()
                ),
                style,
            ),
        );
        return;
    }

    if let Some(BlockEntity::Mod(m)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        if let Some(st) = load_state(m) {
            let style = Style {
                color: Some(text::GOLD),
                ..Style::default()
            };
            game.native_tell(
                who.uuid,
                Text::styled(
                    format!(
                        "[Rusty Factory] Energía: {} / {} FE | Progreso: {}% | Estado: {}",
                        st.energy,
                        st.kind.max_energy(),
                        st.progress,
                        if st.enabled { "ACTIVO" } else { "PAUSADO" }
                    ),
                    style,
                ),
            );
        }
    }
}

pub(in crate::game) fn menu_button(entity: &mut ModBlockEntity, id: i32) {
    if let Some(kind) = MachineKind::from_block_id(&entity.id) {
        if id == TOGGLE_BUTTON {
            let mut state = load_state(entity).unwrap_or_else(|| MachineState::new(kind));
            state.enabled = !state.enabled;
            store_state(entity, &state);
        }
    }
}

pub(in crate::game) fn screen(block: &str, screen: &mut Option<NativeScreen>) {
    if MachineKind::from_block_id(block).is_some() {
        *screen = Some(NativeScreen {
            width: 176.0,
            height: 166.0,
            slots: vec![
                Some([44.0, 35.0]),  // Slot 0: Input
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
        "rusty_factory:gui/machine",
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

    // Draw Title
    let title = lang::get(kind.title_key());
    game.native_text(batch, &title, [x + 8.0, y + 6.0], 0x30_3030, false);

    // Draw Status
    let status_text = lang::get(&format!("rusty_factory.status.{status_code}"));
    let status_color = if status_code == StepStatus::Working as i32 {
        0x1E_7828
    } else {
        0x9E_2A2B
    };
    game.native_text(batch, &status_text, [x + 8.0, y + 18.0], status_color, false);

    // Draw Progress Bar (between input [44,35] and output [106,35])
    game.native_fill(batch, [x + 68.0, y + 39.0, 30.0, 8.0], 0xFF55_5555);
    let bar_w = (30.0 * (progress_pct as f32 / 100.0)).clamp(0.0, 30.0);
    if bar_w > 0.0 {
        game.native_fill(batch, [x + 68.0, y + 39.0, bar_w, 8.0], 0xFF38_D6FF);
    }

    // Draw Energy Bar on left edge
    game.native_fill(batch, [x + 8.0, y + 30.0, 6.0, 28.0], 0xFF33_3333);
    let e_ratio = (energy as f32 / max_energy as f32).clamp(0.0, 1.0);
    let e_h = 28.0 * e_ratio;
    if e_h > 0.0 {
        game.native_fill(
            batch,
            [x + 8.0, y + 30.0 + (28.0 - e_h), 6.0, e_h],
            0xFF00_D4FF,
        );
    }

    // Draw Energy & Progress text lines
    let energy_line = fill_template(
        "rusty_factory.screen.energy",
        &[energy.to_string(), max_energy.to_string()],
    );
    let rate_line = fill_template("rusty_factory.screen.rate", &[rate.to_string()]);
    game.native_text(batch, &energy_line, [x + 8.0, y + 60.0], 0x35_3535, false);
    game.native_text(batch, &rate_line, [x + 8.0, y + 71.0], 0x35_3535, false);

    // Draw ON/OFF Button
    let [bx, by, bw, bh] = BUTTON_RECT;
    game.native_fill(batch, [x + bx, y + by, bw, bh], 0xFF40_4040);
    let btn_bg = if enabled { 0xFF2B_8A3E } else { 0xFFA6_1E22 };
    game.native_fill(
        batch,
        [x + bx + 1.0, y + by + 1.0, bw - 2.0, bh - 2.0],
        btn_bg,
    );
    let btn_label = lang::get(if enabled {
        "rusty_factory.screen.start"
    } else {
        "rusty_factory.screen.stop"
    });
    let tw = game.native_text_width(&btn_label);
    game.native_text(
        batch,
        &btn_label,
        [x + bx + (bw - tw) / 2.0, y + by + 4.0],
        0xFF_FFFF,
        false,
    );
}

pub(in crate::game) fn screen_click(
    block: &str,
    at: [f32; 2],
    right: bool,
    button: &mut Option<i32>,
) {
    let [bx, by, bw, bh] = BUTTON_RECT;
    if MachineKind::from_block_id(block).is_some()
        && !right
        && (bx..bx + bw).contains(&at[0])
        && (by..by + bh).contains(&at[1])
    {
        *button = Some(TOGGLE_BUTTON);
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
        "rusty_factory:coal_generator 2",
        "rusty_factory:copper_cable 32",
        "rusty_factory:electric_crusher 1",
        "rusty_factory:electric_furnace 1",
        "rusty_factory:mechanical_press 1",
        "rusty_factory:conveyor_belt 16",
        "rusty_factory:speed_upgrade 8",
        "rusty_factory:energy_battery 4",
    ] {
        game.native_run_command(&format!("give @s {item}"));
    }
    game.native_feedback(Text::literal(
        "[Rusty Factory] ¡Kit de fábrica industrial entregado!",
    ));
}
