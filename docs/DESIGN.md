# Emberdeep — Game Design Document

*A story-driven, turn-based fantasy RPG. Proof of concept.*

## The pitch

A lamplighter descends twenty floors beneath her mountain village to find the
sister who went down a year ago to tend the sacred flame — and learns that the
flame has always been fed with a Warden's soul.

Emberdeep is a game about grief: what it does to people, what we owe the ones
we lose, and how remembering with love can end what grief started. It plays
like a classic 16-bit JRPG (explore, talk, fight, grow) with the lighting,
pacing and writing of a modern indie.

## Pillars

1. **Story first.** Every floor, companion and boss exists to move the story.
   The twist lands at the midpoint and recolours everything before it.
2. **Readable, tactical battles.** You always see who acts next; every enemy
   has weaknesses to discover; nothing is decided by grinding alone.
3. **A world that reacts.** The village changes act by act — lanterns go out,
   people leave, conversations turn — and your choices at the end are
   informed by how much of the past you found.

## Story

**Hollowmere**, a village high in the Greyspine peaks, is built around the
**Vaultgate**, a stair into the Deep. At the bottom burns the **Hearthflame**,
which holds back **the Pale** — a colourless cold that devours warmth, memory
and will. Once a generation the Lantern Order sends a **Warden** down to
"tend" it. None return.

A year ago **Ilsa Varn** went down. Now the lanterns are failing, frost climbs
the wells in summer, and the dead walk in the cellars. Her younger sister
**Wren**, the village lamplighter, takes their father's lantern and follows.

| Act | Floors | Place | What happens |
| --- | --- | --- | --- |
| I | 1–4 | The Undercroft | Smugglers, rats and ghouls. Wren frees **Brannoc**, a dwarf who lost his crew below. The Rat King falls; the rats were fleeing something. |
| II | 5–8 | The Drowned Archive | **Maelis**, an archivist hiding in the flooded library, joins. The Curator's ledger reveals the truth: Wardens are burned to feed the flame. Ilsa refused. Maelis knew all along. |
| III | 9–12 | The Mycelium Hollows | **Pip**, last keeper of a dying grove, joins. The Mother of Spores — the grove's own corrupted heart — must be put to rest. |
| IV | 13–16 | The Ember Forge | Brannoc finds his crew's tags. The Iron Warden, a Warden from sixty years ago bound in iron, let Ilsa pass: "She had your eyes." |
| V | 17–20 | The Pale Reach | Ilsa, half-consumed, holds the last door. Below: the Pale itself — and the grieving man it used to be. |

**The secret.** Three hundred years ago a healer named **Aurelian** lost his
daughter **Lira** to fever. His grief, magnified by the Well of Returning,
became the Pale. Twelve **Memory Shards** of his life are scattered through
the Deep. Find them all and the finale offers a third choice.

**Endings.**
- *The Warden's Oath* — Wren takes Ilsa's place in the flame.
- *Let It Go Dark* — the flame dies; Hollowmere flees down the mountain.
- *Dawn Below* (all 12 shards) — Wren gives Aurelian back his memories; he
  finishes his daughter's lullaby; the Pale melts into light.

A single leitmotif, *Lira's Lullaby*, runs from the title screen through an
old woman's humming in Act I to the final battle and the true ending.

## Systems

**Exploration.** Tile-based, turn-based movement: monsters step when you step,
notice you in their line of sight and give chase. Walking into a monster from
behind gives the party the first move; being caught can mean an ambush. The
lantern lights a radius around the party; torches, braziers, glowing fungus
and lava light the rest. Explored areas are remembered dimly and appear on the
minimap.

**Battles.** Timeline turn order (a forecast of the next eight turns is always
on screen). Speed and the weight of each action push a combatant's next turn
forward. Commands: Attack, Skills, Items, Defend (halves damage, recovers MP),
Flee, and Auto (the party plays itself sensibly — healing, reviving, exploiting
known weaknesses — for fast grinding). Seven elements; enemies can be weak,
resistant, immune, or absorb. Discovered weaknesses are remembered.

**Status effects.** Poison, Burn, Frozen, Stun, Blind, Weak, Sunder, Slow on
one side; Haste, Regen, Shield, Taunt, Focus, Might on the other. Fire thaws
ice; frost puts out burns; bosses shrug off most disabling effects.

**Party.** Wren (balanced, light magic, healing), Brannoc (tank: taunt,
armour-breaking, earthquakes), Maelis (elemental mage, haste, curses), Pip
(healer, revives, poison). Each learns 8–10 skills up to level 32; levels cap
at 50.

**Gear.** Weapons, armour and accessories in five tiers sold as the story
progresses, found in chests, or earned from quests (a legendary weapon for
each hero). Accessories add resistances, regeneration, critical chance,
first strike or ailment protection.

**Quests.** Eight side quests, four from townsfolk and four from companions,
each tied to a character arc and a unique reward.

**Waystones** at every floor's entrance: rest, save, or return to town. The
Vaultgate takes you back to any floor you have reached.

## Content

| | |
| --- | --- |
| Playable heroes | 4 |
| Skills | 100 (34 for heroes, the rest for monsters and bosses) |
| Monsters / bosses | 36 / 8 (one with two phases) |
| Items | 80 (12 consumables, 7 quest items, 61 pieces of equipment) |
| Floors | 20 procedural, in 5 biomes, plus the hand-built town |
| Story | 167 scenes, 975 lines (~16,000 words), 3 endings |
| Music | 16 original tracks (synthesized in engine), 34 sound effects |
| Sprites | 199, from the CC0 Dungeon Crawl Stone Soup set |

## Length

Measured with the automated player, which plays the real game through its
UI: it fights every monster, collects every shard and page, rests at
waystones, and finishes on the true ending. It never reads (it skips each
line of dialogue in a fraction of a second), never takes a turn by hand
(every battle is on Auto), and never walks back to town to shop.

| Milestone | Bot game time |
| --- | --- |
| Act I done (floor 5) | ~17 min |
| Act II done (floor 9) | ~48 min |
| Act III done (floor 13) | ~70 min |
| Act IV done (floor 17) | ~99 min |
| Credits (true ending) | ~2.4 h |

A person reading ~16,000 words of dialogue, choosing battle commands, and
shopping between acts adds well over an hour on top: the expected first
playthrough is **4–5 hours**, and longer for players who seek out every
side quest and memory by hand.

## Technology

- Rust, egui/eframe; one standalone executable per platform.
- Game rules are UI-free: battles emit events that the battle screen
  animates; the story runner executes script commands against game state.
- Content is data: `src/data` tables and `assets/story/*.story` scripts, both
  validated by tests.
- Quality gates: unit tests; a balance simulation that auto-plays all twenty
  floors and every boss; story completeness and consistency checks; map
  connectivity checks for every floor; and a bot that plays the real UI from
  the title screen to an ending.
