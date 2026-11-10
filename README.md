# ⚙️ Rusty Factory v0.3.0 — Mod de Automatización Industrial para Horizon (Rust)

Un mod completo de máquinas, energía eléctrica, excavación 3D, agricultura automatizada y logística industrial diseñado nativamente para **Horizon (Minecraft: Rust Edition 26.3)**.

Incluye tanto el paquete de **Contenido (`.hmod`)** con **16 bloques**, **18 ítems**, modelos 3D personalizados, animaciones `.png.mcmeta`, 4 interfaces gráficas (GUI), pestaña creativa y recetas, como el **Módulo Nativo en Rust (`.hnmod`)** con simulación de redes eléctricas y neumáticas por grafos (BFS), renderizado 3D en mundo (`world_primitives` y `block_entity_submit`) y controles interactivos.

---

## 🧱 Bloques Incluidos (16 Bloques — `rusty_factory:*`)

| ID del Bloque | Nombre | Función Principal |
| :--- | :--- | :--- |
| `quarry` | **Excavadora Industrial (Quarry 3D)** | Construye un marco 3D estilo BuildCraft detrás de la máquina (`5x5`, `7x7` o `9x9`) con puente grúa móvil, láser de minería animado y holograma 3D. Excava hasta Bedrock y envía todo por tuberías o cofres. |
| `auto_harvester` | **Cosechadora y Taladora 3D** | Escanea un área configurable (`5x5`, `7x7` o `9x9`) con marco láser verde 3D. Cosecha cultivos maduros (`trigo`, `zanahorias`, `papas`, `remolacha`, `bayas`) replantándolos automáticamente (`age=0`), además de caña, bambú, melones, calabazas y troncos. |
| `coal_generator` | **Generador Industrial a Carbón** | Quema carbón/combustible y genera **40+ FE/t** (hasta `20,000 FE`), distribuyendo energía por cables de cobre. |
| `solar_generator` | **Panel Solar Fotovoltaico (3D)** | Bloque de media altura (`0.375` de alto) con celdas solares animadas. Genera energía limpia durante el día según la luz solar (`native_light`) y el clima (`native_weather`). |
| `energy_bank` | **Banco de Capacitores (100k FE)** | Almacena hasta **100,000 FE**, muestra un holograma 3D flotante con su porcentaje de carga y distribuye hasta `400 FE/t` a las máquinas conectadas. |
| `copper_cable` | **Cable de Energía de Cobre** | Tubería eléctrica 6-direcciones (`PipeBlock`) animada que conecta generadores, paneles solares, bancos de energía y máquinas. |
| `item_pipe` | **Tubería Neumática de Ítems** | Tubería neumática 6-direcciones (`PipeBlock`) que se conecta a todas las máquinas, cintas y contenedores vanilla (`cofres`, `barriles`, `tolvas`, `hornos`). |
| `item_extractor` | **Extractor Neumático de Ítems** | Bomba neumática que extrae ítems automáticamente de cofres, barriles u hornos adyacentes y los inyecta en la red de `item_pipe`. Permite poner un ítem en la ranura 0 como **Filtro Inteligente**. |
| `conveyor_belt` | **Cinta Transportadora 3D** | Bloque mediano (`0.5` de altura) con rieles laterales 3D, rodillos, textura animada (8 frames) y renderizado 3D del ítem deslizándose sobre la cinta. Recoge ítems tirados encima y auto-extrae de contenedores traseros. |
| `electric_crusher` | **Trituradora Eléctrica** | Duplica menas en **2x Polvos Minerales** (`iron_dust`, `copper_dust`, `gold_dust`), tritura cuarzo/arena en **Silicio** (`silicon`), y multiplica diamantes, esmeraldas y redstone. Compatible con `fortune_upgrade`. |
| `electric_furnace` | **Horno Eléctrico de Inducción** | Funde polvos, menas, lingotes de hierro en **Acero** (`steel_ingot`), vidrio y comida a alta velocidad usando solo `FE`. |
| `mechanical_press` | **Prensa Mecánica Hidráulica** | Prensa lingotes en placas (`iron_plate`, `copper_plate`, `gold_plate`), acero en engranajes (`steel_gear`), y fabrica cables y tuberías con mayor rendimiento. |
| `industrial_assembler` | **Ensambladora Industrial Automática** | Aserradero y ensambladora robótica: corta troncos en **6 tablones**, y fabrica automáticamente circuitos, celdas solares, chasis, cintas, mejoras y cabezales de diamante. |
| `block_breaker` | **Perforadora Rompe-Bloques** | Rompe automáticamente el bloque situado frente a ella (ideal para generadores de adoquín, basalto u obsidiana) y expulsa los drops a tuberías o cofres. |
| `block_placer` | **Colocador Automático de Bloques** | Coloca automáticamente en el mundo los bloques que reciba en su ranura de entrada frente a su cara frontal. |
| `machine_casing` | **Chasis de Máquina Industrial** | Bloque estructural base para fabricar todas las máquinas. |

---

## 🔧 Ítems, Herramientas y Componentes (18 Ítems — `rusty_factory:*`)

* **`wrench` (Llave Inglesa Industrial):** Haz clic derecho sobre cables de cobre o tuberías de ítems para inspeccionar la red o impulsar extracción manual.
* **`mining_drill` (Taladro Minero Eléctrico de Diamante):** Herramienta de minería avanzada de nivel Netherite/Diamante.
* **`jet_boots` (Exo-Botas Industriales de Titanio):** Botas industriales reforzadas de nivel épico.
* **`speed_upgrade` (Mejora de Velocidad Overclock):** Acelera hasta un `+80%` la velocidad de cualquier máquina, cinta, extractor o generador.
* **`fortune_upgrade` (Mejora de Fortuna Industrial):** Aumenta el rendimiento de minerales, polvos y cultivos en la **Quarry**, **Trituradora**, **Cosechadora** y **Rompe-Bloques**.
* **`energy_battery` (Celda de Batería Industrial):** Recarga máquinas y bancos de energía al colocarse en la ranura de combustible.
* **`electronic_circuit`, `solar_cell`, `silicon`, `diamond_drill_head`:** Componentes electrónicos y de perforación avanzada.
* **`steel_ingot`, `steel_gear`, `iron_plate`, `copper_plate`, `gold_plate`, `iron_dust`, `copper_dust`, `gold_dust`:** Materiales metalúrgicos del árbol tecnológico.

---

## 🚀 Cómo instalar o importar en Horizon

### Opción 1: Desde GitHub en Horizon Launcher
Importa directamente el repositorio:
`https://github.com/Viega095/Rusty-Factory-Industria-.git`

### Opción 2: Instalación automática local
Ejecuta en PowerShell:
```powershell
python tools/make_textures.py
python tools/install_to_horizon.py
```
Dentro del juego puedes usar la pestaña creativa **"Rusty Factory (Industria)"** o ejecutar el comando `/rustyfactory kit` para recibir todas las máquinas, paneles solares, quarry, cosechadora y herramientas.
