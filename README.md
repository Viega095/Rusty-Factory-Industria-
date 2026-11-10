# ⚙️ Rusty Factory — Mod de Automatización Industrial para Horizon (Rust)

Un mod completo de máquinas, energía eléctrica y logística industrial diseñado nativamente para **Horizon (Minecraft: Rust Edition 26.3)**.

Incluye tanto la parte de **Contenido (`.hmod`)** con bloques, ítems, modelos 3D, pestaña creativa, recetas y texturas pixel-art propias, como el **Módulo Nativo en Rust (`.hnmod`)** con simulación de redes eléctricas por grafos (BFS), procesamiento en cada *tick* e interfaz gráfica (GUI).

---

## 🧱 Bloques Incluidos (`rusty_factory:*`)

| ID del Bloque | Nombre | Función |
| :--- | :--- | :--- |
| `machine_casing` | **Chasis de Máquina Industrial** | Bloque estructural base para fabricar todas las máquinas. |
| `coal_generator` | **Generador Industrial a Carbón** | Quema carbón/combustible y genera **40 FE/t** (hasta `50,000 FE`), distribuyendo energía por cables conectados. |
| `copper_cable` | **Cable de Energía de Cobre** | Bloque tipo tubería (`PipeBlock`) que se conecta automáticamente en las 6 direcciones con máquinas y otros cables. |
| `electric_crusher` | **Trituradora Eléctrica** | Usa **20 FE/t** para duplicar menas en **2x Polvos Minerales** (`iron_dust`, `copper_dust`, `gold_dust`), diamantes, carbón o grava/arena. |
| `electric_furnace` | **Horno Eléctrico de Inducción** | Usa **15 FE/t** para fundir polvos, menas, acero y comida al doble de velocidad sin usar carbón. |
| `mechanical_press` | **Prensa Mecánica Hidráulica** | Usa **25 FE/t** para prensar lingotes en placas (`iron_plate`, `copper_plate`, `gold_plate`) y `steel_ingot` en `steel_gear`. |
| `conveyor_belt` | **Cinta Transportadora Industrial** | Mueve ítems automáticamente hacia el contenedor o máquina que tiene enfrente. |

---

## 🔧 Ítems y Componentes (`rusty_factory:*`)

* **`wrench` (Llave Inglesa Industrial):** Haz clic derecho sobre cualquier máquina o cable de cobre para inspeccionar su energía almacenada (`FE`), estado de la red y progreso.
* **`speed_upgrade` (Mejora de Velocidad Overclock):** Se coloca en la ranura de mejora de cualquier máquina para acelerar su velocidad de trabajo.
* **`energy_battery` (Celda de Batería Industrial):** Batería portátil que puede colocarse en la ranura de energía de una máquina para cargarla instantáneamente (`+2000 FE`).
* **`electronic_circuit` (Circuito de Control Electrónico):** Componente avanzado para máquinas eléctricas.
* **`steel_ingot` y `steel_gear`:** Lingote y engranaje de acero templado.
* **`iron_plate`, `copper_plate`, `gold_plate`:** Placas metálicas prensadas.
* **`iron_dust`, `copper_dust`, `gold_dust`:** Polvos minerales obtenidos en la trituradora (se funden en lingotes).

---

## 🎨 Cómo reemplazar mis texturas y modelos por los tuyos

Todas las texturas generadas son archivos PNG estándar de `16x16` píxeles (y `176x166` para la GUI). Puedes editarlas con **Blockbench**, **Aseprite** o cualquier editor de imágenes en:

* **Texturas de Bloques (`16x16`):** `content/assets/rusty_factory/textures/block/`
  * `machine_casing.png`, `machine_side.png`, `machine_top.png`
  * `coal_generator_front.png`, `electric_crusher_front.png`, `electric_furnace_front.png`, `mechanical_press_front.png`
  * `conveyor_belt_top.png`, `conveyor_belt_front.png`, `copper_cable.png`
* **Texturas de Ítems (`16x16`):** `content/assets/rusty_factory/textures/item/`
  * `wrench.png`, `speed_upgrade.png`, `energy_battery.png`, `electronic_circuit.png`
  * `steel_ingot.png`, `steel_gear.png`, `iron_plate.png`, `copper_plate.png`, `gold_plate.png`
  * `iron_dust.png`, `copper_dust.png`, `gold_dust.png`
* **Textura de Interfaz GUI (`176x166`):** `content/assets/rusty_factory/textures/gui/machine.png`

Una vez que reemplaces tus texturas o edites recetas/modelos, ejecuta:

```powershell
python tools/package.py
```

Esto actualizará automáticamente los paquetes listos para instalar en la carpeta `dist/`:
* `dist/rusty_factory-0.1.0.hmod`
* `dist/rusty_factory_native-0.1.0.hnmod`

---

## 🚀 Cómo probarlo en Horizon

1. **Para probar los bloques, ítems, recetas y modelos al instante (Content Mod):**
   * Copia `dist/rusty_factory-0.1.0.hmod` (o la carpeta `content/`) dentro de la carpeta `horizon-mods/` de tu instancia de Horizon.
   * Encontrarás todos los bloques e ítems en la pestaña creativa **"Rusty Factory (Industria)"**.
2. **Para compilar con la lógica nativa en Rust (`.hnmod`):**
   * Importa `dist/rusty_factory_native-0.1.0.hnmod` en la pestaña **Mods** de **Horizon Launcher** y presiona **Build**.
   * Dentro del juego también puedes usar el comando de operador `/rustyfactory kit` para recibir un kit completo de máquinas y cables.
