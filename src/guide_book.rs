use mod_api::*;
use std::sync::Mutex;

const MOD_ID: &str = "rusty_factory";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Rubber,
    Energy,
    Network,
    Machines,
    Automation,
    Silo,
    Equipment,
}

impl Category {
    const ALL: [Category; 7] = [
        Self::Rubber,
        Self::Energy,
        Self::Network,
        Self::Machines,
        Self::Automation,
        Self::Silo,
        Self::Equipment,
    ];

    const fn title(self) -> &'static str {
        match self {
            Self::Rubber => "Inicio y Caucho",
            Self::Energy => "Generación FE",
            Self::Network => "Redes y Cables",
            Self::Machines => "Procesamiento",
            Self::Automation => "Automatización",
            Self::Silo => "Sistema Silo",
            Self::Equipment => "Equipo y Módulos",
        }
    }

    const fn short(self) -> &'static str {
        match self {
            Self::Rubber => "Caucho",
            Self::Energy => "Energía",
            Self::Network => "Redes",
            Self::Machines => "Máquinas",
            Self::Automation => "Auto",
            Self::Silo => "Silo",
            Self::Equipment => "Equipo",
        }
    }

    const fn icon(self) -> &'static str {
        match self {
            Self::Rubber => "rusty_factory:treetap",
            Self::Energy => "rusty_factory:coal_generator",
            Self::Network => "rusty_factory:cable_gold",
            Self::Machines => "rusty_factory:electric_crusher",
            Self::Automation => "rusty_factory:quarry",
            Self::Silo => "rusty_factory:silo_controller",
            Self::Equipment => "rusty_factory:mining_drill_mk3",
        }
    }

    const fn accent(self) -> u32 {
        match self {
            Self::Rubber => 0xFFD4_882A,
            Self::Energy => 0xFFE6_5C28,
            Self::Network => 0xFF28_A8D8,
            Self::Machines => 0xFF4A_90D9,
            Self::Automation => 0xFF38_B878,
            Self::Silo => 0xFF9B_68D9,
            Self::Equipment => 0xFFD9_A420,
        }
    }
}

#[derive(Clone, Copy)]
struct ScenePart {
    pos: [i32; 3],
    block: &'static str,
    facing: Option<&'static str>,
}

const fn sb(x: i32, y: i32, z: i32, block: &'static str) -> ScenePart {
    ScenePart {
        pos: [x, y, z],
        block,
        facing: None,
    }
}

const fn sbf(x: i32, y: i32, z: i32, block: &'static str, facing: &'static str) -> ScenePart {
    ScenePart {
        pos: [x, y, z],
        block,
        facing: Some(facing),
    }
}

#[derive(Clone, Copy)]
struct RecipeGrid {
    grid: [&'static str; 9],
    out: (&'static str, u32),
}

struct GuideEntry {
    category: Category,
    icon: &'static str,
    title: &'static str,
    tier: &'static str,
    summary: &'static [&'static str],
    specs: &'static [(&'static str, &'static str)],
    tip: &'static [&'static str],
    scene_title: &'static str,
    scene: &'static [ScenePart],
    legend: &'static [&'static str],
    recipe: RecipeGrid,
}

// ── 3D Example Scenes ────────────────────────────────────────────────────────

const SCENE_RUBBER_TREE: &[ScenePart] = &[
    sb(-1, 0, -1, "minecraft:grass_block"),
    sb(0, 0, -1, "minecraft:grass_block"),
    sb(1, 0, -1, "minecraft:grass_block"),
    sb(-1, 0, 0, "minecraft:grass_block"),
    sb(0, 0, 0, "minecraft:dirt"),
    sb(1, 0, 0, "minecraft:grass_block"),
    sb(-1, 0, 1, "minecraft:grass_block"),
    sb(0, 0, 1, "minecraft:grass_block"),
    sb(1, 0, 1, "minecraft:grass_block"),
    sb(0, 1, 0, "rusty_factory:rubber_log"),
    sbf(0, 2, 0, "rusty_factory:rubber_sap_log", "south"),
    sb(0, 3, 0, "rusty_factory:rubber_log"),
    sb(0, 4, 0, "rusty_factory:rubber_leaves"),
    sb(-1, 3, 0, "rusty_factory:rubber_leaves"),
    sb(1, 3, 0, "rusty_factory:rubber_leaves"),
    sb(0, 3, -1, "rusty_factory:rubber_leaves"),
    sb(0, 3, 1, "rusty_factory:rubber_leaves"),
    sbf(0, 2, 1, "rusty_factory:sap_extractor", "north"),
    sb(1, 2, 1, "rusty_factory:item_pipe"),
    sb(1, 1, 1, "minecraft:chest"),
];

const SCENE_VULCANIZER: &[ScenePart] = &[
    sb(-1, 0, 0, "rusty_factory:coal_generator"),
    sb(0, 0, 0, "rusty_factory:cable_copper"),
    sbf(1, 0, 0, "rusty_factory:vulcanizer", "south"),
    sb(1, 0, 1, "rusty_factory:conveyor_belt"),
    sb(2, 0, 1, "minecraft:chest"),
];

const SCENE_COAL_GEN: &[ScenePart] = &[
    sbf(-1, 0, 0, "rusty_factory:coal_generator", "south"),
    sb(0, 0, 0, "rusty_factory:cable_copper"),
    sb(0, 0, 1, "rusty_factory:energy_bank"),
    sb(1, 0, 0, "rusty_factory:cable_copper"),
    sbf(2, 0, 0, "rusty_factory:electric_furnace", "south"),
];

const SCENE_SOLAR: &[ScenePart] = &[
    sb(-1, 1, -1, "rusty_factory:solar_generator"),
    sb(0, 1, -1, "rusty_factory:solar_generator"),
    sb(1, 1, -1, "rusty_factory:solar_generator"),
    sb(-1, 1, 0, "rusty_factory:solar_generator"),
    sb(0, 1, 0, "rusty_factory:solar_generator"),
    sb(1, 1, 0, "rusty_factory:solar_generator"),
    sb(-1, 1, 1, "rusty_factory:solar_generator"),
    sb(0, 1, 1, "rusty_factory:solar_generator"),
    sb(1, 1, 1, "rusty_factory:solar_generator"),
    sb(0, 0, 0, "rusty_factory:cable_gold"),
    sb(1, 0, 0, "rusty_factory:energy_bank"),
    sb(2, 0, 0, "rusty_factory:energy_monitor"),
];

const SCENE_WIND: &[ScenePart] = &[
    sb(0, 0, 0, "rusty_factory:energy_bank"),
    sb(0, 1, 0, "rusty_factory:cable_copper"),
    sb(0, 2, 0, "rusty_factory:cable_copper"),
    sbf(0, 3, 0, "rusty_factory:wind_turbine", "south"),
    sb(1, 0, 0, "rusty_factory:cable_copper"),
    sbf(2, 0, 0, "rusty_factory:electric_crusher", "south"),
];

const SCENE_STEAM: &[ScenePart] = &[
    sb(-2, 0, 0, "minecraft:water"),
    sb(-2, 1, 0, "rusty_factory:fluid_pump"),
    sb(-1, 1, 0, "rusty_factory:fluid_pipe"),
    sbf(0, 1, 0, "rusty_factory:steam_boiler", "south"),
    sb(1, 1, 0, "rusty_factory:fluid_pipe_mk2"),
    sbf(2, 1, 0, "rusty_factory:steam_turbine", "south"),
    sb(2, 0, 0, "rusty_factory:cable_gold"),
    sb(1, 0, 0, "rusty_factory:energy_bank"),
];

const SCENE_NUCLEAR: &[ScenePart] = &[
    sbf(-1, 0, 0, "rusty_factory:nuclear_reactor", "south"),
    sb(-1, 1, 0, "rusty_factory:fluid_pipe_mk3"),
    sb(0, 1, 0, "rusty_factory:fluid_tank"),
    sb(-1, 0, 1, "rusty_factory:cable_red"),
    sb(0, 0, 1, "rusty_factory:energy_bank"),
    sb(1, 0, 1, "rusty_factory:tesseract"),
];

