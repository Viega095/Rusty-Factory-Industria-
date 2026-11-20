#![allow(dead_code)]
use crate::runtime::MachineState;
use mod_api::*;

const STATE_TAG: &str = "rusty_machine";

/// Returns the canonical block ID (e.g. `"rusty_factory:quarry"`) at `pos` in the active dimension,
/// or `"minecraft:air"` if unloaded.
pub fn block_id_at(host: &dyn Host, pos: [i32; 3]) -> String {
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return "minecraft:air".to_owned();
    };
    let Some(key) = level.block(host, pos) else {
        return "minecraft:air".to_owned();
    };
    key.id(host)
}

/// Returns a block state property at `pos` (e.g. `"facing"`, `"lit"`).
pub fn block_property_at(host: &dyn Host, pos: [i32; 3], key: &str) -> Option<String> {
    let dim = host.dimension();
    let level = host.level(dim)?;
    level.block_property(host, pos, key)
}

/// Sets the block at `pos` to the default state of `block_id` with flags `3` (UPDATE_ALL).
pub fn set_block_by_id(host: &mut dyn ServerHost, pos: [i32; 3], block_id: &str) -> bool {
    let Some(key) = host.block_key(block_id) else {
        return false;
    };
    let Some(state) = host.block_state_default(key) else {
        return false;
    };
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return false;
    };
    level.set_block(host, pos, state, BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS).unwrap_or(false)
}

/// Returns the canonical item ID string for `stack`.
pub fn item_id(host: &dyn Host, stack: &ItemStack) -> String {
    stack.item().id(host)
}

/// Builds an `ItemStack` from an item ID string and count.
pub fn make_stack(host: &dyn Host, id: &str, count: u32) -> Option<ItemStack> {
    let ident = Identifier::parse(id).ok()?;
    let key = host.item_key(&ident)?;
    ItemStack::new(key, count)
}

/// Reads the `minecraft:custom_data` compound from an `ItemStack`, if present.
pub fn get_custom_data(stack: &ItemStack) -> Option<&Compound> {
    let components = stack.components()?;
    match components.as_nbt().get(&"minecraft:custom_data".into())? {
        Tag::Compound(c) => Some(c),
        _ => None,
    }
}

/// Writes the `minecraft:custom_data` compound onto an `ItemStack`.
pub fn set_custom_data(stack: &mut ItemStack, data: Compound) {
    let mut patch = Compound::new();
    patch.insert("minecraft:custom_data".into(), Tag::Compound(data));
    let _ = stack.apply_components(&patch);
}

/// Inserts as much of `stack` as possible into the `ITEMS` capability at `pos` from `face`.
/// Returns the number of items inserted.
pub fn insert_item_at(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    face: Option<Direction>,
    stack: &ItemStack,
) -> u32 {
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return 0;
    };
    let Some(cap) = level.capability(host, &ITEMS, pos, face) else {
        return 0;
    };
    let resource = ItemResource::of(stack);
    let want = stack.count();
    host.transaction(|host, tx| {
        let moved = cap.insert_any(host, tx, &resource, want);
        if moved > 0 {
            Ok(moved)
        } else {
            Err(())
        }
    })
    .unwrap_or(0)
}

/// Extracts up to `max` items from the first non-empty slot of the `ITEMS` capability at `pos`.
pub fn extract_item_at(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    face: Option<Direction>,
    max: u32,
) -> Option<ItemStack> {
    let dim = host.dimension();
    let level = host.level(dim)?;
    let cap = level.capability(host, &ITEMS, pos, face)?;
    let slots = cap.size(host);
    host.transaction(|host, tx| {
        for slot in 0..slots {
            if let Some(resource) = cap.resource(host, slot) {
                let taken = cap.extract(host, tx, slot, &resource, max);
                if taken > 0 {
                    if let Some(stack) = resource.to_stack(taken) {
                        return Ok(stack);
                    }
                }
            }
        }
        Err(())
    })
    .ok()
}

