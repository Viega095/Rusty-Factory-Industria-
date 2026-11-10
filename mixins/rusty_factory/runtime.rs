// Core industrial logic and simulation rules for Rusty Factory.
// This file serves both as the standalone Cargo library (for unit tests)
// and as the injected `crate::game::rusty_factory` module inside Horizon.

use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

pub const ID: &str = "rusty_factory";
pub const VERSION: &str = "0.1.0";
pub const MENU: &str = "rusty_factory:machine";

pub const BLOCK_CASING: &str = "rusty_factory:machine_casing";
pub const BLOCK_GENERATOR: &str = "rusty_factory:coal_generator";
pub const BLOCK_CABLE: &str = "rusty_factory:copper_cable";
pub const BLOCK_CRUSHER: &str = "rusty_factory:electric_crusher";
pub const BLOCK_FURNACE: &str = "rusty_factory:electric_furnace";
pub const BLOCK_PRESS: &str = "rusty_factory:mechanical_press";
pub const BLOCK_BELT: &str = "rusty_factory:conveyor_belt";

pub const ITEM_WRENCH: &str = "rusty_factory:wrench";
pub const ITEM_SPEED_UPGRADE: &str = "rusty_factory:speed_upgrade";
pub const ITEM_BATTERY: &str = "rusty_factory:energy_battery";

pub const SLOTS: usize = 4;
pub const SLOT_INPUT: usize = 0;
pub const SLOT_FUEL: usize = 1;
pub const SLOT_OUTPUT: usize = 2;
pub const SLOT_UPGRADE: usize = 3;

pub const MAX_CABLE_SEARCH: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineKind {
    CoalGenerator,
    ElectricCrusher,
    ElectricFurnace,
    MechanicalPress,
    ConveyorBelt,
}

impl MachineKind {
    pub fn from_block_id(id: &str) -> Option<Self> {
        match id {
            BLOCK_GENERATOR => Some(Self::CoalGenerator),
            BLOCK_CRUSHER => Some(Self::ElectricCrusher),
            BLOCK_FURNACE => Some(Self::ElectricFurnace),
            BLOCK_PRESS => Some(Self::MechanicalPress),
            BLOCK_BELT => Some(Self::ConveyorBelt),
            _ => None,
        }
    }

    pub fn max_energy(self) -> i32 {
        match self {
            Self::CoalGenerator => 50_000,
            Self::ElectricCrusher => 20_000,
            Self::ElectricFurnace => 20_000,
            Self::MechanicalPress => 25_000,
            Self::ConveyorBelt => 0,
        }
    }

    pub fn energy_per_tick(self) -> i32 {
        match self {
            Self::CoalGenerator => 40, // Generates +40 FE/t
            Self::ElectricCrusher => 20, // Consumes 20 FE/t
            Self::ElectricFurnace => 15, // Consumes 15 FE/t
            Self::MechanicalPress => 25, // Consumes 25 FE/t
            Self::ConveyorBelt => 0,
        }
    }

    pub fn base_work_ticks(self) -> i32 {
        match self {
            Self::CoalGenerator => 200,
            Self::ElectricCrusher => 100,
            Self::ElectricFurnace => 80,
            Self::MechanicalPress => 90,
            Self::ConveyorBelt => 10,
        }
    }

