use super::{img_ids::Imgs, Fonts, TEXT_COLOR, XP_COLOR};
use conrod_core::{
    color,
    widget::{self, Button, Image, Rectangle, Text},
    widget_ids, Colorable, Labelable, Positionable, Sizeable, Widget, WidgetCommon,
};

widget_ids! {
    pub struct Ids {
        charwindow,
        charwindow_bg,
        charwindow_close,
        charwindow_exp_progress_rectangle,
        charwindow_exp_rectangle,
        charwindow_frame,
        content_align,
        // charwindow_icon,
        charwindow_rectangle,
        charwindow_tab1,
        charwindow_tab1_exp,
        charwindow_tab1_expbar,
        charwindow_tab1_level,
        charwindow_tab1_statnames,
        charwindow_tab1_stats,
        charwindow_tab_bg,
        charwindow_title,
        window_3,
        tab_bg,
        tab_small_open,
        tab_small_closed,
        xp_charwindow,
        divider,
        head_bg,
        shoulders_bg,
        hands_bg,
        belt_bg,
        legs_bg,
        feet_bg,
        ring_r_bg,
        ring_l_bg,
        tabard_bg,
        chest_bg,
        back_bg,
        gem_bg,
        necklace_bg,
        head_grid,
        shoulders_grid,
        hands_grid,
        belt_grid,
        legs_grid,
        feet_grid,
        ring_r_grid,
        ring_l_grid,
        tabard_grid,
        chest_grid,
        back_grid,
        gem_grid,
        necklace_grid,


    }
}

#[derive(WidgetCommon)]
pub struct CharacterWindow<'a> {
    imgs: &'a Imgs,
    fonts: &'a Fonts,

    #[conrod(common_builder)]
    common: widget::CommonBuilder,
}

impl<'a> CharacterWindow<'a> {
    pub fn new(imgs: &'a Imgs, fonts: &'a Fonts) -> Self {
        Self {
            imgs,
            fonts,
            common: widget::CommonBuilder::default(),
        }
    }
}

pub enum Event {
    Close,
}