/// Extracts up to `max` items from a specific `slot` of the `ITEMS` capability at `pos`.
pub fn extract_item_slot(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    face: Option<Direction>,
    slot: usize,
    max: u32,
) -> Option<ItemStack> {
    let dim = host.dimension();
    let level = host.level(dim)?;
    let cap = level.capability(host, &ITEMS, pos, face)?;
    host.transaction(|host, tx| {
        let resource = cap.resource(host, slot).ok_or(())?;
        let taken = cap.extract(host, tx, slot, &resource, max);
        if taken > 0 {
            resource.to_stack(taken).ok_or(())
        } else {
            Err(())
        }
    })
    .ok()
}

pub fn machine_state_to_tag(s: &MachineState) -> Tag {
    let mut c = Compound::new();
    c.insert("kind".into(), Tag::Byte(s.kind.to_u8() as i8));
    c.insert("energy".into(), Tag::Int(s.energy));
    c.insert("progress".into(), Tag::Int(s.progress));
    c.insert("burn_left".into(), Tag::Int(s.burn_left));
    c.insert("burn_max".into(), Tag::Int(s.burn_max));
    c.insert("enabled".into(), Tag::Byte(i8::from(s.enabled)));
    c.insert("auto_io".into(), Tag::Byte(i8::from(s.auto_io)));
    c.insert("q_rad".into(), Tag::Int(s.quarry_radius));
    c.insert("q_cur".into(), Tag::Int(s.quarry_cursor));
    c.insert("q_y".into(), Tag::Int(s.quarry_y_offset));
    c.insert("q_mined".into(), Tag::Int(s.quarry_mined));
    c.insert("last_t".into(), Tag::IntArray(s.last_target.to_vec()));
    c.insert("b_item".into(), Tag::String(s.belt_item.as_str().into()));
    c.insert("b_cnt".into(), Tag::Int(s.belt_count as i32));
    c.insert("p_from".into(), Tag::Byte(s.pipe_from as i8));
    c.insert("p_to".into(), Tag::Byte(s.pipe_to as i8));
    c.insert("p_tgt".into(), Tag::IntArray(s.pipe_target.to_vec()));
    c.insert("p_has".into(), Tag::Byte(i8::from(s.pipe_has_target)));
    c.insert("sf0".into(), Tag::String(s.splitter_filters[0].as_str().into()));
    c.insert("sf1".into(), Tag::String(s.splitter_filters[1].as_str().into()));
    c.insert("sf2".into(), Tag::String(s.splitter_filters[2].as_str().into()));
    c.insert("bf_att".into(), Tag::Byte(i8::from(s.belt_filter_attached)));
    c.insert("bf_blk".into(), Tag::Byte(i8::from(s.belt_filter_blacklist)));
    c.insert("s_cap".into(), Tag::Int(s.silo_capacity as i32));
    c.insert("f_kind".into(), Tag::String(s.fluid_kind.as_str().into()));
    c.insert("f_amt".into(), Tag::Int(s.fluid_amount));
    Tag::Compound(c)
}

