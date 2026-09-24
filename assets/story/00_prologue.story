# EMBERDEEP — Prologue, Hollowmere set pieces, waystones and town returns.
# Format: see the story bible. Speakers, commands and ids are validated by the story validator.

=== intro
!music Story
@narrator: High in the Greyspine peaks, where the snow never quite lets go, the village of Hollowmere is built around a stair.
@narrator: The Vaultgate leads down into the Deep: ruin stacked on ruin, twenty levels of it, older than anyone's grandmother's stories.
@narrator: At the bottom burns the Hearthflame. The Lantern Order teaches: while the Flame burns, the Pale sleeps.
@narrator: Once a generation, a Warden is chosen to go down and tend it. Wardens never come back. The Order says they are too honoured to leave.
@narrator: And every night, the village lamplighter lights the lanterns in their name.
!fade
@narrator: One year ago. The Vaultgate, at dusk.
@ilsa: You're shaking.
@wren: It's cold. I'm allowed to shake when it's cold. You're the one walking into a hole in the ground.
@wren: Ilsa, I can't do the rounds without you. I'm a lamplighter who's scared of the dark. They tell jokes about me at the Lark.
@ilsa: The dark's only the part nobody's lit yet. So light it. Chapel Row first, always Chapel Row first.
@wren: Promise me you'll come back.
@ilsa: I promise, little ember. Keep Old Tom burning, and I'll follow the light home.
!sfx DoorOpen
@narrator: The gate closed behind her. The Order rang the bell for a new Warden, Hollowmere cheered, and Wren lit every lantern in town.
!fade
@narrator: Now the lanterns are going out.
@narrator: One by one, night after night, however much oil Wren gives them. Frost creeps up the wells in high summer.
@narrator: And something is walking in the cellars.
!goto wake

=== wake
!music Town
@narrator: Night. The Varn house, two rooms above the chandler's shop. Wren is asleep in her boots again.
!sfx DoorOpen
!shake
@tobin: WREN! Wren, wake up! It's gone out! The big one!
@wren: Tobin, it's the middle of the — which big one?
@tobin: The VAULT lantern! The one that never goes out! It went out, and there's frost all over the gate, and it's SUMMER!
@wren: That lantern hasn't gone out in three hundred years.
@tobin: I know! That's why I ran! I only fell over twice!
@narrator: Wren lights Old Tom. The flame gutters in its dented glass, then catches.
@wren: Ilsa's down there. If the Flame is failing, she's in trouble. I'm going down.
@tobin: I'm coming too!
@wren: You're going home. And tomorrow at dusk you're lighting Chapel Row for me. Lamplighter's orders.
@tobin: Is that a real order?
@wren: Signed, sealed and lit.
@wren: Right, Tom. Elder Marrow will try to stop me, so we see him first and get it over with. Then supplies: Bess, Fen, Dagna.

=== elder_first
@elder: Wren Varn. Half the village is in the street in its nightclothes, and here you are. Let me guess. Something foolish.
@wren: I've come to tell you. I'm going through the Vaultgate. The Flame is failing, and my sister is down there.
@elder: Your sister is the Warden. She is where she chose to be. The Order will see to the Flame.
@wren: The Order has been seeing to it for a month. The lanterns keep going out.
@elder: The Deep is not a cellar, child. It is twenty floors of everything our ancestors buried and meant to stay buried.
@wren: Then I'll bring a lantern.
@elder: I forbid it. As Elder of the Lantern Order, I forbid you to set one foot past that gate.
@wren: You sent my sister past that gate with a bell and a blessing. You don't get to forbid me anything.
@elder: Ilsa said you were stubborn. She was being kind.
@narrator: Marrow looks at Old Tom for a long moment, as if the lantern has said something he would rather not hear.
@elder: Take these. I cannot stop you, and I will not have it said the Order sent you down empty-handed.
!give tonic 3
!sfx ItemGet
@elder: And a warning. Whatever you find down there, whatever you read, remember the Order has kept this village alive for three hundred years.
@elder: I chose your sister because she was the best of us. Remember that too.
!flag elder_met

