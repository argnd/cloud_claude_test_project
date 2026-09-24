//! Side quests. Their progress lives in the save as flags
//! (`quest_<id>_active`, `quest_<id>_done`) plus the items carried.

use serde::{Deserialize, Serialize};

use super::items::ItemId;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum QuestId {
    LostMouser,
    Smugglers,
    HolyRelics,
    TobinsGlowcap,
    Starmetal,
    LostPages,
    Everbloom,
    CrewTags,
}

/// What finishes a quest.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Requirement {
    Items(ItemId, u32),
    Flag(&'static str),
}

pub struct QuestDef {
    pub name: &'static str,
    /// Town NPC who offers and completes it; None for companion quests,
    /// which complete on their own once the requirement is met.
    pub giver: Option<&'static str>,
    pub giver_name: &'static str,
    pub summary: &'static str,
    pub hint: &'static str,
    pub requirement: Requirement,
    /// Story act from which the giver offers it.
    pub from_act: u8,
}

impl QuestId {
    pub const ALL: [QuestId; 8] = [
        QuestId::LostMouser,
        QuestId::Smugglers,
        QuestId::HolyRelics,
        QuestId::TobinsGlowcap,
        QuestId::Starmetal,
        QuestId::LostPages,
        QuestId::Everbloom,
        QuestId::CrewTags,
    ];

    pub fn script_id(self) -> &'static str {
        match self {
            QuestId::LostMouser => "lost_mouser",
            QuestId::Smugglers => "smugglers",
            QuestId::HolyRelics => "holy_relics",
            QuestId::TobinsGlowcap => "tobins_glowcap",
            QuestId::Starmetal => "starmetal",
            QuestId::LostPages => "lost_pages",
            QuestId::Everbloom => "everbloom",
            QuestId::CrewTags => "crew_tags",
        }
    }

    pub fn from_script_id(id: &str) -> Option<QuestId> {
        QuestId::ALL.into_iter().find(|q| q.script_id() == id)
    }

    pub fn active_flag(self) -> String {
        format!("quest_{}_active", self.script_id())
    }

    pub fn done_flag(self) -> String {
        format!("quest_{}_done", self.script_id())
    }

    pub fn def(self) -> &'static QuestDef {
        match self {
            QuestId::LostMouser => &QuestDef {
                name: "The Lost Cat",
                giver: Some("hesta"),
                giver_name: "Old Hesta",
                summary: "Hesta's cat Mouser ran down into the Undercroft.",
                hint: "Search the second floor of the Undercroft, then return to Hesta.",
                requirement: Requirement::Flag("mouser_found"),
                from_act: 1,
            },
            QuestId::Smugglers => &QuestDef {
                name: "Smugglers Below",
                giver: Some("rennick"),
                giver_name: "Captain Rennick",
                summary: "Someone is running contraband through the Undercroft.",
                hint: "Find the smugglers' leader in the Undercroft (floor 3), then report to Rennick.",
                requirement: Requirement::Flag("vex_defeated"),
                from_act: 1,
            },
            QuestId::HolyRelics => &QuestDef {
                name: "Relics of the First Wardens",
                giver: Some("oriel"),
                giver_name: "Sister Oriel",
                summary: "Three keepsakes of the first Wardens lie somewhere in the Deep.",
                hint: "Relics rest on floors 6, 10 and 14. Bring all three to Oriel.",
                requirement: Requirement::Items(ItemId::HolyRelic, 3),
                from_act: 2,
            },
            QuestId::TobinsGlowcap => &QuestDef {
                name: "A Lantern That Never Goes Out",
                giver: Some("tobin"),
                giver_name: "Tobin",
                summary: "Tobin wants a glowing mushroom from the Hollows.",
                hint: "Glowcaps grow in the Mycelium Hollows (floors 9-11); fungal creatures carry them too.",
                requirement: Requirement::Items(ItemId::Glowcap, 1),
                from_act: 3,
            },
            QuestId::Starmetal => &QuestDef {
                name: "Starmetal",
                giver: Some("dagna"),
                giver_name: "Dagna",
                summary: "Dagna can forge a legendary blade from five pieces of starmetal ore.",
                hint: "Ember Golems and Efreets in the Forge (floors 13-16) carry starmetal.",
                requirement: Requirement::Items(ItemId::StarmetalOre, 5),
                from_act: 4,
            },
            QuestId::LostPages => &QuestDef {
                name: "The Forbidden Index",
                giver: None,
                giver_name: "Maelis",
                summary: "Four pages of the Forbidden Index are scattered through the Archive.",
                hint: "Pages lie on floors 5 to 8.",
                requirement: Requirement::Items(ItemId::LostPage, 4),
                from_act: 2,
            },
            QuestId::Everbloom => &QuestDef {
                name: "The Everbloom",
                giver: None,
                giver_name: "Pip",
                summary: "The Glowroot Grove's last seed is somewhere in the Hollows.",
                hint: "Pip believes the heart of the Grove still holds it — below, on floor 12.",
                requirement: Requirement::Items(ItemId::EverbloomSeed, 1),
                from_act: 3,
            },
            QuestId::CrewTags => &QuestDef {
                name: "The Ironvein Crew",
                giver: None,
                giver_name: "Brannoc",
                summary: "Brannoc's crew vanished in the Ember Forge. Their tags might still be there.",
                hint: "Search floors 13, 14 and 15.",
                requirement: Requirement::Items(ItemId::CrewTag, 3),
                from_act: 4,
            },
        }
    }
}