pub fn machine_state_from_tag(tag: &Tag) -> Option<MachineState> {
    let Tag::Compound(c) = tag else {
        return None;
    };
    let get_i = |k: &str, def: i32| match c.get(&k.into()) {
        Some(Tag::Int(v)) => *v,
        _ => def,
    };
    let get_b = |k: &str, def: bool| match c.get(&k.into()) {
        Some(Tag::Byte(v)) => *v != 0,
        _ => def,
    };
    let get_u8 = |k: &str, def: u8| match c.get(&k.into()) {
        Some(Tag::Byte(v)) => *v as u8,
        _ => def,
    };
    let get_s = |k: &str| match c.get(&k.into()) {
        Some(Tag::String(s)) => s.to_string().unwrap_or_default(),
        _ => String::new(),
    };
    let get_pos = |k: &str| match c.get(&k.into()) {
        Some(Tag::IntArray(a)) if a.len() == 3 => [a[0], a[1], a[2]],
        _ => [0, 0, 0],
    };
    let kind = crate::runtime::MachineKind::from_u8(get_u8("kind", 255))?;
    Some(MachineState {
        kind,
        energy: get_i("energy", 0),
        progress: get_i("progress", 0),
        burn_left: get_i("burn_left", 0),
        burn_max: get_i("burn_max", 0),
        enabled: get_b("enabled", true),
        auto_io: get_b("auto_io", true),
        quarry_radius: get_i("q_rad", 2),
        quarry_cursor: get_i("q_cur", 0),
        quarry_y_offset: get_i("q_y", 0),
        quarry_mined: get_i("q_mined", 0),
        last_target: get_pos("last_t"),
        belt_item: get_s("b_item"),
        belt_count: get_i("b_cnt", 0).max(0) as u32,
        pipe_from: get_u8("p_from", 6),
        pipe_to: get_u8("p_to", 6),
        pipe_target: get_pos("p_tgt"),
        pipe_has_target: get_b("p_has", false),
        splitter_filters: [get_s("sf0"), get_s("sf1"), get_s("sf2")],
        belt_filter_attached: get_b("bf_att", false),
        belt_filter_blacklist: get_b("bf_blk", false),
        silo_capacity: get_i("s_cap", 0).max(0) as u32,
        fluid_kind: get_s("f_kind"),
        fluid_amount: get_i("f_amt", 0),
    })
}

/// Reads and mutates a `MachineState` stored in a loaded block entity at `pos`, persisting via `replace_data`.
pub fn with_machine_state<R>(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    f: impl FnOnce(&mut MachineState) -> R,
) -> Option<R> {
    let dim = host.dimension();
    let level = host.level(dim)?;
    let be = host.block_entity(level, pos)?;
    let mut nbt = be.data(host)?;
    let mut state = machine_state_from_tag(nbt.get(&STATE_TAG.into())?)?;
    let out = f(&mut state);
    nbt.insert(STATE_TAG.into(), machine_state_to_tag(&state));
    let _ = be.replace_data(host, nbt);
    Some(out)
}

/// Reads a `MachineState` from a loaded block entity at `pos` without mutating it.
pub fn read_machine_state(host: &dyn Host, pos: [i32; 3]) -> Option<MachineState> {
    let dim = host.dimension();
    let level = host.level(dim)?;
    let be = host.block_entity(level, pos)?;
    let nbt = be.data(host)?;
    machine_state_from_tag(nbt.get(&STATE_TAG.into())?)
}

/// Runs a server command line.
pub fn run_command(host: &mut dyn ServerHost, line: &str) -> bool {
    host.commands().run_command(line)
}

/// Sends an action-bar message to `player`.
pub fn action_bar(host: &mut dyn ServerHost, player: PlayerRef, msg: impl Into<String>) {
    let _ = player.action_bar(host, Text::literal(msg.into()));
}

/// Sends a chat feedback message to `player`.
pub fn tell(host: &mut dyn ServerHost, player: PlayerRef, msg: impl Into<String>) {
    let _ = player.tell(host, Text::literal(msg.into()));
}

/// Sends an action-bar message to the currently acting player, if any.
pub fn acting_action_bar(host: &mut dyn ServerHost, msg: impl Into<String>) {
    if let Some(p) = host.players().acting_player() {
        action_bar(host, p, msg);
    }
}

/// Sends a chat feedback message to the currently acting player, if any.
pub fn acting_feedback(host: &mut dyn ServerHost, msg: impl Into<String>) {
    if let Some(p) = host.players().acting_player() {
        tell(host, p, msg);
    }
}

