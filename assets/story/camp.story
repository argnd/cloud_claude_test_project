# EMBERDEEP — Waystone campfires, one per act. Companions who may not have joined yet are gated with !if has_x.

# ---------------------------------------------------------------- Act I
=== camp_1
!music Story
!if has_brannoc => camp_1_brannoc
@narrator: The waystone hums. Wren builds a small fire out of a smuggler's broken crate and sits with Old Tom in her lap.
@wren: Well, Tom. First night down. How are we doing?
@wren: I think we're doing well. I've only screamed twice. Once was at a coat.
@wren: Ilsa used to talk to you when she thought I was asleep. Give you the day's news. "Wren burned the porridge." "Wren's sweet on the chandler's boy."
@wren: Lies. Mostly lies.
@wren: I miss her, Tom. It's been a year, and I still set out two cups.
@wren: She promised she'd come back, and Ilsa doesn't break promises. So something's stopping her. So I'll go and fetch her.
@narrator: Old Tom's flame steadies in its dented glass. Wren decides to take that as agreement.
!heal

=== camp_1_brannoc
@narrator: The waystone hums. Brannoc builds the fire the dwarven way: small, hot, and without asking anyone's opinion.
@wren: Tom says thank you for the fire.
@brannoc: Tom's a lantern.
@wren: Tom is a very good lantern. He's seen me through every night round since I was twelve.
@brannoc: Tomas Varn's lantern. Knew your da. Worked the Silverdeep seam. Laughed like a rockfall.
@wren: You knew him?
@brannoc: Knew everyone who went down a hole. That's the trade.
@wren: Why did you stay at the Lark, after the Forge? Why not go back to the mountain halls?
@brannoc: Lark's warm. Bess feeds me. Nobody asks.
@wren: I'm asking.
@brannoc: Not tonight, lamplighter.
@narrator: Later, when he thinks Wren is asleep, she hears him mutter to the lantern: "Goodnight, Tom."
!heal

# ---------------------------------------------------------------- Act II
=== camp_2
!music Story
!if has_maelis => camp_2_maelis
@narrator: Wren wrings out her socks over the fire. Brannoc wrings out his beard.
@wren: Can I ask you about Dagna?
@brannoc: No.
@wren: Bess says you courted her.
@brannoc: Bess says a lot.
@wren: Bess says you stood her up on Lantern Night and never told her why.
@brannoc: Got stuck down a shaft. Three days. Came up, she'd danced with a tinker. Never explained. Too proud.
@wren: How long ago?
@brannoc: Forty-three years. Thereabouts.
@wren: "Thereabouts". You know it to the day.
@brannoc: To the hour. Go to sleep.
!heal

=== camp_2_maelis
@narrator: The fire is small and smoky. Maelis sits a little way back from it, the way people do when they aren't sure they've been invited.
@brannoc: Sit closer. Fire won't bite.
@maelis: Footnote: dwarven beards hold up to three times their weight in water. It's why you're steaming.
@brannoc: Footnote this.
@narrator: Wren is mending her glove and, without noticing, humming. An old tune, slow and rocking.
@maelis: Where did you learn that?
@wren: This? Ilsa used to sing it when I couldn't sleep. She got it from Old Hesta. Why?
@maelis: No reason. I've heard it before. Somewhere down here.
@narrator: Maelis stares into the fire for the rest of the night and does not say another word. Wren puts it down to the cold.
!heal

