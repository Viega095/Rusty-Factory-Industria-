// Pure Rust logic for the "Rusty Factory" (rusty_factory) Horizon native mod.
// No engine imports here so `cargo test` runs standalone in milliseconds.

use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

pub const ID: &str = "rusty_factory";
pub const VERSION: &str = "0.4.1";
pub const MENU: &str = "rusty_factory:machine";
pub const MENU_SINGLE: &str = "rusty_factory:single";

// Block IDs (24 blocks)
pub const BLOCK_CASING: &str = "rusty_factory:machine_casing";
pub const BLOCK_GENERATOR: &str = "rusty_factory:coal_generator";
pub const BLOCK_BIO_GENERATOR: &str = "rusty_factory:bio_generator";
pub const BLOCK_SOLAR: &str = "rusty_factory:solar_generator";
pub const BLOCK_ENERGY_BANK: &str = "rusty_factory:energy_bank";
pub const BLOCK_CABLE: &str = "rusty_factory:copper_cable";
pub const BLOCK_CABLE_WHITE: &str = "rusty_factory:cable_white";
pub const BLOCK_CABLE_COPPER: &str = "rusty_factory:copper_cable";
pub const BLOCK_CABLE_GOLD: &str = "rusty_factory:cable_gold";
pub const BLOCK_CABLE_RED: &str = "rusty_factory:cable_red";

pub fn is_cable(block: &str) -> bool {
    matches!(
        block,
        BLOCK_CABLE_WHITE
            | BLOCK_CABLE_COPPER
            | BLOCK_CABLE_GOLD
            | BLOCK_CABLE_RED
    )
}

pub fn cable_transfer_limit(block: &str) -> i32 {
    match block {
        BLOCK_CABLE_WHITE => 120,
        BLOCK_CABLE_COPPER => 360,
        BLOCK_CABLE_GOLD => 1_200,
        BLOCK_CABLE_RED => 10_000,
        _ => 360,
    }
}
pub const BLOCK_ITEM_PIPE: &str = "rusty_factory:item_pipe";
pub const BLOCK_EXTRACTOR: &str = "rusty_factory:item_extractor";
pub const BLOCK_CRUSHER: &str = "rusty_factory:electric_crusher";
pub const BLOCK_FURNACE: &str = "rusty_factory:electric_furnace";
pub const BLOCK_PRESS: &str = "rusty_factory:mechanical_press";
pub const BLOCK_ASSEMBLER: &str = "rusty_factory:industrial_assembler";
pub const BLOCK_VULCANIZER: &str = "rusty_factory:vulcanizer";
pub const BLOCK_SAP_EXTRACTOR: &str = "rusty_factory:sap_extractor";
pub const BLOCK_QUARRY: &str = "rusty_factory:quarry";
pub const BLOCK_HARVESTER: &str = "rusty_factory:auto_harvester";
pub const BLOCK_BREAKER: &str = "rusty_factory:block_breaker";
pub const BLOCK_PLACER: &str = "rusty_factory:block_placer";
pub const BLOCK_CONVEYOR: &str = "rusty_factory:conveyor_belt";
pub const BLOCK_CONVEYOR_MK2: &str = "rusty_factory:conveyor_belt_mk2";
pub const BLOCK_CONVEYOR_MK3: &str = "rusty_factory:conveyor_belt_mk3";

pub fn is_conveyor_block(block: &str) -> bool {
    matches!(block, BLOCK_CONVEYOR | BLOCK_CONVEYOR_MK2 | BLOCK_CONVEYOR_MK3)
}

/// Returns `(ticks_per_block, max_stack_per_batch)` for a conveyor belt tier.
pub fn conveyor_tier(block: &str) -> Option<(i32, u32)> {
    match block {
        BLOCK_CONVEYOR => Some((20, 1)),
        BLOCK_CONVEYOR_MK2 => Some((10, 4)),
        BLOCK_CONVEYOR_MK3 => Some((5, 16)),
        _ => None,
    }
}
pub const BLOCK_RUBBER_LOG: &str = "rusty_factory:rubber_log";
pub const BLOCK_RUBBER_SAP_LOG: &str = "rusty_factory:rubber_sap_log";
pub const BLOCK_RUBBER_LEAVES: &str = "rusty_factory:rubber_leaves";
pub const BLOCK_RUBBER_SAPLING: &str = "rusty_factory:rubber_sapling";
pub const BLOCK_RUBBER_PLANKS: &str = "rusty_factory:rubber_planks";
pub const BLOCK_ENERGY_MONITOR: &str = "rusty_factory:energy_monitor";
pub const BLOCK_FLUID_TANK: &str = "rusty_factory:fluid_tank";
pub const BLOCK_STEAM_BOILER: &str = "rusty_factory:steam_boiler";
pub const BLOCK_FLUID_PUMP: &str = "rusty_factory:fluid_pump";
pub const BLOCK_TRASH_CAN: &str = "rusty_factory:void_trash_can";
pub const BLOCK_ENERGY_RELAY: &str = "rusty_factory:energy_relay";
pub const BLOCK_CHARGING_PAD: &str = "rusty_factory:charging_pad";
pub const BLOCK_FLUID_PIPE: &str = "rusty_factory:fluid_pipe";
pub const BLOCK_ALLOY_SMELTER: &str = "rusty_factory:alloy_smelter";