const SCENE_CABLES: &[ScenePart] = &[
    sbf(-2, 0, 0, "rusty_factory:coal_generator", "south"),
    sb(-1, 0, 0, "rusty_factory:cable_white"),
    sb(0, 0, 0, "rusty_factory:cable_copper"),
    sb(1, 0, 0, "rusty_factory:cable_gold"),
    sb(2, 0, 0, "rusty_factory:cable_red"),
    sb(2, 0, 1, "rusty_factory:energy_bank"),
    sb(0, 1, 0, "rusty_factory:energy_monitor"),
];

const SCENE_WIRELESS: &[ScenePart] = &[
    sbf(-2, 0, 0, "rusty_factory:solar_generator", "south"),
    sb(-1, 0, 0, "rusty_factory:energy_bank"),
    sb(-1, 1, 0, "rusty_factory:energy_relay"),
    sb(1, 1, 0, "rusty_factory:energy_relay"),
    sb(1, 0, 0, "rusty_factory:wireless_charger"),
    sb(2, 0, 0, "rusty_factory:charging_pad"),
];

const SCENE_CRUSHER_FURNACE: &[ScenePart] = &[
    sb(-2, 0, 0, "minecraft:chest"),
    sbf(-1, 0, 0, "rusty_factory:conveyor_belt", "east"),
    sbf(0, 0, 0, "rusty_factory:electric_crusher", "south"),
    sb(0, 1, 0, "rusty_factory:cable_copper"),
    sbf(1, 0, 0, "rusty_factory:electric_furnace", "south"),
    sb(1, 1, 0, "rusty_factory:cable_copper"),
    sb(2, 0, 0, "minecraft:chest"),
];

const SCENE_ALLOY_ASSEMBLER: &[ScenePart] = &[
    sbf(-1, 0, 0, "rusty_factory:alloy_smelter", "south"),
    sb(0, 0, 0, "rusty_factory:item_pipe_mk2"),
    sbf(1, 0, 0, "rusty_factory:industrial_assembler", "south"),
    sb(-1, 1, 0, "rusty_factory:cable_gold"),
    sb(0, 1, 0, "rusty_factory:energy_bank"),
    sb(1, 1, 0, "rusty_factory:cable_gold"),
];

const SCENE_HYDROPONIC: &[ScenePart] = &[
    sb(-1, 0, 0, "minecraft:water"),
    sb(-1, 1, 0, "rusty_factory:fluid_pump"),
    sb(0, 1, 0, "rusty_factory:fluid_pipe"),
    sbf(1, 1, 0, "rusty_factory:hydroponic_chamber", "south"),
    sb(1, 0, 0, "rusty_factory:energy_bank"),
    sb(2, 1, 0, "minecraft:chest"),
];

const SCENE_CONVEYORS: &[ScenePart] = &[
    sb(-2, 0, 0, "minecraft:chest"),
    sbf(-1, 0, 0, "rusty_factory:conveyor_belt", "east"),
    sbf(0, 0, 0, "rusty_factory:item_splitter", "east"),
    sbf(1, 0, 0, "rusty_factory:conveyor_belt_mk2", "east"),
    sb(2, 0, 0, "minecraft:chest"),
    sbf(0, 0, 1, "rusty_factory:conveyor_belt_mk3", "south"),
    sb(0, 0, 2, "minecraft:barrel"),
];

const SCENE_PIPES: &[ScenePart] = &[
    sb(-2, 0, 0, "minecraft:chest"),
    sb(-1, 0, 0, "rusty_factory:item_pipe"),
    sb(0, 0, 0, "rusty_factory:item_pipe_mk2"),
    sb(1, 0, 0, "rusty_factory:item_pipe_mk3"),
    sb(2, 0, 0, "minecraft:chest"),
    sb(-1, 1, 0, "rusty_factory:fluid_tank"),
    sb(0, 1, 0, "rusty_factory:fluid_pipe_mk2"),
    sb(1, 1, 0, "rusty_factory:fluid_tank"),
];

const SCENE_QUARRY: &[ScenePart] = &[
    sb(-2, 0, 0, "rusty_factory:energy_bank"),
    sb(-1, 0, 0, "rusty_factory:cable_gold"),
    sbf(0, 0, 0, "rusty_factory:quarry", "south"),
    sb(1, 0, 0, "rusty_factory:item_pipe_mk2"),
    sb(2, 0, 0, "minecraft:chest"),
    sb(-1, 0, 2, "minecraft:stone"),
    sb(0, 0, 2, "minecraft:iron_ore"),
    sb(1, 0, 2, "minecraft:deepslate_diamond_ore"),
];

const SCENE_SILO: &[ScenePart] = &[
    sbf(-1, 0, 0, "rusty_factory:industrial_silo", "south"),
    sbf(-1, 1, 0, "rusty_factory:industrial_silo", "south"),
    sbf(0, 0, 0, "rusty_factory:silo_controller", "south"),
    sbf(1, 0, 0, "rusty_factory:silo_crafting_terminal", "south"),
    sb(-1, 0, -1, "rusty_factory:item_pipe_mk2"),
    sb(0, 0, -1, "minecraft:chest"),
];

const SCENE_EQUIPMENT: &[ScenePart] = &[
    sbf(-1, 0, 0, "rusty_factory:coal_generator", "south"),
    sb(0, 0, 0, "rusty_factory:cable_copper"),
    sb(1, 0, 0, "rusty_factory:charging_pad"),
    sbf(2, 0, 0, "rusty_factory:upgrade_station", "south"),
];

