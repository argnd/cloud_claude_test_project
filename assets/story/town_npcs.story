# EMBERDEEP — Hollowmere townsfolk: ambient lines npc_<id>_<stage>.
# Stage = current act. 1: lanterns failing, Wren just went down. 2: after Gristlemaw, lanterns flicker back.
# 3: the truth is out, the square is tense. 4: people leaving. 5: snow in summer, a handful remain.
# Companions: Brannoc guaranteed from stage 2, Maelis from 3, Pip from 4. Earlier, gate with !if has_x.
# Ambient scenes may replay, so they never give items (except one flag-guarded gift from Bess).

# ---------------------------------------------------------------- Elder Marrow
=== npc_elder_1
!ifnot elder_met => elder_first
@elder: The Order is seeing to the lanterns, Wren. Every Warden before your sister kept the Flame faithfully. She will too.
@elder: She was the best of us. Remember that, whatever you hear.
@narrator: He does not look toward the Vaultgate as he says it. He looks everywhere else.

=== npc_elder_2
!if has_maelis => npc_elder_2_maelis
@elder: Captain Rennick tells me you've been through the Undercroft. And that you intend to go further.
@elder: The Archive below is flooded, and its records are unreliable. Water-damaged. I would trust nothing you read down there.
@narrator: His hands are shaking. He folds them behind his back.

=== npc_elder_2_maelis
@elder: Archivist Thorne. You're alive.
@maelis: Elder Marrow. You sound disappointed.
@elder: The Archive's records are water-damaged, Wren. Unreliable. I would trust nothing you read down there.
@maelis: Oh, I'd trust some of it.
@narrator: For a moment the Elder and the Archivist look at each other like two people holding the same knife.

=== npc_elder_3
!ifnot elder_confronted => elder_confront
@narrator: The Order hall is dark. Marrow sits alone at the long table. He has not lit a single lamp.
@elder: I can't light them. Every one I light, I see a name.
@elder: Go on, Wren. You don't need anything from me. You never did.

=== npc_elder_4
!ifnot elder_confronted => elder_confront
@narrator: Marrow is in the square in his shirtsleeves, lifting an old woman's trunk onto a cart. His chain of office is nowhere to be seen.
@elder: I can't give back what I took. But I can carry things. It turns out I'm good at carrying things. I've had the practice.
@elder: Your sister would have made a better Elder than me in a single afternoon.

=== npc_elder_5
!ifnot elder_confronted => elder_confront
@elder: I light the lanterns now, while you're below. Tobin is teaching me. He says I'm the worst he has ever seen.
@elder: When you find her, tell her I was wrong. And tell her Chapel Row is lit.

# ---------------------------------------------------------------- Bess
=== npc_bess_1
@bess: Wren, love! Sit. Eat. You're not going down any hole on an empty stomach, not while I've a pot on.
@bess: Frost in the miller's well this morning. In summer! And Hesta's cat's gone off, and Rennick says there's smugglers in the cellars. Under my inn!
@bess: Mind you, that would explain the brandy.

=== npc_bess_2
@bess: Brannoc Ironvein. Walking about in daylight. Sober. With a lamplighter. Hold me, I'm going to faint.
@brannoc: Don't.
@bess: You owe me for twelve years of ale, you old boulder. I'll take it in rescued sisters. Bring Ilsa home and we're square.
@brannoc: Deal.

=== npc_bess_3
@bess: I told them, love. Everyone. I know you didn't mean me to. But folk had a right to know what they were lighting lanterns for.
@bess: And you, Miss Archivist. You're all elbows. Sit down and eat something before I take it personally.
@maelis: I'm not sure I deserve —
@bess: Nobody deserves stew. That's what makes it stew. Sit.

=== npc_bess_4
@bess: Half my rooms are empty. Folk are going down the pass by the cartload. Well, I'm not. Somebody has to feed whoever's left.
@pip: Is that mushroom soup, Bread-lady?
@bess: It is, pet. Would you like a bowl?
@pip: That is somebody's cousin.

=== npc_bess_5
@bess: Snow in high summer. My mother would never have believed it.
!if bess_bread_given => npc_bess_5_again
@bess: Here. Hearth-bread, from the last of the good flour. Don't argue. I'd sooner it went down there with you than sat up here with me.
!give hearth_bread 3
!sfx ItemGet
!flag bess_bread_given
@bess: Come back and I'll bake more. That's a threat, love. Come back.

