# ⚙️ Rusty Factory v0.4.0 — Mod de Automatización Industrial, Árbol de Caucho y Química para Horizon (Rust)

Un mod completo de máquinas, energía eléctrica, excavación 3D, agricultura automatizada, **Árboles de Caucho con Huecos de Savia**, vulcanización de goma y logística industrial diseñado nativamente para **Horizon (Minecraft: Rust Edition 26.3)**.

Incluye tanto el paquete de **Contenido (`.hmod`)** con **24 bloques**, **25 ítems**, modelos 3D personalizados, animaciones `.png.mcmeta`, 4 interfaces gráficas (GUI), pestaña creativa y recetas de fundición/fabricación, como el **Módulo Nativo en Rust (`.hnmod`)** con simulación de redes eléctricas y neumáticas por grafos (BFS), crecimiento de Árboles de Caucho con regeneración natural de savia, renderizado 3D en mundo (`world_primitives` y `block_entity_submit`) y controles interactivos.

---

## 🌳 Nuevo en v0.4.0: Árbol de Caucho, Savia y Vulcanización de Goma

* **`rubber_sapling` (Brote de Árbol de Caucho):** Al plantarlo crece automáticamente (o al instante haciendo clic derecho) formando un **Árbol de Caucho** completo con hojas (`rubber_leaves`), troncos de caucho (`rubber_log`) y un porcentaje de bloques de tronco con **Huecos de Savia (`rubber_sap_log`)** que brillan y gotean resina dorada (`lit=true`).
* **`rubber_sap_log` (Tronco de Caucho con Hueco de Savia):** Bloque tipo roble oscuro con una cavidad frontal animada llena de savia dorada.
  * **Extracción manual:** Haz clic derecho (con la mano o con la **Espita `treetap`**) para extraer **3x Savia Pegajosa (`raw_sap`)** sin romper el árbol. El hueco se vacía (`lit=false`) y **regenera su savia automáticamente** pasados unos segundos.
  * **Extracción 100% automatizada:** Coloca un **Extractor Automático de Savia (`sap_extractor`)** junto a la base del árbol para recolectar savia infinitamente y enviarla por tuberías (`item_pipe`) o cintas (`conveyor_belt`).
* **De Savia a Goma (`raw_sap` -> `rubber_bar`):**
  * **En Horno / Alto Horno / Horno Eléctrico:** Quema o funde **1x Savia Pegajosa (`raw_sap`)** para obtener **Goma Vulcanizada (`rubber_bar`)** (1x en horno tradicional, **2x en el Horno Eléctrico**).
  * **En la Vulcanizadora Química (`vulcanizer`):** Procesa **1x Savia Pegajosa** para obtener **3x Barras de Goma (`rubber_bar`)**, prensa barras en **2x Láminas de Goma (`rubber_sheet`)**, recubre cobre en **4x Bobinas de Cable Aislado (`insulated_copper_cable`)** o destila cultivos en **Bio-Combustible (`bio_fuel`)**.

---

## 🧱 Bloques Incluidos (24 Bloques — `rusty_factory:*`)