const ENTRIES: &[GuideEntry] = &[
    // ── 1. INICIO Y CAUCHO ───────────────────────────────────────────────────
    GuideEntry {
        category: Category::Rubber,
        icon: "rusty_factory:treetap",
        title: "Árbol de Caucho y Savia",
        tier: "Tier 1 · Base",
        summary: &[
            "Todo empieza con el Árbol de Caucho.",
            "Sus troncos tienen cavidades doradas de",
            "savia. Usá el Treetap (clic derecho) para",
            "extraer Savia Cruda, o automatizalo con",
            "un Extractor de Savia pegado al tronco.",
        ],
        specs: &[
            ("Recolección", "Treetap o Sap Extractor"),
            ("Regeneración", "Cada 10 s automática"),
            ("Uso principal", "Barras y Láminas de Caucho"),
        ],
        tip: &[
            "No tales el tronco con cavidad dorada:",
            "regenera savia infinita automáticamente.",
        ],
        scene_title: "Ejemplo 3D: Extracción Automática",
        scene: SCENE_RUBBER_TREE,
        legend: &[
            "rusty_factory:rubber_sap_log",
            "rusty_factory:sap_extractor",
            "rusty_factory:item_pipe",
            "minecraft:chest",
        ],
        recipe: RecipeGrid {
            grid: [
                "", "minecraft:oak_planks", "",
                "minecraft:oak_planks", "minecraft:oak_planks", "minecraft:oak_planks",
                "minecraft:oak_planks", "", "",
            ],
            out: ("rusty_factory:treetap", 1),
        },
    },
    GuideEntry {
        category: Category::Rubber,
        icon: "rusty_factory:vulcanizer",
        title: "Vulcanizador y Caucho",
        tier: "Tier 1 · Proceso",
        summary: &[
            "El Vulcanizador procesa Savia Cruda o",
            "Resina con electricidad para fabricar",
            "Barras de Caucho y Láminas aislantes",
            "necesarias para cables, cintas y máquinas.",
        ],
        specs: &[
            ("Capacidad", "10,000 FE"),
            ("Consumo", "20 FE/t (100 ticks)"),
            ("Slots", "Entrada, Salida, Mejora"),
        ],
        tip: &[
            "Conectalo a un Generador de Carbón con",
            "Cable de Cobre para empezar rápido.",
        ],
        scene_title: "Ejemplo 3D: Cadena de Vulcanizado",
        scene: SCENE_VULCANIZER,
        legend: &[
            "rusty_factory:coal_generator",
            "rusty_factory:cable_copper",
            "rusty_factory:vulcanizer",
            "rusty_factory:conveyor_belt",
        ],
        recipe: RecipeGrid {
            grid: [
                "rusty_factory:rubber_bar", "minecraft:furnace", "rusty_factory:rubber_bar",
                "minecraft:piston", "rusty_factory:machine_casing", "minecraft:piston",
                "rusty_factory:copper_cable", "rusty_factory:electronic_circuit", "rusty_factory:copper_cable",
            ],
            out: ("rusty_factory:vulcanizer", 1),
        },
    },
    // ── 2. ENERGÍA ───────────────────────────────────────────────────────────
    GuideEntry {
        category: Category::Energy,
        icon: "rusty_factory:coal_generator",
        title: "Generador de Carbón y Bio",
        tier: "Tier 1 · Energía",
        summary: &[
            "Queman combustibles (carbón, madera,",
            "bloques de carbón, o biocombustible en",
            "el Bio-Generador) para producir FE.",
            "Empujan energía automáticamente a cables",
            "y máquinas adyacentes.",
        ],
        specs: &[
            ("Carbón", "+40 FE/t (Cap: 25,000 FE)"),
            ("Bio-Gen", "+60 FE/t (Cap: 40,000 FE)"),
            ("Auto-salida", "A todos los lados"),
        ],
        tip: &[
            "Poné un Banco de Energía al lado para no",
            "desperdiciar FE cuando no haya máquinas.",
        ],
        scene_title: "Ejemplo 3D: Red Inicial",
        scene: SCENE_COAL_GEN,
        legend: &[
            "rusty_factory:coal_generator",
            "rusty_factory:cable_copper",
            "rusty_factory:energy_bank",
            "rusty_factory:electric_furnace",
        ],
        recipe: RecipeGrid {
            grid: [
                "minecraft:iron_ingot", "minecraft:iron_ingot", "minecraft:iron_ingot",
                "minecraft:iron_ingot", "minecraft:furnace", "minecraft:iron_ingot",
                "rusty_factory:copper_cable", "rusty_factory:machine_casing", "rusty_factory:copper_cable",
            ],
            out: ("rusty_factory:coal_generator", 1),
        },
    },
    GuideEntry {
        category: Category::Energy,
        icon: "rusty_factory:solar_generator",
        title: "Panel Solar y Matriz 3x3",
        tier: "Tier 1-2 · Renovable",
        summary: &[
            "Genera electricidad limpia durante el día",
            "bajo cielo abierto. Si colocás 9 Paneles",
            "Solares juntos en una matriz 3x3, activan",
            "el bono de Matriz 3x3 (x2 de producción).",
        ],
        specs: &[
            ("Panel 1x1", "+25 FE/t de día"),
            ("Matriz 3x3", "+50 FE/t por panel (x2)"),
            ("Capacidad", "30,000 FE por panel"),
        ],
        tip: &[
            "La GUI te indica si detectó el bono",
            "MATRIZ 3x3 (x2) automáticamente.",
        ],
        scene_title: "Ejemplo 3D: Matriz Solar 3x3",
        scene: SCENE_SOLAR,
        legend: &[
            "rusty_factory:solar_generator",
            "rusty_factory:cable_gold",
            "rusty_factory:energy_bank",
            "rusty_factory:energy_monitor",
        ],
        recipe: RecipeGrid {
            grid: [
                "rusty_factory:solar_cell", "rusty_factory:solar_cell", "rusty_factory:solar_cell",
                "rusty_factory:copper_cable", "rusty_factory:electronic_circuit", "rusty_factory:copper_cable",
                "rusty_factory:iron_plate", "rusty_factory:machine_casing", "rusty_factory:iron_plate",
            ],
            out: ("rusty_factory:solar_generator", 1),
        },
    },
    GuideEntry {
        category: Category::Energy,
        icon: "rusty_factory:wind_turbine",
        title: "Turbina Eólica y Geotérmica",
        tier: "Tier 2 · Renovable",
        summary: &[
            "La Turbina Eólica produce energía 24/7",
            "y rinde mucho más cuanto más alta esté.",
            "La Geotérmica usa lava adyacente o baldes",
            "de lava para generar energía constante.",
        ],
        specs: &[
            ("Eólica", "Hasta +180 FE/t según altura"),
            ("Geotérmica", "+120 FE/t con lava"),
            ("Capacidad", "80,000 FE"),
        ],
        tip: &[
            "Subí la Turbina Eólica por encima de Y=90",
            "y bajá la corriente con cables.",
        ],
        scene_title: "Ejemplo 3D: Torre Eólica",
        scene: SCENE_WIND,
        legend: &[
            "rusty_factory:wind_turbine",
            "rusty_factory:cable_copper",
            "rusty_factory:energy_bank",
            "rusty_factory:electric_crusher",
        ],
        recipe: RecipeGrid {
            grid: [
                "rusty_factory:iron_plate", "rusty_factory:steel_gear", "rusty_factory:iron_plate",
                "rusty_factory:copper_cable", "rusty_factory:machine_casing", "rusty_factory:copper_cable",
                "rusty_factory:steel_ingot", "rusty_factory:electronic_circuit", "rusty_factory:steel_ingot",
            ],
            out: ("rusty_factory:wind_turbine", 1),
        },
    },
    GuideEntry {
        category: Category::Energy,
        icon: "rusty_factory:steam_boiler",
        title: "Caldera y Turbina de Vapor",
        tier: "Tier 2 · Vapor",
        summary: &[
            "La Bomba extrae agua del mundo y la envía",
            "por pipes de fluidos a la Caldera de Vapor.",
            "La Caldera calienta agua en Vapor y alimenta",
            "a la Turbina de Vapor para generar mucho FE.",
        ],
        specs: &[
            ("Caldera", "Agua + Calor -> Vapor"),
            ("Turbina", "+240 FE/t desde Vapor"),
            ("Capacidad", "120,000 FE"),
        ],
        tip: &[
            "Usá Pipes de Fluidos MK2 entre la Caldera",
            "y la Turbina para mantener el flujo.",
        ],
        scene_title: "Ejemplo 3D: Planta de Vapor",
        scene: SCENE_STEAM,
        legend: &[
            "rusty_factory:fluid_pump",
            "rusty_factory:fluid_pipe",
            "rusty_factory:steam_boiler",
            "rusty_factory:steam_turbine",
        ],
        recipe: RecipeGrid {
            grid: [
                "minecraft:copper_ingot", "rusty_factory:fluid_pipe", "minecraft:copper_ingot",
                "rusty_factory:iron_plate", "rusty_factory:machine_casing", "rusty_factory:iron_plate",
                "minecraft:furnace", "rusty_factory:copper_cable", "minecraft:furnace",
            ],
            out: ("rusty_factory:steam_boiler", 1),
        },
    },
    GuideEntry {
        category: Category::Energy,
        icon: "rusty_factory:nuclear_reactor",
        title: "Reactor Nuclear",
        tier: "Tier 3 · Endgame",
        summary: &[
            "El generador más potente del mod.",
            "Produce miles de FE/t usando combustible",
            "nuclear y refrigerante (agua). Vigila",
            "su temperatura o usá un Tanque de reserva.",
        ],
        specs: &[
            ("Producción", "+1,200 FE/t"),
            ("Capacidad", "1,000,000 FE"),
            ("Cable rec.", "Cable Rojo (Superconductor)"),
        ],
        tip: &[
            "Conectalo con Cable Rojo y un Tesseract",
            "para distribuir energía a toda tu base.",
        ],
        scene_title: "Ejemplo 3D: Reactor + Tesseract",
        scene: SCENE_NUCLEAR,
        legend: &[
            "rusty_factory:nuclear_reactor",
            "rusty_factory:fluid_pipe_mk3",
            "rusty_factory:cable_red",
            "rusty_factory:tesseract",
        ],
        recipe: RecipeGrid {
            grid: [
                "rusty_factory:steel_ingot", "rusty_factory:advanced_circuit", "rusty_factory:steel_ingot",
                "rusty_factory:cable_red", "rusty_factory:machine_casing", "rusty_factory:cable_red",
                "rusty_factory:steel_ingot", "minecraft:nether_star", "rusty_factory:steel_ingot",
            ],
            out: ("rusty_factory:nuclear_reactor", 1),
        },
    },
    // ── 3. REDES Y CABLES ────────────────────────────────────────────────────
    GuideEntry {
        category: Category::Network,
        icon: "rusty_factory:cable_gold",
        title: "Cables y Banco de Energía",
        tier: "Tier 1-3 · Red",
        summary: &[
            "Los cables conectan generadores con",
            "bancos y máquinas. Cada tier soporta",
            "mayor transferencia por tick. El Monitor",
            "muestra toda la red con clic derecho.",
        ],
        specs: &[
            ("Blanco / Cobre", "64 / 256 FE/t"),
            ("Oro / Rojo", "1,024 / 8,192 FE/t"),
            ("Banco Energía", "100,000 FE (x celdas)"),
        ],
        tip: &[
            "Usá el Escáner de Energía en cualquier",
            "máquina para ver su carga al instante.",
        ],
        scene_title: "Ejemplo 3D: Tiers de Cables",
        scene: SCENE_CABLES,
        legend: &[
            "rusty_factory:cable_white",
            "rusty_factory:cable_copper",
            "rusty_factory:cable_gold",
            "rusty_factory:cable_red",
        ],
        recipe: RecipeGrid {
            grid: [
                "rusty_factory:rubber_sheet", "rusty_factory:rubber_sheet", "rusty_factory:rubber_sheet",
                "minecraft:gold_ingot", "minecraft:redstone", "minecraft:gold_ingot",
                "rusty_factory:rubber_sheet", "rusty_factory:rubber_sheet", "rusty_factory:rubber_sheet",
            ],
            out: ("rusty_factory:cable_gold", 6),
        },
    },
    GuideEntry {
        category: Category::Network,
        icon: "rusty_factory:wireless_charger",
        title: "Relé, Cargador y Tesseract",
        tier: "Tier 2-3 · Inalámbrico",
        summary: &[
            "El Relé Inalámbrico enlaza redes a distancia.",
            "El Cargador de Área recarga tus herramientas",
            "y armadura en un radio cercano sin cables.",
            "El Tesseract transporta FE entre dimensiones.",
        ],
        specs: &[
            ("Relé", "Enlace inalámbrico de red"),
            ("Cargador Área", "Recarga inventario cercano"),
            ("Plataforma", "Recarga al pararte encima"),
        ],
        tip: &[
            "Dejá una Plataforma de Carga en tu base",
            "para recargar el Taladro y Jet Boots.",
        ],
        scene_title: "Ejemplo 3D: Carga Inalámbrica",
        scene: SCENE_WIRELESS,
        legend: &[
            "rusty_factory:energy_relay",
            "rusty_factory:wireless_charger",
            "rusty_factory:charging_pad",
            "rusty_factory:energy_bank",
        ],
        recipe: RecipeGrid {
            grid: [
                "minecraft:ender_pearl", "rusty_factory:cable_gold", "minecraft:ender_pearl",
                "rusty_factory:advanced_circuit", "rusty_factory:energy_bank", "rusty_factory:advanced_circuit",
                "rusty_factory:steel_ingot", "rusty_factory:cable_gold", "rusty_factory:steel_ingot",
            ],
            out: ("rusty_factory:wireless_charger", 1),
        },
    },
    // ── 4. PROCESAMIENTO ─────────────────────────────────────────────────────
    GuideEntry {
        category: Category::Machines,
        icon: "rusty_factory:electric_crusher",
        title: "Trituradora y Horno Eléctrico",
        tier: "Tier 1 · Proceso",
        summary: &[
            "La Trituradora Eléctrica duplica menas",
            "(1 mena -> 2 polvos) y muele piedra en",
            "grava/arena. El Horno Eléctrico funde",
            "polvos y materiales rápido sin carbón.",
        ],
        specs: &[
            ("Trituradora", "Duplica minerales (20 FE/t)"),
            ("Horno Eléc.", "Fundición rápida (20 FE/t)"),
            ("Mejoras", "Acepta Speed Upgrades"),
        ],
        tip: &[
            "Poné la Trituradora antes del Horno en",
            "línea con una cinta para duplicar lingotes.",
        ],
        scene_title: "Ejemplo 3D: Duplicación de Menas",
        scene: SCENE_CRUSHER_FURNACE,
        legend: &[
            "rusty_factory:conveyor_belt",
            "rusty_factory:electric_crusher",
            "rusty_factory:electric_furnace",
            "minecraft:chest",
        ],
        recipe: RecipeGrid {
            grid: [
                "minecraft:flint", "minecraft:piston", "minecraft:flint",
                "rusty_factory:iron_plate", "rusty_factory:machine_casing", "rusty_factory:iron_plate",
                "rusty_factory:copper_cable", "rusty_factory:electronic_circuit", "rusty_factory:copper_cable",
            ],
            out: ("rusty_factory:electric_crusher", 1),
        },
    },
    GuideEntry {
        category: Category::Machines,
        icon: "rusty_factory:alloy_smelter",
        title: "Aleaciones, Prensa y Ensamblador",
        tier: "Tier 2 · Industria",
        summary: &[
            "El Horno de Aleaciones combina 2 inputs",
            "(ej: Hierro + Carbón = Acero). La Prensa",
            "crea placas/engranajes y el Ensamblador",
            "fabrica circuitos y componentes complejos.",
        ],
        specs: &[
            ("Alloy Smelter", "2 entradas -> Aleación"),
            ("Prensa / Ensam.", "Placas, engranajes, chips"),
            ("Consumo", "40 FE/t (escalable)"),
        ],
        tip: &[
            "El Reciclador convierte basura en chatarra",
            "útil, y acepta Speed Upgrades.",
        ],
        scene_title: "Ejemplo 3D: Línea Industrial",
        scene: SCENE_ALLOY_ASSEMBLER,
        legend: &[
            "rusty_factory:alloy_smelter",
            "rusty_factory:item_pipe_mk2",
            "rusty_factory:industrial_assembler",
            "rusty_factory:energy_bank",
        ],
        recipe: RecipeGrid {
            grid: [
                "minecraft:blast_furnace", "rusty_factory:electronic_circuit", "minecraft:blast_furnace",
                "rusty_factory:steel_ingot", "rusty_factory:machine_casing", "rusty_factory:steel_ingot",
                "rusty_factory:cable_gold", "rusty_factory:steel_gear", "rusty_factory:cable_gold",
            ],
            out: ("rusty_factory:alloy_smelter", 1),
        },
    },
    GuideEntry {
        category: Category::Machines,
        icon: "rusty_factory:hydroponic_chamber",
        title: "Cámara Hidropónica",
        tier: "Tier 2 · Cultivos",
        summary: &[
            "Cultiva plantas, madera de caucho y",
            "cultivos automáticamente usando energía",
            "y agua bombeada por pipes de fluidos,",
            "sin necesitar espacio de granja.",
        ],
        specs: &[
            ("Requiere", "FE + Agua (mB)"),
            ("Salida", "Cultivos automáticos"),
            ("Auto-I/O", "Compatible con pipes"),
        ],
        tip: &[
            "Alimentala con una Bomba de Fluidos y",
            "sacá los cultivos directo a un Silo.",
        ],
        scene_title: "Ejemplo 3D: Granja Compacta",
        scene: SCENE_HYDROPONIC,
        legend: &[
            "rusty_factory:fluid_pump",
            "rusty_factory:fluid_pipe",
            "rusty_factory:hydroponic_chamber",
            "minecraft:chest",
        ],
        recipe: RecipeGrid {
            grid: [
                "minecraft:glass", "minecraft:glowstone_dust", "minecraft:glass",
                "rusty_factory:fluid_pipe", "rusty_factory:machine_casing", "rusty_factory:fluid_pipe",
                "rusty_factory:copper_cable", "minecraft:dirt", "rusty_factory:copper_cable",
            ],
            out: ("rusty_factory:hydroponic_chamber", 1),
        },
    },
    // ── 5. AUTOMATIZACIÓN ────────────────────────────────────────────────────
    GuideEntry {
        category: Category::Automation,
        icon: "rusty_factory:conveyor_belt",
        title: "Cintas MK1-MK3 y Divisor",
        tier: "Logística · 3D",
        summary: &[
            "Las Cintas transportan ítems en 3D con",
            "curvas y rampas automáticas. El Divisor",
            "reparte y filtra ítems por cara. Podés",
            "poner filtros de lista blanca/negra.",
        ],
        specs: &[
            ("MK1 / MK2 / MK3", "1 / 4 / 16 ítems por ciclo"),
            ("Divisor", "Filtro por cara (clic der.)"),
            ("Rotación", "Llave Inglesa (Wrench)"),
        ],
        tip: &[
            "Al tener una cinta en la mano ves una",
            "previsualización fantasma antes de ponerla.",
        ],
        scene_title: "Ejemplo 3D: Cintas y Divisor",
        scene: SCENE_CONVEYORS,
        legend: &[
            "rusty_factory:conveyor_belt",
            "rusty_factory:item_splitter",
            "rusty_factory:conveyor_belt_mk2",
            "rusty_factory:conveyor_belt_mk3",
        ],
        recipe: RecipeGrid {
            grid: [
                "", "", "",
                "rusty_factory:rubber_sheet", "rusty_factory:rubber_sheet", "rusty_factory:rubber_sheet",
                "minecraft:iron_ingot", "rusty_factory:copper_cable", "minecraft:iron_ingot",
            ],
            out: ("rusty_factory:conveyor_belt", 6),
        },
    },
    GuideEntry {
        category: Category::Automation,
        icon: "rusty_factory:item_pipe",
        title: "Pipes de Ítems y Fluidos",
        tier: "Logística · Pipes",
        summary: &[
            "Las Pipes de Ítems conectan cofres,",
            "máquinas y silos mostrando los ítems",
            "viajando por dentro. Las Pipes de Fluidos",
            "mueven agua, lava y vapor entre tanques.",
        ],
        specs: &[
            ("Item Pipe MK1-3", "Transporte rápido en red"),
            ("Fluid Pipe MK1-3", "Agua, Lava, Vapor, Savia"),
            ("Tanque Fluidos", "Almacena mB con medidor"),
        ],
        tip: &[
            "Usá MK2 o MK3 cuando conectes varias",
            "Quarrys o Reactores a la misma red.",
        ],
        scene_title: "Ejemplo 3D: Red de Pipes",
        scene: SCENE_PIPES,
        legend: &[
            "rusty_factory:item_pipe",
            "rusty_factory:item_pipe_mk2",
            "rusty_factory:fluid_tank",
            "rusty_factory:fluid_pipe_mk2",
        ],
        recipe: RecipeGrid {
            grid: [
                "minecraft:iron_ingot", "minecraft:glass", "minecraft:iron_ingot",
                "", "", "",
                "minecraft:iron_ingot", "minecraft:glass", "minecraft:iron_ingot",
            ],
            out: ("rusty_factory:item_pipe", 6),
        },
    },
    GuideEntry {
        category: Category::Automation,
        icon: "rusty_factory:quarry",
        title: "Quarry MK1-MK3 y Cosechadora",
        tier: "Tier 1-3 · Minería",
        summary: &[
            "La Quarry arma una estructura 3D y excava",
            "capa por capa hasta bedrock, enviando",
            "los minerales a cofres o pipes adjuntas.",
            "La Cosechadora recoge cultivos en su área.",
        ],
        specs: &[
            ("Área", "5x5, 7x7 o 9x9 (botón GUI)"),
            ("MK1 / MK2 / MK3", "Mayor velocidad y Fortuna"),
            ("Botón Reset", "Reinicia escaneo desde arriba"),
        ],
        tip: &[
            "Colocá un cofre o pipe pegado a la Quarry",
            "para que nunca se llene su salida.",
        ],
        scene_title: "Ejemplo 3D: Setup de Quarry",
        scene: SCENE_QUARRY,
        legend: &[
            "rusty_factory:quarry",
            "rusty_factory:energy_bank",
            "rusty_factory:item_pipe_mk2",
            "minecraft:chest",
        ],
        recipe: RecipeGrid {
            grid: [
                "rusty_factory:steel_gear", "minecraft:diamond_pickaxe", "rusty_factory:steel_gear",
                "rusty_factory:electronic_circuit", "rusty_factory:machine_casing", "rusty_factory:electronic_circuit",
                "rusty_factory:copper_cable", "rusty_factory:item_pipe", "rusty_factory:copper_cable",
            ],
            out: ("rusty_factory:quarry", 1),
        },
    },
    // ── 6. SISTEMA SILO ──────────────────────────────────────────────────────
    GuideEntry {
        category: Category::Silo,
        icon: "rusty_factory:silo_controller",
        title: "Silo Industrial y Terminal",
        tier: "Almacén · Red AE",
        summary: &[
            "El Silo Industrial guarda miles de ítems",
            "de un tipo con pantalla frontal. Conectá",
            "varios al Controlador de Silo y crafteá",
            "usando toda la red desde la Terminal.",
        ],
        specs: &[
            ("Silo Base", "2,048 ítems (+Upgrades 500K)"),
            ("Controlador", "Grilla 9x3 + Buscador"),
            ("Terminal", "Crafteo 3x3 directo de red"),
        ],
        tip: &[
            "La Tablet Inalámbrica de Silo te deja",
            "acceder a tus ítems desde lejos.",
        ],
        scene_title: "Ejemplo 3D: Red de Silos",
        scene: SCENE_SILO,
        legend: &[
            "rusty_factory:industrial_silo",
            "rusty_factory:silo_controller",
            "rusty_factory:silo_crafting_terminal",
            "rusty_factory:item_pipe_mk2",
        ],
        recipe: RecipeGrid {
            grid: [
                "rusty_factory:steel_ingot", "rusty_factory:electronic_circuit", "rusty_factory:steel_ingot",
                "minecraft:ender_pearl", "rusty_factory:cable_gold", "minecraft:ender_pearl",
                "rusty_factory:steel_ingot", "rusty_factory:industrial_silo", "rusty_factory:steel_ingot",
            ],
            out: ("rusty_factory:silo_controller", 1),
        },
    },
    // ── 7. EQUIPO Y MÓDULOS ──────────────────────────────────────────────────
    GuideEntry {
        category: Category::Equipment,
        icon: "rusty_factory:mining_drill_mk3",
        title: "Taladros, Láser, Armadura y Módulos",
        tier: "Tier 1-3 · Equipo",
        summary: &[
            "Taladros MK1-3 (1x1, 3x3, 5x5), Láser",
            "Minero, Omni-Tool, Espadas de Plasma,",
            "Armadura Cuántica, Jet Boots y Mochila.",
            "Instalá módulos en la Estación de Mejoras.",
        ],
        specs: &[
            ("Módulos", "Speed, Eff, Fortune, Silk, Magnet"),
            ("Extra", "Auto-Smelt y Visión Nocturna"),
            ("Llave Inglesa", "Rota/desmonta con Shift+Clic"),
        ],
        tip: &[
            "Cargá un módulo en la Estación de Mejoras",
            "y hacé clic derecho con tu Taladro/Armadura.",
        ],
        scene_title: "Ejemplo 3D: Estación de Equipo",
        scene: SCENE_EQUIPMENT,
        legend: &[
            "rusty_factory:charging_pad",
            "rusty_factory:upgrade_station",
            "rusty_factory:cable_copper",
            "rusty_factory:coal_generator",
        ],
        recipe: RecipeGrid {
            grid: [
                "rusty_factory:steel_ingot", "minecraft:diamond_pickaxe", "rusty_factory:steel_ingot",
                "rusty_factory:steel_gear", "rusty_factory:energy_battery_mk2", "rusty_factory:steel_gear",
                "rusty_factory:advanced_circuit", "rusty_factory:steel_ingot", "rusty_factory:advanced_circuit",
            ],
            out: ("rusty_factory:mining_drill_mk3", 1),
        },
    },
];