# ---------------------------------------------------------------- Act III
=== camp_3
!music Story
!if has_pip => camp_3_pip
@narrator: The Hollows glow soft blue around the waystone. Wren sits on one side of the fire, Maelis on the other. Brannoc sits in the middle, like a wall.
@brannoc: Going to have this out? Or do I sit here all night?
@wren: There's nothing to have out.
@brannoc: Then I'll say it. She was a coward. So was I. Twelve years in the Lark instead of going back for my crew. Cowards can turn round. What matters is if they keep walking.
@maelis: I'm not asking you to forgive me, Wren. I'd think less of you if you did. Footnote: I'd still be grateful.
@wren: Why didn't you open the door?
@maelis: Because once you say a thing like that aloud, it's true for everyone. While it stayed in my head, it was only a very bad book I'd read.
@wren: That's the stupidest thing I've ever heard a clever person say.
@maelis: I know. It sounds worse out loud. Most true things do.
@wren: Tomorrow you walk up front with me and read the signs. I can't carry all this and a map.
@maelis: Up front. Yes.
!heal

=== camp_3_pip
@narrator: The Hollows glow soft blue around the waystone. Wren sits on one side of the fire, Maelis on the other. Brannoc sits in the middle, like a wall.
@pip: Why is everybody sitting so far apart? Is it a game? I'm very good at games.
@brannoc: Not a game.
@maelis: I kept a secret that hurt people, Pip. Wren's sister most of all.
@pip: In the grove, when a root grows crooked, we don't cut it. We turn it toward the light, and we wait. Sometimes a whole year.
@wren: And if it never grows straight?
@pip: Then it's a crooked root that holds the ground up anyway. The Mother always said the crooked ones were her favourites.
@wren: Maelis. Why didn't you open the door?
@maelis: Because once I said it aloud, it would be true for everyone. While it stayed in my head, it was only a very bad book I'd read.
@wren: That's the stupidest thing I've ever heard a clever person say.
@maelis: I know. It sounds worse out loud. Most true things do.
@wren: Tomorrow you walk up front with me and read the signs. I can't carry all this and a map.
@pip: Crooked root!
@brannoc: Go to sleep, twig.
!heal

# ---------------------------------------------------------------- Act IV
=== camp_4
!music Story
@narrator: The Forge is never cold, even at night. They camp in the lee of a great broken anvil, and Brannoc turns Kell's old pipe over and over in his hands.
@brannoc: Kell sang while he worked. Couldn't hold a tune in a bucket. Durra had the best eye for a seam I ever saw. Fisk was the oldest. Told the worst jokes in the Greyspine.
@pip: Tell one! Tell a Fisk joke!
@brannoc: What do you call a dwarf who won't go down a mine?
@pip: What?
@brannoc: Alive.
@narrator: Nobody laughs. Then Brannoc does, a short cracked bark, and then everyone does, even Maelis, who snorts and looks appalled at herself.
@brannoc: He'd have liked that. Nobody ever laughed at that one.
@wren: Why didn't you ever come back for them?
@brannoc: Because if I came back and they were dead, they'd be dead. While I sat in the Lark, they were only missing.
@maelis: I know that particular kind of cowardice. Rather intimately.
@brannoc: Aye. Thought you might.
@wren: Then we carry them up, Brannoc. Their names, Kell's singing, and that terrible joke. All of it.
!heal

# ---------------------------------------------------------------- Act V
=== camp_5
!music Story
@narrator: The last waystone. Beyond it there is only snow falling upward, and the bottom of the world. Nobody wants to sleep, so nobody does.
@pip: When this is done, what will everyone do? Me first. I'll plant a grove. A big one, with a room in it for each of you.
@brannoc: Walk into Dagna's forge. Say what I should've said forty-three years back.
@maelis: Which is?
@brannoc: "Sorry I'm late."
@maelis: I'll write the true history of the Lantern Order. Every name. It will have so many footnotes they'll need their own shelf.
@pip: Will I be in it?
@maelis: In the footnotes. That's where I keep the best parts.
@wren: I'll take Ilsa home and make her breakfast. I'll burn it on purpose, so she knows how it feels.
@wren: And after that I don't know. I'm scared to think that far.
@pip: Then we'll think it for you, Lantern-friend. That's what a grove is for.
@narrator: One by one they sleep. Wren sits up with Old Tom a while longer, listening to the snow.
@wren: Tomorrow, Tom. Stay lit.
!heal