    pub fn title_key(self) -> &'static str {
        match self {
            Self::CoalGenerator => "block.rusty_factory.coal_generator",
            Self::ElectricCrusher => "block.rusty_factory.electric_crusher",
            Self::ElectricFurnace => "block.rusty_factory.electric_furnace",
            Self::MechanicalPress => "block.rusty_factory.mechanical_press",
            Self::ConveyorBelt => "block.rusty_factory.conveyor_belt",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineState {
    pub kind: MachineKind,
    pub energy: i32,
    pub progress: i32,
    pub burn_remaining: i32,
    pub burn_total: i32,
    pub enabled: bool,
}

impl MachineState {
    pub fn new(kind: MachineKind) -> Self {
        Self {
            kind,
            energy: 0,
            progress: 0,
            burn_remaining: 0,
            burn_total: 0,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecipeOutput {
    pub item: &'static str,
    pub count: u32,
}

/// Returns the recipe output for a given machine kind and input item ID.
pub fn match_recipe(kind: MachineKind, input_item: &str) -> Option<RecipeOutput> {
    match kind {
        MachineKind::ElectricCrusher => match input_item {
            "minecraft:iron_ore" | "minecraft:deepslate_iron_ore" | "minecraft:raw_iron" => {
                Some(RecipeOutput {
                    item: "rusty_factory:iron_dust",
                    count: 2,
                })
            }
            "minecraft:copper_ore" | "minecraft:deepslate_copper_ore" | "minecraft:raw_copper" => {
                Some(RecipeOutput {
                    item: "rusty_factory:copper_dust",
                    count: 2,
                })
            }
            "minecraft:gold_ore" | "minecraft:deepslate_gold_ore" | "minecraft:raw_gold" => {
                Some(RecipeOutput {
                    item: "rusty_factory:gold_dust",
                    count: 2,
                })
            }
            "minecraft:coal_ore" | "minecraft:deepslate_coal_ore" => Some(RecipeOutput {
                item: "minecraft:coal",
                count: 3,
            }),
            "minecraft:diamond_ore" | "minecraft:deepslate_diamond_ore" => Some(RecipeOutput {
                item: "minecraft:diamond",
                count: 2,
            }),
            "minecraft:redstone_ore" | "minecraft:deepslate_redstone_ore" => Some(RecipeOutput {
                item: "minecraft:redstone",
                count: 6,
            }),
            "minecraft:cobblestone" => Some(RecipeOutput {
                item: "minecraft:gravel",
                count: 1,
            }),
            "minecraft:gravel" => Some(RecipeOutput {
                item: "minecraft:sand",
                count: 1,
            }),
            "minecraft:bone" => Some(RecipeOutput {
                item: "minecraft:bone_meal",
                count: 5,
            }),
            "minecraft:blaze_rod" => Some(RecipeOutput {
                item: "minecraft:blaze_powder",
                count: 4,
            }),
            _ => None,
        },
        MachineKind::ElectricFurnace => match input_item {
            "rusty_factory:iron_dust" | "minecraft:raw_iron" | "minecraft:iron_ore" => {
                Some(RecipeOutput {
                    item: "minecraft:iron_ingot",
                    count: 1,
                })
            }
            "rusty_factory:copper_dust" | "minecraft:raw_copper" | "minecraft:copper_ore" => {
                Some(RecipeOutput {
                    item: "minecraft:copper_ingot",
                    count: 1,
                })
            }
            "rusty_factory:gold_dust" | "minecraft:raw_gold" | "minecraft:gold_ore" => {
                Some(RecipeOutput {
                    item: "minecraft:gold_ingot",
                    count: 1,
                })
            }
            "minecraft:iron_ingot" => Some(RecipeOutput {
                item: "rusty_factory:steel_ingot",
                count: 1,
            }),
            "minecraft:sand" => Some(RecipeOutput {
                item: "minecraft:glass",
                count: 1,
            }),
            "minecraft:cobblestone" => Some(RecipeOutput {
                item: "minecraft:stone",
                count: 1,
            }),
            "minecraft:clay_ball" => Some(RecipeOutput {
                item: "minecraft:brick",
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
        MachineKind::MechanicalPress => match input_item {
            "minecraft:iron_ingot" => Some(RecipeOutput {
                item: "rusty_factory:iron_plate",
                count: 1,
            }),
            "minecraft:copper_ingot" => Some(RecipeOutput {
                item: "rusty_factory:copper_plate",
                count: 1,
            }),
            "minecraft:gold_ingot" => Some(RecipeOutput {
                item: "rusty_factory:gold_plate",
                count: 1,
            }),
            "rusty_factory:steel_ingot" => Some(RecipeOutput {
                item: "rusty_factory:steel_gear",
                count: 1,
            }),
            "minecraft:sugar_cane" => Some(RecipeOutput {
                item: "minecraft:paper",
                count: 2,
            }),
            _ => None,
        },
        MachineKind::CoalGenerator | MachineKind::ConveyorBelt => None,
    }
}

/// Calculates effective work ticks required given the number of speed upgrades (0..=16).
pub fn effective_work_ticks(kind: MachineKind, speed_upgrades: u32) -> i32 {
    let base = kind.base_work_ticks();
    let clamped = speed_upgrades.min(8) as i32;
    let reduction = (base * clamped * 10) / 100;
    (base - reduction).max(15)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepStatus {
    Working = 0,
    Idle = 1,
    LowEnergy = 2,
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

/// Advances a machine's state by 1 tick and returns what inventory changes should happen.
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
        MachineKind::CoalGenerator => {
            let mut consume_fuel = false;
            let max_e = state.kind.max_energy();
            let gen_rate = state.kind.energy_per_tick();

            if state.burn_remaining <= 0 && state.energy < max_e {
                if let Some(burn) = fuel_burn_ticks.filter(|b| *b > 0) {
                    state.burn_remaining = burn;
                    state.burn_total = burn;
                    consume_fuel = true;
                }
            }

            if state.burn_remaining > 0 {
                state.burn_remaining -= 1;
                state.energy = (state.energy + gen_rate).min(max_e);
                let pct = if state.burn_total > 0 {
                    ((state.burn_total - state.burn_remaining) * 100) / state.burn_total
                } else {
                    0
                };
                state.progress = pct.clamp(0, 100);
                StepOutcome {
                    status: StepStatus::Working,
                    consume_input: false,
                    consume_fuel,
                    produced: None,
                    net_energy_rate: gen_rate,
                }
            } else {
                state.progress = 0;
                StepOutcome {
                    status: if state.energy >= max_e {
                        StepStatus::OutputFull
                    } else {
                        StepStatus::Idle
                    },
                    consume_input: false,
                    consume_fuel,
                    produced: None,
                    net_energy_rate: 0,
                }
            }
        }
        MachineKind::ElectricCrusher
        | MachineKind::ElectricFurnace
        | MachineKind::MechanicalPress => {
            // Optional battery discharge in fuel slot
            let mut consume_fuel = false;
            let max_e = state.kind.max_energy();
            if fuel_burn_ticks == Some(-1) && state.energy + 2000 <= max_e {
                state.energy += 2000;
                consume_fuel = true;
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

            let Some(recipe) = match_recipe(state.kind, in_id) else {
                state.progress = 0;
                return StepOutcome {
                    status: StepStatus::Idle,
                    consume_input: false,
                    consume_fuel,
                    produced: None,
                    net_energy_rate: 0,
                };
            };

            // Check if output slot can fit the recipe result
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

            let cost = state.kind.energy_per_tick() + (speed_upgrades.min(8) as i32 * 3);
            if state.energy < cost {
                return StepOutcome {
                    status: StepStatus::LowEnergy,
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
        MachineKind::ConveyorBelt => {
            state.progress = (state.progress + 1) % 10;
            StepOutcome {
                status: StepStatus::Working,
                consume_input: false,
                consume_fuel: false,
                produced: None,
                net_energy_rate: 0,
            }
        }
    }
}

/// Six cardinal neighbors of a 3D block position.
pub fn neighbors(pos: [i32; 3]) -> [[i32; 3]; 6] {
    [
        [pos[0] + 1, pos[1], pos[2]],
        [pos[0] - 1, pos[1], pos[2]],
        [pos[0], pos[1] + 1, pos[2]],
        [pos[0], pos[1] - 1, pos[2]],
        [pos[0], pos[1], pos[2] + 1],
        [pos[0], pos[1], pos[2] - 1],
    ]
}

/// Finds all energy consumer positions reachable from `start` via copper cables or direct adjacency.
pub fn find_connected_consumers<FIsCable, FIsConsumer>(
    start: [i32; 3],
    is_cable: FIsCable,
    is_consumer: FIsConsumer,
) -> Vec<[i32; 3]>
where
    FIsCable: Fn([i32; 3]) -> bool,
    FIsConsumer: Fn([i32; 3]) -> bool,
{
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut consumers = Vec::new();

    visited.insert(start);
    for adj in neighbors(start) {
        if visited.insert(adj) {
            if is_consumer(adj) {
                consumers.push(adj);
            } else if is_cable(adj) {
                queue.push_back(adj);
            }
        }
    }

    while let Some(curr) = queue.pop_front() {
        if visited.len() >= MAX_CABLE_SEARCH {
            break;
        }
        for adj in neighbors(curr) {
            if visited.insert(adj) {
                if is_consumer(adj) {
                    consumers.push(adj);
                } else if is_cable(adj) {
                    queue.push_back(adj);
                }
            }
        }
    }

    consumers
}

/// Converts a cardinal facing string into a `(front_offset, back_offset)` pair.
pub fn conveyor_offsets(facing: &str) -> ([i32; 3], [i32; 3]) {
    match facing {
        "south" => ([0, 0, 1], [0, 0, -1]),
        "east" => ([1, 0, 0], [-1, 0, 0]),
        "west" => ([-1, 0, 0], [1, 0, 0]),
        _ => ([0, 0, -1], [0, 0, 1]), // default north
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crusher_doubles_ores_into_dusts() {
        let out = match_recipe(MachineKind::ElectricCrusher, "minecraft:iron_ore").unwrap();
        assert_eq!(out.item, "rusty_factory:iron_dust");
        assert_eq!(out.count, 2);
    }

    #[test]
    fn generator_produces_energy_from_fuel() {
        let mut gen = MachineState::new(MachineKind::CoalGenerator);
        let outcome = step_machine(&mut gen, None, Some(1600), None, 0);
        assert!(outcome.consume_fuel);
        assert_eq!(outcome.status, StepStatus::Working);
        assert_eq!(gen.energy, 40);
        assert_eq!(gen.burn_remaining, 1599);
    }

    #[test]
    fn crusher_processes_item_when_powered() {
        let mut crusher = MachineState::new(MachineKind::ElectricCrusher);
        crusher.energy = 5000;
        let ticks = effective_work_ticks(MachineKind::ElectricCrusher, 0);
        let mut last = None;
        for _ in 0..ticks {
            last = Some(step_machine(
                &mut crusher,
                Some("minecraft:iron_ore"),
                None,
                None,
                0,
            ));
        }
        let res = last.unwrap();
        assert!(res.consume_input);
        assert_eq!(
            res.produced,
            Some(RecipeOutput {
                item: "rusty_factory:iron_dust",
                count: 2
            })
        );
    }

    #[test]
    fn bfs_cable_network_finds_distant_machines() {
        let cables: HashSet<[i32; 3]> = [[1, 0, 0], [2, 0, 0], [3, 0, 0]].into_iter().collect();
        let target = [4, 0, 0];
        let found = find_connected_consumers(
            [0, 0, 0],
            |p| cables.contains(&p),
            |p| p == target,
        );
        assert_eq!(found, vec![target]);
    }
}