// ── Client UI State ──────────────────────────────────────────────────────────

struct BookUiState {
    category: Category,
    /// `None` = category index page; `Some(idx)` = index into `ENTRIES`
    selected_entry: Option<usize>,
    yaw: f32,
    pitch: f32,
    scale: f32,
    auto_rotate: bool,
    dragging_scene: bool,
    show_recipe: bool,
}

impl Default for BookUiState {
    fn default() -> Self {
        Self {
            category: Category::Rubber,
            selected_entry: Some(0),
            yaw: -38.0,
            pitch: 24.0,
            scale: 18.0,
            auto_rotate: true,
            dragging_scene: false,
            show_recipe: false,
        }
    }
}

static BOOK_UI: Mutex<Option<BookUiState>> = Mutex::new(None);
static PENDING_CLOSE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn with_state<R>(f: impl FnOnce(&mut BookUiState) -> R) -> R {
    let mut guard = BOOK_UI.lock().unwrap();
    let st = guard.get_or_insert_with(BookUiState::default);
    f(st)
}

pub fn open_book(host: &mut dyn ClientHost) -> bool {
    PENDING_CLOSE.store(false, std::sync::atomic::Ordering::Relaxed);
    with_state(|st| {
        st.dragging_scene = false;
    });
    host.mod_screen_open(MOD_ID)
}