=== elder_confront
!music TownSorrow
@wren: You knew.
@elder: Close the door, Wren.
@wren: No. Let them hear it. You knew what a Warden is. You knew what she'd become when you sent her down.
@elder: Yes.
@narrator: He says it the way a man sets down something he has carried too long. It is not relief. It is only weight, finally on the floor.
@elder: Every Elder learns it the day they take the chain of office. I learned it twenty-five years ago. By then my brother Aldous had been burning for ten.
@wren: And you still did it. You picked my sister.
@elder: I picked the best of us. The ledgers say the Flame burns longer on a strong heart. I did the arithmetic, and I rang the bell, and I went home and ate my supper.
@elder: She came to me the night before. She asked whether Wardens ever wrote home. I told her the Flame keeps them busy.
@wren: She was twenty-six. She did my washing. She burned the porridge every single morning and called it "toasted".
@elder: I know. I know.
@narrator: Elder Marrow of the Lantern Order sits down on the floor of his own hall, and weeps like a boy.
@elder: Three hundred years we lit lanterns in their honour. I told myself it was enough. It was never enough. It was only light.
@wren: I'm going to end this. Not your way.
@elder: Then when you find her, don't tell her I'm sorry. She'll know that already. Tell her I was wrong.
!flag elder_confronted

=== vaultgate_first
!music Undercroft
@narrator: The Vaultgate stands at the foot of Chapel Row: two leaves of black iron, taller than the temple, furred white with frost.
@guard: Vaultgate's shut by the Elder's order. Nobody goes — oh. It's you.
@guard: I didn't see you. I'm looking at this wall. It's a very interesting wall.
@narrator: Above the gate hangs the great Vault lantern, cold and dark for the first time in three hundred years.
@wren: All right, Tom. It's only stairs. Stairs and dark. The dark's only the part nobody's lit yet.
@wren: Ilsa made that sound much braver.
!sfx DoorOpen
@narrator: The gate groans open. Warm air should rise from the Deep. Instead, the cold comes up the stairs to meet her.

=== waystone_first
@narrator: A standing stone, waist-high, carved with the Lantern Order's sigil. It hums under Wren's palm, low and warm, like a cat deciding to forgive you.
!sfx Waystone
@wren: It's warm. The only warm thing down here.
@narrator: Through the hum she can feel Hollowmere above her: the inn's hearth, the temple bell, every step of the stair she came down.
@wren: I could go home from here. And come straight back to this spot.
@wren: The Order must have set these for the Wardens. So why did none of them ever use one to come home?

=== inn_rest
@bess: Boots off, bellies full, and not one word about the Deep till morning. House rules.
!heal
!sfx Heal

=== town_return_2
!music Town
@narrator: Hollowmere at dusk. For the first time in weeks, three lanterns on Chapel Row are burning on their own.
@tobin: Wren! WREN! Look! They came back on! I think it was me. I've been practising.
@brannoc: It was the rat.
@wren: It was definitely you, Tobin.
@narrator: People have come out of their houses to look. Someone is smiling. It has been a long time since anyone in Hollowmere smiled at a lantern.

=== town_return_3
!music TownSorrow
@narrator: Wren does not mean to tell anyone. She walks into the Lantern & Lark, Bess asks how she is, and Wren tells her.
@narrator: By sundown, all of Hollowmere knows.
@narrator: The square is full and silent, the bad kind of silent. Someone has thrown a stone through the Order hall's window. Nobody has swept up the glass.
@narrator: On the temple steps Sister Oriel begins the Wardens' hymn, as she does every evening. Nobody sings with her. Halfway through, she stops.
@rennick: Nobody touches that hall. Go home. Please. It's late, and I'm too tired to arrest all of you.
@maelis: This is why I never said anything.
@wren: No. This is why you should have.

=== town_return_4
!music TownSorrow
@narrator: The road down the pass is full of carts: looms, cradles, hens in baskets. Hollowmere is leaving, one family at a time.
@narrator: Fen's shop has a new sign on the door. STILL OPEN. Underneath, smaller, crossed out: PLEASE DON'T GO.
@pip: Lantern-friend, why is everybody going away?
@wren: Because it's getting cold, Pip. And they're frightened.
@pip: Like my grove. Everybody left, and I stayed. Staying was very lonely.
@brannoc: Then we'd best be quick.

=== town_return_5
!music TownSorrow
@narrator: Snow in high summer. It settles on Chapel Row, on the empty market, on the cold chimney of the Order hall.
@narrator: Hollowmere is nearly empty now. But in the windows that are left, there are candles. Dozens of them.
@bess: They're for you, love. All of you. Seemed only right, after all the lanterns you've lit for everybody else.
@wren: I don't know what to say.
@bess: Then don't. Eat something. You're all bones and bravery.
@pip: It's a grove. A grove of little suns.
