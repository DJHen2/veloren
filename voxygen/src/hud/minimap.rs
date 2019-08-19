use conrod_core::{
    color,
    widget::{self, Button, Image, Rectangle, Text},
    widget_ids, Color, Colorable, Positionable, Sizeable, Widget, WidgetCommon,
};

use std::time::{Duration, Instant};

use super::{img_ids::Imgs, Fonts, Show, HP_COLOR, TEXT_COLOR};
use client::{self, Client};

widget_ids! {
    struct Ids {
        mmap_frame,
        mmap_frame_bg,
        mmap_location,
        mmap_button,
        zone_display,
    }
}

#[derive(WidgetCommon)]
pub struct MiniMap<'a> {
    show: &'a Show,

    client: &'a Client,

    imgs: &'a Imgs,
    _fonts: &'a Fonts,
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
}

impl<'a> MiniMap<'a> {
    pub fn new(show: &'a Show, client: &'a Client, imgs: &'a Imgs, fonts: &'a Fonts) -> Self {
        Self {
            show,
            client,
            imgs,
            _fonts: fonts,
            common: widget::CommonBuilder::default(),
        }
    }
}

pub struct State {
    ids: Ids,

    last_region_name: Option<String>,
    last_update: Instant,
}

pub enum Event {
    Toggle,
}

impl<'a> Widget for MiniMap<'a> {
    type State = State;
    type Style = ();
    type Event = Option<Event>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),

            last_region_name: None,
            last_update: Instant::now(),
        }
    }

    fn style(&self) -> Self::Style {
        ()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, ui, .. } = args;

        if self.show.mini_map {
            Image::new(self.imgs.mmap_frame)
                .w_h(100.0 * 2.0, 100.0 * 2.0)
                .top_right_with_margins_on(ui.window, 5.0, 5.0)
                .set(state.ids.mmap_frame, ui);

            Rectangle::fill_with([92.0 * 2.0, 82.0 * 2.0], color::TRANSPARENT)
                .mid_top_with_margin_on(state.ids.mmap_frame, 13.0 * 2.0 + 2.0)
                .set(state.ids.mmap_frame_bg, ui);
        } else {
            Image::new(self.imgs.mmap_frame_closed)
                .w_h(100.0 * 2.0, 11.0 * 2.0)
                .top_right_with_margins_on(ui.window, 5.0, 5.0)
                .set(state.ids.mmap_frame, ui);
        }

        if Button::image(if self.show.mini_map {
            self.imgs.mmap_open
        } else {
            self.imgs.mmap_closed
        })
        .w_h(100.0 * 0.2, 100.0 * 0.2)
        .hover_image(if self.show.mini_map {
            self.imgs.mmap_open_hover
        } else {
            self.imgs.mmap_closed_hover
        })
        .press_image(if self.show.mini_map {
            self.imgs.mmap_open_press
        } else {
            self.imgs.mmap_closed_press
        })
        .top_right_with_margins_on(state.ids.mmap_frame, 0.0, 0.0)
        .set(state.ids.mmap_button, ui)
        .was_clicked()
        {
            return Some(Event::Toggle);
        }

        // Display zone name on entry

        const FADE_IN: f32 = 0.5;
        const FADE_HOLD: f32 = 1.0;
        const FADE_OUT: f32 = 3.0;

        match self.client.current_chunk() {
            Some(chunk) => {
                let current = chunk.meta().name();
                // Check if no other popup is displayed and a new one is needed
                if state.last_update.elapsed()
                    > Duration::from_secs_f32(FADE_IN + FADE_HOLD + FADE_OUT)
                    && state
                        .last_region_name
                        .as_ref()
                        .map(|l| l != current)
                        .unwrap_or(true)
                {
                    // Update last_region
                    state.update(|s| s.last_region_name = Some(current.to_owned()));
                    state.update(|s| s.last_update = Instant::now());
                }

                let seconds = state.last_update.elapsed().as_secs_f32();
                let fade = if seconds < FADE_IN {
                    seconds / FADE_IN
                } else if seconds < FADE_IN + FADE_HOLD {
                    1.0
                } else {
                    (1.0 - (seconds - FADE_IN - FADE_HOLD) / FADE_OUT).max(0.0)
                };
                // Region Name
                Text::new(state.last_region_name.as_ref().unwrap_or(&"".to_owned()))
                    .mid_top_with_margin_on(ui.window, 200.0)
                    .font_size(80)
                    .color(Color::Rgba(1.0, 1.0, 1.0, fade))
                    .set(state.ids.zone_display, ui);
            }
            None => Text::new(" ")
                .middle_of(ui.window)
                .font_size(14)
                .color(HP_COLOR)
                .set(state.ids.zone_display, ui),
        }

        // TODO: Subregion name display

        // Title
        match self.client.current_chunk() {
            Some(chunk) => Text::new(chunk.meta().name())
                .mid_top_with_margin_on(state.ids.mmap_frame, 3.0)
                .font_size(14)
                .color(TEXT_COLOR)
                .set(state.ids.mmap_location, ui),
            None => Text::new(" ")
                .mid_top_with_margin_on(state.ids.mmap_frame, 3.0)
                .font_size(14)
                .color(TEXT_COLOR)
                .set(state.ids.mmap_location, ui),
        }

        None
    }
}