pub fn tick_close(host: &mut dyn ClientHost) {
    if PENDING_CLOSE.swap(false, std::sync::atomic::Ordering::Relaxed) && is_book_open(host) {
        host.mod_screen_close(MOD_ID);
    }
}

pub fn is_book_open(host: &dyn ClientHost) -> bool {
    host.mod_screen() == Some(MOD_ID)
}

fn entries_in_category(cat: Category) -> Vec<usize> {
    ENTRIES
        .iter()
        .enumerate()
        .filter_map(|(i, e)| (e.category == cat).then_some(i))
        .collect()
}

// ── Layout Constants (GUI pixels) ────────────────────────────────────────────

const BOOK_W: f32 = 368.0;
const BOOK_H: f32 = 224.0;

fn book_origin(screen_rect: [f32; 4]) -> [f32; 2] {
    // `screen.rect` on a mod screen is `[x, y, w, h]` of the viewport
    let x = screen_rect[0] + ((screen_rect[2] - BOOK_W) * 0.5).max(8.0);
    let y = screen_rect[1] + ((screen_rect[3] - BOOK_H) * 0.5).max(8.0);
    [x, y]
}

fn in_rect(pt: [f32; 2], r: [f32; 4]) -> bool {
    pt[0] >= r[0] && pt[0] <= r[0] + r[2] && pt[1] >= r[1] && pt[1] <= r[1] + r[3]
}