=== npc_bess_5_again
@bess: Come back, all of you. I've nobody else left to feed.

# ---------------------------------------------------------------- Dagna
=== npc_dagna_1
!if has_brannoc => npc_dagna_1_brannoc
@dagna: Varn. I mended your sister's lantern hook twice. She kept bending it on the chapel railings.
@dagna: Going down, are you? Then you'll want something sharper than hope. I sell that. Hope's extra.

=== npc_dagna_1_brannoc
@dagna: Well. Brannoc Ironvein. Forty-three years, and you walk into my forge smelling like a smuggler's boot.
@brannoc: Dagna.
@dagna: That's all? Forty-three years, and one word.
@brannoc: You look well.
@dagna: Three words. Careful, Ironvein. You'll turn my head.

=== npc_dagna_2
@dagna: Varn. Is he eating? He doesn't eat when he's brooding, and he's been brooding since before you were born.
@brannoc: Standing right here.
@dagna: I know. I can smell you. Have a bath, Ironvein. The Archive doesn't count.

=== npc_dagna_3
@dagna: Thirty years I made the Order's lantern hooks. Brackets for the Warden's hall. I polished their chain of office every spring.
@dagna: I'd like to melt the lot down. Make horseshoes. Something honest.
@brannoc: Make a doorstop. For the Order hall. Hold it open.
@dagna: That's the first clever thing you've said in forty-three years.

=== npc_dagna_4
@dagna: The Ember Forge. Ironvein, you swore you'd never set foot in it again.
@brannoc: Swore a lot of things.
@dagna: Then swear this one. You come back up those stairs. I'm not standing alone at another Lantern Night like a fool.
@brannoc: I swear it.

=== npc_dagna_5
@dagna: Half the town's gone down the pass. I'm staying. A forge doesn't pack.
@dagna: Tell that old fool I'm not leaving until he's back. And tell him it isn't sentiment. It's stubbornness.
@brannoc: She knows I'm right here.
@dagna: I'm aware.

# ---------------------------------------------------------------- Fen
=== npc_fen_1
@fen: Wren! Hello! Sorry. Hello. You're going down? Into the — of course you are, I heard, everyone heard, the whole square heard, you weren't quiet about it.
@fen: I measured the frost in three wells. It's climbing a thumb's width a day, which can't happen in summer, so it isn't weather, so it's — I don't know what it is. Tonic?

=== npc_fen_2
@fen: I went down into my own cellar today. On purpose. On my own. There was a rat.
@fen: I didn't scream. I made a noise, but it was a brave noise. Draughts? I tested them on myself. I'm fine. Mostly fine.

=== npc_fen_3
@fen: I did the sums. A Warden every thirty years for three hundred years. That's ten people. Eleven. More, with the ones who...
@fen: I made the tonics for every Warden's send-off. I put a sprig of mint in, so it would taste nice on the journey.
@fen: I'm keeping the shop open. Somebody should help people properly for once.

=== npc_fen_4
@fen: Everyone's leaving, and I'm staying. The Pale-sick can't walk down a mountain, so somebody has to stay with them.
@fen: I'm terrified all the time. I've decided that's just my face now.

=== npc_fen_5
@fen: I'm not scared any more. That's a lie. I'm very scared. I'm just here anyway.
@fen: Come back, all of you. I'll have tonics ready. A hundred of them. I don't sleep much now.

# ---------------------------------------------------------------- Sister Oriel
=== npc_oriel_1
@oriel: The Flame keep you, Wren. While the Flame burns, the Pale sleeps.
@oriel: I pray for your sister every night, by name. The Wardens are the Flame's own children. She is in good hands.

=== npc_oriel_2
@oriel: The lanterns on Chapel Row came back. I lit a candle in thanks. Then another. Then I ran out of candles.
@oriel: The Flame is good, Wren. I have to believe that. Otherwise, what have I been singing to all these years?

=== npc_oriel_3
@oriel: For twenty years I have sung the Wardens' hymn every evening. I thought it was a hymn of praise.
@oriel: It's a funeral song. It always was. I've been singing their funeral, and smiling while I did it.
@oriel: I don't know what I believe now. I only know I'm not stopping. Someone should sing for them and mean it.

