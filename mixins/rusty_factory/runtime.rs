// Pure Rust logic for the "Rusty Factory" (rusty_factory) Horizon native mod.
// No engine imports here so `cargo test` runs standalone in milliseconds.

use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

pub const ID: &str = "rusty_factory";
pub const VERSION: &str = "0.2.0";
pub const MENU: &str = "rusty_factory:machine";

// Block IDs
pub const BLOCK_CASING: &str = "rusty_factory:machine_casing";
pub const BLOCK_GENERATOR: &str = "rusty_factory:coal_generator";
pub const BLOCK_CABLE: &str = "rusty_factory:copper_cable";
pub const BLOCK_ITEM_PIPE: &str = "rusty_factory:item_pipe";
pub const BLOCK_CRUSHER: &str = "rusty_factory:electric_crusher";
pub const BLOCK_FURNACE: &str = "rusty_factory:electric_furnace";
pub const BLOCK_PRESS: &str = "rusty_factory:mechanical_press";
pub const BLOCK_QUARRY: &str = "rusty_factory:quarry";
pub const BLOCK_CONVEYOR: &str = "rusty_factory:conveyor_belt";

// Item IDs
pub const ITEM_WRENCH: &str = "rusty_factory:wrench";
pub const ITEM_SPEED_UPGRADE: &str = "rusty_factory:speed_upgrade";
pub const ITEM_BATTERY: &str = "rusty_factory:energy_battery";
pub const ITEM_CIRCUIT: &str = "rusty_factory:electronic_circuit";
pub const ITEM_STEEL_INGOT: &str = "rusty_factory:steel_ingot";
pub const ITEM_STEEL_GEAR: &str = "rusty_factory:steel_gear";
pub const ITEM_IRON_PLATE: &str = "rusty_factory:iron_plate";
pub const ITEM_COPPER_PLATE: &str = "rusty_factory:copper_plate";
pub const ITEM_GOLD_PLATE: &str = "rusty_factory:gold_plate";
pub const ITEM_IRON_DUST: &str = "rusty_factory:iron_dust";
pub const ITEM_COPPER_DUST: &str = "rusty_factory:copper_dust";
pub const ITEM_GOLD_DUST: &str = "rusty_factory:gold_dust";

// Container slot layout
pub const SLOTS: usize = 4;
pub const SLOT_INPUT: usize = 0;
pub const SLOT_FUEL: usize = 1;
pub const SLOT_OUTPUT: usize = 2;
pub const SLOT_UPGRADE: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineKind {
    CoalGenerator,
    ElectricCrusher,
    ElectricFurnace,
    MechanicalPress,
    Quarry,
    ConveyorBelt,
}

impl MachineKind {
    pub fn from_block_id(block: &str) -> Option<Self> {
        match block {
            BLOCK_GENERATOR => Some(Self::CoalGenerator),
            BLOCK_CRUSHER => Some(Self::ElectricCrusher),
            BLOCK_FURNACE => Some(Self::ElectricFurnace),
            BLOCK_PRESS => Some(Self::MechanicalPress),
            BLOCK_QUARRY => Some(Self::Quarry),
            BLOCK_CONVEYOR => Some(Self::ConveyorBelt),
            _ => None,
        }
    }

    pub const fn max_energy(self) -> i32 {
        match self {
            Self::CoalGenerator => 20_000,
            Self::ElectricCrusher => 12_000,
            Self::ElectricFurnace => 12_000,
            Self::MechanicalPress => 12_000,
            Self::Quarry => 25_000,
            Self::ConveyorBelt => 0,
        }
    }

    pub const fn base_work_ticks(self) -> i32 {
        match self {
            Self::CoalGenerator => 1,
            Self::ElectricCrusher => 70,
            Self::ElectricFurnace => 50,
            Self::MechanicalPress => 60,
            Self::Quarry => 16,
            Self::ConveyorBelt => 8,
        }
    }