fn scene_rect(ox: f32, oy: f32) -> [f32; 4] {
    // [x0, y0, x1, y1] for draw_scene
    [ox + 196.0, oy + 32.0, ox + 354.0, oy + 156.0]
}

fn scene_rect_wh(ox: f32, oy: f32) -> [f32; 4] {
    [ox + 196.0, oy + 32.0, 158.0, 124.0]
}

fn build_scene(host: &dyn ClientHost, parts: &[ScenePart]) -> (Scene, [f32; 3]) {
    let mut blocks = Vec::with_capacity(parts.len());
    let mut min_p = [i32::MAX; 3];
    let mut max_p = [i32::MIN; 3];
    for part in parts {
        let Some(key) = host.block_key(part.block) else {
            continue;
        };
        let state = if let Some(facing) = part.facing {
            BlockState::of(
                &host.block_states(),
                key,
                &[StateConstraint::new("facing", facing)],
            )
            .ok()
            .or_else(|| host.block_state_default(key))
        } else {
            host.block_state_default(key)
        };
        let Some(state) = state else {
            continue;
        };
        for axis in 0..3 {
            min_p[axis] = min_p[axis].min(part.pos[axis]);
            max_p[axis] = max_p[axis].max(part.pos[axis]);
        }
        blocks.push(SceneBlock::new(part.pos, state, None));
    }
    let center = if blocks.is_empty() {
        [0.5, 0.5, 0.5]
    } else {
        [
            (min_p[0] + max_p[0]) as f32 * 0.5 + 0.5,
            (min_p[1] + max_p[1]) as f32 * 0.5 + 0.5,
            (min_p[2] + max_p[2]) as f32 * 0.5 + 0.5,
        ]
    };
    (Scene::new(blocks), center)
}

fn draw_item_id(host: &dyn ClientHost, draw: &mut dyn DrawContext, id: &str, at: [f32; 2], size: f32) {
    if let Ok(ident) = Identifier::parse(id) {
        if let Some(key) = host.item_key(&ident) {
            let _ = draw.item(key, at, size);
        }
    }
}

fn draw_btn(draw: &mut dyn DrawContext, r: [f32; 4], label: &str, bg: u32, fg: u32) {
    draw.fill(r, 0xFF3E_3428);
    draw.fill([r[0] + 1.0, r[1] + 1.0, r[2] - 2.0, r[3] - 2.0], bg);
    let tw = draw.text_width(label);
    let tx = r[0] + (r[2] - tw) * 0.5;
    let ty = r[1] + (r[3] - 8.0) * 0.5;
    draw.text(label, [tx, ty], fg, false);
}

// ── Rendering ────────────────────────────────────────────────────────────────

