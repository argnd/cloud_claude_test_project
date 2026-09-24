//! The story scripts in assets/story/*.story: parsing, validation, and a
//! runner that walks a scene line by line, executing its commands against
//! the game through `StoryContext`.
//!
//! Format (one statement per line):
//! ```text
//! === scene_id
//! @speaker: Dialogue text.
//! !command args
//! ? Choice text => target_scene
//! ?[flag] Choice shown only when flag is set => target_scene
//! ```

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::audio::{Sfx, Track};
use crate::data::enemies::BattleId;
use crate::data::heroes::HeroId;
use crate::data::items::ItemId;
use crate::data::quests::QuestId;
use crate::gfx::sprites::Sprite;

include!(concat!(env!("OUT_DIR"), "/story_files.rs"));

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ending {
    Oath,
    Dark,
    Dawn,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Visual {
    Shake,
    Flash,
    Fade,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Flag(String),
    Unflag(String),
    If { flag: String, target: String, negate: bool },
    Goto(String),
    Give(ItemId, u32),
    Take(ItemId, u32),
    Gold(i32),
    Join(HeroId),
    QuestStart(QuestId),
    QuestDone(QuestId),
    Battle(BattleId),
    Heal,
    Music(Track),
    Sfx(Sfx),
    Visual(Visual),
    Ending(Ending),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceOpt {
    pub flag: Option<String>,
    pub text: String,
    pub target: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Line { speaker: String, text: String },
    Choice(Vec<ChoiceOpt>),
    Cmd(Command),
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub id: String,
    pub ops: Vec<Op>,
}

#[derive(Default)]
pub struct Script {
    pub scenes: HashMap<String, Scene>,
}

#[derive(Debug)]
pub struct ParseError {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}

fn track_named(name: &str) -> Option<Track> {
    Some(match name {
        "Title" => Track::Title,
        "Town" => Track::Town,
        "TownSorrow" => Track::TownSorrow,
        "Undercroft" => Track::Undercroft,
        "Archive" => Track::Archive,
        "Hollows" => Track::Hollows,
        "Forge" => Track::Forge,
        "PaleReach" => Track::PaleReach,
        "Battle" => Track::Battle,
        "Boss" => Track::Boss,
        "FinalBoss" => Track::FinalBoss,
        "Victory" => Track::Victory,
        "GameOver" => Track::GameOver,
        "Story" => Track::Story,
        "Ending" => Track::Ending,
        "Dawn" => Track::Dawn,
        _ => return None,
    })
}

fn sfx_named(name: &str) -> Option<Sfx> {
    Some(match name {
        "Shard" => Sfx::Shard,
        "LevelUp" => Sfx::LevelUp,
        "ItemGet" => Sfx::ItemGet,
        "Coin" => Sfx::Coin,
        "DoorOpen" => Sfx::DoorOpen,
        "BossRoar" => Sfx::BossRoar,
        "Explosion" => Sfx::Explosion,
        "Light" => Sfx::Light,
        "Shadow" => Sfx::Shadow,
        "Frost" => Sfx::Frost,
        "Heal" => Sfx::Heal,
        "Buff" => Sfx::Buff,
        "Waystone" => Sfx::Waystone,
        "Fire" => Sfx::Fire,
        "Save" => Sfx::Save,
        "Stairs" => Sfx::Stairs,
        "Encounter" => Sfx::Encounter,
        "ChestOpen" => Sfx::ChestOpen,
        _ => return None,
    })
}

fn parse_command(text: &str) -> Result<Command, String> {
    let mut words = text.split_whitespace();
    let name = words.next().ok_or("empty command")?;
    let rest: Vec<&str> = words.collect();
    let arg = |i: usize| rest.get(i).copied().ok_or_else(|| format!("!{name} needs more arguments"));
    let count = |i: usize| -> Result<u32, String> {
        match rest.get(i) {
            None => Ok(1),
            Some(n) => n.parse().map_err(|_| format!("bad count {n:?}")),
        }
    };
    let item = |id: &str| ItemId::from_script_id(id).ok_or_else(|| format!("unknown item {id:?}"));
    let jump = |negate: bool| -> Result<Command, String> {
        // `!if flag => scene`
        if rest.len() != 3 || rest[1] != "=>" {
            return Err(format!("expected `!{name} flag => scene`"));
        }
        Ok(Command::If { flag: rest[0].to_string(), target: rest[2].to_string(), negate })
    };
    Ok(match name {
        "flag" => Command::Flag(arg(0)?.to_string()),
        "unflag" => Command::Unflag(arg(0)?.to_string()),
        "if" => jump(false)?,
        "ifnot" => jump(true)?,
        "goto" => Command::Goto(arg(0)?.to_string()),
        "give" => Command::Give(item(arg(0)?)?, count(1)?),
        "take" => Command::Take(item(arg(0)?)?, count(1)?),
        "gold" => Command::Gold(arg(0)?.parse().map_err(|_| "bad gold amount".to_string())?),
        "join" => Command::Join(HeroId::from_script_id(arg(0)?).ok_or("unknown hero")?),
        "quest" => {
            let quest = QuestId::from_script_id(arg(1)?).ok_or_else(|| format!("unknown quest {:?}", rest[1]))?;
            match arg(0)? {
                "start" => Command::QuestStart(quest),
                "done" => Command::QuestDone(quest),
                other => return Err(format!("!quest {other}?")),
            }
        }
        "battle" => Command::Battle(BattleId::from_script_id(arg(0)?).ok_or_else(|| format!("unknown battle {:?}", rest[0]))?),
        "heal" => Command::Heal,
        "music" => Command::Music(track_named(arg(0)?).ok_or_else(|| format!("unknown track {:?}", rest[0]))?),
        "sfx" => Command::Sfx(sfx_named(arg(0)?).ok_or_else(|| format!("unknown sfx {:?}", rest[0]))?),
        "shake" => Command::Visual(Visual::Shake),
        "flash" => Command::Visual(Visual::Flash),
        "fade" => Command::Visual(Visual::Fade),
        "ending" => Command::Ending(match arg(0)? {
            "oath" => Ending::Oath,
            "dark" => Ending::Dark,
            "dawn" => Ending::Dawn,
            other => return Err(format!("unknown ending {other:?}")),
        }),
        other => return Err(format!("unknown command !{other}")),
    })
}

fn parse_choice(text: &str) -> Result<ChoiceOpt, String> {
    let body = text.trim_start_matches('?').trim();
    let (flag, body) = if let Some(rest) = body.strip_prefix('[') {
        let (flag, after) = rest.split_once(']').ok_or("unclosed [flag]")?;
        (Some(flag.trim().to_string()), after.trim())
    } else {
        (None, body)
    };
    let (label, target) = body.rsplit_once("=>").ok_or("choice needs `=> target`")?;
    Ok(ChoiceOpt { flag, text: label.trim().to_string(), target: target.trim().to_string() })
}

impl Script {
    pub fn parse_files(files: &[(&str, &str)]) -> Result<Script, ParseError> {
        let mut script = Script::default();
        for &(file, text) in files {
            let mut current: Option<Scene> = None;
            for (n, raw) in text.lines().enumerate() {
                let err = |message: String| ParseError { file: file.to_string(), line: n + 1, message };
                let line = raw.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some(id) = line.strip_prefix("===") {
                    if let Some(done) = current.take() {
                        script.insert(done).map_err(err)?;
                    }
                    current = Some(Scene { id: id.trim().to_string(), ops: Vec::new() });
                    continue;
                }
                let Some(scene) = current.as_mut() else {
                    return Err(err("statement before the first === scene".into()));
                };
                if let Some(body) = line.strip_prefix('@') {
                    let (speaker, text) = body.split_once(':').ok_or_else(|| err("expected `@speaker: text`".into()))?;
                    scene.ops.push(Op::Line { speaker: speaker.trim().to_string(), text: text.trim().to_string() });
                } else if let Some(body) = line.strip_prefix('!') {
                    scene.ops.push(Op::Cmd(parse_command(body).map_err(err)?));
                } else if line.starts_with('?') {
                    let opt = parse_choice(line).map_err(err)?;
                    if let Some(Op::Choice(opts)) = scene.ops.last_mut() {
                        opts.push(opt);
                    } else {
                        scene.ops.push(Op::Choice(vec![opt]));
                    }
                } else {
                    return Err(err(format!("can't read line: {line:?}")));
                }
            }
            if let Some(done) = current.take() {
                script
                    .insert(done)
                    .map_err(|message| ParseError { file: file.to_string(), line: 0, message })?;
            }
        }
        Ok(script)
    }

    fn insert(&mut self, scene: Scene) -> Result<(), String> {
        if self.scenes.contains_key(&scene.id) {
            return Err(format!("scene {:?} defined twice", scene.id));
        }
        self.scenes.insert(scene.id.clone(), scene);
        Ok(())
    }

    pub fn has(&self, id: &str) -> bool {
        self.scenes.contains_key(id)
    }

    /// Problems a parse can't see: dangling jumps, unknown speakers.
    #[cfg(test)]
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let exists = |t: &str| t == "end" || self.scenes.contains_key(t);
        for scene in self.scenes.values() {
            for op in &scene.ops {
                match op {
                    Op::Line { speaker, .. } if speaker_info(speaker).is_none() => {
                        problems.push(format!("{}: unknown speaker {speaker:?}", scene.id))
                    }
                    Op::Choice(opts) => {
                        for o in opts {
                            if !exists(&o.target) {
                                problems.push(format!("{}: choice to missing scene {:?}", scene.id, o.target));
                            }
                        }
                    }
                    Op::Cmd(Command::Goto(t)) | Op::Cmd(Command::If { target: t, .. }) if !exists(t) => {
                        problems.push(format!("{}: jump to missing scene {t:?}", scene.id))
                    }
                    _ => {}
                }
            }
        }
        problems.sort();
        problems
    }
}

/// The script compiled into the game. Parse errors are reported once and
/// the broken file is skipped rather than crashing the game.
pub fn script() -> &'static Script {
    static SCRIPT: OnceLock<Script> = OnceLock::new();
    SCRIPT.get_or_init(|| match Script::parse_files(STORY_FILES) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("story script error: {e}");
            let mut merged = Script::default();
            for file in STORY_FILES {
                if let Ok(part) = Script::parse_files(&[*file]) {
                    for (_, scene) in part.scenes {
                        let _ = merged.insert(scene);
                    }
                }
            }
            merged
        }
    })
}

