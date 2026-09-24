# EMBERDEEP — Act V: The Pale Reach (floors 17-20). Party: Wren, Brannoc, Maelis, Pip.

=== floor_17_enter
!music PaleReach
@narrator: The Pale Reach. The stair ends at the lip of an abyss, and the abyss is full of snow falling upward, slow and silent, into a sky that isn't there.
@narrator: There is no colour here. Wren's red scarf has gone the grey of old ash. Pip's moss is white. Even Old Tom's flame burns thin and pale, like moonlight through a window.
@pip: Lantern-friend, my leaves are cold. They've never been cold before.
@maelis: Keep talking, all of you. About anything. The Pale takes the quiet first.
@brannoc: Don't like talking.
@maelis: Then complain. You have a gift for it.
@brannoc: Hate this place. Hate the cold. Hate snow that goes the wrong way.
@maelis: Wonderful. Keep going.

=== floor_18_enter
@maelis: The little note at the bottom of a page. What is it called? I've used the word every day of my life, and it's gone.
@pip: Footnote! It's footnote, Book-lady. I'll remember it for you.
@brannoc: The smith. Up top. Her name's... Stone and ember. Dagna. Her name is Dagna.
@wren: Everyone. One thing you'll never forget. Out loud. Now.
@brannoc: Kell's singing.
@maelis: The smell of a new book.
@pip: The Mother saying "not yet".
@wren: Ilsa burning the porridge. Calling it toasted.

=== ilsa_lantern
@narrator: Half buried in the snow lies a lamplighter's lantern. The glass is cracked. The brass is worn smooth in one place, just where a thumb would rest.
@wren: That's hers. That's Ilsa's.
@narrator: Wren lifts it. The metal is bitter cold, but when she cups it in her hands there is warmth still at its heart. Faint. Stubborn.
@wren: Still warm, Tom. She's still warm.
!give ilsa_lantern
!sfx ItemGet

=== floor_19_enter
@narrator: The last door before the bottom of the world. The cold here has a voice, and it is humming a tune Wren has known all her life.
@wren: That's the lullaby. Hesta's song. Ilsa's song.
@pip: Somebody is singing to keep warm.
@brannoc: Or something's singing to draw us in.
@wren: It's her. I'd know those flat high notes anywhere.

=== ilsa_pre
!music Boss
@narrator: Ilsa Varn stands before the last door with her back pressed to it, as if she has held it shut with her body for a year. Perhaps she has.
@narrator: Frost crowns her head in a ragged circlet of ice. Half of her face is Ilsa. The other half is snow.
@wren: Ilsa!
@ilsa: Wren? No. No, no. You're not here. You're up top. Chapel Row first. I told you —
@ilsa_pale: Little ember. Come closer. Come and be cold with us.
@ilsa: Don't listen. It wears my voice. Wren, run.
@ilsa_pale: Why run? You came all this way to keep her. So keep her. Keep her in the cold, forever, where nothing is ever lost again.
@wren: Let her go.
@ilsa_pale: She holds the door. She holds us. And we are so tired of being held.
@ilsa: Wren. If you fight me, fight properly. Don't you dare go easy on me. You never could beat me at anything.
@wren: I beat you at cards every winter.
@ilsa: I let you. Now. Please.
!sfx BossRoar
!shake
!battle ilsa