=== npc_oriel_4
@oriel: I've taken the Flame's sigil down from the altar and put up the Wardens' names instead. Maelis gave me the list.
@oriel: The Flame was never holy, Wren. They were. The people who went down.
@maelis: I added footnotes. Oriel took them down.
@oriel: Gently.

=== npc_oriel_5
@oriel: Every evening I light a candle for each of you. Four candles, five with Ilsa. The wind keeps blowing them out, and I keep lighting them.
@oriel: I don't pray to the Flame any more. I pray for you. It turns out that's a better use of the time.

# ---------------------------------------------------------------- Captain Rennick
=== npc_rennick_1
@rennick: Varn. My whole night watch is three men, and two of them are asleep.
@rennick: The Vaultgate's shut by the Elder's order. Not that I'd see anyone going through. Terrible eyes. Night blindness. Came on very sudden.

=== npc_rennick_2
@rennick: Harlan's in my cells, singing. Twice a day. Three times on market days.
@rennick: I asked him to stop. He said a true showman never stops for an audience of one. I said I'm not an audience. He said they all say that.

=== npc_rennick_3
@rennick: I've a mob outside the Order hall, a hole in its window, and one guard who can't find his other boot.
@rennick: I don't blame them. I've half a mind to throw a stone myself. But somebody has to keep the peace, and it seems it's me.

=== npc_rennick_4
@rennick: The road down the pass is open, for now. I'm moving families out in whatever order they'll let me, which is no order at all.
@rennick: If you can end this, Varn, do it quickly. I'm running out of carts.

=== npc_rennick_5
@rennick: Somebody has to be last out of the gate. Captain's privilege.
@rennick: I've picked a spot on the wall with a good view of the Vaultgate. I'll see you come up. That's an order.

# ---------------------------------------------------------------- Old Hesta
=== npc_hesta_1
!if quest_lost_mouser_done => npc_hesta_1_mouser
@hesta: Back again, dearie? Your sister used to sit on my step and learn my song. Pretty voice. Flat on the high notes.
@hesta: The dark's not so bad when you've a tune to keep you company.

=== npc_hesta_1_mouser
@mouser: Mrrp.
@hesta: He says thank you. He doesn't mean it, but he says it.
@hesta: The dark's not so bad, dearie, when you've a tune to keep you company. Your sister knew that. Flat on the high notes, but she knew it.

=== npc_hesta_2
@hesta: You've something on you, dearie. Something from far down, that hums. I can't see it, but I can hear it.
@hesta: Hold on to it. Some things are only lost while nobody's holding them.

=== npc_hesta_3
@hesta: My mother said that song came from a healer, long ago, before there was any Order. He sang it to his little girl, and it went round the village after.
@hesta: Funny, what lasts. Kings and temples fall down, and a lullaby just keeps going.

=== npc_hesta_4
@hesta: They want me on a cart. I told them I'm too old to walk down a mountain and too stubborn to be carried.
@hesta: Besides, somebody has to be here to hear the end of that song. I've waited my whole life. I can wait a little longer.

=== npc_hesta_5
@hesta: The snow's singing, dearie. Can you hear it? It's that tune. It's all the snow knows.
@hesta: Bring me the rest of the song, if you find it. I'd like to hear how it ends before I do.

# ---------------------------------------------------------------- Tobin
=== npc_tobin_1
@tobin: Wren! I lit Chapel Row! All of it! Well, most of it. The first four. One of them was a door.
@tobin: When I'm big I'm going to be a lamplighter, like you and Ilsa. I've already got the walk.
@narrator: He demonstrates the walk. It is mostly swagger.

=== npc_tobin_2
@tobin: The lanterns came back! I've been lighting them every night, and they came back! Brannoc, can I hold your pick?
@brannoc: No.
@tobin: Can I know its name?
@brannoc: Hilde.
@tobin: That's the best name I ever heard.

=== npc_tobin_3
@tobin: Everyone's shouting at the Order hall. I shouted too. I shouted "you're rotten" through the keyhole.
@tobin: Ilsa's not a candle. She's Ilsa. She let me carry the ladder.