// Item IDs (29 items)
pub const ITEM_WRENCH: &str = "rusty_factory:wrench";
pub const ITEM_TREETAP: &str = "rusty_factory:treetap";
pub const ITEM_RAW_SAP: &str = "rusty_factory:raw_sap";
pub const ITEM_RUBBER_BAR: &str = "rusty_factory:rubber_bar";
pub const ITEM_RUBBER_SHEET: &str = "rusty_factory:rubber_sheet";
pub const ITEM_INSULATED_CABLE: &str = "rusty_factory:insulated_copper_cable";
pub const ITEM_SULFUR_DUST: &str = "rusty_factory:sulfur_dust";
pub const ITEM_BIO_FUEL: &str = "rusty_factory:bio_fuel";
pub const ITEM_MINING_DRILL: &str = "rusty_factory:mining_drill";
pub const ITEM_JET_BOOTS: &str = "rusty_factory:jet_boots";
pub const ITEM_JETPACK: &str = "rusty_factory:jetpack";
pub const ITEM_ITEM_MAGNET: &str = "rusty_factory:item_magnet";
pub const ITEM_MINING_LASER: &str = "rusty_factory:mining_laser";
pub const ITEM_SOLAR_HELMET: &str = "rusty_factory:solar_helmet";
pub const ITEM_SPEED_UPGRADE: &str = "rusty_factory:speed_upgrade";
pub const ITEM_FORTUNE_UPGRADE: &str = "rusty_factory:fortune_upgrade";
pub const ITEM_BATTERY: &str = "rusty_factory:energy_battery";
pub const ITEM_CIRCUIT: &str = "rusty_factory:electronic_circuit";
pub const ITEM_SOLAR_CELL: &str = "rusty_factory:solar_cell";
pub const ITEM_SILICON: &str = "rusty_factory:silicon";
pub const ITEM_DRILL_HEAD: &str = "rusty_factory:diamond_drill_head";
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
pub enum MachineTier {
    Tier1,    // Basic machinery / Early game
    Tier2,    // Advanced industrial machinery
    Storage,  // Energy storage modular banks
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineKind {
    CoalGenerator,
    BioGenerator,
    SolarGenerator,
    EnergyBank,
    ItemExtractor,
    ElectricCrusher,
    ElectricFurnace,
    MechanicalPress,
    IndustrialAssembler,
    Vulcanizer,
    SapExtractor,
    Quarry,
    AutoHarvester,
    BlockBreaker,
    BlockPlacer,
    ConveyorBelt,
    ItemPipe,
    EnergyMonitor,
    FluidTank,
    SteamBoiler,
    FluidPump,
    VoidTrashCan,
    EnergyRelay,
    ChargingPad,
    AlloySmelter,
}

impl MachineKind {
    pub fn from_block_id(block: &str) -> Option<Self> {
        match block {
            BLOCK_GENERATOR => Some(Self::CoalGenerator),
            BLOCK_BIO_GENERATOR => Some(Self::BioGenerator),
            BLOCK_SOLAR => Some(Self::SolarGenerator),
            BLOCK_ENERGY_BANK => Some(Self::EnergyBank),
            BLOCK_EXTRACTOR => Some(Self::ItemExtractor),
            BLOCK_CRUSHER => Some(Self::ElectricCrusher),
            BLOCK_FURNACE => Some(Self::ElectricFurnace),
            BLOCK_PRESS => Some(Self::MechanicalPress),
            BLOCK_ASSEMBLER => Some(Self::IndustrialAssembler),
            BLOCK_VULCANIZER => Some(Self::Vulcanizer),
            BLOCK_SAP_EXTRACTOR => Some(Self::SapExtractor),
            BLOCK_QUARRY => Some(Self::Quarry),
            BLOCK_HARVESTER => Some(Self::AutoHarvester),
            BLOCK_BREAKER => Some(Self::BlockBreaker),
            BLOCK_PLACER => Some(Self::BlockPlacer),
            BLOCK_CONVEYOR | BLOCK_CONVEYOR_MK2 | BLOCK_CONVEYOR_MK3 => Some(Self::ConveyorBelt),
            BLOCK_ITEM_PIPE => Some(Self::ItemPipe),
            BLOCK_ENERGY_MONITOR => Some(Self::EnergyMonitor),
            BLOCK_FLUID_TANK => Some(Self::FluidTank),
            BLOCK_STEAM_BOILER => Some(Self::SteamBoiler),
            BLOCK_FLUID_PUMP => Some(Self::FluidPump),
            BLOCK_TRASH_CAN => Some(Self::VoidTrashCan),
            BLOCK_ENERGY_RELAY => Some(Self::EnergyRelay),
            BLOCK_CHARGING_PAD => Some(Self::ChargingPad),
            BLOCK_ALLOY_SMELTER => Some(Self::AlloySmelter),
            _ => None,
        }
    }

    pub const fn tier(self) -> MachineTier {
        match self {
            Self::CoalGenerator
            | Self::BioGenerator
            | Self::SolarGenerator
            | Self::ElectricCrusher
            | Self::ElectricFurnace
            | Self::MechanicalPress
            | Self::Vulcanizer
            | Self::SapExtractor
            | Self::BlockPlacer
            | Self::ItemExtractor
            | Self::ConveyorBelt
            | Self::ItemPipe
            | Self::FluidPump
            | Self::VoidTrashCan => MachineTier::Tier1,

            Self::IndustrialAssembler
            | Self::AutoHarvester
            | Self::BlockBreaker
            | Self::Quarry
            | Self::SteamBoiler
            | Self::EnergyRelay
            | Self::ChargingPad
            | Self::AlloySmelter => MachineTier::Tier2,

            Self::EnergyBank | Self::EnergyMonitor | Self::FluidTank => MachineTier::Storage,
        }
    }

