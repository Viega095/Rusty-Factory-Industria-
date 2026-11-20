pub mod runtime;
mod host_compat;
mod glue;
mod guide_book;

use mod_api::*;

#[derive(Default)]
pub struct RustyFactory;

static STARTER_GUIDE_GIVEN: AttachmentType<bool> = AttachmentType {
    namespace: "rusty_factory",
    name: "starter_guide_given",
    default: || false,
    codec: Some(AttachmentCodec {
        encode: |v| Ok([("given".into(), Tag::Byte(i8::from(*v)))].into()),
        decode: |c| match c.get(&"given".into()) {
            Some(Tag::Byte(b)) => Ok(*b != 0),
            _ => Err(AttachmentError::Decode),
        },
    }),
    copy_on_death: true,
    sync: false,
};

pub struct RustyMachinePart;

#[mod_api::horizon_part]
impl BlockPart for RustyMachinePart {
    fn use_without_item(
        &self,
        host: &mut dyn ServerHost,
        event: &mut UseWithoutItemEvent<'_>,
    ) -> Option<InteractionResult> {
        let click = event.click;
        let dim = host.dimension();
        let level = host.level(dim)?;
        let block_id = level.block(host, click.pos)?.id(host);
        if glue::container(&block_id).is_some() {
            let player = host.player(click.player)?;
            let be = host.block_entity(level, click.pos)?;
            if player.open_menu(host, be).unwrap_or(false) {
                return Some(InteractionResult::Success);
            }
        }
        None
    }
}

#[mod_api::horizon_part]
impl BlockEntityPart for RustyMachinePart {
    fn block_entity(&self, _: LevelSide, event: &mut BlockEntityEvent<'_>) -> Option<Compound> {
        glue::block_entity(event.block)
    }

    fn block_entity_ticks(&self, _: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        if glue::block_entity_ticks(event.block) {
            event.ticks = true;
        }
    }

    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        glue::tick(host, event.pos, &mut event.entity);
    }

    fn block_entity_update_tag(&self, event: &mut BlockEntityUpdateTagEvent<'_>) -> Option<Compound> {
        glue::update_tag(event.pos, event.entity)
    }

    fn block_entity_submit(&self, host: &dyn ClientHost, event: &mut BlockEntitySubmitEvent<'_>) {
        glue::submit(host, event.pos, event.block, event.state, event.entity, event.draw);
    }

    fn screen(&self, _: &dyn Host, event: &mut ScreenEvent<'_>) -> Option<Screen> {
        glue::screen(event.block)
    }

    fn screen_draw(&self, host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        glue::screen_draw(host, event.block, event.data, event.draw, event.origin);
    }

    fn screen_click(&self, _: &dyn ClientHost, event: &mut ScreenClickEvent<'_>) -> Option<i32> {
        glue::screen_click(event.block, event.at, event.right)
    }

    fn screen_tooltip(&self, _: &dyn ClientHost, event: &mut ScreenTooltipEvent<'_>) {
        glue::screen_tooltip(event.block, event.data, event.at, event.lines);
    }

    fn menu_button(&self, host: &mut dyn ServerHost, event: &mut MenuButtonEvent<'_>) {
        glue::menu_button_v2(
            host,
            event.context.pos,
            &mut event.entity,
            event.context.id,
            event.carried,
        );
    }
}

#[mod_api::horizon_mod]
impl Hooks for RustyFactory {
    fn register(&self, registrar: &mut dyn Registrar, _: &RegistrationContext) {
        let machine_blocks = glue::all_block_entity_blocks();
        for &block_id in machine_blocks {
            if let Some(container) = glue::container(block_id) {
                let key = registrar.block_key(block_id);
                registrar.block_container(&key, container);
            }
        }
        let machine_be = registrar
            .block_entity_type("machine")
            .blocks(machine_blocks)
            .build();
        registrar
            .block_entity_part(&machine_be, RustyMachinePart)
            .expect("rusty_factory block entity type");
    }

