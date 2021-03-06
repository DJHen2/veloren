#![deny(clippy::clone_on_ref_ptr)]

use std::error::Error;
use structopt::StructOpt;

use common::comp;
use comp::item::{
    armor::{ArmorKind, Protection},
    tool::ToolKind,
    ItemKind,
};

#[derive(StructOpt)]
struct Cli {
    /// Available arguments: "armor_stats", "weapon_stats", "all_items"
    function: String,
}

fn armor_stats() -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path("armorstats.csv")?;
    wtr.write_record(&["Path", "Kind", "Name", "Protection"])?;

    for item in comp::item::Item::new_from_asset_glob("common.items.armor.*")
        .expect("Failed to iterate over item folders!")
    {
        match item.kind() {
            comp::item::ItemKind::Armor(armor) => {
                let protection = match armor.get_protection() {
                    Protection::Invincible => "Invincible".to_string(),
                    Protection::Normal(value) => value.to_string(),
                };
                let kind = get_armor_kind(&armor.kind);

                wtr.write_record(&[item.item_definition_id(), &kind, item.name(), &protection])?;
            },
            _ => println!("Skipping non-armor item: {:?}", item),
        }
    }

    wtr.flush()?;
    Ok(())
}

fn weapon_stats() -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path("weaponstats.csv")?;
    wtr.write_record(&["Path", "Kind", "Name", "Power", "Equip Time (ms)"])?;

    for item in comp::item::Item::new_from_asset_glob("common.items.weapons.*")
        .expect("Failed to iterate over item folders!")
    {
        match item.kind() {
            comp::item::ItemKind::Tool(tool) => {
                let power = tool.base_power().to_string();
                let equip_time = tool.equip_time().subsec_millis().to_string();
                let kind = get_tool_kind(&tool.kind);

                wtr.write_record(&[
                    item.item_definition_id(),
                    &kind,
                    item.name(),
                    &power,
                    &equip_time,
                ])?;
            },
            _ => println!("Skipping non-weapon item: {:?}", item),
        }
    }

    wtr.flush()?;
    Ok(())
}

fn get_tool_kind(kind: &ToolKind) -> String {
    match kind {
        ToolKind::Sword(_) => "Sword".to_string(),
        ToolKind::Axe(_) => "Axe".to_string(),
        ToolKind::Hammer(_) => "Hammer".to_string(),
        ToolKind::Bow(_) => "Bow".to_string(),
        ToolKind::Dagger(_) => "Dagger".to_string(),
        ToolKind::Staff(_) => "Staff".to_string(),
        ToolKind::Sceptre(_) => "Sceptre".to_string(),
        ToolKind::Shield(_) => "Shield".to_string(),
        ToolKind::Debug(_) => "Debug".to_string(),
        ToolKind::Farming(_) => "Farming".to_string(),
        ToolKind::NpcWeapon(_) => "NpcWeapon".to_string(),
        ToolKind::Empty => "Empty".to_string(),
    }
}

fn get_tool_kind_kind(kind: &ToolKind) -> String {
    match kind {
        ToolKind::Sword(x) => x.clone(),
        ToolKind::Axe(x) => x.clone(),
        ToolKind::Hammer(x) => x.clone(),
        ToolKind::Bow(x) => x.clone(),
        ToolKind::Dagger(x) => x.clone(),
        ToolKind::Staff(x) => x.clone(),
        ToolKind::Sceptre(x) => x.clone(),
        ToolKind::Shield(x) => x.clone(),
        ToolKind::Debug(x) => x.clone(),
        ToolKind::Farming(x) => x.clone(),
        ToolKind::NpcWeapon(x) => x.clone(),
        ToolKind::Empty => "".to_string(),
    }
}

fn get_armor_kind(kind: &ArmorKind) -> String {
    match kind {
        ArmorKind::Shoulder(_) => "Shoulder".to_string(),
        ArmorKind::Chest(_) => "Chest".to_string(),
        ArmorKind::Belt(_) => "Belt".to_string(),
        ArmorKind::Hand(_) => "Hand".to_string(),
        ArmorKind::Pants(_) => "Pants".to_string(),
        ArmorKind::Foot(_) => "Foot".to_string(),
        ArmorKind::Back(_) => "Back".to_string(),
        ArmorKind::Ring(_) => "Ring".to_string(),
        ArmorKind::Neck(_) => "Neck".to_string(),
        ArmorKind::Head(_) => "Head".to_string(),
        ArmorKind::Tabard(_) => "Tabard".to_string(),
    }
}

fn get_armor_kind_kind(kind: &ArmorKind) -> String {
    match kind {
        ArmorKind::Shoulder(x) => x.clone(),
        ArmorKind::Chest(x) => x.clone(),
        ArmorKind::Belt(x) => x.clone(),
        ArmorKind::Hand(x) => x.clone(),
        ArmorKind::Pants(x) => x.clone(),
        ArmorKind::Foot(x) => x.clone(),
        ArmorKind::Back(x) => x.clone(),
        ArmorKind::Ring(x) => x.clone(),
        ArmorKind::Neck(x) => x.clone(),
        ArmorKind::Head(x) => x.clone(),
        ArmorKind::Tabard(x) => x.clone(),
    }
}

fn all_items() -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path("items.csv")?;
    wtr.write_record(&["Path", "Name", "Kind"])?;

    for item in comp::item::Item::new_from_asset_glob("common.items.*")
        .expect("Failed to iterate over item folders!")
    {
        let kind = match item.kind() {
            ItemKind::Armor(armor) => get_armor_kind_kind(&armor.kind),
            ItemKind::Lantern(lantern) => lantern.kind.clone(),
            ItemKind::Tool(tool) => get_tool_kind_kind(&tool.kind),
            _ => "".to_owned(),
        };

        wtr.write_record(&[item.item_definition_id(), item.name(), &kind])?;
    }

    wtr.flush()?;
    Ok(())
}

fn main() {
    let args = Cli::from_args();
    if args.function.eq_ignore_ascii_case("armor_stats") {
        if let Err(e) = armor_stats() {
            println!("Error: {}", e)
        }
    } else if args.function.eq_ignore_ascii_case("weapon_stats") {
        if let Err(e) = weapon_stats() {
            println!("Error: {}", e)
        }
    } else if args.function.eq_ignore_ascii_case("all_items") {
        if let Err(e) = all_items() {
            println!("Error: {}", e)
        }
    } else {
        println!(
            "Invalid argument, available \
             arguments:\n\"armor_stats\"\n\"weapon_stats\"\n\"all_items\""
        )
    }
}