    pub const fn max_energy(self) -> i32 {
        match self {
            Self::CoalGenerator => 4_000,
            Self::BioGenerator => 4_000,
            Self::SolarGenerator => 2_000, // 4_000 when part of 3x3 Solar Array
            Self::EnergyBank => 10_000,    // 10_000 per connected cluster cell
            Self::ItemExtractor => 0,
            Self::ElectricCrusher => 2_500,
            Self::ElectricFurnace => 2_500,
            Self::MechanicalPress => 2_500,
            Self::IndustrialAssembler => 6_000,
            Self::Vulcanizer => 3_000,
            Self::SapExtractor => 2_500,
            Self::Quarry => 10_000,
            Self::AutoHarvester => 6_000,
            Self::BlockBreaker => 5_000,
            Self::BlockPlacer => 2_000,
            Self::ConveyorBelt => 0,
            Self::ItemPipe => 0,
            Self::EnergyMonitor => 0,
            Self::FluidTank => 0,
            Self::SteamBoiler => 10_000,
            Self::FluidPump => 4_000,
            Self::VoidTrashCan => 0,
            Self::EnergyRelay => 20_000,
            Self::ChargingPad => 8_000,
            Self::AlloySmelter => 5_000,
        }
    }

    pub const fn base_work_ticks(self) -> i32 {
        match self {
            Self::CoalGenerator
            | Self::BioGenerator
            | Self::SolarGenerator
            | Self::EnergyBank
            | Self::EnergyMonitor
            | Self::FluidTank
            | Self::SteamBoiler
            | Self::FluidPump
            | Self::VoidTrashCan
            | Self::EnergyRelay
            | Self::ChargingPad => 1,
            Self::ItemExtractor => 10,
            Self::ElectricCrusher => 70,
            Self::ElectricFurnace => 50,
            Self::MechanicalPress => 60,
            Self::IndustrialAssembler => 55,
            Self::Vulcanizer => 45,
            Self::SapExtractor => 30,
            Self::Quarry => 16,
            Self::AutoHarvester => 14,
            Self::BlockBreaker => 18,
            Self::BlockPlacer => 12,
            Self::ConveyorBelt => 20,
            Self::ItemPipe => 10,
            Self::AlloySmelter => 50,
        }
    }

    pub const fn energy_per_tick(self) -> i32 {
        match self {
            Self::CoalGenerator => 20, // Generates +20 FE/t (Tier 1 generator)
            Self::BioGenerator => 24,  // Generates +24 FE/t from organic/sap fuels
            Self::SolarGenerator => 12, // Base daytime generation +12 FE/t (+24 FE/t in 3x3 array!)
            Self::SteamBoiler => 80,    // Massive +80 FE/t high-pressure steam generation
            Self::EnergyBank | Self::ItemExtractor | Self::ConveyorBelt | Self::ItemPipe | Self::EnergyMonitor | Self::FluidTank | Self::VoidTrashCan | Self::EnergyRelay => 0,
            Self::FluidPump => 15,
            Self::ElectricCrusher => 14,
            Self::ElectricFurnace => 12,
            Self::MechanicalPress => 14,
            Self::IndustrialAssembler => 20,
            Self::Vulcanizer => 15,
            Self::SapExtractor => 10,
            Self::Quarry => 25,
            Self::AutoHarvester => 18,
            Self::BlockBreaker => 15,
            Self::BlockPlacer => 8,
            Self::ChargingPad => 25,
            Self::AlloySmelter => 16,
        }
    }

    pub const fn is_energy_consumer(self) -> bool {
        matches!(
            self,
            Self::EnergyBank
                | Self::ElectricCrusher
                | Self::ElectricFurnace
                | Self::MechanicalPress
                | Self::IndustrialAssembler
                | Self::Vulcanizer
                | Self::SapExtractor
                | Self::Quarry
                | Self::AutoHarvester
                | Self::BlockBreaker
                | Self::BlockPlacer
                | Self::FluidPump
                | Self::EnergyRelay
                | Self::ChargingPad
                | Self::AlloySmelter
        )
    }