=== npc_tobin_4
@tobin: Mum says we're going down the pass tomorrow. I don't want to. Who's going to light the lanterns?
@tobin: I wrote down all the streets for you, in order. Chapel Row first. In case you forget.

=== npc_tobin_5
@tobin: Mum said we'd go on the last cart. Then Bess stayed, and Hesta stayed, and Mum said, well, then. So we stayed.
@tobin: I light every lantern in town now. Chapel Row first. Elder Marrow carries the ladder. I'm keeping them lit for you, like you kept them lit for Ilsa.

# ---------------------------------------------------------------- Gwen the Weaver (villager_a)
=== npc_villager_a_1
@villager_a: I'm weaving a shawl for your sister, Wren. For when she comes home. Lantern-gold, with a border of snowdrops.
@villager_a: Only the frost keeps getting into my loom. The threads go stiff and grey. I've never seen the like.

=== npc_villager_a_2
@villager_a: Three lanterns lit again on Chapel Row! I cried, Wren. In public. Over a lantern.
@villager_a: Ilsa's shawl is finished. It's waiting on the peg by my door.

=== npc_villager_a_3
@villager_a: My great-aunt Senna was a Warden. Her portrait hung over our hearth, and we said good morning to it every day.
@villager_a: She wasn't keeping anything. She was burning. And we said good morning to her.

=== npc_villager_a_4
@villager_a: I'm taking my loom down the pass. I can weave anywhere. I can't breathe here any more.
@villager_a: I've left Ilsa's shawl with Bess. Give it to her when you bring her home. Promise me.

=== npc_villager_a_5
@villager_a: I got as far as the foot of the pass, and then I turned the cart round. Don't ask me why.
@villager_a: Somebody ought to be here with a shawl when she comes home.

# ---------------------------------------------------------------- Osric the Miner (villager_b)
=== npc_villager_b_1
@villager_b: Frost in the Silverdeep shaft this morning. Forty years I've mined, and I've never seen frost below the fourth level.
@villager_b: Your da would have known what to make of it. Tomas Varn could smell a bad seam through a wall.

=== npc_villager_b_2
@villager_b: Is that Brannoc Ironvein walking about in daylight? Last time I saw him he was asleep in a horse trough.
@villager_b: Best miner in the Greyspine, once. Before the Forge. Buy him a drink. No. Don't.

=== npc_villager_b_3
@villager_b: They'd send a Warden down and ring the bell, and we'd all cheer. I cheered for your sister.
@villager_b: And my brother Garrick walked out of the mill at midwinter and never came back. Folk say he's gone Hollow.
@villager_b: Is he down there, Wren? No. Don't tell me. I'd rather keep hoping.

=== npc_villager_b_4
@villager_b: Frost in every shaft now. Can't dig ice, and can't feed children on it either. I'm sending the family down to Brackenford.
@villager_b: If you see Garrick down there, tell him... He won't hear you. Tell him anyway.

=== npc_villager_b_5
@villager_b: Sent the family down to Brackenford. I stayed. Somebody ought to be here if Garrick ever walks back up.
@villager_b: Can't mine snow, mind. I tried. Dug a tunnel to the inn instead. It's a very good tunnel.
@villager_b: Bring the sun back, lamplighter. I'd like to see my own feet again.

# ---------------------------------------------------------------- Gate Guard
=== npc_guard_1
@guard: Vaultgate's shut. Elder's orders. Nobody goes down.
@guard: I didn't see you, mind. I'm looking at this wall. It's a very interesting wall.

=== npc_guard_2
@guard: Didn't see you go down. Didn't see you come back up. Didn't see a dwarf, either. I don't see much, me.
@guard: Heard about the rat, though. Big as a cart, Tobin says. It gets bigger every time he tells it.

=== npc_guard_3
@guard: The Elder says the gate stays shut. The Captain says it stays open. The mob says burn it down.
@guard: I've decided I'm on my break. It's a long break. It started this morning.

=== npc_guard_4
@guard: Half the watch has gone down the pass. I'm my own relief now. I've relieved myself twice today.
@guard: That came out wrong.

=== npc_guard_5
@guard: I saw you this time.
@guard: Go on down. And come back up. I'll be watching for you, properly this time.
