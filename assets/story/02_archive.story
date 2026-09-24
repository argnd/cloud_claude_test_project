# EMBERDEEP — Act II: The Drowned Archive (floors 5-8). Party: Wren, Brannoc; Maelis joins on floor 6.

=== floor_5_enter
!music Archive
@narrator: The Drowned Archive. Shelves rise into the dark like the ribs of a sunken ship, and between them the water stands black and still, breathing cold.
@narrator: Books float face-down. Ink blooms from them in slow, dark clouds. Somewhere in the stacks, a page turns by itself.
@narrator: The lamps on the reading desks are all out. Beside some of them, hands still rest where they were set down.
@brannoc: Water.
@wren: It's only knee-deep.
@brannoc: Your knee. My chest.
@wren: Can dwarves swim?
@brannoc: Dwarves sink. Proudly.

=== floor_6_enter
@narrator: Deeper stacks. On a landing above the waterline, a round iron door is sealed with wax and chain. A thin line of warm light shows beneath it.
@wren: Brannoc. Somebody's left a lamp on.
@brannoc: Nobody's lived down here in two years.
@wren: Listen. Pages turning. Somebody's reading.
@brannoc: Or something's eating a book.
@wren: Only one way to find out.

=== maelis_meet
@narrator: Wren knocks. The pages stop. Nothing else happens for a long, long moment.
@wren: Hello? I'm not a ghost. I'm Wren Varn, from Hollowmere. I'm looking for my sister, Ilsa Varn. She came down here a year ago.
@narrator: Another silence, longer than the first. Then the chains rattle, and the door opens a crack.
@maelis: You're not a ghost. Ghosts don't say "hello". They say "whoooo", which isn't even a complete question.
@narrator: The woman behind the door is thin and ink-stained, wrapped in three Order robes at once. Her eyes are very sharp and very tired.
@maelis: Maelis Thorne, Archivist of the Lantern Order. The Archivist, I should say. The others left when the water came in.
@brannoc: And you stayed.
@maelis: The water was getting into the forbidden stacks. Somebody had to hold the umbrella.
@wren: Did you see my sister? Did she come this way?
@narrator: Something crosses Maelis's face and is gone before Wren can name it.
@maelis: People come through, now and then. I don't open the door. It's a sealed room. That's rather the point of it.
@maelis: You're going deeper. You'll want someone who reads the old signs, knows which floors flood, and can tell a haunted tome from a merely rude one.
@maelis: Also, four pages of the Forbidden Index were torn loose in the flood. They're somewhere below. I would very much like them back.
@brannoc: She's inviting herself.
@maelis: I'm volunteering. It's different. There's a footnote about it.
!join maelis
!quest start lost_pages
!sfx LevelUp

=== floor_7_enter
@maelis: Mind the third shelf. It bites.
@brannoc: Books don't bite.
@maelis: That one is bound in the hide of something that did. Footnote: the Order has not always been a tasteful institution.
@wren: Maelis, the Order kept records of its Wardens, didn't it? Where they went. What became of them.
@maelis: It kept records. Watch your step here. The floor is softer than it looks.
@brannoc: She changed the subject.
@wren: I noticed.

=== floor_8_enter
@narrator: The Reading Hall of the Curators. Candles burn along the walls, grey-flamed and cold, as they have burned for a hundred years. The water will not come in here.
@maelis: Wren. Before we go on.
@wren: What is it?
@maelis: Whatever waits in there has guarded something for a century. When we find out what, there's something I should have told you. Some time ago.
@wren: Then tell me now.
@maelis: I can't. Not yet. I'm sorry. That's the first honest thing I've said to you, and it's a coward's one.
@brannoc: Later. Something's moving.

=== curator_pre
!music Boss
@narrator: At the end of the hall, behind a lectern of black oak, stands a tall figure in the robes of an Archivist Superior. The robes are a century out of date. So is the man.
@curator: Maelis Thorne. You were always my favourite reader. Such appetite. Such discretion.
@maelis: Archivist Superior Casimir Vell. You died in the year of the long frost. I've read your obituary. It was very flattering.
@curator: I wrote it.
@curator: And the lamplighter's sister. You've come for the ledger. They all come for it, sooner or later. She did.
@wren: Ilsa was here?
@curator: I let her read it. I was tired, and she asked so politely. And see what it did: the lanterns going out, the frost on your wells.
@curator: Knowledge is a flame too, child. It burns the hands that hold it. I have kept this secret for a hundred years so that no one else need burn.
@wren: Someone is already burning. I can feel it in the walls.
@curator: Yes. The right ones. Quietly. That is what mercy looks like, when you are old enough to see it.
@brannoc: Heard enough.
!sfx BossRoar
!battle curator

=== curator_post
!music Story
@narrator: The Curator comes apart like wet paper. On the lectern lies a ledger bound in grey leather, its title scraped away.
@curator: Read it, then. I am so tired of being kind.
!fade
@narrator: The handwriting changes every thirty years or so. The entries all end the same way.
@wren: "Aldous Marrow. Descended. Consumed." That's the Elder's brother. "Hadric Moll. Refused the Flame. Bound in iron at the Forge."
@narrator: Brannoc goes very still at that name.
@wren: "The Warden does not tend the Flame. The Warden is given to it. A strong heart burns some thirty years before the Pale wakes hungry again."
@wren: Given to it. Consumed. Every name. Consumed, consumed, consumed.
@wren: The last entry is her. "Ilsa Varn." And then nothing. No "consumed". Nothing.
@brannoc: She never went into the fire.
@wren: The lanterns. The frost. The Flame is dying because she won't feed it. She read this, and she said no.
@brannoc: Stone and ember. Good lass.
@maelis: Wren. I knew.
@maelis: I found it in the Forbidden Index before the flood. When they chose your sister, I said nothing. And when she came down, she knocked on my door.
@maelis: She was humming an old lullaby to keep her courage up. I knew her voice from the lamp rounds. I sat very still until she went away.
@wren: She knocked. She was down here alone, finding this, and you were ten feet away with a lock on the door.
@maelis: Yes.
@wren: I should leave you here with your books. You'd like that. It's quiet.
@narrator: Nobody speaks. Somewhere in the stacks, water drips onto an open page, one slow drop at a time.
@wren: She's alive. She's down there somewhere, fighting it. That's all that matters now.
@wren: You know these depths. So you come. And the moment you know anything, you tell me. Everything.
@maelis: Everything. I swear it on every book I own.
@wren: Swear it on something you'd miss.
@maelis: On my name, then.
!flag truth_known
!music TownSorrow