    pub const fn title_key(self) -> &'static str {
        match self {
            Self::CoalGenerator => "block.rusty_factory.coal_generator",
            Self::BioGenerator => "block.rusty_factory.bio_generator",
            Self::SolarGenerator => "block.rusty_factory.solar_generator",
            Self::EnergyBank => "block.rusty_factory.energy_bank",
            Self::ItemExtractor => "block.rusty_factory.item_extractor",
            Self::ElectricCrusher => "block.rusty_factory.electric_crusher",
            Self::ElectricFurnace => "block.rusty_factory.electric_furnace",
            Self::MechanicalPress => "block.rusty_factory.mechanical_press",
            Self::IndustrialAssembler => "block.rusty_factory.industrial_assembler",
            Self::Vulcanizer => "block.rusty_factory.vulcanizer",
            Self::SapExtractor => "block.rusty_factory.sap_extractor",
            Self::Quarry => "block.rusty_factory.quarry",
            Self::AutoHarvester => "block.rusty_factory.auto_harvester",
            Self::BlockBreaker => "block.rusty_factory.block_breaker",
            Self::BlockPlacer => "block.rusty_factory.block_placer",
            Self::ConveyorBelt => "block.rusty_factory.conveyor_belt",
            Self::ItemPipe => "block.rusty_factory.item_pipe",
            Self::EnergyMonitor => "block.rusty_factory.energy_monitor",
            Self::FluidTank => "block.rusty_factory.fluid_tank",
            Self::SteamBoiler => "block.rusty_factory.steam_boiler",
            Self::FluidPump => "block.rusty_factory.fluid_pump",
            Self::VoidTrashCan => "block.rusty_factory.void_trash_can",
            Self::EnergyRelay => "block.rusty_factory.energy_relay",
            Self::ChargingPad => "block.rusty_factory.charging_pad",
            Self::AlloySmelter => "block.rusty_factory.alloy_smelter",
        }
    }

    pub const fn gui_sprite(self) -> &'static str {
        match self {
            Self::CoalGenerator => "rusty_factory:gui/coal_generator",
            Self::BioGenerator => "rusty_factory:gui/bio_generator",
            Self::SolarGenerator => "rusty_factory:gui/solar_generator",
            Self::EnergyBank => "rusty_factory:gui/energy_bank",
            Self::ChargingPad => "rusty_factory:gui/charging_pad",
            Self::ElectricCrusher => "rusty_factory:gui/electric_crusher",
            Self::ElectricFurnace => "rusty_factory:gui/electric_furnace",
            Self::AlloySmelter => "rusty_factory:gui/alloy_smelter",
            Self::MechanicalPress => "rusty_factory:gui/mechanical_press",
            Self::IndustrialAssembler => "rusty_factory:gui/industrial_assembler",
            Self::Vulcanizer => "rusty_factory:gui/vulcanizer",
            Self::SapExtractor => "rusty_factory:gui/sap_extractor",
            Self::Quarry => "rusty_factory:gui/quarry",
            Self::AutoHarvester => "rusty_factory:gui/auto_harvester",
            Self::ConveyorBelt => "rusty_factory:gui/conveyor",
            Self::ItemExtractor => "rusty_factory:gui/item_extractor",
            Self::BlockBreaker => "rusty_factory:gui/block_breaker",
            Self::BlockPlacer => "rusty_factory:gui/block_placer",
            Self::FluidTank => "rusty_factory:gui/fluid_tank",
            Self::SteamBoiler => "rusty_factory:gui/steam_boiler",
            Self::FluidPump => "rusty_factory:gui/fluid_pump",
            Self::VoidTrashCan => "rusty_factory:gui/void_trash_can",
            Self::ItemPipe | Self::EnergyMonitor | Self::EnergyRelay => "rusty_factory:gui/machine",
        }
    }
}


fn default_true() -> bool {
    true
}

fn default_quarry_radius() -> i32 {
    2 // 5x5 area (2*2 + 1)
}

pub const FLUID_TANK_MAX_MB: i32 = 8000;

pub fn fluid_code(kind: &str) -> i32 {
    match kind {
        "water" => 1,
        "lava" => 2,
        "sap" => 3,
        "steam" => 4,
        _ => 0,
    }
}