/// Display name and portrait for a speaker id.
pub fn speaker_info(id: &str) -> Option<(&'static str, Option<Sprite>)> {
    Some(match id {
        "narrator" => ("", None),
        "wren" => ("Wren", Some(Sprite::Wren)),
        "ilsa" => ("Ilsa", Some(Sprite::Ilsa)),
        "ilsa_pale" => ("Ilsa?", Some(Sprite::IlsaPale)),
        "brannoc" => ("Brannoc", Some(Sprite::Brannoc)),
        "maelis" => ("Maelis", Some(Sprite::Maelis)),
        "pip" => ("Pip", Some(Sprite::Pip)),
        "elder" => ("Elder Marrow", Some(Sprite::Elder)),
        "bess" => ("Bess", Some(Sprite::Innkeeper)),
        "dagna" => ("Dagna", Some(Sprite::Smith)),
        "fen" => ("Fen", Some(Sprite::Apothecary)),
        "oriel" => ("Sister Oriel", Some(Sprite::Priestess)),
        "rennick" => ("Captain Rennick", Some(Sprite::Captain)),
        "hesta" => ("Old Hesta", Some(Sprite::OldWoman)),
        "tobin" => ("Tobin", Some(Sprite::Child)),
        "villager_a" => ("Gwen the Weaver", Some(Sprite::VillagerA)),
        "villager_b" => ("Osric the Miner", Some(Sprite::VillagerB)),
        "guard" => ("Gate Guard", Some(Sprite::GuardNpc)),
        "vex" => ("Vex Harlan", Some(Sprite::VexHarlan)),
        "gristlemaw" => ("Gristlemaw", Some(Sprite::Gristlemaw)),
        "curator" => ("The Curator", Some(Sprite::Curator)),
        "mother" => ("Mother of Spores", Some(Sprite::MotherOfSpores)),
        "hadric" => ("The Iron Warden", Some(Sprite::IronWarden)),
        "aurelian" => ("Aurelian", Some(Sprite::Aurelian)),
        "aurelian_memory" => ("Aurelian", Some(Sprite::AurelianTrue)),
        "lira" => ("Lira", None),
        "mouser" => ("Mouser", Some(Sprite::Cat)),
        _ => return None,
    })
}