pub fn render(host: &dyn ClientHost, screen: &ScreenView, draw: &mut dyn DrawContext) {
    if !is_book_open(host) {
        return;
    }

    // Dim backdrop over the world
    draw.fill(screen.rect, 0x990C_1018);

    let [ox, oy] = book_origin(screen.rect);
    let mouse = screen.mouse;

    let (cat, selected_entry, yaw, pitch, scale, auto_rotate, show_recipe) = with_state(|st| {
        if st.auto_rotate && !st.dragging_scene {
            st.yaw = (st.yaw + 0.35) % 360.0;
        }
        (
            st.category,
            st.selected_entry,
            st.yaw,
            st.pitch,
            st.scale,
            st.auto_rotate,
            st.show_recipe,
        )
    });

    // Left Category Tabs (sticking out on the left side of the book)
    for (i, &c) in Category::ALL.iter().enumerate() {
        let ty = oy + 14.0 + i as f32 * 27.0;
        let active = c == cat;
        let tab_x = if active { ox - 56.0 } else { ox - 50.0 };
        let tab_w = if active { 60.0 } else { 54.0 };
        let tab_r = [tab_x, ty, tab_w, 23.0];
        let hovered = in_rect(mouse, tab_r);
        let bg = if active {
            0xFFF2_E6CE
        } else if hovered {
            0xFFD9_C9AC
        } else {
            0xFFB8_A486
        };
        draw.fill(tab_r, 0xFF3B_2618);
        draw.fill([tab_x + 1.0, ty + 1.0, tab_w - 1.0, 21.0], bg);
        draw.fill([tab_x + 1.0, ty + 1.0, 3.0, 21.0], c.accent());
        draw_item_id(host, draw, c.icon(), [tab_x + 6.0, ty + 3.5], 16.0);
        draw.text(
            c.short(),
            [tab_x + 23.0, ty + 7.5],
            if active { 0xFF2B_1D12 } else { 0xFF3A_2A1D },
            false,
        );
    }

    // Book Outer Leather Cover + Copper Trim
    draw.fill([ox, oy, BOOK_W, BOOK_H], 0xFF2E_1E14);
    draw.fill([ox + 2.0, oy + 2.0, BOOK_W - 4.0, BOOK_H - 4.0], 0xFF5A_3824);
    draw.fill([ox + 4.0, oy + 4.0, BOOK_W - 8.0, BOOK_H - 8.0], 0xFFB8_7333);

    // Left & Right Parchment Pages
    draw.fill([ox + 8.0, oy + 8.0, 174.0, BOOK_H - 16.0], 0xFFF3_E7CF);
    draw.fill([ox + 186.0, oy + 8.0, 174.0, BOOK_H - 16.0], 0xFFF3_E7CF);
    // Center Spine Shading
    draw.fill([ox + 181.0, oy + 8.0, 6.0, BOOK_H - 16.0], 0xFFC7_B494);
    draw.fill([ox + 183.0, oy + 8.0, 2.0, BOOK_H - 16.0], 0xFF9C_886A);

    // Close [X] button top-right
    let close_r = [ox + BOOK_W - 22.0, oy + 10.0, 12.0, 12.0];
    draw_btn(
        draw,
        close_r,
        "X",
        if in_rect(mouse, close_r) { 0xFFC8_4444 } else { 0xFF8C_3636 },
        0xFFFF_FFFF,
    );

    let cat_entries = entries_in_category(cat);
    let entry_idx = selected_entry.filter(|idx| cat_entries.contains(idx));

    if let Some(idx) = entry_idx {
        let entry = &ENTRIES[idx];

        // ── LEFT PAGE: Header, Description, Specs, Tip ──
        draw_item_id(host, draw, entry.icon, [ox + 14.0, oy + 13.0], 18.0);
        draw.text(entry.title, [ox + 35.0, oy + 14.0], 0xFF2B_1D12, false);
        draw.text(entry.tier, [ox + 35.0, oy + 24.0], entry.category.accent(), false);
        draw.fill([ox + 14.0, oy + 34.0, 162.0, 1.0], 0xFFC8_B596);

        // Summary lines
        let mut sy = oy + 40.0;
        for line in entry.summary {
            draw.text(line, [ox + 14.0, sy], 0xFF3A_2A1D, false);
            sy += 10.0;
        }

        // Specs Card
        sy += 4.0;
        let specs_h = 14.0 + entry.specs.len() as f32 * 11.0;
        draw.fill([ox + 14.0, sy, 162.0, specs_h], 0xFFE4_D4B7);
        draw.fill([ox + 14.0, sy, 162.0, 11.0], 0xFF3A_3028);
        draw.text("Especificaciones", [ox + 18.0, sy + 1.5], 0xFFFF_D68A, false);
        let mut spy = sy + 14.0;
        for (k, v) in entry.specs {
            draw.text(&format!("{k}:"), [ox + 18.0, spy], 0xFF4A_3522, false);
            let vw = draw.text_width(v);
            draw.text(v, [ox + 172.0 - vw, spy], 0xFF1A_5C7A, false);
            spy += 11.0;
        }

        // Tip box
        let tip_y = sy + specs_h + 5.0;
        if tip_y < oy + 178.0 {
            draw.fill([ox + 14.0, tip_y, 162.0, 26.0], 0xFFE9_DEC5);
            draw.fill([ox + 14.0, tip_y, 2.5, 26.0], entry.category.accent());
            for (i, tline) in entry.tip.iter().enumerate() {
                draw.text(tline, [ox + 20.0, tip_y + 4.0 + i as f32 * 10.0], 0xFF4A_3B2C, false);
            }
        }

        // ── RIGHT PAGE: Interactive 3D Ponder Scene OR 3x3 Recipe ──
        let tab_3d = [ox + 194.0, oy + 12.0, 44.0, 14.0];
        let tab_rec = [ox + 241.0, oy + 12.0, 50.0, 14.0];
        draw_btn(
            draw,
            tab_3d,
            "Vista 3D",
            if !show_recipe { 0xFF2B_6E46 } else { 0xFF7A_654E },
            0xFFFF_FFFF,
        );
        draw_btn(
            draw,
            tab_rec,
            "Receta",
            if show_recipe { 0xFFB8_7333 } else { 0xFF7A_654E },
            0xFFFF_FFFF,
        );

        let swh = scene_rect_wh(ox, oy);
        if !show_recipe {
            draw.fill(swh, 0xFF16_1B24);
            draw.fill([swh[0] + 1.0, swh[1] + 1.0, swh[2] - 2.0, swh[3] - 2.0], 0xFF1E_2633);

            let (scene, center) = build_scene(host, entry.scene);
            let view = SceneView::new(center, yaw, pitch, scale);
            let _ = draw.draw_scene(scene_rect(ox, oy), &scene, view);

            draw.text(
                entry.scene_title,
                [swh[0] + 6.0, swh[1] + 5.0],
                0xDDE8_F0F8,
                false,
            );
            draw.text(
                "Arrastrá para rotar · Rueda zoom",
                [swh[0] + 6.0, swh[1] + swh[3] - 11.0],
                0xAA9A_B0C8,
                false,
            );

            // Camera control buttons
            let auto_r = [ox + 196.0, oy + 160.0, 52.0, 14.0];
            let reset_r = [ox + 252.0, oy + 160.0, 48.0, 14.0];
            draw_btn(
                draw,
                auto_r,
                if auto_rotate { "↻ Auto: ON" } else { "↻ Auto: OFF" },
                if auto_rotate { 0xFF2B_6E46 } else { 0xFF5A_4E40 },
                0xFFFF_FFFF,
            );
            draw_btn(
                draw,
                reset_r,
                "↺ Reset",
                if in_rect(mouse, reset_r) { 0xFF6B_5B4A } else { 0xFF5A_4E40 },
                0xFFFF_FFFF,
            );
        } else {
            // 3x3 Crafting Recipe Grid
            draw.fill(swh, 0xFFE6_D7BC);
            draw.fill([swh[0] + 1.0, swh[1] + 1.0, swh[2] - 2.0, swh[3] - 2.0], 0xFFEC_DFC8);
            draw.text("Mesa de Crafteo (3x3)", [swh[0] + 8.0, swh[1] + 8.0], 0xFF3A_2A1D, false);

            let gx0 = swh[0] + 12.0;
            let gy0 = swh[1] + 28.0;
            let mut hovered_item: Option<&str> = None;

            for r in 0..3 {
                for c in 0..3 {
                    let sx = gx0 + c as f32 * 22.0;
                    let sy = gy0 + r as f32 * 22.0;
                    let sr = [sx, sy, 20.0, 20.0];
                    let hov = in_rect(mouse, sr);
                    draw.fill(sr, 0xFF3A_3028);
                    draw.fill([sx + 1.0, sy + 1.0, 18.0, 18.0], if hov { 0xFFB8_A890 } else { 0xFF8B_8B8B });
                    let ing = entry.recipe.grid[r * 3 + c];
                    if !ing.is_empty() {
                        draw_item_id(host, draw, ing, [sx + 2.0, sy + 2.0], 16.0);
                        if hov {
                            hovered_item = Some(ing);
                        }
                    }
                }
            }

            // Arrow + Output slot
            draw.text("➔", [gx0 + 73.0, gy0 + 27.0], 0xFF4A_3522, false);
            let out_r = [gx0 + 94.0, gy0 + 20.0, 24.0, 24.0];
            let out_hov = in_rect(mouse, out_r);
            draw.fill(out_r, 0xFF3A_3028);
            draw.fill(
                [out_r[0] + 1.5, out_r[1] + 1.5, 21.0, 21.0],
                if out_hov { 0xFFD9_C088 } else { 0xFFA6_9478 },
            );
            draw_item_id(host, draw, entry.recipe.out.0, [out_r[0] + 4.0, out_r[1] + 4.0], 16.0);
            if entry.recipe.out.1 > 1 {
                draw.text(
                    &format!("x{}", entry.recipe.out.1),
                    [out_r[0] + 12.0, out_r[1] + 14.0],
                    0xFFFF_FFFF,
                    true,
                );
            }
            if out_hov {
                hovered_item = Some(entry.recipe.out.0);
            }

            draw.text(
                "Pasá el mouse por los ítems",
                [swh[0] + 8.0, swh[1] + swh[3] - 14.0],
                0xFF6B_5844,
                false,
            );
            if let Some(iid) = hovered_item {
                let pretty = iid.split(':').nth(1).unwrap_or(iid).replace('_', " ");
                let _ = draw.gui_tooltip(&[Text::literal(pretty)], mouse);
            }
        }

        // Component Legend icons under controls
        draw.text("Bloques:", [ox + 196.0, oy + 180.0], 0xFF4A_3522, false);
        let mut hovered_block: Option<&str> = None;
        for (li, &blk) in entry.legend.iter().enumerate() {
            let lx = ox + 242.0 + li as f32 * 24.0;
            let ly = oy + 176.0;
            let lr = [lx, ly, 18.0, 18.0];
            let hov = in_rect(mouse, lr);
            draw.fill(lr, if hov { 0xFFD4_BA92 } else { 0xFFE2_D2B5 });
            draw_item_id(host, draw, blk, [lx + 1.0, ly + 1.0], 16.0);
            if hov {
                hovered_block = Some(blk);
            }
        }
        if let Some(blk) = hovered_block {
            let pretty = blk
                .split(':')
                .nth(1)
                .unwrap_or(blk)
                .replace('_', " ");
            let _ = draw.gui_tooltip(&[Text::literal(pretty)], mouse);
        }

        // Bottom Navigation
        let idx_btn = [ox + 14.0, oy + BOOK_H - 24.0, 54.0, 13.0];
        draw_btn(draw, idx_btn, "◀ Índice", 0xFF7A_5E42, 0xFFFF_FFFF);

        let pos_in_cat = cat_entries.iter().position(|&x| x == idx).unwrap_or(0);
        let page_lbl = format!("{} / {}", pos_in_cat + 1, cat_entries.len());
        let pw = draw.text_width(&page_lbl);
        draw.text(&page_lbl, [ox + 184.0 - pw * 0.5, oy + BOOK_H - 21.0], 0xFF4A_3522, false);

        if pos_in_cat > 0 {
            let prev_r = [ox + 196.0, oy + BOOK_H - 24.0, 54.0, 13.0];
            draw_btn(draw, prev_r, "◀ Anterior", 0xFF7A_5E42, 0xFFFF_FFFF);
        }
        if pos_in_cat + 1 < cat_entries.len() {
            let next_r = [ox + 298.0, oy + BOOK_H - 24.0, 56.0, 13.0];
            draw_btn(draw, next_r, "Siguiente ▶", 0xFF7A_5E42, 0xFFFF_FFFF);
        }
    } else {
        // ── CATEGORY INDEX VIEW ──
        draw.text("Manual de Rusty Factory", [ox + 14.0, oy + 14.0], 0xFF2B_1D12, false);
        draw.text(cat.title(), [ox + 14.0, oy + 26.0], cat.accent(), false);
        draw.fill([ox + 14.0, oy + 37.0, 162.0, 1.0], 0xFFC8_B596);

        for (i, &e_idx) in cat_entries.iter().enumerate() {
            let entry = &ENTRIES[e_idx];
            let ry = oy + 44.0 + i as f32 * 26.0;
            let row_r = [ox + 14.0, ry, 162.0, 22.0];
            let hov = in_rect(mouse, row_r);
            draw.fill(row_r, if hov { 0xFFE0_CDA9 } else { 0xFFE9_DEC5 });
            draw.fill([ox + 14.0, ry, 3.0, 22.0], cat.accent());
            draw_item_id(host, draw, entry.icon, [ox + 20.0, ry + 3.0], 16.0);
            draw.text(entry.title, [ox + 40.0, ry + 3.5], 0xFF2B_1D12, false);
            draw.text(entry.tier, [ox + 40.0, ry + 12.5], 0xFF6B_5844, false);
        }

        // Right page: Welcome / Overview + 3D preview of first entry in category
        draw.text("Vista Rápida 3D", [ox + 194.0, oy + 15.0], 0xFF2B_1D12, false);
        if let Some(&first_idx) = cat_entries.first() {
            let entry = &ENTRIES[first_idx];
            let swh = scene_rect_wh(ox, oy);
            draw.fill(swh, 0xFF16_1B24);
            let (scene, center) = build_scene(host, entry.scene);
            let _ = draw.draw_scene(scene_rect(ox, oy), &scene, SceneView::new(center, yaw, pitch, scale));
            draw.text("Hacé clic en un tema a la", [ox + 196.0, oy + 166.0], 0xFF4A_3522, false);
            draw.text("izquierda para ver su guía.", [ox + 196.0, oy + 177.0], 0xFF4A_3522, false);
        }
    }
}