    pub const fn energy_per_tick(self) -> i32 {
        match self {
            Self::CoalGenerator => 40, // Generates +40 FE/t
            Self::ElectricCrusher => 20,
            Self::ElectricFurnace => 16,
            Self::MechanicalPress => 18,
            Self::Quarry => 28,
            Self::ConveyorBelt => 0,
        }
    }

    pub const fn title_key(self) -> &'static str {
        match self {
            Self::CoalGenerator => "block.rusty_factory.coal_generator",
            Self::ElectricCrusher => "block.rusty_factory.electric_crusher",
            Self::ElectricFurnace => "block.rusty_factory.electric_furnace",
            Self::MechanicalPress => "block.rusty_factory.mechanical_press",
            Self::Quarry => "block.rusty_factory.quarry",
            Self::ConveyorBelt => "block.rusty_factory.conveyor_belt",
        }
    }

    pub const fn gui_sprite(self) -> &'static str {
        match self {
            Self::CoalGenerator => "rusty_factory:gui/generator",
            Self::Quarry => "rusty_factory:gui/quarry",
            Self::ConveyorBelt => "rusty_factory:gui/conveyor",
            Self::ElectricCrusher | Self::ElectricFurnace | Self::MechanicalPress => {
                "rusty_factory:gui/machine"
            }
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_quarry_radius() -> i32 {
    2 // 5x5 area (2*2 + 1)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineState {
    pub kind: MachineKind,
    pub energy: i32,
    pub progress: i32,
    pub burn_left: i32,
    pub burn_max: i32,
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub auto_io: bool,
    #[serde(default = "default_quarry_radius")]
    pub quarry_radius: i32,
    #[serde(default)]
    pub quarry_cursor: i32,
    #[serde(default)]
    pub quarry_y_offset: i32,
    #[serde(default)]
    pub quarry_mined: i32,
    #[serde(default)]
    pub last_target: [i32; 3],
    #[serde(default)]
    pub belt_item: String,
}

impl MachineState {
    pub fn new(kind: MachineKind) -> Self {
        Self {
            kind,
            energy: 0,
            progress: 0,
            burn_left: 0,
            burn_max: 0,
            enabled: true,
            auto_io: true,
            quarry_radius: 2,
            quarry_cursor: 0,
            quarry_y_offset: 0,
            quarry_mined: 0,
            last_target: [0, 0, 0],
            belt_item: String::new(),
        }
    }

    pub fn quarry_side_len(&self) -> i32 {
        self.quarry_radius.clamp(2, 4) * 2 + 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeOutput {
    pub item: &'static str,
    pub count: u32,
}

/// Returns the processing recipe for a given machine kind and input item ID.
pub fn lookup_recipe(kind: MachineKind, input: &str) -> Option<RecipeOutput> {
    match kind {
        MachineKind::ElectricCrusher => match input {
            // Ore doubling into dusts
            "minecraft:raw_iron" | "minecraft:iron_ore" | "minecraft:deepslate_iron_ore" => Some(
                RecipeOutput {
                    item: ITEM_IRON_DUST,
                    count: 2,
                },
            ),
            "minecraft:raw_copper" | "minecraft:copper_ore" | "minecraft:deepslate_copper_ore" => {
                Some(RecipeOutput {
                    item: ITEM_COPPER_DUST,
                    count: 2,
                })
            }
            "minecraft:raw_gold" | "minecraft:gold_ore" | "minecraft:deepslate_gold_ore" => Some(
                RecipeOutput {
                    item: ITEM_GOLD_DUST,
                    count: 2,
                },
            ),
            // Rock & mineral crushing
            "minecraft:stone" | "minecraft:deepslate" | "minecraft:cobbled_deepslate" => Some(
                RecipeOutput {
                    item: "minecraft:cobblestone",
                    count: 1,
                },
            ),
            "minecraft:cobblestone" => Some(RecipeOutput {
                item: "minecraft:gravel",
                count: 1,
            }),
            "minecraft:gravel" => Some(RecipeOutput {
                item: "minecraft:sand",
                count: 1,
            }),
            "minecraft:sandstone" => Some(RecipeOutput {
                item: "minecraft:sand",
                count: 2,
            }),
            "minecraft:coal_ore" | "minecraft:deepslate_coal_ore" => Some(RecipeOutput {
                item: "minecraft:coal",
                count: 3,
            }),
            "minecraft:diamond_ore" | "minecraft:deepslate_diamond_ore" => Some(RecipeOutput {
                item: "minecraft:diamond",
                count: 2,
            }),
            "minecraft:emerald_ore" | "minecraft:deepslate_emerald_ore" => Some(RecipeOutput {
                item: "minecraft:emerald",
                count: 2,
            }),
            "minecraft:redstone_ore" | "minecraft:deepslate_redstone_ore" => Some(RecipeOutput {
                item: "minecraft:redstone",
                count: 6,
            }),
            "minecraft:lapis_ore" | "minecraft:deepslate_lapis_ore" => Some(RecipeOutput {
                item: "minecraft:lapis_lazuli",
                count: 8,
            }),
            "minecraft:bone" => Some(RecipeOutput {
                item: "minecraft:bone_meal",
                count: 6,
            }),
            "minecraft:blaze_rod" => Some(RecipeOutput {
                item: "minecraft:blaze_powder",
                count: 4,
            }),
            "minecraft:amethyst_cluster" | "minecraft:amethyst_block" => Some(RecipeOutput {
                item: "minecraft:amethyst_shard",
                count: 4,
            }),
            "minecraft:glowstone" => Some(RecipeOutput {
                item: "minecraft:glowstone_dust",
                count: 4,
            }),
            _ => None,
        },

        MachineKind::ElectricFurnace => match input {
            // Mod dusts -> ingots
            ITEM_IRON_DUST | "minecraft:raw_iron" | "minecraft:iron_ore" | "minecraft:deepslate_iron_ore" => {
                Some(RecipeOutput {
                    item: "minecraft:iron_ingot",
                    count: 1,
                })
            }
            ITEM_COPPER_DUST
            | "minecraft:raw_copper"
            | "minecraft:copper_ore"
            | "minecraft:deepslate_copper_ore" => Some(RecipeOutput {
                item: "minecraft:copper_ingot",
                count: 1,
            }),
            ITEM_GOLD_DUST | "minecraft:raw_gold" | "minecraft:gold_ore" | "minecraft:deepslate_gold_ore" => {
                Some(RecipeOutput {
                    item: "minecraft:gold_ingot",
                    count: 1,
                })
            }
            // High-temp induction: Iron Ingot -> Steel Ingot
            "minecraft:iron_ingot" => Some(RecipeOutput {
                item: ITEM_STEEL_INGOT,
                count: 1,
            }),
            // Vanilla smelting favorites
            "minecraft:cobblestone" => Some(RecipeOutput {
                item: "minecraft:stone",
                count: 1,
            }),
            "minecraft:cobbled_deepslate" => Some(RecipeOutput {
                item: "minecraft:deepslate",
                count: 1,
            }),
            "minecraft:sand" | "minecraft:red_sand" => Some(RecipeOutput {
                item: "minecraft:glass",
                count: 1,
            }),
            "minecraft:clay_ball" => Some(RecipeOutput {
                item: "minecraft:brick",
                count: 1,
            }),
            "minecraft:netherrack" => Some(RecipeOutput {
                item: "minecraft:nether_brick",
                count: 1,
            }),
            "minecraft:kelp" => Some(RecipeOutput {
                item: "minecraft:dried_kelp",
                count: 1,
            }),
            "minecraft:oak_log"
            | "minecraft:spruce_log"
            | "minecraft:birch_log"
            | "minecraft:jungle_log"
            | "minecraft:acacia_log"
            | "minecraft:dark_oak_log"
            | "minecraft:cherry_log" => Some(RecipeOutput {
                item: "minecraft:charcoal",
                count: 1,
            }),
            "minecraft:beef" => Some(RecipeOutput {
                item: "minecraft:cooked_beef",
                count: 1,
            }),
            "minecraft:porkchop" => Some(RecipeOutput {
                item: "minecraft:cooked_porkchop",
                count: 1,
            }),
            "minecraft:chicken" => Some(RecipeOutput {
                item: "minecraft:cooked_chicken",
                count: 1,
            }),
            "minecraft:potato" => Some(RecipeOutput {
                item: "minecraft:baked_potato",
                count: 1,
            }),
            _ => None,
        },

        MachineKind::MechanicalPress => match input {
            "minecraft:iron_ingot" => Some(RecipeOutput {
                item: ITEM_IRON_PLATE,
                count: 1,
            }),
            "minecraft:copper_ingot" => Some(RecipeOutput {
                item: ITEM_COPPER_PLATE,
                count: 1,
            }),
            "minecraft:gold_ingot" => Some(RecipeOutput {
                item: ITEM_GOLD_PLATE,
                count: 1,
            }),
            ITEM_STEEL_INGOT => Some(RecipeOutput {
                item: ITEM_STEEL_GEAR,
                count: 1,
            }),
            ITEM_COPPER_PLATE => Some(RecipeOutput {
                item: BLOCK_CABLE,
                count: 3,
            }),
            ITEM_IRON_PLATE => Some(RecipeOutput {
                item: BLOCK_ITEM_PIPE,
                count: 2,
            }),
            "minecraft:sugar_cane" => Some(RecipeOutput {
                item: "minecraft:paper",
                count: 2,
            }),
            _ => None,
        },

        MachineKind::CoalGenerator | MachineKind::Quarry | MachineKind::ConveyorBelt => None,
    }
}

/// Returns what item stack a Quarry produces when mining `block_id`, or `None` if unmineable/air.
pub fn quarry_drop_for_block(block_id: &str) -> Option<(&'static str, u32)> {
    if block_id.is_empty()
        || block_id.starts_with("rusty_factory:")
        || matches!(
            block_id,
            "minecraft:air"
                | "minecraft:cave_air"
                | "minecraft:void_air"
                | "minecraft:bedrock"
                | "minecraft:water"
                | "minecraft:lava"
                | "minecraft:barrier"
                | "minecraft:command_block"
                | "minecraft:end_portal_frame"
                | "minecraft:end_portal"
                | "minecraft:nether_portal"
                | "minecraft:spawner"
                | "minecraft:trial_spawner"
                | "minecraft:vault"
        )
    {
        return None;
    }

    Some(match block_id {
        "minecraft:stone" | "minecraft:infested_stone" => ("minecraft:cobblestone", 1),
        "minecraft:deepslate" | "minecraft:infested_deepslate" => ("minecraft:cobbled_deepslate", 1),
        "minecraft:grass_block" | "minecraft:dirt_path" | "minecraft:farmland" | "minecraft:mycelium"
        | "minecraft:podzol" => ("minecraft:dirt", 1),
        "minecraft:coal_ore" | "minecraft:deepslate_coal_ore" => ("minecraft:coal", 2),
        "minecraft:iron_ore" | "minecraft:deepslate_iron_ore" => ("minecraft:raw_iron", 1),
        "minecraft:copper_ore" | "minecraft:deepslate_copper_ore" => ("minecraft:raw_copper", 2),
        "minecraft:gold_ore" | "minecraft:deepslate_gold_ore" => ("minecraft:raw_gold", 1),
        "minecraft:redstone_ore" | "minecraft:deepslate_redstone_ore" => ("minecraft:redstone", 4),
        "minecraft:lapis_ore" | "minecraft:deepslate_lapis_ore" => ("minecraft:lapis_lazuli", 4),
        "minecraft:diamond_ore" | "minecraft:deepslate_diamond_ore" => ("minecraft:diamond", 1),
        "minecraft:emerald_ore" | "minecraft:deepslate_emerald_ore" => ("minecraft:emerald", 1),
        "minecraft:nether_quartz_ore" => ("minecraft:quartz", 2),
        "minecraft:nether_gold_ore" => ("minecraft:gold_nugget", 4),
        "minecraft:ancient_debris" => ("minecraft:ancient_debris", 1),
        "minecraft:clay" => ("minecraft:clay_ball", 4),
        "minecraft:glowstone" => ("minecraft:glowstone_dust", 4),
        "minecraft:amethyst_cluster" | "minecraft:budding_amethyst" => ("minecraft:amethyst_shard", 4),
        "minecraft:gravel" => ("minecraft:gravel", 1),
        "minecraft:sand" => ("minecraft:sand", 1),
        "minecraft:red_sand" => ("minecraft:red_sand", 1),
        "minecraft:andesite" => ("minecraft:andesite", 1),
        "minecraft:diorite" => ("minecraft:diorite", 1),
        "minecraft:granite" => ("minecraft:granite", 1),
        "minecraft:tuff" => ("minecraft:tuff", 1),
        "minecraft:calcite" => ("minecraft:calcite", 1),
        "minecraft:dripstone_block" => ("minecraft:dripstone_block", 1),
        "minecraft:Cobblestone" | "minecraft:cobblestone" => ("minecraft:cobblestone", 1),
        "minecraft:cobbled_deepslate" => ("minecraft:cobbled_deepslate", 1),
        "minecraft:dirt" | "minecraft:coarse_dirt" | "minecraft:rooted_dirt" => ("minecraft:dirt", 1),
        "minecraft:netherrack" => ("minecraft:netherrack", 1),
        "minecraft:basalt" | "minecraft:smooth_basalt" => ("minecraft:basalt", 1),
        "minecraft:blackstone" => ("minecraft:blackstone", 1),
        "minecraft:end_stone" => ("minecraft:end_stone", 1),
        "minecraft:obsidian" => ("minecraft:obsidian", 1),
        "minecraft:sandstone" => ("minecraft:sandstone", 1),
        "minecraft:terracotta" => ("minecraft:terracotta", 1),
        "minecraft:moss_block" => ("minecraft:moss_block", 1),
        "minecraft:mud" => ("minecraft:mud", 1),
        _ => ("minecraft:cobblestone", 1),
    })
}

/// Computes the horizontal `[min_x, max_x, min_z, max_z]` mining region behind the Quarry.
/// The Quarry sits at `pos` with its front pointing toward `facing`, so the excavation frame
/// extends behind the Quarry (`-facing`).
pub fn quarry_area_bounds(pos: [i32; 3], facing: &str, radius: i32) -> (i32, i32, i32, i32) {
    let r = radius.clamp(2, 4);
    let side = r * 2 + 1;
    match facing {
        // Front is North (-Z), so behind is South (+Z: pos[2]+1 ..= pos[2]+side)
        "north" => (pos[0] - r, pos[0] + r, pos[2] + 1, pos[2] + side),
        // Front is South (+Z), so behind is North (-Z: pos[2]-side ..= pos[2]-1)
        "south" => (pos[0] - r, pos[0] + r, pos[2] - side, pos[2] - 1),
        // Front is East (+X), so behind is West (-X: pos[0]-side ..= pos[0]-1)
        "east" => (pos[0] - side, pos[0] - 1, pos[2] - r, pos[2] + r),
        // Front is West (-X), so behind is East (+X: pos[0]+1 ..= pos[0]+side)
        "west" => (pos[0] + 1, pos[0] + side, pos[2] - r, pos[2] + r),
        _ => (pos[0] - r, pos[0] + r, pos[2] + 1, pos[2] + side),
    }
}

/// Computes the 3D world coordinates `[x, y, z]` of the Quarry's current target block
/// from `quarry_cursor` and `quarry_y_offset` in a serpentine scan pattern.
pub fn quarry_target_pos(pos: [i32; 3], facing: &str, state: &MachineState) -> [i32; 3] {
    let (min_x, max_x, min_z, max_z) = quarry_area_bounds(pos, facing, state.quarry_radius);
    let width_x = (max_x - min_x + 1).max(1);
    let width_z = (max_z - min_z + 1).max(1);
    let layer_area = (width_x * width_z).max(1);
    let idx = state.quarry_cursor.rem_euclid(layer_area);
    let row_z = idx / width_x;
    let col_x = idx % width_x;
    // Serpentine X scan so the gantry moves smoothly back and forth
    let x = if row_z % 2 == 0 {
        min_x + col_x
    } else {
        max_x - col_x
    };
    let z = min_z + row_z;
    let y = (pos[1] - 1 - state.quarry_y_offset).max(-60);
    [x, y, z]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepStatus {
    Working = 0,
    Idle = 1,
    NoPower = 2,
    OutputFull = 3,
    Paused = 4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepOutcome {
    pub status: StepStatus,
    pub consume_input: bool,
    pub consume_fuel: bool,
    pub produced: Option<RecipeOutput>,
    pub net_energy_rate: i32,
}

pub fn effective_work_ticks(kind: MachineKind, speed_upgrades: u32) -> i32 {
    let base = kind.base_work_ticks();
    let clamped = speed_upgrades.min(8) as i32;
    let reduction = (base * clamped * 10) / 100;
    (base - reduction).max(2)
}

pub fn effective_energy_cost(kind: MachineKind, speed_upgrades: u32) -> i32 {
    let base = kind.energy_per_tick();
    let clamped = speed_upgrades.min(8) as i32;
    base + (base * clamped * 12) / 100
}

/// Steps a machine's simulation by 1 tick (20 ticks = 1 second).
pub fn step_machine(
    state: &mut MachineState,
    input_item: Option<&str>,
    fuel_burn_ticks: Option<i32>,
    output_slot: Option<(&str, u32, u32)>,
    speed_upgrades: u32,
) -> StepOutcome {
    if !state.enabled {
        return StepOutcome {
            status: StepStatus::Paused,
            consume_input: false,
            consume_fuel: false,
            produced: None,
            net_energy_rate: 0,
        };
    }

    match state.kind {
        MachineKind::CoalGenerator => step_generator(state, fuel_burn_ticks, speed_upgrades),
        MachineKind::ConveyorBelt => {
            let period = effective_work_ticks(MachineKind::ConveyorBelt, speed_upgrades);
            state.progress = (state.progress + 1) % period.max(1);
            StepOutcome {
                status: StepStatus::Working,
                consume_input: false,
                consume_fuel: false,
                produced: None,
                net_energy_rate: 0,
            }
        }
        MachineKind::Quarry => {
            // Quarry step is driven by `step_quarry` in glue with world access,
            // but we provide fuel/battery top-up here as well.
            let mut consume_fuel = false;
            if state.energy < state.kind.max_energy() {
                if let Some(burn) = fuel_burn_ticks {
                    if burn == -1 {
                        state.energy = (state.energy + 120).min(state.kind.max_energy());
                    } else if burn > 0 && state.energy + 800 <= state.kind.max_energy() {
                        state.energy = (state.energy + 1600).min(state.kind.max_energy());
                        consume_fuel = true;
                    }
                }
            }
            let cost = effective_energy_cost(MachineKind::Quarry, speed_upgrades);
            if state.energy < cost {
                return StepOutcome {
                    status: StepStatus::NoPower,
                    consume_input: false,
                    consume_fuel,
                    produced: None,
                    net_energy_rate: 0,
                };
            }
            StepOutcome {
                status: StepStatus::Working,
                consume_input: false,
                consume_fuel,
                produced: None,
                net_energy_rate: -cost,
            }
        }
        MachineKind::ElectricCrusher
        | MachineKind::ElectricFurnace
        | MachineKind::MechanicalPress => {
            step_processor(state, input_item, fuel_burn_ticks, output_slot, speed_upgrades)
        }
    }
}

fn step_generator(
    state: &mut MachineState,
    fuel_burn_ticks: Option<i32>,
    speed_upgrades: u32,
) -> StepOutcome {
    let mut consume_fuel = false;
    let gen_rate =
        MachineKind::CoalGenerator.energy_per_tick() + (speed_upgrades.min(8) as i32) * 10;

    if state.burn_left <= 0 && state.energy < state.kind.max_energy() {
        if let Some(ticks) = fuel_burn_ticks {
            if ticks > 0 {
                state.burn_left = ticks;
                state.burn_max = ticks;
                consume_fuel = true;
            } else if ticks == -1 {
                state.energy = (state.energy + 100).min(state.kind.max_energy());
            }
        }
    }

    if state.burn_left > 0 {
        state.burn_left -= 1;
        let before = state.energy;
        state.energy = (state.energy + gen_rate).min(state.kind.max_energy());
        let added = state.energy - before;
        state.progress = if state.burn_max > 0 {
            ((state.burn_left * 100) / state.burn_max).clamp(0, 100)
        } else {
            0
        };
        StepOutcome {
            status: StepStatus::Working,
            consume_input: false,
            consume_fuel,
            produced: None,
            net_energy_rate: added,
        }
    } else {
        state.progress = 0;
        StepOutcome {
            status: StepStatus::Idle,
            consume_input: false,
            consume_fuel,
            produced: None,
            net_energy_rate: 0,
        }
    }
}

fn step_processor(
    state: &mut MachineState,
    input_item: Option<&str>,
    fuel_burn_ticks: Option<i32>,
    output_slot: Option<(&str, u32, u32)>,
    speed_upgrades: u32,
) -> StepOutcome {
    let mut consume_fuel = false;
    if state.energy < state.kind.max_energy() {
        if let Some(burn) = fuel_burn_ticks {
            if burn == -1 {
                state.energy = (state.energy + 80).min(state.kind.max_energy());
            } else if burn > 0 && state.energy + 400 <= state.kind.max_energy() {
                state.energy = (state.energy + 1200).min(state.kind.max_energy());
                consume_fuel = true;
            }
        }
    }

    let Some(in_id) = input_item else {
        state.progress = 0;
        return StepOutcome {
            status: StepStatus::Idle,
            consume_input: false,
            consume_fuel,
            produced: None,
            net_energy_rate: 0,
        };
    };

    let Some(recipe) = lookup_recipe(state.kind, in_id) else {
        state.progress = 0;
        return StepOutcome {
            status: StepStatus::Idle,
            consume_input: false,
            consume_fuel,
            produced: None,
            net_energy_rate: 0,
        };
    };

    if let Some((out_id, out_count, out_max)) = output_slot {
        if out_id != recipe.item || out_count + recipe.count > out_max {
            return StepOutcome {
                status: StepStatus::OutputFull,
                consume_input: false,
                consume_fuel,
                produced: None,
                net_energy_rate: 0,
            };
        }
    }

    let cost = effective_energy_cost(state.kind, speed_upgrades);
    if state.energy < cost {
        return StepOutcome {
            status: StepStatus::NoPower,
            consume_input: false,
            consume_fuel,
            produced: None,
            net_energy_rate: 0,
        };
    }

    state.energy -= cost;
    state.progress += 1;
    let needed = effective_work_ticks(state.kind, speed_upgrades);

    if state.progress >= needed {
        state.progress = 0;
        StepOutcome {
            status: StepStatus::Working,
            consume_input: true,
            consume_fuel,
            produced: Some(recipe),
            net_energy_rate: -cost,
        }
    } else {
        StepOutcome {
            status: StepStatus::Working,
            consume_input: false,
            consume_fuel,
            produced: None,
            net_energy_rate: -cost,
        }
    }
}

/// 6-way neighbor offsets `[dx, dy, dz]`.
pub const NEIGHBORS_6: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// BFS across copper cables (`BLOCK_CABLE`) starting from `generator_pos` to find connected consumer machines.
pub fn find_connected_consumers<F, M>(
    generator_pos: [i32; 3],
    mut is_cable: F,
    mut is_consumer: M,
) -> Vec<[i32; 3]>
where
    F: FnMut([i32; 3]) -> bool,
    M: FnMut([i32; 3]) -> bool,
{
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut consumers = Vec::new();
    let mut seen_consumers = HashSet::new();

    visited.insert(generator_pos);
    queue.push_back((generator_pos, 0usize));

    while let Some((cur, depth)) = queue.pop_front() {
        if depth >= 48 {
            continue;
        }
        for [dx, dy, dz] in NEIGHBORS_6 {
            let next = [cur[0] + dx, cur[1] + dy, cur[2] + dz];
            if is_consumer(next) && seen_consumers.insert(next) {
                consumers.push(next);
            }
            if visited.insert(next) && is_cable(next) {
                queue.push_back((next, depth + 1));
            }
        }
    }

    consumers
}

/// BFS across `rusty_factory:item_pipe` blocks starting from `source_pos` to find connected
/// destination containers/machines in order of distance.
pub fn find_connected_item_targets<P, T>(
    source_pos: [i32; 3],
    mut is_pipe: P,
    mut is_target: T,
) -> Vec<[i32; 3]>
where
    P: FnMut([i32; 3]) -> bool,
    T: FnMut([i32; 3]) -> bool,
{
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut targets = Vec::new();
    let mut seen_targets = HashSet::new();

    visited.insert(source_pos);

    // First check direct neighbors of `source_pos`:
    // - If a neighbor is a pipe, enqueue it to traverse the pipe network.
    // - If a neighbor is directly a target container, also allow direct adjacent output.
    for [dx, dy, dz] in NEIGHBORS_6 {
        let next = [source_pos[0] + dx, source_pos[1] + dy, source_pos[2] + dz];
        if is_pipe(next) && visited.insert(next) {
            queue.push_back((next, 1usize));
        } else if is_target(next) && seen_targets.insert(next) {
            targets.push(next);
        }
    }

    while let Some((cur, depth)) = queue.pop_front() {
        if depth >= 64 {
            continue;
        }
        for [dx, dy, dz] in NEIGHBORS_6 {
            let next = [cur[0] + dx, cur[1] + dy, cur[2] + dz];
            if next == source_pos {
                continue;
            }
            if is_target(next) && seen_targets.insert(next) {
                targets.push(next);
            }
            if visited.insert(next) && is_pipe(next) {
                queue.push_back((next, depth + 1));
            }
        }
    }

    targets
}

/// Returns `(front_offset, back_offset)` for a horizontal `facing` property.
pub fn conveyor_offsets(facing: &str) -> ([i32; 3], [i32; 3]) {
    match facing {
        "north" => ([0, 0, -1], [0, 0, 1]),
        "south" => ([0, 0, 1], [0, 0, -1]),
        "east" => ([1, 0, 0], [-1, 0, 0]),
        "west" => ([-1, 0, 0], [1, 0, 0]),
        _ => ([0, 0, -1], [0, 0, 1]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crusher_doubles_raw_ores_into_dusts() {
        assert_eq!(
            lookup_recipe(MachineKind::ElectricCrusher, "minecraft:raw_iron"),
            Some(RecipeOutput {
                item: ITEM_IRON_DUST,
                count: 2
            })
        );
    }

    #[test]
    fn quarry_area_and_target_serpentine() {
        let mut st = MachineState::new(MachineKind::Quarry);
        st.quarry_radius = 2; // 5x5
        let t0 = quarry_target_pos([10, 70, 10], "north", &st);
        assert_eq!(t0, [8, 69, 11]);
        st.quarry_cursor = 4;
        let t4 = quarry_target_pos([10, 70, 10], "north", &st);
        assert_eq!(t4, [12, 69, 11]);
        st.quarry_cursor = 5; // Next row reverses X!
        let t5 = quarry_target_pos([10, 70, 10], "north", &st);
        assert_eq!(t5, [12, 69, 12]);
    }

    #[test]
    fn item_pipe_bfs_finds_distant_containers() {
        let pipes: HashSet<[i32; 3]> = [[0, 0, 1], [0, 0, 2], [0, 0, 3]].into_iter().collect();
        let chests: HashSet<[i32; 3]> = [[0, 0, 4]].into_iter().collect();
        let found = find_connected_item_targets(
            [0, 0, 0],
            |p| pipes.contains(&p),
            |p| chests.contains(&p),
        );
        assert_eq!(found, vec![[0, 0, 4]]);
    }
}