/// What the runner needs from the game.
pub trait StoryContext {
    fn flag(&self, name: &str) -> bool;
    fn set_flag(&mut self, name: &str, on: bool);
    fn give(&mut self, item: ItemId, count: u32);
    fn take(&mut self, item: ItemId, count: u32);
    fn gold(&mut self, amount: i32);
    fn join(&mut self, hero: HeroId);
    fn quest_start(&mut self, quest: QuestId);
    fn quest_done(&mut self, quest: QuestId);
    fn heal_party(&mut self);
    fn music(&mut self, track: Track);
    fn sfx(&mut self, sfx: Sfx);
    fn visual(&mut self, visual: Visual);
}

/// What a scene wants shown next.
#[derive(Clone, Debug, PartialEq)]
pub enum Beat {
    Line { speaker: String, text: String },
    Choice(Vec<(String, String)>),
    Battle(BattleId),
    Ending(Ending),
    Done,
}

/// A cursor walking through scenes.
#[derive(Clone, Debug)]
pub struct Runner {
    pub scene: String,
    pc: usize,
    /// Scenes seen by this runner (to stop runaway goto loops).
    jumps: u32,
}

impl Runner {
    pub fn new(scene: &str) -> Self {
        Self { scene: scene.to_string(), pc: 0, jumps: 0 }
    }

