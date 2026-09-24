# EMBERDEEP — Lore pickups: Memory Shards (Aurelian's life, in order) and Ilsa's journal.
# Shard floors: 1=F2 2=F3 3=F6 4=F7 5=F9 6=F10 7=F11 8=F13 9=F14 10=F15 11=F17 12=F18
# Journal floors: 1=F2 2=F7 3=F11 4=F15 5=F18
# Reactions only use companions guaranteed on that floor; others are gated with !if has_x.

=== shard_1
!sfx Shard
@narrator: The shard is no bigger than a thumbnail, and it glows like a coal that has forgotten how to be warm. When Wren touches it, the cellar falls away.
@narrator: A snowstorm, long ago. A stone cottage at the edge of Hollowmere, drifted to the eaves. Inside, one candle, and a man with a healer's steady hands.
@aurelian_memory: Breathe. Good. Once more. The midwife can't get through the drifts, love, so you'll have to make do with me.
@narrator: A cry: thin, furious, alive. The healer laughs, and then he is weeping, and he doesn't trouble to hide either.
@aurelian_memory: A girl. Look at her. Look at all that temper. Lira, just like you wanted.
@narrator: Her mother holds her until the candle burns down, and does not live to see the thaw. All that winter the healer keeps a light in the window, so the baby never wakes in the dark.
@wren: A light in the window, so she'd never wake in the dark. Who were you?

=== shard_2
!sfx Shard
@narrator: A small bedroom under the eaves. A girl of five, with hair like a haystack, is fighting sleep with everything she has.
@lira: One more song. Then I'll sleep. Probably.
@aurelian_memory: You said that three songs ago, little ember.
@aurelian_memory: "Hush now, little ember, the long night's nearly through. I'll keep the lantern burning, till morning comes for you."
@lira: Now sing my part.
@aurelian_memory: Your part is the answer. You'll learn it when you're bigger, and then you can sing it back to me.
@lira: I'm bigger now. I got bigger during the song.
@wren: That's Hesta's tune. Ilsa sang it to me every night after Mother died. The same words. She called me little ember too.
!if has_brannoc => shard_2_brannoc

=== shard_2_brannoc
@brannoc: Something in my eye. Ash. From somewhere.

=== shard_3
!sfx Shard
@narrator: Spring. A healer's kitchen, buried in scraps of coloured paper and pots of paste, and one extremely serious seven-year-old.
@lira: It's a lantern. For the window. So the sick people can find your door at night.
@aurelian_memory: Why paper, Lira? Paper burns.
@lira: So the light can get out, Papa. Glass keeps it in. Paper lets it go.
@narrator: By summer every window on the lane has one of Lira's lanterns. By autumn the whole village lights them at dusk, and nobody remembers whose idea it was.
@wren: Lanterns in the windows, so the sick can find the door. That's why we light them. That's where it started. With her.
!if has_maelis => shard_3_maelis

=== shard_3_maelis
@maelis: The Order's charter calls the lighting of lanterns "a sacred ordinance handed down by the Flame". Footnote: it was a seven-year-old with a paste pot.

=== shard_4
!sfx Shard
@narrator: Autumn. Overnight, the wells of the lower town turn grey: grey as ash, grey as old dishwater. Then the coughing starts.
@aurelian_memory: Boil every drop. Burn the bedding. Lira, you're to stay upstairs. Do you hear me? Upstairs.
@lira: But who'll make the lanterns for the sick-houses?
@aurelian_memory: Make them upstairs, then. Fold them small. I'll hang them myself.
@narrator: That night twelve paper lanterns hang in the windows of the lower town, and twelve families find the healer's door.
@maelis: Greywater Fever. It took a third of Hollowmere. The records name one healer who never closed his door: Aurelian. Then they stop mentioning him, as if someone took a knife to the page.
@wren: Grey water in the wells. Like the frost in ours.

=== shard_5
!sfx Shard
@narrator: Winter again. The healer's house is dark. He has not been home in three weeks. He sleeps in chairs in other people's houses, when he sleeps at all.
@narrator: Every night a small figure climbs onto the sill of the dark house and lights a paper lantern, so her father can find his way home.
@lira: Papa's coming home tonight. Probably. Papa always comes home.
@aurelian_memory: One more house. One more, and then I'll go home to her. Just one more.
@maelis: The ledger's first page says the Flame was lit "to bind the grief of the healer Aurelian". Wren, the Pale isn't a curse or a god. I think it's him. What's left of him.
@wren: Then these shards are what the Pale forgot.