/// Gives `count` of `item_id` to the acting player (or drops at their feet if full).
pub fn give_items(host: &mut dyn ServerHost, item_id: &str, count: u32) {
    let Some(player) = host.players().acting_player() else {
        return;
    };
    let Some(stack) = make_stack(host, item_id, count) else {
        return;
    };
    let _ = player.give_stack(host, stack);
}

/// Removes up to `count` of `item_id` from the acting player's inventory. Returns true if full count was taken.
pub fn take_items(host: &mut dyn ServerHost, item_id: &str, mut count: u32) -> bool {
    let Some(player) = host.players().acting_player() else {
        return false;
    };
    let Some(slots) = player.inventory_slots(host) else {
        return false;
    };
    let mut have = 0u32;
    for (_, st) in &slots {
        if st.item().id(host) == item_id {
            have += st.count();
        }
    }
    if have < count {
        return false;
    }
    for (slot_idx, st) in slots {
        if count == 0 {
            break;
        }
        if st.item().id(host) != item_id {
            continue;
        }
        let take = count.min(st.count());
        let rem = st.shrink(take);
        let _ = player.set_inventory_slot(host, slot_idx, rem);
        count -= take;
    }
    count == 0
}

/// Damages the acting player's main-hand item by `amount` durability.
pub fn damage_held(host: &mut dyn ServerHost, amount: u32) {
    let Some(player) = host.players().acting_player() else {
        return;
    };
    if player.is_creative(host).unwrap_or(false) {
        return;
    }
    let living = LivingRef::new(player.entity());
    if let Some(mut stack) = living.hand_item(host, Hand::Main) {
        let next_dmg = stack.damage().saturating_add(amount);
        stack.set_damage(next_dmg);
        let _ = living.set_hand_item(host, Hand::Main, Some(stack));
    }
}

/// Inserts an `ItemStack` directly into a block entity's `slot` if it is currently empty.
pub fn insert_mod_be_slot_if_empty(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    slot: u32,
    stack: ItemStack,
) -> bool {
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return false;
    };
    let Some(be) = host.block_entity(level, pos) else {
        return false;
    };
    if !matches!(be.item_slot(host, slot), Ok(None)) {
        return false;
    }
    host.transaction(|host, tx| {
        be.set_item_slot(host, tx, slot, Some(stack.clone()))
            .map_err(|_| ())
            .and_then(|ok| if ok { Ok(()) } else { Err(()) })
    })
    .is_ok()
}

/// Returns the number of item slots in a container at `pos`, or 0 if none.
pub fn container_slots(host: &mut dyn ServerHost, pos: [i32; 3]) -> usize {
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return 0;
    };
    let Some(cap) = level.capability(host, &ITEMS, pos, Some(Direction::Up)) else {
        return 0;
    };
    cap.size(host)
}

/// Returns `(block_light, sky_light)` at `pos` (0..15).
pub fn light_at(host: &dyn Host, pos: [i32; 3]) -> (i32, i32) {
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return (0, 15);
    };
    let block = level.brightness(host, LightLayer::Block, pos).unwrap_or(0);
    let sky = level.brightness(host, LightLayer::Sky, pos).unwrap_or(15);
    (block, sky)
}

/// Returns `(rain_level, thunder_level)` in the active dimension.
pub fn weather(host: &dyn Host) -> (f32, f32) {
    let dim = host.dimension();
    host.level(dim)
        .and_then(|l| l.weather(host))
        .unwrap_or((0.0, 0.0))
}

/// Returns the current day time in ticks.
pub fn day_time(host: &dyn Host) -> i64 {
    let dim = host.dimension();
    host.level(dim)
        .and_then(|l| l.day_time(host))
        .unwrap_or(0)
}

/// Returns the current game time in ticks.
pub fn game_time(host: &dyn Host) -> i64 {
    let dim = host.dimension();
    host.level(dim)
        .and_then(|l| l.game_time(host))
        .unwrap_or(0)
}

