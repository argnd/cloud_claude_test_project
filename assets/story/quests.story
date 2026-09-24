# EMBERDEEP — Side quests. Giver quests: offer / remind / done. Party quests start in join scenes; done scenes here.

# ---------------------------------------------------------------- Lost Mouser (Old Hesta, stage 1)
=== quest_lost_mouser_offer
!if mouser_found => quest_lost_mouser_offer_found
@hesta: Is that little Wren Varn? Come closer, dearie, my eyes are all fog. Ah. There you are. You've your mother's frown.
@hesta: Mouser's gone. Ran off down the cellar steps a week ago, after something only he could see. He never goes down. Never.
@hesta: If you're going below, and you are, I can hear it in your boots, keep an eye out for him. Grey. One ear torn. Hates everybody.
@wren: I'll bring him home, Hesta.
!quest start lost_mouser
@narrator: As Wren turns to go, Hesta starts to hum: a slow, rocking tune, older than the houses. Wren has known it all her life.

=== quest_lost_mouser_offer_found
@hesta: Is that little Wren Varn? And what's that grumbling in your pack? I know that grumble. MOUSER!
!quest start lost_mouser
!goto quest_lost_mouser_done

=== quest_lost_mouser_remind
@hesta: Any sign of my Mouser, dearie? Grey, one ear, hates everybody.
@hesta: He'll be somewhere cold, sulking. He likes to be found. He'd never admit it.

=== quest_lost_mouser_done
@narrator: Mouser climbs out of Wren's pack, stretches, and walks into Hesta's lap as though he had never been away.
@mouser: Mrrp.
@hesta: There's my wicked boy. Down in the dark with smugglers and rats, and not a whisker out of place.
@hesta: Thank you, dearie. Take this. It was my grandmother's. She said it keeps the cold off. I've never been cold a day in my life, so it must work.
!quest done lost_mouser
!give hesta_charm
!sfx ItemGet
@narrator: Hesta strokes the cat and begins to sing, soft and cracked.
@hesta: "Hush now, little ember, the long night's nearly through. I'll keep the lantern burning, till morning comes for you."
@wren: Ilsa used to sing me that. It always feels like it stops halfway. Is there more?
@hesta: There's more. Nobody's known it for a long, long time. My gran used to say the song was waiting for its answer.

# ---------------------------------------------------------------- Smugglers (Captain Rennick, stage 1)
=== quest_smugglers_offer
@rennick: Varn. I hear you're going down. I won't stop you. I haven't the men to stop anyone.
@rennick: Smugglers have been using the Undercroft to run goods under the town. Their boss is Vex Harlan. Fancies himself a showman.
@rennick: If you run into him, put him down and I'll make it worth your while. Out of my own pocket. It's a small pocket, but it's honest.
!quest start smugglers

=== quest_smugglers_remind
@rennick: Harlan's still down there. My men hear him singing through the drains at night.
@rennick: Deal with him, Varn, before I have to arrest him for crimes against music.

=== quest_smugglers_done
@rennick: My watch found Vex Harlan tied to a chair with his own rope, courtesy of the lamplighter. They are never going to live it down.
@rennick: He's singing now. Names, routes, stashes. Can't shut him up.
@rennick: The purse is the town's. The mail is mine. I wore it when I was young and quick. Better it keeps you alive than rusts on my wall.
!quest done smugglers
!give guard_mail
!gold 200
!sfx Coin

# ---------------------------------------------------------------- Holy Relics (Sister Oriel, stage 2+)
=== quest_holy_relics_offer
!if truth_known => quest_holy_relics_offer_truth
@oriel: Wren. I've been praying for the lanterns every hour. I think the lanterns are tired of me.
@oriel: The temple scrolls tell of three relics of the first Wardens, lost in the Deep long ago: a ring, a bell, and a lantern hook.
@oriel: If you find them, bring them home. Holy things, back on the altar. Perhaps the Flame will remember what it owes us.
!quest start holy_relics