=== shard_6
!sfx Shard
@narrator: Lira's last night. The room under the eaves is very warm and very quiet. The healer sits on the edge of the bed. He has not slept in days, and he will not sleep tonight.
@lira: Sing it, Papa. The ember song.
@aurelian_memory: "Hush now, little ember, the long night's nearly through. I'll keep the lantern burning, till morning comes for you."
@lira: I learned my part. I was saving it for when I got bigger. "Rest now, lantern-keeper..."
@lira: I'm too sleepy. I'll sing it tomorrow. I promise.
@aurelian_memory: Tomorrow, then. Sleep, little ember. I'm right here.
@narrator: He keeps the lantern burning until morning comes. She does not.
@wren: "Tomorrow." She promised him tomorrow.
!if has_pip => shard_6_pip

=== shard_6_pip
@pip: Her song stopped in the middle. Songs shouldn't stop in the middle. It hurts them.

=== shard_7
!sfx Shard
@narrator: The funeral is small, and snow falls on it. The healer stands at the grave and does not speak or move. When everyone else has gone home, he is still standing there.
@narrator: That night, for the first time in seven years, no paper lantern hangs in the healer's window. The village lights theirs anyway, all down the lane, for her. He cannot look at them.
@aurelian_memory: Such a quiet house. I never knew a house could be so quiet.
@pip: When a glowroot dies, the others lean toward where it was. For a long, long time. They can't help it.
@brannoc: Aye. Folk do too.

=== shard_8
!sfx Shard
@narrator: Spring comes, and the healer does not notice. He reads: old books, forbidden books, travellers' tales traded at the back door for medicine.
@aurelian_memory: "At the bottom of the world lies the Well of Returning. What is lost may be asked for there." Asked for. Not given. Asked.
@narrator: He reads that page until it is soft as cloth. Beside the book sits the last paper lantern Lira ever made. It is a little crooked. He will not let anyone straighten it.
@brannoc: Every miner's heard of that Well. The ones with sense stayed up top.
@wren: The ones with sense hadn't lost what he had.

=== shard_9
!sfx Shard
@narrator: The Vaultgate, three hundred years ago: a raw hole in the mountain, no gate at all. Three of the healer's friends stand at its edge in the dusk, pleading.
@narrator: Aurelian has a pack on his back and Lira's crooked lantern in his hand. He has lit it. It is the first light he has lit in a year.
@aurelian_memory: I know what you'll say. Grief passes. Time heals. I'm a healer. I've said it myself, to a hundred families.
@aurelian_memory: I was lying. I just didn't know it.
@narrator: He goes down. His friends light a fire at the top of the stair so he can find his way back. He does not come back.
@maelis: He went down after someone he'd lost, carrying a lantern, while the people who loved him kept a light burning up top.
@wren: I know what that sounds like, Maelis. I know.

=== shard_10
!sfx Shard
@narrator: The bottom of the world. A well, wide and silver and still. Aurelian kneels at its edge, and the paper lantern in his hand is nearly burned through.
@aurelian_memory: You are the Well of Returning. So return her. Give her back.
@narrator: The Well answers without a voice, and the answer settles on him like snow: It cannot return her. It can only keep you from forgetting.
@aurelian_memory: Then let me never forget. Let it hurt forever. Let it hurt so much it never, ever goes away. The pain is all I have left of her.
@narrator: The Well takes his grief and his wish, and makes them vast.
@pip: Oh. Oh, no. He asked it to keep the hurt. You mustn't ever ask for that. Hurt is the fastest-growing thing there is.
@maelis: And the Well kept its word. That's the cruelty of it.

=== shard_11
!sfx Shard
@narrator: Years pass above. Below, the grief grows. It forgets it was ever a man. It knows only that it is cold, and that warmth is a thing other people are allowed to keep.
@narrator: The first frost climbs the stair. The wells of Hollowmere freeze in high summer. People begin to forget their own names.
@aurelian_memory: Warm... so warm up there... give it to me...
@narrator: His three friends carry fire all the way down to the Well, and build a flame that can hold him. They know what it needs. His apprentice, Edda, steps forward first.
@narrator: They weep as they chain him. Edda weeps as she walks into the fire. The Pale sleeps. The Lantern Order is born that night, out of three people who loved him.
@maelis: Edda. The first Warden. I never knew why she volunteered.
@wren: Because she loved him. They all did. That's the worst of it. They did all this out of love.

