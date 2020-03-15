mod ui;

use crate::{
    i18n::{i18n_asset_key, VoxygenLocalization},
    scene::simple::{self as scene, Scene},
    session::SessionState,
    window::Event as WinEvent,
    Direction, GlobalState, PlayState, PlayStateResult,
};
use client::{self, Client};
use common::{assets, clock::Clock, comp, msg::ClientState, state::DeltaTime};
use log::error;
use specs::WorldExt;
use std::{cell::RefCell, rc::Rc, time::Duration};
use ui::CharSelectionUi;

pub struct CharSelectionState {
    char_selection_ui: CharSelectionUi,
    client: Rc<RefCell<Client>>,
    scene: Scene,
}

impl CharSelectionState {
    /// Create a new `CharSelectionState`.
    pub fn new(global_state: &mut GlobalState, client: Rc<RefCell<Client>>) -> Self {
        Self {
            char_selection_ui: CharSelectionUi::new(global_state),
            client,
            scene: Scene::new(
                global_state.window.renderer_mut(),
                Some("fixture.selection_bg"),
            ),
        }
    }
}

impl PlayState for CharSelectionState {
    fn play(&mut self, _: Direction, global_state: &mut GlobalState) -> PlayStateResult {
        // Set up an fps clock.
        let mut clock = Clock::start();

        let mut current_client_state = self.client.borrow().get_client_state();
        while let ClientState::Pending | ClientState::Registered = current_client_state {
            // Handle window events
            for event in global_state.window.fetch_events(&mut global_state.settings) {
                if self.char_selection_ui.handle_event(event.clone()) {
                    continue;
                }
                match event {
                    WinEvent::Close => {
                        return PlayStateResult::Shutdown;
                    },
                    // Pass all other events to the scene
                    event => {
                        self.scene.handle_input_event(event);
                    }, // TODO: Do something if the event wasn't handled?
                }
            }

            global_state.window.renderer_mut().clear();

            // Maintain the UI.
            let events = self
                .char_selection_ui
                .maintain(global_state, &self.client.borrow());
            for event in events {
                match event {
                    ui::Event::Logout => {
                        return PlayStateResult::Pop;
                    },
                    ui::Event::Play => {
                        let char_data = self
                            .char_selection_ui
                            .get_character_data()
                            .expect("Character data is required to play");
                        self.client.borrow_mut().request_character(
                            char_data.name,
                            char_data.body,
                            char_data.tool,
                        );
                        return PlayStateResult::Switch(Box::new(SessionState::new(
                            global_state,
                            self.client.clone(),
                        )));
                    },
                }
            }

            // Maintain global state.
            global_state.maintain(clock.get_last_delta().as_secs_f32());

            let humanoid_body = self
                .char_selection_ui
                .get_character_data()
                .and_then(|data| match data.body {
                    comp::Body::Humanoid(body) => Some(body),
                    _ => None,
                });

            // Maintain the scene.
            {
                let client = self.client.borrow();
                let scene_data = scene::SceneData {
                    time: client.state().get_time(),
                    delta_time: client.state().ecs().read_resource::<DeltaTime>().0,
                    tick: client.get_tick(),
                    body: humanoid_body.clone(),
                    gamma: global_state.settings.graphics.gamma,
                };
                self.scene
                    .maintain(global_state.window.renderer_mut(), scene_data);
            }

            // Render the scene.
            let item = self
                .char_selection_ui
                .get_character_data()
                .and_then(|data| data.tool)
                .and_then(|tool| assets::load_cloned::<comp::Item>(&tool).ok());
            let item_kind = item.map(|i| i.kind);

            self.scene.render(
                global_state.window.renderer_mut(),
                self.client.borrow().get_tick(),
                humanoid_body.clone(),
                item_kind.as_ref(),
            );

            // Draw the UI to the screen.
            self.char_selection_ui
                .render(global_state.window.renderer_mut(), self.scene.globals());

            // Tick the client (currently only to keep the connection alive).
            let localized_strings = assets::load_expect::<VoxygenLocalization>(&i18n_asset_key(
                &global_state.settings.language.selected_language,
            ));
            if let Err(err) = self.client.borrow_mut().tick(
                comp::ControllerInputs::default(),
                clock.get_last_delta(),
                |_| {},
            ) {
                global_state.info_message =
                    Some(localized_strings.get("common.connection_lost").to_owned());
                error!("[char_selection] Failed to tick the scene: {:?}", err);

                return PlayStateResult::Pop;
            }

            self.client.borrow_mut().cleanup();

            // Finish the frame.
            global_state.window.renderer_mut().flush();
            global_state
                .window
                .swap_buffers()
                .expect("Failed to swap window buffers");

            // Wait for the next tick.
            clock.tick(Duration::from_millis(
                1000 / (global_state.settings.graphics.max_fps as u64),
            ));

            current_client_state = self.client.borrow().get_client_state();
        }

        PlayStateResult::Pop
    }

    fn name(&self) -> &'static str { "Title" }
}