    fn jump(&mut self, target: &str) {
        self.scene = target.to_string();
        self.pc = 0;
        self.jumps += 1;
    }

    /// Runs commands until something must be shown.
    pub fn next(&mut self, script: &Script, ctx: &mut dyn StoryContext) -> Beat {
        loop {
            if self.jumps > 200 || self.scene == "end" {
                return Beat::Done;
            }
            let Some(scene) = script.scenes.get(&self.scene) else {
                return Beat::Done;
            };
            let Some(op) = scene.ops.get(self.pc) else {
                return Beat::Done;
            };
            self.pc += 1;
            match op {
                Op::Line { speaker, text } => {
                    return Beat::Line { speaker: speaker.clone(), text: text.clone() };
                }
                Op::Choice(opts) => {
                    let shown: Vec<(String, String)> = opts
                        .iter()
                        .filter(|o| o.flag.as_ref().is_none_or(|f| ctx.flag(f)))
                        .map(|o| (o.text.clone(), o.target.clone()))
                        .collect();
                    if !shown.is_empty() {
                        return Beat::Choice(shown);
                    }
                }
                Op::Cmd(cmd) => match cmd {
                    Command::Flag(f) => ctx.set_flag(f, true),
                    Command::Unflag(f) => ctx.set_flag(f, false),
                    Command::If { flag, target, negate } => {
                        if ctx.flag(flag) != *negate {
                            self.jump(target);
                        }
                    }
                    Command::Goto(target) => self.jump(target),
                    Command::Give(item, n) => ctx.give(*item, *n),
                    Command::Take(item, n) => ctx.take(*item, *n),
                    Command::Gold(n) => ctx.gold(*n),
                    Command::Join(h) => ctx.join(*h),
                    Command::QuestStart(q) => ctx.quest_start(*q),
                    Command::QuestDone(q) => ctx.quest_done(*q),
                    Command::Battle(b) => return Beat::Battle(*b),
                    Command::Heal => ctx.heal_party(),
                    Command::Music(t) => ctx.music(*t),
                    Command::Sfx(s) => ctx.sfx(*s),
                    Command::Visual(v) => ctx.visual(*v),
                    Command::Ending(e) => return Beat::Ending(*e),
                },
            }
        }
    }

    /// Follows a picked choice.
    pub fn choose(&mut self, target: &str) {
        self.jump(target);
    }
}

#[cfg(test)]
mod tests;