/// Drops an `ItemStack` at block `pos`.
pub fn pop_resource(host: &mut dyn ServerHost, pos: [i32; 3], stack: Option<ItemStack>) {
    if let Some(st) = stack {
        spawn_item(
            host,
            st,
            [pos[0] as f64 + 0.5, pos[1] as f64 + 0.5, pos[2] as f64 + 0.5],
        );
    }
}

/// Spawns an `ItemStack` at world coordinates `pos`.
pub fn spawn_item(host: &mut dyn ServerHost, stack: impl Into<Option<ItemStack>>, pos: [f64; 3]) {
    let Some(stack) = stack.into() else {
        return;
    };
    let id = stack.item().id(host);
    let cnt = stack.count();
    let _ = host.commands().run_command(&format!(
        "summon minecraft:item {:.2} {:.2} {:.2} {{Item:{{id:\"{}\",count:{}}}}}",
        pos[0], pos[1], pos[2], id, cnt
    ));
}

/// Moves up to `max` items (optionally matching `filter`) from `src_pos` (`src_face`) to `dst_pos` (`dst_face`).
pub fn move_items(
    host: &mut dyn ServerHost,
    src_pos: [i32; 3],
    src_face: Direction,
    dst_pos: [i32; 3],
    dst_face: Direction,
    filter: Option<&str>,
    max: u32,
) -> u32 {
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return 0;
    };
    let Some(src) = level.capability(host, &ITEMS, src_pos, Some(src_face)) else {
        return 0;
    };
    let Some(dst) = level.capability(host, &ITEMS, dst_pos, Some(dst_face)) else {
        return 0;
    };
    let slots = src.size(host);
    host.transaction(|host, tx| {
        for slot in 0..slots {
            if let Some(res) = src.resource(host, slot) {
                if let Some(f) = filter {
                    if !res.to_stack(1).is_some_and(|s| s.item().id(host) == f) {
                        continue;
                    }
                }
                let space = dst.insert_any(host, tx, &res, max);
                if space > 0 {
                    let took = src.extract(host, tx, slot, &res, space);
                    if took == space {
                        return Ok(took);
                    }
                }
            }
        }
        Err(())
    })
    .unwrap_or(0)
}

/// Inserts `stack` into the container at `pos` from `face`, returning any uninserted remainder.
pub fn insert_item_rem(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    face: Direction,
    stack: impl Into<Option<ItemStack>>,
) -> Option<ItemStack> {
    let Some(stack) = stack.into() else {
        return None;
    };
    let moved = insert_item_at(host, pos, Some(face), &stack);
    if moved == 0 {
        Some(stack)
    } else {
        stack.shrink(moved)
    }
}

/// Draws an item icon by item ID string on a `DrawContext`.
pub fn draw_item_id(host: &dyn Host, draw: &mut dyn DrawContext, item_id: &str, at: [f32; 2], size: f32) -> bool {
    let Ok(ident) = Identifier::parse(item_id) else {
        return false;
    };
    let Some(key) = host.item_key(&ident) else {
        return false;
    };
    draw.item(key, at, size)
}

pub trait HasNbt {
    fn nbt(&self) -> &Compound;
    fn nbt_mut(&mut self) -> &mut Compound;
}

impl HasNbt for ModBlockEntity {
    fn nbt(&self) -> &Compound {
        &self.nbt
    }
    fn nbt_mut(&mut self) -> &mut Compound {
        &mut self.nbt
    }
}

impl HasNbt for ModBlockEntityView<'_> {
    fn nbt(&self) -> &Compound {
        &self.nbt
    }
    fn nbt_mut(&mut self) -> &mut Compound {
        &mut self.nbt
    }
}

pub struct CompatBe {
    pub id: String,
    pub nbt: Compound,
    pub items: Vec<Option<ItemStack>>,
}

