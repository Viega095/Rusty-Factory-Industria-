# ⚙️ Rusty Factory v1.0.0 — Mod de Automatización Industrial, Energía, Silos y Equipo Tech para Horizon (Rust)

Un mod completo de automatización industrial, redes eléctricas y neumáticas, almacenamiento masivo tipo AE2, agricultura automatizada, **Árboles de Caucho** y equipamiento eléctrico por **Tiers (Tier 1 → Tier 2 → Tier 3)** diseñado nativamente para **Horizon (Minecraft: Rust Edition)**.

Incluye tanto el paquete de **Contenido (`.hmod`)** con **57 bloques** y **76 ítems** organizados por tiers en **4 pestañas creativas** (`Armaduras y Herramientas`, `Logística y Energía`, `Máquinas`, `Materiales y Componentes`), modelos 3D personalizados, texturas pixel-art, menús interactivos y traducciones (`en_us`, `es_es`, `es_ar`), como el **Módulo Nativo en Rust (`.hnmod`)** con simulación por grafos, renderizado 3D en mundo y HUD de armadura/herramientas.

---

## 🚀 Novedades Principales en v1.0.0

### 1. Progresión Organizada por Tiers (`Tier 1 → Tier 2 → Tier 3`)
* **Pestañas creativas ordenadas** por familia y tier (`equipment`, `logistics`, `machines`, `materials`), con etiquetas `[Tier 1]`, `[Tier 2]` y `[Tier 3]` en tooltips.
* **Síntesis balanceada de minerales y Netherite:**
  * Compresión de `8× coal_block + 1× obsidian` → `1× minecraft:diamond`.
  * `netherite_seed_core` (`4× diamond + 4× gold_ingot + 4× obsidian + 1× blaze_powder`) → procesado en `alloy_smelter` → `1× minecraft:netherite_scrap`.

### 2. Generación de Energía (FE) y Redes Inalámbricas
* **Tier 1:** `coal_generator` (+20 FE/t), `bio_generator` (+24 FE/t), `solar_generator` (+12 FE/t, doble en matriz 3×3) y `steam_boiler` (+80 FE/t).
* **Tier 2:** `wind_turbine` (+15 a +120 FE/t según altura Y), `geothermal_generator` (+120 FE/t con lava) y `steam_turbine` (+160 FE/t con vapor).
* **Tier 3:** `nuclear_reactor` (+1,000 FE/t con diamantes, esmeraldas, estrellas del Nether o núcleos de netherite).
* **Cables y Energía:** `cable_white` (120 FE/t), `copper_cable` (360 FE/t), `cable_gold` (1,200 FE/t), `cable_red` (10,000 FE/t), `energy_bank`, `energy_relay`, `energy_monitor`, `tesseract` (transmisión inalámbrica a distancia) y `charging_pad` (recarga todo tu inventario y armadura al pararte encima).

### 3. Almacenamiento Masivo, Terminales y Logística
* **Silos Industriales (`industrial_silo`):** 2,000 ítems base ampliables con `silo_upgrade_5k`, `10k`, `100k` y `500k`, pantalla 3D frontal iluminada y barra de capacidad.
* **Controlador de Silos (`silo_controller`) y Terminal de Crafteo (`silo_crafting_terminal`):**
  * Unifican todos los `industrial_silo` conectados y cofres/barriles cercanos en una grilla **`9×3`** con íconos reales, paginación y orden por cantidad o nombre.
  * **Clic directo con cursor:** depositá ítems desde el mouse o retirá `x64` directo al cursor.
  * **Buscador en tiempo real** por nombre + botón `X` para limpiar.
  * **Crafteo `3×3` automático** en `silo_crafting_terminal` con **auto-reposición** desde tus Silos al retirar el resultado.
* **Cintas y Tuberías por Tiers:**
  * Cintas `conveyor_belt` (`MK1`, `MK2`, `MK3`), `belt_filter` (con ícono 3D), `belt_filter_advanced` y `item_splitter`.
  * Tuberías de ítems `item_pipe` (4 ítems), `item_pipe_mk2` (16 ítems, 2.5× velocidad) e `item_pipe_mk3` (64 ítems al instante), sin tráfico visual en 3D.
  * Tuberías de fluidos `fluid_pipe` (250 mB/t), `fluid_pipe_mk2` (1,000 mB/t) y `fluid_pipe_mk3` (4,000 mB/t).

### 4. Máquinas, Quarrys, Agricultura y Defensa
* **Procesamiento:** `electric_furnace`, `electric_crusher`, `mechanical_press`, `alloy_smelter`, `vulcanizer`, `industrial_assembler` y `recycler`.
* **Quarrys por Tiers:** `quarry` (5×5–9×9), `quarry_mk2` (13×13, 2× velocidad, Fortuna I) y `quarry_mk3` (21×21, 4× velocidad, Fortuna II), con salto rápido de capas vacías de aire.
* **Agricultura y Utilidad:** `sap_extractor`, `fluid_pump`, `fluid_tank`, `auto_harvester`, `block_breaker`, `block_placer`, `void_trash_can`, `upgrade_station` y **Cámara Hidropónica (`hydroponic_chamber`)** que cultiva semillas, kelp, madera y savia en 1 solo bloque.
* **Defensa:** **Torreta Láser (`laser_turret`)** Tier 3 que ataca mobs cercanos en un radio de 16 bloques (10 de daño).

### 5. Armaduras, Herramientas Eléctricas y Módulos
* **Armaduras por Tiers + HUD 2D de personaje:**
  * **Tier 1:** `steel_*` (25k FE).
  * **Tier 2:** `industrial_*`, `solar_helmet`, `jetpack` y `jet_boots`.
  * **Tier 3:** `quantum_*` (250k FE) y `jetpack_mk2` con Modo Hover.
* **Herramientas por Tiers:**
  * **Taladros:** `mining_drill` (`1x1`/`3x3`), `mining_drill_mk2` y `mining_drill_mk3` (`5x5`) — pican con clic izquierdo consumiendo FE por cada bloque roto; `Shift + Clic Derecho` alterna modo.
  * **Motosierras:** `electric_chainsaw` (64 troncos), `mk2` (128 + hojas) y `mk3` (256 + hojas).
  * **Láseres Mineros:** `mining_laser` (4 blq), `mk2` (8 blq) y `mk3` (16 blq).
  * **Sables de Plasma:** `plasma_sword_mk1` (9 daño), `mk2` (12 daño + descarga) y `mk3` (16 daño + Blink con `Shift + Clic Derecho`).
  * **Omni-Herramientas:** `omni_tool_mk2` y `omni_tool_mk3` (Taladro + Motosierra + Llave Inglesa).
  * **Imanes:** `item_magnet` (12 blq), `mk2` y `mk3` (modo pasivo automático en inventario).
  * **Portátiles y Baterías:** `wrench`, `treetap`, `energy_scanner`, `energy_battery` (50k), `energy_battery_mk2` (250k), `battery_pack` (1M), `quantum_battery` (5M), **`wireless_silo_tablet`** (Control Remoto de Silos) y **`quantum_backpack`** (auto-envío de stacks minados a tus Silos).
* **Estación de Mejoras (`upgrade_station`):** 7 módulos instalables (`Visión Nocturna`, `Imán`, `Eficiencia`, `Auto-Fundición`, `Fortuna`, `Toque de Seda`, `Velocidad`).