=== ilsa_post
!music Story
@narrator: The frost-crown shatters. Ilsa falls, Wren catches her, and for a while they are both on their knees in the snow, holding on.
@wren: I've got you. I've got you. You're so cold.
@ilsa: You came all the way down. I told you not to.
@wren: You told me to keep Old Tom lit so you could follow the light home. You were taking too long.
@ilsa: You look taller.
@wren: It's the boots. They're yours.
@narrator: Ilsa laughs, and the laugh breaks in the middle, and she holds her sister very tight.
@ilsa: I read the ledger. I knew what I was meant to do, and I couldn't. I kept seeing you lighting Chapel Row alone, for thirty years, with my name on every lamp.
@ilsa: So I came down to find it. The Pale. Wren, it's a man. A father. His name is Aurelian. He lost his little girl, and it hurt so much it swallowed the world.
@ilsa: He doesn't even remember her now. Only that it hurts.
@wren: Then come with me. We'll finish it together.
@ilsa: I can't. If I let go of this door, everything below comes up at once. Someone has to hold it while you go down.
@wren: I only just got you back.
@ilsa: You have me. Whatever happens. Listen: down there, it will try to make you forget. Me, Tom, your own name. It wants everyone as empty as it is.
@ilsa: Don't let it make you forget.
@wren: I won't. Not you. Not ever.
@ilsa: Brannoc Ironvein, out of the Lark at last. Look after her, all of you. She's scared of the dark. She's braver than any of us.
@brannoc: You kept telling me to go home. Took the long way.
@pip: We're her grove now, Lantern-sister. We'll keep her.
@maelis: Ilsa. It was me, behind the door. When you knocked.
@ilsa: I know. I heard you breathing. You're here now. That counts.
!heal
@narrator: Ilsa sets her back against the door, closes her eyes, and begins to hum. The old tune. The one with the lantern in it.

=== floor_20_enter
!music PaleReach
@narrator: The bottom of the world. A round chamber of black stone, and at its centre a well as wide as a village square, brimming with still, silver water. The Well of Returning.
@narrator: Above it, in an iron brazier, the Hearthflame gutters: a flame the size of a candle where there should be a bonfire. It is the only colour left anywhere.
@pip: The fire is crying.
@maelis: Three hundred years. Every one of them, burned here.
@brannoc: Tom. Look after her.
@wren: Did you just talk to my lantern?
@brannoc: No.

=== aurelian_pre
!music FinalBoss
@narrator: The water of the Well stirs. Something rises out of it, slow and enormous: a shape like a man, built of snowstorm and silence, so tall its head is lost in the dark.
@aurelian: Who comes down?
@wren: Wren Varn. Lamplighter of Hollowmere.
@aurelian: Lamps. Little fires. They all go out. Everything goes out.
@aurelian: I had something. I had something, and it went out, and now no one may keep anything. No one. Ever.
@wren: Aurelian. That's your name. Do you remember it?
@aurelian: There is no name. There is only the cold, and what was taken.
@maelis: It doesn't know itself. Three hundred years of grief, with nothing left to grieve for.
@pip: Tom-lantern, shine your very brightest.
@wren: You're wrong. Not everything goes out. My sister taught me that. The dark's only the part nobody's lit yet.
!sfx BossRoar
!shake
!battle aurelian

=== aurelian_phase2
!flash
@narrator: The great grey shape screams, and folds, and falls in on itself like a snowdrift in the thaw.
@narrator: What is left, kneeling at the Well's edge, is an old man. Frost for hair. Frost for tears. His hands are empty, and they will not stop reaching.
@aurelian: Where did it go? I was holding it. I was holding it so tightly.
@ilsa: Wren! I've got the door! Keep going, little ember!
@wren: You heard her, Tom. Once more.

=== final_choice
!music Story
@narrator: The old man sinks down at the edge of the Well. The cold has gone out of him. So has everything else.
@aurelian: I don't remember what I lost. Isn't that strange? I remember losing it. Only that.
@aurelian: Three hundred years of holding on, and I cannot remember its face. Please. Let me go. Let it all go out.
@narrator: Above them the Hearthflame shrinks to a single blue bead. Without a heart to feed it, it will go out. When it does, nothing will hold the cold.
@maelis: The ledger was right about one thing, Wren. That fire won't burn without a heart.
@brannoc: Your call, lass. We're with you.
@pip: Whatever you choose, we're your grove.
!if shards_all => final_choice_shards
? Take the oath. Become the flame. => ending_oath
? Let the flame die. Take Ilsa home. => ending_dark
?[shards_all] Give him back his memories. => ending_dawn

=== final_choice_shards
@narrator: In Wren's pack, the twelve Memory Shards are warm. All of them at once, humming, like a song waiting for someone to sing it.
? Take the oath. Become the flame. => ending_oath
? Let the flame die. Take Ilsa home. => ending_dark
?[shards_all] Give him back his memories. => ending_dawn