impl CompatBe {
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl HasNbt for CompatBe {
    fn nbt(&self) -> &Compound {
        &self.nbt
    }
    fn nbt_mut(&mut self) -> &mut Compound {
        &mut self.nbt
    }
}

/// Reads and mutates a mod block entity at `pos` via `BlockEntityRef`, persisting NBT and changed item slots.
pub fn with_mod_block_entity<R>(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    f: impl FnOnce(&mut CompatBe) -> R,
) -> Option<R> {
    let block_id = block_id_at(host, pos);
    let slot_count = crate::glue::container(&block_id).map_or(0, |c| c.slots);
    let dim = host.dimension();
    let level = host.level(dim)?;
    let be = host.block_entity(level, pos)?;
    let nbt = be.data(host)?;
    let mut items = Vec::with_capacity(slot_count);
    for s in 0..slot_count {
        items.push(be.item_slot(host, s as u32).ok().flatten());
    }
    let orig_items = items.clone();
    let orig_nbt = nbt.clone();
    let mut compat = CompatBe {
        id: block_id,
        nbt,
        items,
    };
    let out = f(&mut compat);
    if compat.nbt != orig_nbt {
        let _ = be.replace_data(host, compat.nbt);
    }
    if compat.items != orig_items {
        let new_items = compat.items;
        let _ = host.transaction(|host, tx| {
            for (s, st) in new_items.into_iter().enumerate() {
                if orig_items.get(s) != Some(&st) {
                    let _ = be.set_item_slot(host, tx, s as u32, st);
                }
            }
            Ok::<(), ()>(())
        });
    }
    Some(out)
}

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Client-side cache of visible mod block entities updated in `block_entity_submit` for `world_primitives`.
pub static CLIENT_VISIBLE_BE: LazyLock<Mutex<HashMap<[i32; 3], (String, MachineState)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Server-side cache of known active mod block entities (e.g. TeleporterPads, EnergyRelays).
pub static SERVER_ACTIVE_BE: LazyLock<Mutex<HashMap<[i32; 3], String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn record_client_be(pos: [i32; 3], id: &str, state: MachineState) {
    if let Ok(mut map) = CLIENT_VISIBLE_BE.lock() {
        map.insert(pos, (id.to_owned(), state));
    }
}

pub fn record_server_be(pos: [i32; 3], id: &str) {
    if let Ok(mut map) = SERVER_ACTIVE_BE.lock() {
        map.insert(pos, id.to_owned());
    }
}

pub fn remove_server_be(pos: [i32; 3]) {
    if let Ok(mut map) = SERVER_ACTIVE_BE.lock() {
        map.remove(&pos);
    }
}

pub fn is_crouching(host: &dyn Host, entity: EntityRef) -> bool {
    entity
        .pose(host)
        .is_some_and(|p| p.eq_ignore_ascii_case("crouching"))
}

pub fn acting_creative(host: &dyn ServerHost) -> bool {
    host.players()
        .acting_player()
        .and_then(|p| p.is_creative(host))
        .unwrap_or(false)
}

pub fn acting_held(host: &dyn ServerHost) -> Option<ItemStack> {
    let p = host.players().acting_player()?;
    LivingRef::new(p.entity()).hand_item(host, Hand::Main)
}

pub fn acting_held_mut(host: &mut dyn ServerHost, f: impl FnOnce(&mut Option<ItemStack>)) {
    let Some(p) = host.players().acting_player() else {
        return;
    };
    let living = LivingRef::new(p.entity());
    let mut stack = living.hand_item(host, Hand::Main);
    f(&mut stack);
    let _ = living.set_hand_item(host, Hand::Main, stack);
}

pub fn acting_consume_held(host: &mut dyn ServerHost, amount: u32) {
    if acting_creative(host) {
        return;
    }
    acting_held_mut(host, |slot| {
        if let Some(st) = slot.take() {
            *slot = st.shrink(amount);
        }
    });
}

pub fn acting_offhand(host: &dyn ServerHost) -> Option<ItemStack> {
    let p = host.players().acting_player()?;
    LivingRef::new(p.entity()).hand_item(host, Hand::Off)
}

pub fn set_acting_offhand(host: &mut dyn ServerHost, stack: Option<ItemStack>) {
    let Some(p) = host.players().acting_player() else {
        return;
    };
    let _ = LivingRef::new(p.entity()).set_hand_item(host, Hand::Off, stack);
}

pub fn acting_yaw_pitch(host: &dyn ServerHost) -> (f32, f32) {
    let Some(p) = host.players().acting_player() else {
        return (0.0, 0.0);
    };
    let rot = p.entity().rotation(host).unwrap_or([0.0, 0.0]);
    (rot[0], rot[1])
}

pub fn local_held(host: &dyn ClientHost) -> Option<ItemStack> {
    let p = host.local_player()?;
    LivingRef::new(p.entity()).hand_item(host, Hand::Main)
}

pub fn local_armor_slot(host: &dyn ClientHost, idx: usize) -> Option<ItemStack> {
    let p = host.local_player()?;
    let slot = match idx {
        0 => EquipmentSlot::Head,
        1 => EquipmentSlot::Chest,
        2 => EquipmentSlot::Legs,
        3 => EquipmentSlot::Feet,
        _ => return None,
    };
    LivingRef::new(p.entity()).equipment(host, slot)
}

pub fn local_inventory(host: &dyn ClientHost) -> Vec<ItemStack> {
    let Some(p) = host.local_player() else {
        return Vec::new();
    };
    p.inventory(host).unwrap_or_default()
}

pub fn local_yaw_pitch(host: &dyn ClientHost) -> (f32, f32) {
    let Some(p) = host.local_player() else {
        return (0.0, 0.0);
    };
    let rot = p.entity().rotation(host).unwrap_or([0.0, 0.0]);
    (rot[0], rot[1])
}

/// Reads all item stacks in a container at `pos` via `ITEMS` capability.
pub fn read_container_items(host: &mut dyn ServerHost, pos: [i32; 3]) -> Vec<Option<ItemStack>> {
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return Vec::new();
    };
    let Some(cap) = level.capability(host, &ITEMS, pos, Some(Direction::Up)) else {
        return Vec::new();
    };
    let n = cap.size(host);
    let mut out = Vec::with_capacity(n);
    for s in 0..n {
        let Some(res) = cap.resource(host, s) else {
            out.push(None);
            continue;
        };
        let amt = cap.amount(host, s) as u32;
        out.push(if amt > 0 { res.to_stack(amt) } else { None });
    }
    out
}

