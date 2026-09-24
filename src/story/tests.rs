use std::collections::HashSet;

use super::*;

#[derive(Default)]
struct Mock {
    flags: HashSet<String>,
    given: Vec<(ItemId, u32)>,
    joined: Vec<HeroId>,
}

impl StoryContext for Mock {
    fn flag(&self, name: &str) -> bool {
        self.flags.contains(name)
    }
    fn set_flag(&mut self, name: &str, on: bool) {
        if on {
            self.flags.insert(name.to_string());
        } else {
            self.flags.remove(name);
        }
    }
    fn give(&mut self, item: ItemId, count: u32) {
        self.given.push((item, count));
    }
    fn take(&mut self, _: ItemId, _: u32) {}
    fn gold(&mut self, _: i32) {}
    fn join(&mut self, hero: HeroId) {
        self.joined.push(hero);
    }
    fn quest_start(&mut self, _: QuestId) {}
    fn quest_done(&mut self, _: QuestId) {}
    fn heal_party(&mut self) {}
    fn music(&mut self, _: Track) {}
    fn sfx(&mut self, _: Sfx) {}
    fn visual(&mut self, _: Visual) {}
}

const SAMPLE: &str = "
# a comment
=== start
@wren: Hello.
!give tonic 3
!flag met
!if met => second
@wren: never shown
=== second
@brannoc: Stone and ember!
!join brannoc
? Fight => fight
?[secret] Hidden => end
=== fight
!battle vex
@narrator: After.
";

#[test]
fn runs_a_scene_with_commands_jumps_and_choices() {
    let script = Script::parse_files(&[("sample.story", SAMPLE)]).unwrap();
    assert!(script.validate().is_empty(), "{:?}", script.validate());
    let mut ctx = Mock::default();
    let mut r = Runner::new("start");
    assert_eq!(r.next(&script, &mut ctx), Beat::Line { speaker: "wren".into(), text: "Hello.".into() });
    assert_eq!(r.next(&script, &mut ctx), Beat::Line { speaker: "brannoc".into(), text: "Stone and ember!".into() });
    assert_eq!(ctx.given, vec![(ItemId::Tonic, 3)]);
    let Beat::Choice(opts) = r.next(&script, &mut ctx) else { panic!() };
    assert_eq!(opts.len(), 1, "flagged choice hidden");
    assert_eq!(ctx.joined, vec![HeroId::Brannoc]);
    r.choose(&opts[0].1);
    assert_eq!(r.next(&script, &mut ctx), Beat::Battle(BattleId::Vex));
    assert!(matches!(r.next(&script, &mut ctx), Beat::Line { .. }));
    assert_eq!(r.next(&script, &mut ctx), Beat::Done);
}

#[test]
fn reports_bad_lines() {
    assert!(Script::parse_files(&[("x", "=== a\n!give nothing_real")]).is_err());
    assert!(Script::parse_files(&[("x", "@wren: before any scene")]).is_err());
    let s = Script::parse_files(&[("x", "=== a\n@nobody: hi\n!goto nowhere")]).unwrap();
    assert_eq!(s.validate().len(), 2);
}

/// Every scene id the engine triggers by name.
pub fn required_scenes() -> Vec<String> {
    let mut ids: Vec<String> = [
        "intro", "wake", "elder_first", "elder_confront", "inn_rest", "vaultgate_first", "waystone_first",
        "mouser_found", "vex_pre", "brannoc_join", "gristlemaw_pre", "gristlemaw_post", "maelis_meet",
        "curator_pre", "curator_post", "pip_meet", "mother_pre", "mother_post", "iron_warden_pre",
        "iron_warden_post", "ilsa_lantern", "ilsa_pre", "ilsa_post", "aurelian_pre", "aurelian_phase2",
        "final_choice", "ending_oath", "ending_dark", "ending_dawn", "credits",
        "quest_lost_mouser_offer", "quest_lost_mouser_remind", "quest_lost_mouser_done",
        "quest_smugglers_offer", "quest_smugglers_remind", "quest_smugglers_done",
        "quest_holy_relics_offer", "quest_holy_relics_remind", "quest_holy_relics_done",
        "quest_tobins_glowcap_offer", "quest_tobins_glowcap_remind", "quest_tobins_glowcap_done",
        "quest_starmetal_offer", "quest_starmetal_remind", "quest_starmetal_done",
        "quest_lost_pages_done", "quest_everbloom_done", "quest_crew_tags_done",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for n in 1..=20 {
        ids.push(format!("floor_{n}_enter"));
    }
    for n in 1..=12 {
        ids.push(format!("shard_{n}"));
    }
    for n in 1..=5 {
        ids.push(format!("journal_{n}"));
        ids.push(format!("camp_{n}"));
    }
    for n in 2..=5 {
        ids.push(format!("town_return_{n}"));
    }
    for npc in ["elder", "bess", "dagna", "fen", "oriel", "rennick", "hesta", "tobin", "villager_a", "villager_b", "guard"] {
        for stage in 1..=5 {
            ids.push(format!("npc_{npc}_{stage}"));
        }
    }
    ids
}

#[test]
fn the_game_script_is_complete_and_consistent() {
    let script = Script::parse_files(STORY_FILES).unwrap_or_else(|e| panic!("{e}"));
    let problems = script.validate();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    let missing: Vec<String> = required_scenes().into_iter().filter(|id| !script.has(id)).collect();
    assert!(missing.is_empty(), "missing scenes: {missing:?}");
}