| ID del Bloque | Nombre | Función Principal |
| :--- | :--- | :--- |
| `rubber_sap_log` | **Tronco de Caucho con Hueco de Savia** | Tronco especial con cavidad generadora de savia animada (`lit=true`). Da `3x raw_sap` al hacer clic derecho o con `sap_extractor` y se regenera solo. |
| `rubber_log` | **Tronco de Caucho** | Madera natural del Árbol de Caucho (se corta en 4 tablones o 6 en la Ensambladora). |
| `rubber_leaves` | **Hojas de Árbol de Caucho** | Follaje del Árbol de Caucho; suelta `rubber_sapling` y `raw_sap`. |
| `rubber_sapling` | **Brote de Árbol de Caucho** | Crece generando un Árbol de Caucho con porcentaje de huecos de savia (`rubber_sap_log`). |
| `rubber_planks` | **Tablones de Madera de Caucho** | Madera industrial tratada para construcción y artesanía. |
| `sap_extractor` | **Extractor Automático de Savia** | Máquina que detecta troncos y huecos de savia adyacentes, extrae `raw_sap` sin talar el árbol y la expulsa por tuberías. |
| `vulcanizer` | **Vulcanizadora Química de Goma** | Convierte `raw_sap -> 3x rubber_bar`, `rubber_bar -> 2x rubber_sheet`, `copper_ingot -> 4x insulated_copper_cable` y biomasa en `2x bio_fuel`. |
| `bio_generator` | **Bio-Generador de Savia y Biomasa** | Genera **+36 FE/t** quemando `raw_sap`, `bio_fuel`, brotes, semillas y cultivos. |
| `quarry` | **Excavadora Industrial (Quarry 3D)** | Construye un marco 3D estilo BuildCraft detrás de la máquina (`5x5`, `7x7` o `9x9`) con puente grúa móvil, láser de minería animado y holograma 3D. Excava hasta Bedrock y envía todo por tuberías o cofres. |
| `auto_harvester` | **Cosechadora y Taladora 3D** | Escanea un área configurable (`5x5`, `7x7` o `9x9`) con marco láser verde 3D. Cosecha cultivos maduros, caña, bambú y también recolecta `raw_sap` de `rubber_sap_log`. |
| `coal_generator` | **Generador Industrial a Carbón** | Quema carbón/combustible y genera **40+ FE/t** (hasta `20,000 FE`), distribuyendo energía por cables de cobre. |
| `solar_generator` | **Panel Solar Fotovoltaico (3D)** | Bloque de media altura (`0.375` de alto) con celdas solares animadas. Genera energía limpia durante el día. |
| `energy_bank` | **Banco de Capacitores (100k FE)** | Almacena hasta **100,000 FE**, muestra un holograma 3D flotante con su porcentaje de carga y distribuye hasta `400 FE/t`. |
| `copper_cable` | **Cable de Energía de Cobre** | Tubería eléctrica 6-direcciones (`PipeBlock`) animada que conecta generadores, paneles solares, bancos de energía y máquinas. |
| `item_pipe` | **Tubería Neumática de Ítems** | Tubería neumática 6-direcciones (`PipeBlock`) que se conecta a todas las máquinas, cintas y contenedores vanilla. |
| `item_extractor` | **Extractor Neumático de Ítems** | Extrae ítems automáticamente de cofres, barriles u hornos adyacentes y los inyecta en la red de `item_pipe` (con soporte de filtro en Slot 0). |
| `conveyor_belt` | **Cinta Transportadora 3D** | Bloque mediano (`0.5` de altura) con rieles laterales 3D, rodillos, textura animada (8 frames) y renderizado 3D del ítem deslizándose sobre la cinta. |
| `electric_crusher` | **Trituradora Eléctrica** | Duplica menas en **2x Polvos Minerales**, tritura carbón/netherrack en **Polvo de Azufre (`sulfur_dust`)** y cuarzo/arena en **Silicio (`silicon`)**. |
| `electric_furnace` | **Horno Eléctrico de Inducción** | Funde polvos, menas, **Savia en 2x Goma (`rubber_bar`)**, hierro en **Acero (`steel_ingot`)** y más usando `FE`. |
| `mechanical_press` | **Prensa Mecánica Hidráulica** | Prensa lingotes en placas, goma en **Láminas de Goma (`rubber_sheet`)** y acero en engranajes (`steel_gear`). |
| `industrial_assembler` | **Ensambladora Industrial Automática** | Corta troncos en **6 tablones**, procesa `rubber_sap_log` en **4x Savia (`raw_sap`)** y ensambla circuitos, cintas (con goma) y celdas solares. |
| `block_breaker` | **Perforadora Rompe-Bloques** | Rompe automáticamente el bloque situado frente a ella y expulsa los drops a tuberías o cofres. |
| `block_placer` | **Colocador Automático de Bloques** | Coloca automáticamente en el mundo los bloques que reciba en su ranura de entrada frente a su cara frontal. |
| `machine_casing` | **Chasis de Máquina Industrial** | Bloque estructural base para fabricar todas las máquinas. |

---

## 🔧 Ítems, Herramientas y Componentes (25 Ítems — `rusty_factory:*`)

* **`treetap` (Espita Extractora de Savia):** Herramienta de madera y cobre para extraer savia de los `rubber_sap_log`.
* **`raw_sap` (Savia Pegajosa de Caucho):** Resina dorada extraída del Árbol de Caucho. **Si la quemas en un horno o en la Vulcanizadora te da Goma (`rubber_bar`)**.
* **`rubber_bar` (Barra de Goma Vulcanizada):** Goma elástica obtenida al quemar `raw_sap`. Se usa para fabricar **Cintas Transportadoras**, **Cables Aislados**, **Botas Industriales** y **Láminas de Goma**.
* **`rubber_sheet` (Lámina de Goma Industrial):** Goma prensada de alta densidad para aislamiento y cintas industriales.
* **`insulated_copper_cable` (Bobina de Cable de Cobre Aislado):** Cobre recubierto con goma vulcanizada para ensamblar circuitos y redes eléctricas con mayor rendimiento.
* **`sulfur_dust` (Polvo de Azufre):** Obtenido al triturar carbón o netherrack; catalizador químico y componente para pólvora/vulcanización.
* **`bio_fuel` (Celda de Bio-Combustible):** Combustible orgánico de alta duración (`3200 ticks`) para el Bio-Generador.
* **`wrench`, `mining_drill`, `jet_boots`, `speed_upgrade`, `fortune_upgrade`, `energy_battery`, `electronic_circuit`, `solar_cell`, `silicon`, `diamond_drill_head`, `steel_ingot`, `steel_gear`, `iron_plate`, `copper_plate`, `gold_plate`, `iron_dust`, `copper_dust`, `gold_dust`**.

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
Dentro del juego puedes usar la pestaña creativa **"Rusty Factory (Industria)"** o ejecutar el comando `/rustyfactory kit` para recibir el kit completo con brotes de árbol de caucho, troncos con hueco de savia, espita, vulcanizadora, extractor de savia y todas las máquinas.
