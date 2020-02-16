use super::{img_ids::Imgs, Show, TEXT_COLOR};
use crate::{i18n::VoxygenLocalization, ui::fonts::ConrodVoxygenFonts};
use client::{self, Client};
use conrod_core::{
    color,
    widget::{self, Button, Image, Rectangle, Text},
    widget_ids, Colorable, Positionable, Sizeable, Widget, WidgetCommon,
};

widget_ids! {
    pub struct Ids {
        quest_frame,
        quest_close,
        quest_title,
        frame,
        content_align,
    }
}

#[derive(WidgetCommon)]
pub struct Quest<'a> {
    _show: &'a Show,
    _client: &'a Client,

    imgs: &'a Imgs,
    fonts: &'a ConrodVoxygenFonts,
    localized_strings: &'a std::sync::Arc<VoxygenLocalization>,
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
}

impl<'a> Quest<'a> {
    pub fn new(
        show: &'a Show,
        _client: &'a Client,
        imgs: &'a Imgs,
        fonts: &'a ConrodVoxygenFonts,
        localized_strings: &'a std::sync::Arc<VoxygenLocalization>,
    ) -> Self {
        Self {
            _show: show,
            imgs,
            _client,
            fonts,
            localized_strings,
            common: widget::CommonBuilder::default(),
        }
    }
}

/*pub struct State {
    ids: Ids,
}*/

pub enum Event {
    Close,
}

impl<'a> Widget for Quest<'a> {
    type Event = Option<Event>;
    type State = Ids;
    type Style = ();

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State { Ids::new(id_gen) }

    fn style(&self) -> Self::Style { () }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id: _, state, ui, ..
        } = args;

        if self._show.character_window {
            Image::new(self.imgs.window_3)
                .top_left_with_margins_on(ui.window, 200.0, 658.0)
                .w_h(103.0 * 4.0, 122.0 * 4.0)
                .set(state.quest_frame, ui);
        } else {
            Image::new(self.imgs.window_3)
                .top_left_with_margins_on(ui.window, 200.0, 25.0)
                .w_h(103.0 * 4.0, 122.0 * 4.0)
                .set(state.quest_frame, ui);
        }

        // X-Button
        if Button::image(self.imgs.close_button)
            .w_h(28.0, 28.0)
            .hover_image(self.imgs.close_button_hover)
            .press_image(self.imgs.close_button_press)
            .top_right_with_margins_on(state.quest_frame, 0.0, 0.0)
            .set(state.quest_close, ui)
            .was_clicked()
        {
            return Some(Event::Close);
        }

        // Title
        // TODO: Use an actual character name.
        Text::new(&self.localized_strings.get("hud.quests"))
            .mid_top_with_margin_on(state.quest_frame, 6.0)
            .font_id(self.fonts.cyri.conrod_id)
            .font_size(self.fonts.cyri.scale(14))
            .color(TEXT_COLOR)
            .set(state.quest_title, ui);

        // Content Alignment
        Rectangle::fill_with([95.0 * 4.0, 108.0 * 4.0], color::TRANSPARENT)
            .mid_top_with_margin_on(state.quest_frame, 40.0)
            .set(state.content_align, ui);

        // Contents

        // Frame

        None
    }
}
