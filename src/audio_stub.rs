//! Temporary silent stand-in with the same API as src/audio/mod.rs.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Title,
    Town,
    TownSorrow,
    Undercroft,
    Archive,
    Hollows,
    Forge,
    PaleReach,
    Battle,
    Boss,
    FinalBoss,
    Victory,
    GameOver,
    Story,
    Ending,
    Dawn,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    MenuMove,
    MenuSelect,
    MenuBack,
    Denied,
    TextBlip,
    Step,
    DoorOpen,
    ChestOpen,
    Coin,
    ItemGet,
    Encounter,
    Stairs,
    Waystone,
    Save,
    Hit,
    CritHit,
    Miss,
    Block,
    EnemyDie,
    PartyDown,
    Heal,
    Fire,
    Frost,
    Shock,
    Light,
    Shadow,
    Poison,
    Buff,
    Debuff,
    LevelUp,
    Flee,
    Shard,
    BossRoar,
    Explosion,
}

pub struct Audio;

impl Audio {
    pub fn new() -> Self {
        Audio
    }
    pub fn play_music(&mut self, _track: Track) {}
    pub fn stop_music(&mut self) {}
    pub fn play_sfx(&mut self, _sfx: Sfx) {}
    pub fn set_music_volume(&mut self, _volume: f32) {}
    pub fn set_sfx_volume(&mut self, _volume: f32) {}
    pub fn update(&mut self, _dt: f32) {}
}