/// Extracts 1 item matching `item_id` from a container at `pos`.
pub fn take_one_matching_from_container(host: &mut dyn ServerHost, pos: [i32; 3], item_id: &str) -> bool {
    let dim = host.dimension();
    let Some(level) = host.level(dim) else {
        return false;
    };
    let Some(cap) = level.capability(host, &ITEMS, pos, Some(Direction::Up)) else {
        return false;
    };
    let n = cap.size(host);
    host.transaction(|host, tx| {
        for s in 0..n {
            if let Some(res) = cap.resource(host, s) {
                if let Some(st) = res.to_stack(1) {
                    if st.item().id(host) == item_id {
                        if cap.extract(host, tx, s, &res, 1) == 1 {
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(())
    })
    .is_ok()
}

pub struct WorldPrimitivesCompat {
    pub lines: Vec<WorldLine>,
    pub quads: Vec<WorldQuad>,
}

impl WorldPrimitivesCompat {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            quads: Vec::new(),
        }
    }

    pub fn line(&mut self, from: [f64; 3], to: [f64; 3], color: u32, width: f32) {
        self.lines.push(WorldLine::new(
            from,
            to,
            color,
            width.max(1.0),
            WorldDepth::Tested,
        ));
    }

    pub fn box_outline(&mut self, min: [f64; 3], max: [f64; 3], color: u32, width: f32) {
        let [x0, y0, z0] = min;
        let [x1, y1, z1] = max;
        let edges = [
            ([x0, y0, z0], [x1, y0, z0]),
            ([x1, y0, z0], [x1, y0, z1]),
            ([x1, y0, z1], [x0, y0, z1]),
            ([x0, y0, z1], [x0, y0, z0]),
            ([x0, y1, z0], [x1, y1, z0]),
            ([x1, y1, z0], [x1, y1, z1]),
            ([x1, y1, z1], [x0, y1, z1]),
            ([x0, y1, z1], [x0, y1, z0]),
            ([x0, y0, z0], [x0, y1, z0]),
            ([x1, y0, z0], [x1, y1, z0]),
            ([x1, y0, z1], [x1, y1, z1]),
            ([x0, y0, z1], [x0, y1, z1]),
        ];
        for (a, b) in edges {
            self.line(a, b, color, width);
        }
    }

    pub fn filled_box(&mut self, min: [f64; 3], max: [f64; 3], color: u32) {
        let [x0, y0, z0] = min;
        let [x1, y1, z1] = max;
        let faces = [
            [[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], // down
            [[x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]], // up
            [[x0, y0, z0], [x0, y1, z0], [x1, y1, z0], [x1, y0, z0]], // north
            [[x1, y0, z1], [x1, y1, z1], [x0, y1, z1], [x0, y0, z1]], // south
            [[x0, y0, z1], [x0, y1, z1], [x0, y1, z0], [x0, y0, z0]], // west
            [[x1, y0, z0], [x1, y1, z0], [x1, y1, z1], [x1, y0, z1]], // east
        ];
        for corners in faces {
            self.quads.push(WorldQuad::new(
                corners,
                color,
                WorldDepth::Tested,
            ));
        }
    }

    pub fn flush(&self, draw: &mut dyn WorldDrawContext) {
        for chunk in self.quads.chunks(256) {
            if draw.quads(chunk).is_err() {
                break;
            }
        }
        for chunk in self.lines.chunks(256) {
            if draw.lines(chunk).is_err() {
                break;
            }
        }
    }
}

/// Extracts a block state property value from a `BlockState` via `block_state_string`.
pub fn block_state_property(host: &dyn Host, state: BlockState, key: &str) -> Option<String> {
    let s = host.block_state_string(state)?;
    let props = s.split('[').nth(1)?.strip_suffix(']')?;
    for pair in props.split(',') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(v.to_owned());
            }
        }
    }
    None
}

/// Draws an `ItemStack` on a `BlockEntityDrawContext`.
pub fn draw_be_item_stack(
    draw: &mut dyn BlockEntityDrawContext,
    stack: &ItemStack,
    display: ItemDisplayContext,
    owner_yaw: Option<f32>,
    seed: i32,
) -> bool {
    draw.item(stack, display, owner_yaw, seed)
}

/// Returns a localized fallback string for a translation key.
pub fn lang_get(key: &str) -> String {
    match key {
        "rusty_factory.screen.start" => "Encender".to_owned(),
        "rusty_factory.screen.stop" => "Pausar".to_owned(),
        "rusty_factory.screen.auto_on" => "Auto: ON".to_owned(),
        "rusty_factory.screen.auto_off" => "Auto: OFF".to_owned(),
        "rusty_factory.status.0" => "Inactivo".to_owned(),
        "rusty_factory.status.1" => "Trabajando".to_owned(),
        "rusty_factory.status.2" => "Sin Energía".to_owned(),
        "rusty_factory.status.3" => "Pausado".to_owned(),
        "rusty_factory.status.4" => "Salida Llena".to_owned(),
        _ => key
            .rsplit('.')
            .next()
            .unwrap_or(key)
            .replace('_', " "),
    }
}
