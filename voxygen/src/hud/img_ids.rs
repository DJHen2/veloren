use crate::ui::img_ids::{BlankGraphic, ImageGraphic, VoxelGraphic, VoxelPixArtGraphic};

// TODO: Combine with image_ids, see macro definition
rotation_image_ids! {
    pub struct ImgsRot {
        <VoxelGraphic>

        // Tooltip Test
        tt_side: "voxygen/element/frames/tt_test_edge",
        tt_corner: "voxygen/element/frames/tt_test_corner_tr",

        <ImageGraphic>
        indicator_mmap_small: "voxygen.element.buttons.indicator_mmap_small",
//////////////////////////////////////////////////////////////////////////////////////////////////////

        <VoxelPixArtGraphic>
    }
}

image_ids! {
    pub struct Imgs {
        <VoxelGraphic>

        // Skillbar
        xp_bar_mid: "voxygen.element.skillbar.xp_bar_mid",
        xp_bar_left: "voxygen.element.skillbar.xp_bar_left",
        xp_bar_right: "voxygen.element.skillbar.xp_bar_right",
        skillbar_slot: "voxygen.element.skillbar.skillbar_slot",
        skillbar_slot_act: "voxygen.element.skillbar.skillbar_slot_active",
        skillbar_slot_l: "voxygen.element.skillbar.skillbar_slot_l",
        skillbar_slot_r: "voxygen.element.skillbar.skillbar_slot_r",
        skillbar_slot_l_act: "voxygen.element.skillbar.skillbar_slot_l_active",
        skillbar_slot_r_act: "voxygen.element.skillbar.skillbar_slot_r_active",
        skillbar_slot_bg: "voxygen.element.skillbar.skillbar_slot_bg",
        skillbar_slot_big: "voxygen.element.skillbar.skillbar_slot_big",
        skillbar_slot_big_act: "voxygen.element.skillbar.skillbar_slot_big_active",
        skillbar_slot_big_bg: "voxygen.element.skillbar.skillbar_slot_big_bg",
        healthbar_bg: "voxygen.element.skillbar.healthbar_bg",
        energybar_bg: "voxygen.element.skillbar.energybar_bg",
        bar_content: "voxygen.element.skillbar.bar_content",
        level_up: "voxygen.element.misc_bg.level_up",
        level_down:"voxygen.element.misc_bg.level_down",
        stamina_0:"voxygen.element.skillbar.stamina_wheel-empty",
        stamina_1:"voxygen.element.skillbar.stamina_wheel-0",
        stamina_2:"voxygen.element.skillbar.stamina_wheel-1",
        stamina_3:"voxygen.element.skillbar.stamina_wheel-2",
        stamina_4:"voxygen.element.skillbar.stamina_wheel-3",
        stamina_5:"voxygen.element.skillbar.stamina_wheel-4",
        stamina_6:"voxygen.element.skillbar.stamina_wheel-5",
        stamina_7:"voxygen.element.skillbar.stamina_wheel-6",
        stamina_8:"voxygen.element.skillbar.stamina_wheel-7",

        // Window Parts
        window_3: "voxygen.element.frames.window_3",
        tab_bg: "voxygen.element.frames.tab_bg",
        tab_small_open: "voxygen.element.frames.tab_small_open",
        tab_small_closed: "voxygen.element.frames.tab_small_closed",

        // Missing: Buff Frame Animation .gif ?! we could do animation in ui.maintain, or in shader?
        window_frame: "voxygen.element.frames.window2",

        // Social Window
        social_button: "voxygen.element.buttons.social_tab",
        social_button_pressed: "voxygen.element.buttons.social_tab_pressed",
        social_button_hover: "voxygen.element.buttons.social_tab_hover",
        social_button_press: "voxygen.element.buttons.social_tab_press",
        social_frame: "voxygen.element.frames.social_frame",


        // Settings Window
        settings_frame_r: "voxygen.element.frames.settings_r",
        settings_frame_l: "voxygen.element.frames.settings_l",
        settings_button: "voxygen.element.buttons.settings_button",
        settings_button_pressed: "voxygen.element.buttons.settings_button_pressed",
        settings_button_hover: "voxygen.element.buttons.settings_button_hover",
        settings_button_press: "voxygen.element.buttons.settings_button_press",
        slider: "voxygen.element.slider.track",
        slider_indicator: "voxygen.element.slider.indicator",
        esc_frame: "voxygen.element.frames.esc_menu",

        // Chat-Arrows
        chat_arrow: "voxygen.element.buttons.arrow_down",
        chat_arrow_mo: "voxygen.element.buttons.arrow_down_hover",
        chat_arrow_press: "voxygen.element.buttons.arrow_down_press",



////////////////////////////////////////////////////////////////////////
        <VoxelPixArtGraphic>

        // Skill Icons
        bow_m2: "voxygen.element.icons.bow_m2",

        // Icons
        flower: "voxygen.element.icons.item_flower",
        grass: "voxygen.element.icons.item_grass",

        // Minimap

        // Map
        indicator_mmap_2: "voxygen.element.buttons.indicator_mmap_2",
        indicator_mmap_3: "voxygen.element.buttons.indicator_mmap_3",

        // Checkboxes and Radio buttons
        check: "voxygen.element.buttons.radio.inactive",
        check_mo: "voxygen.element.buttons.radio.inactive_hover",
        check_press: "voxygen.element.buttons.radio.press",
        check_checked: "voxygen.element.buttons.radio.active",
        check_checked_mo: "voxygen.element.buttons.radio.hover",
        checkbox: "voxygen.element.buttons.checkbox.inactive",
        checkbox_mo: "voxygen.element.buttons.checkbox.inactive_hover",
        checkbox_press: "voxygen.element.buttons.checkbox.press",
        checkbox_checked: "voxygen.element.buttons.checkbox.active",
        checkbox_checked_mo: "voxygen.element.buttons.checkbox.hover",

        // Grid
        grid: "voxygen.element.buttons.grid",
        grid_hover: "voxygen.element.buttons.grid",
        grid_press: "voxygen.element.buttons.grid",

        settings: "voxygen.element.buttons.settings",
        settings_hover: "voxygen.element.buttons.settings_hover",
        settings_press: "voxygen.element.buttons.settings_press",

        social: "voxygen.element.buttons.social",
        social_hover: "voxygen.element.buttons.social_hover",
        social_press: "voxygen.element.buttons.social_press",

        map_button: "voxygen.element.buttons.map",
        map_hover: "voxygen.element.buttons.map_hover",
        map_press: "voxygen.element.buttons.map_press",

        spellbook_button: "voxygen.element.buttons.spellbook",
        spellbook_hover: "voxygen.element.buttons.spellbook_hover",
        spellbook_press: "voxygen.element.buttons.spellbook_press",

        // Charwindow
        xp_charwindow: "voxygen.element.frames.xp_charwindow",
        divider: "voxygen.element.frames.divider_charwindow",


        // Items
        potion_red: "voxygen.voxel.object.potion_red",
        potion_green: "voxygen.voxel.object.potion_green",
        potion_blue: "voxygen.voxel.object.potion_blue",
        key: "voxygen.voxel.object.key",
        key_gold: "voxygen.voxel.object.key_gold",


//////////////////////////////////////////////////////////////////////////////////////////////////////

        <ImageGraphic>

        // Skill Icons
        twohsword_m1: "voxygen.element.icons.2hsword_m1",
        twohsword_m2: "voxygen.element.icons.2hsword_m2",
        twohhammer_m1: "voxygen.element.icons.2hhammer_m1",
        twohhammer_m2: "voxygen.element.icons.2hhammer_m2",
        twohaxe_m1: "voxygen.element.icons.2haxe_m1",
        twohaxe_m2: "voxygen.element.icons.2haxe_m2",
        bow_m1: "voxygen.element.icons.bow_m1",
        //bow_m2: "voxygen.element.icons.bow_m2",
        staff_m1: "voxygen.element.icons.staff_m1",
        staff_m2: "voxygen.element.icons.staff_m2",
        flyingrod_m1: "voxygen.element.icons.debug_wand_m1",
        flyingrod_m2: "voxygen.element.icons.debug_wand_m2",
        charge: "voxygen.element.icons.skill_charge_3",

        // Other Icons/Art
        skull: "voxygen.element.icons.skull",
        skull_2: "voxygen.element.icons.skull_2",
        fireplace: "voxygen.element.misc_bg.fireplace",

        // Crosshair
        crosshair_inner: "voxygen.element.misc_bg.crosshair_inner",

        crosshair_outer_round: "voxygen.element.misc_bg.crosshair_outer_1",
        crosshair_outer_round_edges: "voxygen.element.misc_bg.crosshair_outer_2",
        crosshair_outer_edges: "voxygen.element.misc_bg.crosshair_outer_3",

        crosshair_bg: "voxygen.element.misc_bg.crosshair_bg",
        crosshair_bg_hover: "voxygen.element.misc_bg.crosshair_bg_hover",
        crosshair_bg_press: "voxygen.element.misc_bg.crosshair_bg_press",
        crosshair_bg_pressed: "voxygen.element.misc_bg.crosshair_bg_pressed",

        // Map
        map_bg: "voxygen.element.misc_bg.map_bg",
        map_frame: "voxygen.element.misc_bg.map_frame",
        map_frame_art: "voxygen.element.misc_bg.map_frame_art",
        indicator_mmap: "voxygen.element.buttons.indicator_mmap",

        // MiniMap
        mmap_frame: "voxygen.element.frames.mmap",
        mmap_frame_2: "voxygen.element.frames.mmap_frame",
        mmap_frame_closed: "voxygen.element.frames.mmap_closed",
        mmap_closed: "voxygen.element.buttons.button_mmap_closed",
        mmap_closed_hover: "voxygen.element.buttons.button_mmap_closed_hover",
        mmap_closed_press: "voxygen.element.buttons.button_mmap_closed_press",
        mmap_open: "voxygen.element.buttons.button_mmap_open",
        mmap_open_hover: "voxygen.element.buttons.button_mmap_open_hover",
        mmap_open_press: "voxygen.element.buttons.button_mmap_open_press",
        mmap_plus: "voxygen.element.buttons.min_plus.mmap_button-plus",
        mmap_plus_hover: "voxygen.element.buttons.min_plus.mmap_button-plus_hover",
        mmap_plus_press: "voxygen.element.buttons.min_plus.mmap_button-plus_press",
        mmap_minus: "voxygen.element.buttons.min_plus.mmap_button-min",
        mmap_minus_hover: "voxygen.element.buttons.min_plus.mmap_button-min_hover",
        mmap_minus_press: "voxygen.element.buttons.min_plus.mmap_button-min_press",

        // Close-Button
        close_btn: "voxygen.element.buttons.close_btn",
        close_btn_hover: "voxygen.element.buttons.close_btn_hover",
        close_btn_press: "voxygen.element.buttons.close_btn_press",
        close_button: "voxygen.element.buttons.close_btn",
        close_button_hover: "voxygen.element.buttons.close_btn_hover",
        close_button_press: "voxygen.element.buttons.close_btn_press",

        // Inventory
        coin_ico: "voxygen.element.icons.coin",
        inv_bg_armor: "voxygen.element.misc_bg.inv_bg_0",
        inv_bg_stats: "voxygen.element.misc_bg.inv_bg_1",
        inv_frame: "voxygen.element.misc_bg.inv_frame",
        char_art: "voxygen.element.icons.character",
        inv_slot: "voxygen.element.buttons.inv_slot",
        inv_slot_sel: "voxygen.element.buttons.inv_slot_sel",
        scrollbar_bg: "voxygen.element.slider.scrollbar",
        inv_tab_active: "voxygen.element.buttons.inv_tab_active",
        inv_tab_inactive: "voxygen.element.buttons.inv_tab_inactive",
        inv_tab_inactive_hover: "voxygen.element.buttons.inv_tab_inactive",
        inv_tab_inactive_press: "voxygen.element.buttons.inv_tab_inactive",
        inv_slots: "voxygen.element.misc_bg.inv_slots",
        inv_runes: "voxygen.element.misc_bg.inv_runes",
        armor_slot: "voxygen.element.buttons.armor_slot",
        head_bg: "voxygen.element.icons.head",
        shoulders_bg: "voxygen.element.icons.shoulders",
        hands_bg: "voxygen.element.icons.hands",
        belt_bg: "voxygen.element.icons.belt",
        legs_bg: "voxygen.element.icons.legs",
        feet_bg: "voxygen.element.icons.feet",
        ring_r_bg: "voxygen.element.icons.ring",
        ring_l_bg: "voxygen.element.icons.ring",
        tabard_bg: "voxygen.element.icons.tabard",
        chest_bg: "voxygen.element.icons.chest",
        back_bg: "voxygen.element.icons.back",
        necklace_bg: "voxygen.element.icons.necklace",
        mainhand_bg: "voxygen.element.icons.mainhand",
        offhand_bg: "voxygen.element.icons.offhand",
        willpower_ico: "voxygen.element.icons.willpower",
        endurance_ico: "voxygen.element.icons.endurance",
        fitness_ico: "voxygen.element.icons.fitness",

        not_found:"voxygen.element.not_found",

        help:"voxygen.element.help",

        death_bg: "voxygen.background.death",
        hurt_bg: "voxygen.background.hurt",

        banner_top: "voxygen.element.frames.banner_top",

        // Icons
        fire_spell_1: "voxygen.element.icons.fire_spell_0",
        heal_0: "voxygen.element.icons.heal_0",

        // Buttons
        button: "voxygen.element.buttons.button",
        button_hover: "voxygen.element.buttons.button_hover",
        button_press: "voxygen.element.buttons.button_press",

        // Enemy Healthbar
        enemy_health: "voxygen.element.frames.enemybar",
        enemy_health_bg: "voxygen.element.frames.enemybar_bg",
        // Enemy Bar Content:
        enemy_bar: "voxygen.element.skillbar.enemy_bar_content",
        // Bag
        bag: "voxygen.element.buttons.bag.closed",
        bag_hover: "voxygen.element.buttons.bag.closed_hover",
        bag_press: "voxygen.element.buttons.bag.closed_press",
        bag_open: "voxygen.element.buttons.bag.open",
        bag_open_hover: "voxygen.element.buttons.bag.open_hover",
        bag_open_press: "voxygen.element.buttons.bag.open_press",

        map_icon: "voxygen.element.icons.map",

        grid_button: "voxygen.element.buttons.border",
        grid_button_hover: "voxygen.element.buttons.border_mo",
        grid_button_press: "voxygen.element.buttons.border_press",
        grid_button_open: "voxygen.element.buttons.border_pressed",

        // Char Window
        progress_frame: "voxygen.element.frames.progress_bar",
        progress: "voxygen.element.misc_bg.progress",

        <BlankGraphic>
        nothing: (),
    }
}