=== quest_holy_relics_offer_truth
@oriel: Wren. I don't know what to pray to any more. I've been sitting here trying.
@oriel: Three relics of the first Wardens lie somewhere in the Deep: a ring, a bell, and a lantern hook. I used to call them holy.
@oriel: Now I think they're all that's left of three people who were burned so we could sleep. They should come home. Will you bring them?
!quest start holy_relics

=== quest_holy_relics_remind
@oriel: A ring, a bell, and a lantern hook. The relics of the first Wardens. If you find them, please, bring them home.

=== quest_holy_relics_done
!take holy_relic 3
!if truth_known => quest_holy_relics_done_truth
@oriel: The ring, the bell, the hook. Oh, Wren. The first Wardens, home at last.
@oriel: Take this: the Amulet of Dawn, the oldest treasure of the temple. They say it holds a spark of the first morning. It should go with someone walking toward the Flame.
!quest done holy_relics
!give amulet_of_dawn
!sfx ItemGet

=== quest_holy_relics_done_truth
@narrator: Oriel takes the relics one by one: a plain ring, a small cracked bell, a lantern hook worn smooth by some long-dead hand.
@oriel: They were people. Someone wore this ring on their wedding day. Someone rang this bell to call their children in for supper.
@oriel: I won't put them on the altar. I'll bury them in the garden under their own names. Maelis found them for me: Edda, Corin and Wynne.
@oriel: Take this. The Amulet of Dawn. I used to say it held a spark of the first morning. Now I only hope it does. Hope will have to do.
!quest done holy_relics
!give amulet_of_dawn
!sfx ItemGet

# ---------------------------------------------------------------- Tobin's Glowcap (Tobin, stage 3+)
=== quest_tobins_glowcap_offer
@tobin: Wren, is it true? What they're saying about Ilsa? That the Flame... that she was meant to...
@wren: It's true. But she didn't. She said no. She's still down there, fighting it.
@tobin: Good. GOOD. I'd have bitten them.
@tobin: I've got a plan. Fen says there's mushrooms down there that glow for a hundred years. Glowcaps. Bring me one, and I'll make a lantern that never goes out.
@tobin: And then nobody ever has to go down and be the fire again. Not ever.
@wren: I'll find you the best one down there.
!quest start tobins_glowcap

=== quest_tobins_glowcap_remind
@tobin: Did you find a glowcap yet? I've got the lantern ready. I made it out of a jam jar. It's a very good jam jar.

=== quest_tobins_glowcap_done
!take glowcap 1
@narrator: Tobin drops the glowcap into his jam jar and screws the lid down tight. The mushroom glows a soft, steady blue.
@tobin: Look! LOOK! It doesn't need oil, it doesn't need a wick, it doesn't need anybody!
@tobin: I'm hanging it on the Vaultgate, so Ilsa sees it when she comes home.
@tobin: And this is for you. It's a bit of the big Vault lantern. It cracked when it went out. I made it into a charm. For luck.
!quest done tobins_glowcap
!give tobin_charm
!sfx ItemGet
@wren: Tobin, this is the best lantern I've ever seen.
@tobin: I KNOW. I'm going to be the best lamplighter ever. After you. And Ilsa. Third best. That's still very good.

# ---------------------------------------------------------------- Starmetal (Dagna, stage 4)
=== quest_starmetal_offer
@dagna: Varn. That blade of yours is a butter knife with ambitions. You're taking it into the Ember Forge?
@dagna: The Forge golems carry starmetal in their chests. Old dwarven work. Bring me five pieces of the ore and I'll make you a real sword. One fit for a lamplighter.
@wren: What makes a sword fit for a lamplighter?
@dagna: It glows, obviously. I'm a smith, girl, not a poet.
!quest start starmetal

=== quest_starmetal_remind
@dagna: Five starmetal ore. From the golems' chests, not their elbows. I can tell the difference, and so will your sword.