    fn register_commands(&self, _: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        glue::commands(event.commands);
    }

    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        glue::perform(host, event.command, event.source)
    }

    fn entity_place(
        &self,
        host: &mut dyn ServerHost,
        event: &mut EntityPlaceEvent,
    ) -> Flow {
        glue::place_block(host, [event.pos.x, event.pos.y, event.pos.z]);
        Flow::Continue
    }

    fn break_block(&self, host: &mut dyn ServerHost, event: &mut BreakBlockEvent<'_>) -> Flow {
        glue::destroy_block(host, event.pos);
        Flow::Continue
    }

    fn right_click_block(
        &self,
        host: HostSide<'_>,
        event: &mut RightClickBlockEvent<'_>,
    ) -> Flow {
        match host {
            HostSide::Server(server) => {
                if glue::right_click(server, event.click) {
                    *event.cancellation_result = InteractionResult::Success;
                    return Flow::Cancel;
                }
                Flow::Continue
            }
            HostSide::Client(_) => Flow::Continue,
        }
    }

    fn right_click_item(
        &self,
        host: HostSide<'_>,
        event: &mut RightClickItemEvent<'_>,
    ) -> Flow {
        match host {
            HostSide::Server(server) => {
                if event.click.stack.item().id(server) == "rusty_factory:guide_book" {
                    *event.cancellation_result = InteractionResult::Success;
                    return Flow::Cancel;
                }
                if glue::use_item(server, Some(event.click.stack), event.click.player) {
                    *event.cancellation_result = InteractionResult::Success;
                    return Flow::Cancel;
                }
                Flow::Continue
            }
            HostSide::Client(client) => {
                if event.click.stack.item().id(client) == "rusty_factory:guide_book" {
                    guide_book::open_book(client);
                    *event.cancellation_result = InteractionResult::Success;
                    return Flow::Cancel;
                }
                Flow::Continue
            }
        }
    }

    fn input_key(&self, host: &mut dyn ClientHost, event: &mut InputKeyEvent<'_>) {
        let pressed = event.action == KeyAction::Press;
        if matches!(event.action, KeyAction::Press | KeyAction::Release) {
            glue::key_event(host, event.key, pressed);
        }
    }

    fn item_tooltip(&self, host: &dyn ClientHost, event: &mut ItemTooltipEvent<'_>) {
        glue::tooltip(host, event.stack, event.lines);
    }

    fn render_gui_post(&self, host: &dyn ClientHost, event: &mut RenderGuiPostEvent<'_>) {
        glue::hud(host, event.draw, event.corners);
    }

    fn render_level_stage(&self, host: &dyn ClientHost, event: &mut RenderLevelStageEvent<'_>) {
        if event.stage == RenderLevelStage::AfterLevel {
            glue::world_primitives(host, event.view, event.draw);
        }
    }

    fn player_logged_in(&self, host: &mut dyn ServerHost, event: &mut PlayerLoggedInEvent<'_>) {
        let Some(player) = host.player(event.player) else {
            return;
        };
        let holder = Holder::Entity(player.entity());
        if !STARTER_GUIDE_GIVEN.get(host, holder).unwrap_or(false) {
            if let Some(stack) = host_compat::make_stack(host, "rusty_factory:guide_book", 1) {
                let _ = player.give_or_drop(host, stack);
                let _ = STARTER_GUIDE_GIVEN.set(host, holder, true);
            }
        }
    }

    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        guide_book::tick_close(host);
    }

    fn screen_render_post(&self, host: &dyn ClientHost, event: &mut ScreenRenderPostEvent<'_>) {
        guide_book::render(host, event.screen, event.draw);
    }

    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseButtonPressedPreEvent<'_>,
    ) -> Flow {
        if guide_book::mouse_pressed(host, event.screen, event.button) {
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn screen_mouse_button_released_pre(
        &self,
        host: &dyn ClientHost,
        _event: &mut ScreenMouseButtonReleasedPreEvent<'_>,
    ) -> Flow {
        if guide_book::mouse_released(host) {
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn screen_mouse_dragged_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseDraggedPreEvent<'_>,
    ) -> Flow {
        if guide_book::mouse_dragged(host, event.delta) {
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn screen_mouse_scrolled_pre(
        &self,
        host: &dyn ClientHost,
        event: &mut ScreenMouseScrolledPreEvent<'_>,
    ) -> Flow {
        if guide_book::mouse_scrolled(host, event.screen, event.scroll[1]) {
            return Flow::Cancel;
        }
        Flow::Continue
    }
}