pub fn fluid_name_from_code(code: i32) -> &'static str {
    match code {
        1 => "Agua",
        2 => "Lava",
        3 => "Savia de Caucho",
        4 => "Vapor",
        _ => "Vacío",
    }
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
    #[serde(default)]
    pub belt_count: u32,
    #[serde(default)]
    pub pipe_from: u8,
    #[serde(default)]
    pub pipe_to: u8,
    #[serde(default)]
    pub pipe_target: [i32; 3],
    #[serde(default)]
    pub pipe_has_target: bool,
    #[serde(default)]
    pub fluid_kind: String,
    #[serde(default)]
    pub fluid_amount: i32,
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
            belt_count: 0,
            pipe_from: 6,
            pipe_to: 6,
            pipe_target: [0, 0, 0],
            pipe_has_target: false,
            fluid_kind: String::new(),
            fluid_amount: 0,
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
            "minecraft:quartz" | "minecraft:quartz_block" | "minecraft:sand" => Some(RecipeOutput {
                item: ITEM_SILICON,
                count: 2,
            }),
            "minecraft:netherrack" | "minecraft:coal" | "minecraft:gunpowder" => Some(RecipeOutput {
                item: ITEM_SULFUR_DUST,
                count: 2,
            }),
            BLOCK_RUBBER_SAP_LOG | BLOCK_RUBBER_LOG => Some(RecipeOutput {
                item: ITEM_RAW_SAP,
                count: 2,
            }),
            _ => None,
        },

        MachineKind::ElectricFurnace => match input {
            // Raw Sap -> Vulcanized Rubber Bar!
            ITEM_RAW_SAP => Some(RecipeOutput {
                item: ITEM_RUBBER_BAR,
                count: 2,
            }),
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
            | "minecraft:cherry_log"
            | BLOCK_RUBBER_LOG => Some(RecipeOutput {
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
            ITEM_RUBBER_BAR => Some(RecipeOutput {
                item: ITEM_RUBBER_SHEET,
                count: 2,
            }),
            ITEM_RAW_SAP => Some(RecipeOutput {
                item: ITEM_RUBBER_BAR,
                count: 1,
            }),
            _ => None,
        },

        MachineKind::IndustrialAssembler => match input {
            // Automated Sawmill & Industrial Component Crafting
            BLOCK_RUBBER_LOG | BLOCK_RUBBER_SAP_LOG => Some(RecipeOutput {
                item: BLOCK_RUBBER_PLANKS,
                count: 6,
            }),
            "minecraft:oak_log" => Some(RecipeOutput {
                item: "minecraft:oak_planks",
                count: 6,
            }),
            "minecraft:spruce_log" => Some(RecipeOutput {
                item: "minecraft:spruce_planks",
                count: 6,
            }),
            "minecraft:birch_log" => Some(RecipeOutput {
                item: "minecraft:birch_planks",
                count: 6,
            }),
            "minecraft:jungle_log" => Some(RecipeOutput {
                item: "minecraft:jungle_planks",
                count: 6,
            }),
            "minecraft:acacia_log" => Some(RecipeOutput {
                item: "minecraft:acacia_planks",
                count: 6,
            }),
            "minecraft:dark_oak_log" => Some(RecipeOutput {
                item: "minecraft:dark_oak_planks",
                count: 6,
            }),
            "minecraft:cherry_log" => Some(RecipeOutput {
                item: "minecraft:cherry_planks",
                count: 6,
            }),
            "minecraft:oak_planks"
            | "minecraft:spruce_planks"
            | "minecraft:birch_planks"
            | "minecraft:jungle_planks"
            | "minecraft:acacia_planks"
            | "minecraft:dark_oak_planks"
            | BLOCK_RUBBER_PLANKS => Some(RecipeOutput {
                item: "minecraft:stick",
                count: 4,
            }),
            ITEM_RUBBER_BAR => Some(RecipeOutput {
                item: ITEM_INSULATED_CABLE,
                count: 4,
            }),
            ITEM_INSULATED_CABLE => Some(RecipeOutput {
                item: BLOCK_CABLE,
                count: 4,
            }),
            ITEM_RUBBER_SHEET => Some(RecipeOutput {
                item: BLOCK_CONVEYOR,
                count: 4,
            }),
            ITEM_IRON_PLATE => Some(RecipeOutput {
                item: BLOCK_CASING,
                count: 1,
            }),
            ITEM_GOLD_PLATE => Some(RecipeOutput {
                item: ITEM_CIRCUIT,
                count: 2,
            }),
            ITEM_SILICON => Some(RecipeOutput {
                item: ITEM_SOLAR_CELL,
                count: 2,
            }),
            ITEM_STEEL_GEAR => Some(RecipeOutput {
                item: BLOCK_CONVEYOR,
                count: 4,
            }),
            ITEM_CIRCUIT => Some(RecipeOutput {
                item: ITEM_SPEED_UPGRADE,
                count: 1,
            }),
            "minecraft:diamond" => Some(RecipeOutput {
                item: ITEM_DRILL_HEAD,
                count: 1,
            }),
            "minecraft:redstone_block" => Some(RecipeOutput {
                item: ITEM_BATTERY,
                count: 1,
            }),
            _ => None,
        },

        MachineKind::Vulcanizer => match input {
            // High-efficiency rubber vulcanization & chemical processing
            ITEM_RAW_SAP => Some(RecipeOutput {
                item: ITEM_RUBBER_BAR,
                count: 3,
            }),
            ITEM_RUBBER_BAR => Some(RecipeOutput {
                item: ITEM_RUBBER_SHEET,
                count: 2,
            }),
            "minecraft:copper_ingot" | ITEM_COPPER_PLATE => Some(RecipeOutput {
                item: ITEM_INSULATED_CABLE,
                count: 4,
            }),
            ITEM_SULFUR_DUST => Some(RecipeOutput {
                item: ITEM_RUBBER_BAR,
                count: 2,
            }),
            BLOCK_RUBBER_SAPLING
            | BLOCK_RUBBER_LEAVES
            | "minecraft:wheat_seeds"
            | "minecraft:sugar_cane"
            | "minecraft:kelp"
            | "minecraft:sweet_berries"
            | "minecraft:potato"
            | "minecraft:carrot" => Some(RecipeOutput {
                item: ITEM_BIO_FUEL,
                count: 2,
            }),
            "minecraft:rotten_flesh" => Some(RecipeOutput {
                item: "minecraft:leather",
                count: 1,
            }),
            "minecraft:Magma_cream" | "minecraft:magma_cream" => Some(RecipeOutput {
                item: "minecraft:slime_ball",
                count: 2,
            }),
            _ => None,
        },

        MachineKind::AlloySmelter => match input {
            "minecraft:iron_ingot" => Some(RecipeOutput { item: ITEM_STEEL_INGOT, count: 2 }),
            ITEM_IRON_DUST => Some(RecipeOutput { item: ITEM_STEEL_INGOT, count: 1 }),
            "minecraft:copper_ingot" | ITEM_COPPER_DUST => Some(RecipeOutput { item: ITEM_GOLD_PLATE, count: 1 }),
            "minecraft:gold_ingot" | ITEM_GOLD_DUST => Some(RecipeOutput { item: ITEM_CIRCUIT, count: 2 }),
            ITEM_RUBBER_BAR => Some(RecipeOutput { item: ITEM_RUBBER_SHEET, count: 3 }),
            _ => None,
        },

        MachineKind::CoalGenerator
        | MachineKind::BioGenerator
        | MachineKind::SolarGenerator
        | MachineKind::EnergyBank
        | MachineKind::ItemExtractor
        | MachineKind::SapExtractor
        | MachineKind::Quarry
        | MachineKind::AutoHarvester
        | MachineKind::BlockBreaker
        | MachineKind::BlockPlacer
        | MachineKind::ConveyorBelt
        | MachineKind::ItemPipe
        | MachineKind::EnergyMonitor
        | MachineKind::FluidTank
        | MachineKind::SteamBoiler
        | MachineKind::FluidPump
        | MachineKind::VoidTrashCan
        | MachineKind::EnergyRelay
        | MachineKind::ChargingPad => None,
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

/// Computes the 3D world coordinates `[x, y, z]` of the Auto Harvester's current scan target
/// across a 2-block high volume (`pos[1]..=pos[1]+1`) behind the Harvester.
pub fn harvester_target_pos(pos: [i32; 3], facing: &str, state: &MachineState) -> [i32; 3] {
    let (min_x, max_x, min_z, max_z) = quarry_area_bounds(pos, facing, state.quarry_radius);
    let width_x = (max_x - min_x + 1).max(1);
    let width_z = (max_z - min_z + 1).max(1);
    let layer_area = (width_x * width_z).max(1);
    let idx = state.quarry_cursor.rem_euclid(layer_area);
    let row_z = idx / width_x;
    let col_x = idx % width_x;
    let x = if row_z % 2 == 0 {
        min_x + col_x
    } else {
        max_x - col_x
    };
    let z = min_z + row_z;
    let y = pos[1] + (state.quarry_y_offset.rem_euclid(2));
    [x, y, z]
}

/// Checks if a block in the Auto Harvester's scan area is a mature crop (`replant = true`)
/// or a harvestable plant/tree log (`replant = false`), returning `(drop_item, drop_count, replant_crop)`.
pub fn harvester_drop_for_block(block_id: &str, age_prop: Option<&str>) -> Option<(&'static str, u32, bool)> {
    match block_id {
        "minecraft:wheat" if age_prop == Some("7") => Some(("minecraft:wheat", 2, true)),
        "minecraft:carrots" if age_prop == Some("7") => Some(("minecraft:carrot", 3, true)),
        "minecraft:potatoes" if age_prop == Some("7") => Some(("minecraft:potato", 3, true)),
        "minecraft:beetroots" if age_prop == Some("3") || age_prop == Some("7") => {
            Some(("minecraft:beetroot", 2, true))
        }
        "minecraft:nether_wart" if age_prop == Some("3") => Some(("minecraft:nether_wart", 3, true)),
        "minecraft:sweet_berry_bush" if matches!(age_prop, Some("2" | "3")) => {
            Some(("minecraft:sweet_berries", 3, true))
        }
        "minecraft:sugar_cane" => Some(("minecraft:sugar_cane", 1, false)),
        "minecraft:bamboo" => Some(("minecraft:bamboo", 1, false)),
        "minecraft:cactus" => Some(("minecraft:cactus", 1, false)),
        "minecraft:melon" => Some(("minecraft:melon_slice", 5, false)),
        "minecraft:pumpkin" | "minecraft:carved_pumpkin" => Some(("minecraft:pumpkin", 1, false)),
        "minecraft:oak_log" => Some(("minecraft:oak_log", 1, false)),
        "minecraft:spruce_log" => Some(("minecraft:spruce_log", 1, false)),
        "minecraft:birch_log" => Some(("minecraft:birch_log", 1, false)),
        "minecraft:jungle_log" => Some(("minecraft:jungle_log", 1, false)),
        "minecraft:acacia_log" => Some(("minecraft:acacia_log", 1, false)),
        "minecraft:dark_oak_log" => Some(("minecraft:dark_oak_log", 1, false)),
        "minecraft:cherry_log" => Some(("minecraft:cherry_log", 1, false)),
        "minecraft:mangrove_log" => Some(("minecraft:mangrove_log", 1, false)),
        BLOCK_RUBBER_LOG => Some((BLOCK_RUBBER_LOG, 1, false)),
        BLOCK_RUBBER_LEAVES => Some((BLOCK_RUBBER_SAPLING, 1, false)),
        _ => None,
    }
}

/// Deterministic layout for a grown Rubber Tree rooted at `base_pos`:
/// - A 5-block vertical trunk (`y = 0..5`), where `y = 1` and `y = 3` have a percentage chance
///   to become `BLOCK_RUBBER_SAP_LOG` (a rubber-oak trunk with an active Sap Cavity `lit=true`),
///   and the remaining trunk blocks are `BLOCK_RUBBER_LOG`.
/// - A lush 2-layer canopy of `BLOCK_RUBBER_LEAVES` around the top (`y = 3..=6`).
pub fn rubber_tree_blocks(base_pos: [i32; 3], seed: i64) -> Vec<([i32; 3], &'static str, Option<(&'static str, &'static str)>)> {
    let mut blocks = Vec::new();
    let hash = (base_pos[0] as i64 * 734287)
        ^ (base_pos[2] as i64 * 912931)
        ^ seed.wrapping_mul(31);

    // 1. Leaves canopy first so trunk overwrites center
    for dy in 3i32..=5i32 {
        let radius: i32 = if dy == 5 { 1 } else { 2 };
        let neg_radius: i32 = -radius;
        for dx in neg_radius..=radius {
            for dz in neg_radius..=radius {
                if dx == 0 && dz == 0 && dy < 5 {
                    continue;
                }
                if dx.abs() == radius && dz.abs() == radius && dy == 3 {
                    continue;
                }
                blocks.push((
                    [base_pos[0] + dx, base_pos[1] + dy, base_pos[2] + dz],
                    BLOCK_RUBBER_LEAVES,
                    Some(("persistent", "true")),
                ));
            }
        }
    }
    blocks.push((
        [base_pos[0], base_pos[1] + 6, base_pos[2]],
        BLOCK_RUBBER_LEAVES,
        Some(("persistent", "true")),
    ));

    // 2. Trunk (5 blocks tall, with 1-2 Sap Cavity blocks at y+1 / y+2 / y+3)
    let faces = ["north", "south", "east", "west"];
    for dy in 0i32..5i32 {
        let p = [base_pos[0], base_pos[1] + dy, base_pos[2]];
        let is_sap_cavity = dy == 1 || (dy == 3 && (hash.rem_euclid(100) < 65));
        if is_sap_cavity {
            let face = faces[((hash.unsigned_abs() as usize) + dy as usize) % 4];
            blocks.push((p, BLOCK_RUBBER_SAP_LOG, Some(("facing", face))));
        } else {
            blocks.push((p, BLOCK_RUBBER_LOG, None));
        }
    }

    blocks
}

/// Checks if a solar generator at `pos` is part of a complete 3x3 horizontal array of solar panels.
pub fn is_solar_array_3x3<F>(pos: [i32; 3], mut is_solar: F) -> bool
where
    F: FnMut([i32; 3]) -> bool,
{
    for min_x in (pos[0] - 2)..=pos[0] {
        for min_z in (pos[2] - 2)..=pos[2] {
            let mut complete = true;
            for x in min_x..=(min_x + 2) {
                for z in min_z..=(min_z + 2) {
                    if !is_solar([x, pos[1], z]) {
                        complete = false;
                        break;
                    }
                }
                if !complete {
                    break;
                }
            }
            if complete {
                return true;
            }
        }
    }
    false
}

/// Finds all connected `EnergyBank` blocks adjacent to `start` (clustering into a unified battery bank).
pub fn find_battery_cluster<F>(start: [i32; 3], mut is_battery: F) -> Vec<[i32; 3]>
where
    F: FnMut([i32; 3]) -> bool,
{
    let mut visited = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    visited.push(start);

    let directions = [
        [1, 0, 0], [-1, 0, 0],
        [0, 1, 0], [0, -1, 0],
        [0, 0, 1], [0, 0, -1],
    ];

    while let Some(current) = queue.pop_front() {
        if visited.len() >= 64 {
            break;
        }
        for d in directions {
            let neighbor = [current[0] + d[0], current[1] + d[1], current[2] + d[2]];
            if !visited.contains(&neighbor) && is_battery(neighbor) {
                visited.push(neighbor);
                queue.push_back(neighbor);
            }
        }
    }
    visited
}

/// Computes Solar Generator output (`FE/t`) based on world time, sky light (`0..=15`), weather, and 3x3 array synergy.
pub fn solar_generation_rate(
    day_time: i64,
    sky_light: u8,
    rain_level: f32,
    speed_upgrades: u32,
    is_array_3x3: bool,
) -> i32 {
    let tod = day_time.rem_euclid(24_000);
    // Daytime in Minecraft is 0..=12500
    if tod > 12_800 || sky_light < 6 {
        return 0;
    }
    let light_factor = i32::from(sky_light).clamp(0, 15);
    let base = (MachineKind::SolarGenerator.energy_per_tick() * light_factor) / 15;
    let upgrade_bonus = (speed_upgrades.min(8) as i32) * 4;
    let mut rate = base + upgrade_bonus;
    if rain_level > 0.2 {
        rate = (rate * 45) / 100;
    }
    // 3x3 Giant Solar Array captures 2x double solar energy!
    if is_array_3x3 {
        rate *= 2;
    }
    rate.max(1)
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
        MachineKind::CoalGenerator | MachineKind::BioGenerator | MachineKind::SteamBoiler => {
            step_generator(state, fuel_burn_ticks, speed_upgrades)
        }
        MachineKind::EnergyMonitor
        | MachineKind::FluidTank
        | MachineKind::VoidTrashCan
        | MachineKind::EnergyRelay
        | MachineKind::ChargingPad => StepOutcome {
            status: StepStatus::Idle,
            consume_input: false,
            consume_fuel: false,
            produced: None,
            net_energy_rate: 0,
        },
        MachineKind::FluidPump => {
            let cost = 15;
            if state.energy < cost {
                StepOutcome {
                    status: StepStatus::NoPower,
                    consume_input: false,
                    consume_fuel: false,
                    produced: None,
                    net_energy_rate: 0,
                }
            } else {
                state.energy -= cost;
                StepOutcome {
                    status: StepStatus::Working,
                    consume_input: false,
                    consume_fuel: false,
                    produced: None,
                    net_energy_rate: -cost,
                }
            }
        },

        MachineKind::SolarGenerator | MachineKind::EnergyBank => {
            let mut consume_fuel = false;
            if state.energy < state.kind.max_energy() {
                if let Some(burn) = fuel_burn_ticks {
                    if burn == -1 {
                        state.energy = (state.energy + 200).min(state.kind.max_energy());
                    } else if burn > 0 && state.energy + 1000 <= state.kind.max_energy() {
                        state.energy = (state.energy + 2000).min(state.kind.max_energy());
                        consume_fuel = true;
                    }
                }
            }
            StepOutcome {
                status: if state.energy > 0 {
                    StepStatus::Working
                } else {
                    StepStatus::Idle
                },
                consume_input: false,
                consume_fuel,
                produced: None,
                net_energy_rate: 0,
            }
        }
        MachineKind::ConveyorBelt | MachineKind::ItemExtractor | MachineKind::ItemPipe => {
            let period = effective_work_ticks(state.kind, speed_upgrades);
            state.progress = (state.progress + 1) % period.max(1);
            StepOutcome {
                status: StepStatus::Working,
                consume_input: false,
                consume_fuel: false,
                produced: None,
                net_energy_rate: 0,
            }
        }
        MachineKind::Quarry
        | MachineKind::AutoHarvester
        | MachineKind::SapExtractor
        | MachineKind::BlockBreaker
        | MachineKind::BlockPlacer => {
            // World-interacting machines are driven in glue with world access,
            // and receive fuel/battery top-up here.
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
        | MachineKind::MechanicalPress
        | MachineKind::IndustrialAssembler
        | MachineKind::Vulcanizer
        | MachineKind::AlloySmelter => {
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
    let gen_rate = state.kind.energy_per_tick() + (speed_upgrades.min(8) as i32) * 10;

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

/// Returns the exit offset for a conveyor belt with `(facing, shape)`.
pub fn conveyor_exit_offset(facing: &str, shape: &str) -> [i32; 3] {
    let (front, _) = conveyor_offsets(facing);
    match shape {
        "turn_left" => match facing {
            "north" => [-1, 0, 0],
            "west" => [0, 0, 1],
            "south" => [1, 0, 0],
            "east" => [0, 0, -1],
            _ => [-1, 0, 0],
        },
        "turn_right" => match facing {
            "north" => [1, 0, 0],
            "east" => [0, 0, 1],
            "south" => [-1, 0, 0],
            "west" => [0, 0, -1],
            _ => [1, 0, 0],
        },
        "up" => [front[0], 1, front[2]],
        "down" => [front[0], -1, front[2]],
        _ => front,
    }
}

/// Maps a unit cardinal offset to a direction index (`0=N, 1=S, 2=E, 3=W, 4=U, 5=D, 6=Center`).
pub fn dir_index_from_offset(off: [i32; 3]) -> u8 {
    match off {
        [0, 0, -1] => 0,
        [0, 0, 1] => 1,
        [1, 0, 0] => 2,
        [-1, 0, 0] => 3,
        [0, 1, 0] => 4,
        [0, -1, 0] => 5,
        _ => 6,
    }
}

pub fn offset_from_dir_index(dir: u8) -> [i32; 3] {
    match dir {
        0 => [0, 0, -1],
        1 => [0, 0, 1],
        2 => [1, 0, 0],
        3 => [-1, 0, 0],
        4 => [0, 1, 0],
        5 => [0, -1, 0],
        _ => [0, 0, 0],
    }
}

pub fn opposite_dir_index(dir: u8) -> u8 {
    match dir {
        0 => 1,
        1 => 0,
        2 => 3,
        3 => 2,
        4 => 5,
        5 => 4,
        _ => 6,
    }
}

pub const DIRS: [[i32; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

/// BFS from `start_pipe` through connected pipes to find the immediate next step toward `target` (or any valid target).
pub fn find_next_pipe_step<FPipe, FTarget>(
    start_pipe: [i32; 3],
    preferred_target: Option<[i32; 3]>,
    is_pipe: FPipe,
    is_target: FTarget,
) -> Option<([i32; 3], [i32; 3])>
where
    FPipe: Fn([i32; 3]) -> bool,
    FTarget: Fn([i32; 3]) -> bool,
{
    let mut visited = std::collections::BTreeSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start_pipe);

    // Check immediate neighbors first; record `first_step` in queue
    for d in DIRS {
        let next = [start_pipe[0] + d[0], start_pipe[1] + d[1], start_pipe[2] + d[2]];
        if preferred_target == Some(next) && is_target(next) {
            return Some((next, next));
        }
        if is_pipe(next) && visited.insert(next) {
            queue.push_back((next, next));
        } else if preferred_target.is_none() && is_target(next) {
            return Some((next, next));
        }
    }

    let mut fallback: Option<([i32; 3], [i32; 3])> = None;
    while let Some((curr, first_step)) = queue.pop_front() {
        if visited.len() > 256 {
            break;
        }
        for d in DIRS {
            let next = [curr[0] + d[0], curr[1] + d[1], curr[2] + d[2]];
            if preferred_target == Some(next) && is_target(next) {
                return Some((first_step, next));
            }
            if is_target(next) && fallback.is_none() {
                fallback = Some((first_step, next));
            }
            if is_pipe(next) && visited.insert(next) {
                queue.push_back((next, first_step));
            }
        }
    }
    fallback
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