=== quest_starmetal_done
!take starmetal_ore 5
!if has_brannoc => quest_starmetal_done_brannoc
@dagna: Five. Good ore. Clean. Give me a night.
!fade
@dagna: Emberbrand. Holds a flame along the edge, and never dulls in the cold. Don't drop it in a snowdrift.
!quest done starmetal
!give emberbrand
!sfx ItemGet

=== quest_starmetal_done_brannoc
@dagna: Five. Good ore. Ironvein, get over here and work the bellows. You're the only one in town with the lungs for it.
@brannoc: Aye.
!fade
@narrator: They work through the night without a word. Neither of them needs one. At dawn the blade comes out of the quench singing.
@dagna: Emberbrand. Holds a flame on the edge, never dulls in the cold. Best thing I've ever made.
@brannoc: Best thing you've made this year.
@dagna: Best thing, Ironvein. You've no taste.
@brannoc: Courted you, didn't I?
@narrator: Dagna hits him with the tongs. She is smiling when she does it.
!quest done starmetal
!give emberbrand
!sfx ItemGet

# ---------------------------------------------------------------- Lost Pages (Maelis; started in maelis_meet)
=== quest_lost_pages_done
!take lost_page 4
@maelis: Four pages. Soaked, torn, and one of them has a snail on it. Exactly as I lost them.
@narrator: Maelis sits up all night with ink, thread and an expression of ferocious joy. By morning the Forbidden Index is whole again.
!ifnot truth_known => quest_lost_pages_done_early
@maelis: It isn't only an index. It's a roll. Every Warden the Order ever sent down: their names, their years, what they liked for breakfast.
@maelis: The Order called them "the stars". Far away, very bright, and nobody's problem. So: the Codex of Stars.
@maelis: I'll carry it into every fight from now on. They ought to see how this ends.
!quest done lost_pages
!give codex_of_stars
!sfx ItemGet

=== quest_lost_pages_done_early
@maelis: The Order called the names in it "the stars". It's the Codex of Stars now. I'll carry it into every fight.
@wren: What names?
@maelis: I'll explain. Soon. I promise. I'm told that's a thing people say before they explain.
!quest done lost_pages
!give codex_of_stars
!sfx ItemGet

# ---------------------------------------------------------------- Everbloom (Pip; started in pip_meet)
=== quest_everbloom_done
@narrator: Pip searches a long time for the right place. At last he finds it: a crack in the stone where a thin thread of real light falls through.
@pip: Here. Light, and stone, and a little bit of damp. The Mother would say it's perfect.
!take everbloom_seed 1
@narrator: He digs a hole with his fingers, settles the seed inside, and pats the earth down. Then he sits back and waits.
@narrator: Nothing happens. Then a single green thread, finer than hair, uncurls toward the light.
@pip: Hello, little grove. I'm Pip. I'm your tender. Take your time. "Not yet" is a good word.
@pip: The old grove left me her last branch. It's a staff now. It still remembers how to grow.
!quest done everbloom
!give grovewood_staff
!sfx ItemGet

# ---------------------------------------------------------------- Crew Tags (Brannoc; started on floor 13)
=== quest_crew_tags_done
!take crew_tag 3
@narrator: Brannoc lays three iron tags on the stone, side by side, and wipes the ash from each with his thumb.
@brannoc: Kell Ironvein. Durra Ironvein. Fisk Ironvein.
@brannoc: That's all of you. Going home.
@narrator: He sits with them a long time. Nobody speaks. Pip leans against his arm, very lightly, the way moss leans against a stone.
@brannoc: Found this by Fisk's tag. My da's axe. Dropped it when I ran, twelve years back. Fisk must have picked it up. Stubborn old goat.
@brannoc: Not running this time.
!quest done crew_tags
!give ironvein_axe
!sfx ItemGet
@pip: Will Hilde be jealous?
@brannoc: Hilde understands.