impl<'a> Widget for CharacterWindow<'a> {
    type State = Ids;
    type Style = ();
    type Event = Option<Event>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        Ids::new(id_gen)
    }

    fn style(&self) -> Self::Style {
        ()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { id, state, ui, .. } = args;

        // TODO: Read from parameter / character struct
        let xp_percentage = 0.4;

        // Frame
        Image::new(self.imgs.window_3)
            .middle_of(id)
            .top_left_with_margins_on(ui.window, 212.0, 215.0)
            .w_h(103.0 * 4.0, 122.0 * 4.0)
            .set(state.charwindow_frame, ui);

        // Icon
        //Image::new(self.imgs.charwindow_icon)
        //.w_h(40.0, 40.0)
        //.top_left_with_margins_on(state.charwindow_frame, 4.0, 4.0)
        //.set(state.charwindow_icon, ui);

        // X-Button
        if Button::image(self.imgs.close_button)
            .w_h(28.0, 28.0)
            .hover_image(self.imgs.close_button_hover)
            .press_image(self.imgs.close_button_press)
            .top_right_with_margins_on(state.charwindow_frame, 0.0, 0.0)
            .set(state.charwindow_close, ui)
            .was_clicked()
        {
            return Some(Event::Close);
        }

        // Title
        Text::new("Character Name") // Add in actual Character Name
            .mid_top_with_margin_on(state.charwindow_frame, 6.0)
            .font_id(self.fonts.metamorph)
            .font_size(14)
            .color(TEXT_COLOR)
            .set(state.charwindow_title, ui);

        // Content Alignment
        Rectangle::fill_with([95.0 * 4.0, 108.0 * 4.0], color::TRANSPARENT)
            .mid_top_with_margin_on(state.charwindow_frame, 40.0)
            .set(state.content_align, ui);

        // Contents

        //Head
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .mid_top_with_margin_on(state.content_align, 5.0)
            .set(state.head_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.head_bg)
            .set(state.head_grid, ui);        
        // Shoulders
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .top_right_with_margins_on(state.content_align, 65.0, 40.0)
            .set(state.shoulders_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.shoulders_bg)
            .set(state.shoulders_grid, ui);
        // Ring R
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .bottom_right_with_margins_on(state.content_align, 20.0, 20.0)
            .set(state.ring_r_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.ring_r_bg)
            .set(state.ring_r_grid, ui);
         // Feet
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .up_from(state.ring_r_bg, 10.0)
            .set(state.feet_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.feet_bg)
            .set(state.feet_grid, ui);
        // Legs
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .up_from(state.feet_bg, 10.0)
            .set(state.legs_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.legs_bg)
            .set(state.legs_grid, ui);
        // Belt
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .up_from(state.legs_bg, 10.0)
            .set(state.belt_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.belt_bg)
            .set(state.belt_grid, ui);
        // Hands
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .up_from(state.belt_bg, 10.0)
            .set(state.hands_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.hands_bg)
            .set(state.hands_grid, ui);            
        // Ring L
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .bottom_left_with_margins_on(state.content_align, 20.0, 20.0)
            .set(state.ring_l_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.ring_l_bg)
            .set(state.ring_l_grid, ui);
        // Tabard
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .up_from(state.ring_l_bg, 10.0)
            .set(state.tabard_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.tabard_bg)
            .set(state.tabard_grid, ui);
        // Chest
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .up_from(state.tabard_bg, 10.0)
            .set(state.chest_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.chest_bg)
            .set(state.chest_grid, ui);
        // Back
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .up_from(state.chest_bg, 10.0)
            .set(state.back_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.back_bg)
            .set(state.back_grid, ui);
        // Gem
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .up_from(state.back_bg, 10.0)
            .set(state.gem_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.gem_bg)
            .set(state.gem_grid, ui);
        //Necklace
        Image::new(self.imgs.head_bg)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .top_left_with_margins_on(state.content_align, 65.0, 40.0)
            .set(state.necklace_bg, ui);
        Button::image(self.imgs.grid)
            .w_h(28.0 * 2.0, 28.0 * 2.0)
            .middle_of(state.necklace_bg)
            .set(state.necklace_grid, ui);
        // Stats Tab

        // Tab BG
        Image::new(self.imgs.tab_bg)
            .w_h(50.0 * 4.0, 115.0 * 4.0)
            .top_left_with_margins_on(state.charwindow_frame, 28.0, -200.0)
            .set(state.charwindow_tab_bg, ui);

        // Tab Rectangle
        Rectangle::fill_with([45.0 * 4.0, 104.0 * 4.0], color::TRANSPARENT)
            .top_left_with_margins_on(state.charwindow_tab_bg, 7.0 * 4.0, 4.0 * 4.0)
            .set(state.charwindow_rectangle, ui);

        // Tab Button -> Add that back in when we have multiple tabs
        // Button::image(self.imgs.charwindow_tab)
        //.w_h(65.0, 23.0)
        //.top_left_with_margins_on(state.charwindow_tab_bg, -18.0, 2.0)
        //.label("Stats")
        //.label_color(TEXT_COLOR)
        //.label_font_size(14)
        //.set(state.charwindow_tab1, ui);

        Text::new("1") //Add in actual Character Level
            .mid_top_with_margin_on(state.charwindow_rectangle, 10.0)
            .font_id(self.fonts.opensans)
            .font_size(30)
            .color(TEXT_COLOR)
            .set(state.charwindow_tab1_level, ui);

        // Exp-Bar Background
        Rectangle::fill_with([170.0, 10.0], color::BLACK)
            .mid_top_with_margin_on(state.charwindow_rectangle, 50.0)
            .set(state.charwindow_exp_rectangle, ui);

        // Exp-Bar Progress
        Rectangle::fill_with([170.0 * (xp_percentage), 6.0], XP_COLOR) // 0.8 = Experience percentage
            .mid_left_with_margin_on(state.charwindow_tab1_expbar, 1.0)
            .set(state.charwindow_exp_progress_rectangle, ui);

        // Exp-Bar Foreground Frame
        Image::new(self.imgs.progress_frame)
            .w_h(170.0, 10.0)
            .middle_of(state.charwindow_exp_rectangle)
            .set(state.charwindow_tab1_expbar, ui);

        // Exp-Text
        Text::new("120/170") // Shows the Exp / Exp to reach the next level
            .mid_top_with_margin_on(state.charwindow_tab1_expbar, 10.0)
            .font_id(self.fonts.opensans)
            .font_size(15)
            .color(TEXT_COLOR)
            .set(state.charwindow_tab1_exp, ui);

        // Divider

        Image::new(self.imgs.divider)
            .w_h(38.0 * 4.0, 5.0 * 4.0)
            .mid_top_with_margin_on(state.charwindow_tab1_exp, 30.0)
            .set(state.divider, ui);

        // Stats
        Text::new(
            "Stamina\n\
             \n\
             Strength\n\
             \n\
             Dexterity\n\
             \n\
             Intelligence",
        )
        .top_left_with_margins_on(state.charwindow_rectangle, 140.0, 5.0)
        .font_id(self.fonts.opensans)
        .font_size(16)
        .color(TEXT_COLOR)
        .set(state.charwindow_tab1_statnames, ui);

        Text::new(
            "1234\n\
             \n\
             12312\n\
             \n\
             12414\n\
             \n\
             124124",
        )
        .top_right_with_margins_on(state.charwindow_rectangle, 140.0, 5.0)
        .font_id(self.fonts.opensans)
        .font_size(16)
        .color(TEXT_COLOR)
        .set(state.charwindow_tab1_stats, ui);

        None
    }
}