// ── Input Handlers ───────────────────────────────────────────────────────────

pub fn mouse_pressed(host: &dyn ClientHost, screen: &ScreenView, button: i32) -> bool {
    if !is_book_open(host) {
        return false;
    }
    let [ox, oy] = book_origin(screen.rect);
    let m = screen.mouse;

    // Close [X]
    if in_rect(m, [ox + BOOK_W - 22.0, oy + 10.0, 12.0, 12.0]) {
        PENDING_CLOSE.store(true, std::sync::atomic::Ordering::Relaxed);
        return true;
    }

    // Category tabs
    for (i, &c) in Category::ALL.iter().enumerate() {
        let ty = oy + 14.0 + i as f32 * 27.0;
        let tab_r = [ox - 56.0, ty, 60.0, 23.0];
        if in_rect(m, tab_r) {
            with_state(|st| {
                st.category = c;
                st.selected_entry = None;
            });
            return true;
        }
    }

    let (cat, selected_entry) = with_state(|st| (st.category, st.selected_entry));
    let cat_entries = entries_in_category(cat);
    let entry_idx = selected_entry.filter(|idx| cat_entries.contains(idx));

    if let Some(idx) = entry_idx {
        // [Vista 3D] / [Receta] tabs
        if in_rect(m, [ox + 194.0, oy + 12.0, 44.0, 14.0]) {
            with_state(|st| st.show_recipe = false);
            return true;
        }
        if in_rect(m, [ox + 241.0, oy + 12.0, 50.0, 14.0]) {
            with_state(|st| st.show_recipe = true);
            return true;
        }
        // 3D viewport drag start
        if button == 0 && !with_state(|st| st.show_recipe) && in_rect(m, scene_rect_wh(ox, oy)) {
            with_state(|st| {
                st.dragging_scene = true;
                st.auto_rotate = false;
            });
            return true;
        }
        // Auto-rotate toggle
        if in_rect(m, [ox + 196.0, oy + 160.0, 52.0, 14.0]) {
            with_state(|st| st.auto_rotate = !st.auto_rotate);
            return true;
        }
        // Reset camera
        if in_rect(m, [ox + 252.0, oy + 160.0, 48.0, 14.0]) {
            with_state(|st| {
                st.yaw = -38.0;
                st.pitch = 24.0;
                st.scale = 18.0;
                st.auto_rotate = true;
            });
            return true;
        }
        // Back to Index
        if in_rect(m, [ox + 14.0, oy + BOOK_H - 24.0, 54.0, 13.0]) {
            with_state(|st| st.selected_entry = None);
            return true;
        }
        let pos_in_cat = cat_entries.iter().position(|&x| x == idx).unwrap_or(0);
        if pos_in_cat > 0 && in_rect(m, [ox + 196.0, oy + BOOK_H - 24.0, 54.0, 13.0]) {
            with_state(|st| st.selected_entry = Some(cat_entries[pos_in_cat - 1]));
            return true;
        }
        if pos_in_cat + 1 < cat_entries.len()
            && in_rect(m, [ox + 298.0, oy + BOOK_H - 24.0, 56.0, 13.0])
        {
            with_state(|st| st.selected_entry = Some(cat_entries[pos_in_cat + 1]));
            return true;
        }
    } else {
        // Index entry click
        for (i, &e_idx) in cat_entries.iter().enumerate() {
            let ry = oy + 44.0 + i as f32 * 26.0;
            if in_rect(m, [ox + 14.0, ry, 162.0, 22.0]) {
                with_state(|st| st.selected_entry = Some(e_idx));
                return true;
            }
        }
        if button == 0 && in_rect(m, scene_rect_wh(ox, oy)) {
            with_state(|st| {
                st.dragging_scene = true;
                st.auto_rotate = false;
            });
            return true;
        }
    }

    true
}

pub fn mouse_released(host: &dyn ClientHost) -> bool {
    if !is_book_open(host) {
        return false;
    }
    with_state(|st| st.dragging_scene = false);
    true
}

pub fn mouse_dragged(host: &dyn ClientHost, delta: [f32; 2]) -> bool {
    if !is_book_open(host) {
        return false;
    }
    with_state(|st| {
        if st.dragging_scene {
            st.yaw = (st.yaw + delta[0] as f32 * 0.9) % 360.0;
            st.pitch = (st.pitch + delta[1] as f32 * 0.7).clamp(-60.0, 80.0);
        }
    });
    true
}

pub fn mouse_scrolled(host: &dyn ClientHost, screen: &ScreenView, scroll_y: f64) -> bool {
    if !is_book_open(host) {
        return false;
    }
    let [ox, oy] = book_origin(screen.rect);
    if in_rect(screen.mouse, scene_rect_wh(ox, oy)) {
        with_state(|st| {
            st.scale = (st.scale + scroll_y as f32 * 2.0).clamp(10.0, 34.0);
        });
    }
    true
}