=== shard_12
!sfx Shard
@narrator: Almost nothing of the man is left now. His name went first. Then her face. Then the snowstorm, and the lanterns, and the song.
@narrator: One thing holds on longest, deep in the cold, like the last coal in a dead fire. A summer afternoon. The healer folding a paper lantern with big clumsy fingers, and making a frog by mistake.
@lira: Papa! That's a FROG!
@narrator: Lira laughs. She laughs until she falls off her chair, and then she laughs on the floor, bright and helpless, like a struck match.
@aurelian_memory: There. There it is. Oh, I'd almost forgotten — no. No, don't go. Please don't go.
@narrator: The laugh goes out. It was the last thing he forgot.
@wren: I'm keeping this one safe. I'm going to give it back to him. I swear I will.

=== journal_1
@ilsa: Wren. I'm writing this on the back of a lamp roster, on the second cellar stair, because I forgot paper and I am not going back up for it.
@ilsa: I'll leave a page at every waystone and collect them on the way home. Then you can have the whole bundle and laugh at my spelling.
@ilsa: I'm not afraid. That's a lie. I'm afraid and I'm going anyway, which Da always said was the same thing, only with more walking.
@ilsa: Chapel Row first. Not too early, or it's out by midnight. You know this. I'm telling you anyway.
@ilsa: Keep Old Tom lit. I'll see you when the Flame is tended. — I.

=== journal_2
@ilsa: Wren. The Archive is flooded. Everything smells of wet paper, and there are fish in the theology section.
@ilsa: I found the Wardens' roll. Three hundred years of names, and every one ends the same way: "Descended. Keeps the Flame." Nothing after. Not a single letter home.
@ilsa: Thirty years is a long time not to write to anyone. Even I'd manage one letter in thirty years.
@ilsa: There's a sealed room on the stair with a light under the door. I knocked. Someone's in there. They didn't answer.
@ilsa: I hummed Hesta's song as I went past, to keep my courage up. I think whoever it was held their breath to listen.
@ilsa: Something is wrong down here. I'm going to find out what.

=== journal_3
@ilsa: I've carried this page down three floors, because I couldn't make myself write on it.
@ilsa: The Warden doesn't tend the Flame. The Warden is the Flame. Burned slowly, for thirty years, so the Pale stays asleep.
@ilsa: The Curator let me read the ledger. He said knowledge burns the hands that hold it. He's right. I'm holding it anyway.
@ilsa: I thought of you lighting Chapel Row for thirty years with my name on every lamp, and I sat down in the water and couldn't get up for a long time.
@ilsa: Then I got up. The ledger's first page says the Flame was lit "to bind the grief of the healer Aurelian". A man. Not a monster.
@ilsa: If it was a man, then maybe someone can talk to him.

=== journal_4
@ilsa: I won't burn. I've decided. I'm going to find the Pale and look it in the face and ask it what it wants.
@ilsa: The Flame is failing without me. I can feel the cold getting stronger behind me. So I'll have to be quick. I've always been quick.
@ilsa: There are dwarven tags in the ash here. Ironvein. I keep thinking of Brannoc at the Lark, who never talks about the Forge. I understand him now. Tell him so.
@ilsa: There has to be another way, Wren. Someone chained a man's grief with a fire. Then someone can find the man.
@ilsa: Onward.

=== journal_5
@ilsa: Wren. Wren. Wren. I'm writing your name so I keep it.
@ilsa: The cold is inside my ribs now. It talks to me. It sounds so tired.
@ilsa: I keep forgetting Mother's face. I try and try, and it's only a coat on a hook. Da's is going too. His laugh like a rockfall.
@ilsa: I will not forget yours. The freckles. The scar on your chin from the chandler's fence. The way you scowl when you're frightened.
@ilsa: I'm going to hold the last door. It's all I can do now. I hum the song, and it helps.
@ilsa: If you find this, go home. I know you won't. Wren. Wren. Wren.
