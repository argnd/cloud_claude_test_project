# EMBERDEEP — Act I: The Undercroft (floors 1-4). Party: Wren alone; Brannoc joins on floor 3.

=== floor_1_enter
!music Undercroft
@narrator: The Undercroft: the cellars beneath Hollowmere, dug by brewers and hoarders and people with things to hide, then dug again by their grandchildren.
@narrator: Barrel-vaults sweat with frost. The air smells of old ale and older mould. Somewhere ahead, a great many small feet are running away from something.
@wren: Chalk marks on the walls. Arrows, crosses, numbers. Someone's been mapping these tunnels, and not long ago.
@wren: Smugglers. Captain Rennick said as much. Wonderful. Rats, dark, and criminals.
@wren: Stay lit, Tom. That's all I ask of you.

=== floor_2_enter
@narrator: The second cellar-level. The smugglers' chalk grows bolder here, and the frost thicker.
@wren: Tom, if you see anything with more than four legs, you tell me first.
@narrator: From somewhere in the dark comes a small, deeply offended "mrrow".
@wren: That's either a cat or the least frightening monster in the Deep.

=== mouser_found
@narrator: On top of a smuggler's crate, puffed to twice his size, sits a grey cat with one torn ear and an expression of total contempt.
@mouser: Mrrrow.
!ifnot quest_lost_mouser_active => mouser_found_unasked
@wren: Mouser! Hesta's been worried sick about you, you horrible thing.
!goto mouser_found_common

=== mouser_found_unasked
@wren: Grey coat, one torn ear, appalling manners. You're Old Hesta's Mouser. She'll be beside herself.
!goto mouser_found_common

=== mouser_found_common
@mouser: Hsss.
@wren: I'm down here to rescue my sister. I do not have time to be hissed at by a cat who ran away into a haunted cellar.
@narrator: Mouser considers this. Then he climbs into Wren's pack and settles in as if he had booked the room.
@wren: Fine. But you're not getting the good tonic.
!sfx ItemGet
!flag mouser_found

=== floor_3_enter
@narrator: The cellars give way to rough tunnels, shored up with stolen timber. Crates are stacked to the roof, stamped with half the merchant marks in the Greyspine.
@narrator: Torchlight ahead. Voices. And someone singing a dwarven drinking song, very badly, with all the wrong words on purpose.
@wren: Smugglers. And a prisoner, by the sound of it. Tom, whatever happens, try to look intimidating.

=== vex_pre
!music Boss
@narrator: A torchlit cavern heaped with stolen goods. In the middle, tied to a chair with enough rope to moor a ship, sits a very large, very grumpy dwarf.
@vex: — and the fourth verse, if you please, Master Ironvein, does not go "Vex Harlan has a face like a dropped pie".
@brannoc: Does in mine.
@vex: Oh! A guest! Welcome to the Undercroft Exchange, finest market beneath Hollowmere. Vex Harlan: proprietor, visionary, and, when the market demands it, murderer.
@wren: You're the smuggler Captain Rennick's been hunting.
@vex: "Smuggler". Such a small, grubby word. I prefer "unlicensed merchant of the depths". It fits better on a sign.
@wren: Why is that dwarf tied to a chair?
@vex: Master Ironvein knows the old roads down to the Ember Forge. Dwarven gold, dwarven steel, dwarven everything. He's going to guide us. Aren't you, dear?
@brannoc: No.
@vex: He's said that for four days. It's very tiresome. Lads! The lamplighter has seen far too much. Put her out.
@wren: I light lanterns for a living. You'll find I'm hard to put out.
!sfx BossRoar
!battle vex
!music Undercroft
@narrator: Vex Harlan sits in a heap of his own merchandise, clutching a nose that now points somewhere new.
@vex: My face! You've ruined my face! It was my best asset!
@wren: It'll match your personality now.
@narrator: Wren ties him to his own chair with Brannoc's rope. There is plenty left over.
!goto brannoc_join

=== brannoc_join
@narrator: Wren saws through the last knot. The dwarf stands, cracks his neck one side and then the other, and looks her up and down.
@brannoc: Varn. The lamplighter. Ilsa's sister.
@wren: You know Ilsa?
@brannoc: Lit the lamp outside the Lark every night. Told me to go home every night. Never did.
@brannoc: Brannoc Ironvein. Obliged.
@wren: You're the one who came back from the Forge. At the inn, everyone says —
@brannoc: Everyone at the inn talks too much.
@wren: I'm going down. All the way. To find her.
@narrator: Brannoc looks at the dark mouth of the tunnel for a long time, the way a man looks at a grave he has been walking around for years.
@brannoc: Twelve years I've been drinking so I'd never go back down there.
@brannoc: Can't let Ilsa's little sister go alone, though. She'd haunt me. Worse than the others do.
!join brannoc
!sfx LevelUp
@wren: The others?
@brannoc: Walk.

=== floor_4_enter
@narrator: The tunnels slope downward, and the rats come up: hundreds of them, pouring past Wren's boots without a glance.
@brannoc: Rats running uphill. Bad sign.
@wren: Worse than rats running at us?
@brannoc: Rats run from two things. Floods, and bigger things.
@wren: Very reassuring. Thank you, Brannoc. Did you hear that, Tom? Bigger things.
@brannoc: You talk to your lantern.
@wren: He was my father's. Da named him after himself. Said he was the only Tom in the house who never complained.
@brannoc: Hm. My pick's called Hilde. Tell anyone, I'll deny it.

=== gristlemaw_pre
!music Boss
@narrator: The tunnel opens into an old cistern, and the cistern is full of rats. They are not running any more. They are waiting.
@narrator: On a throne of bones and bottle-glass squats something that was a rat once. It is the size of a hay cart. Pale veins glow under its hide like cracks in river ice.
@gristlemaw: SKREEEEEE!
@brannoc: That's the bigger thing.
@wren: Tom, forget what I said about more than four legs. Four is plenty.
@wren: Look at its fur, Brannoc. That's frost. The same frost that's on the wells.
@narrator: Gristlemaw the Rat King rears up, and every rat in the cistern screams with it.
@gristlemaw: Skrr... SKREEEEEEEEEEE!
!sfx BossRoar
!shake
!battle gristlemaw

=== gristlemaw_post
!music Undercroft
@narrator: Gristlemaw collapses. The pale light drains out of it like water from a cracked jug, and what's left is only a very large, very old rat.
@wren: It wasn't born like that. Something down here did that to it.
@brannoc: Same thing the rats were running from.
@narrator: Behind the throne, the cistern wall has split. Cold air breathes through the crack, smelling of wet paper and old ink.
@brannoc: That's the Archive. Order's library. Flooded out two years back.
@wren: Then that's our way down.
@brannoc: Water. Stone and ember. It had to be water.
@narrator: Far above, three lanterns on Chapel Row flicker back to life. No one in Hollowmere knows why. Tobin takes the credit.
