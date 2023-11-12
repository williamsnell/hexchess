# Encoding Board State

In order to get a neural net to start predicting the best moves from a given
position, we first need to be able to tell the neural net, in a language it can understand, what the current board state is.

## Possible approaches
Neural nets accept a (typically normalized) sequence of values on their input layer. Never having done this before, (and leaving aside extra board states like en-passant and draw-by-repetition), a few ideas on how to convert a chessboard into a sequence of values are:

-  Allocate one input node for every piece + hexagon combination, and 
write a 1 where that particular hexagon is occupied by that particular piece. This seems to be the conventional approach.
- Allocate one input node for every piece. This approach doesn't work once we consider promotion, since we can't know that there will only be 3 bishops or 1 queen.
- Allocate one input node for every position on the board, with a particular value assigned to each piece. We would then rely on the neural net, as part of its training, to learn the meanings of the different input levels and assign meaning to them. E.g. a White King is +1, a White Queen is +0.8, etc.

While the last approach seems simplest (we would require many many times fewer nodes on the input layer,) my assumption is that the additional learning required could make training more difficult. Furthermore, piece types really are discrete and distinct. In the above example, we will never specify a piece halfway in between a White King and a White Queen with a value of 0.9, and so we don't gain any new functionality by having the ability to pass in an arbitrary floating point value. Since we know each piece + hexagon combination is unique, passing them as distinct values into the neural net seems a bit less lossy.

## Do we need a crazy number of input layer nodes?

There are (6 piece types) * (2 colors) * (91 hexagons) = 1092 inputs. 

We also have 9 pawns which may have double-jumped in the previous move, adding (2 colors) * (9 pawns) = 18 states. Lets ignore draw by repetition for now. 

All up, that's 1110 input nodes. For context, that's about the same as needed for a 33 x 33 pixel image. The MNIST dataset is 28 x 28, so our input layer doesn't seem especially crazy.


## Implementation

Since we've decided to use a series of 1s or 0s to represent pieces being present (or not,) we can start thinking in terms of bits and bitboards. Annoyingly, Hexagonal Chess' 91 hexagons sits halfway between the typical datatypes of u64 or u128 we might want to use. u96 exists, but seems rare. We could play around with a (u64, u32) tuple, but that seems more trouble than it is worth. The odds that using 128 bit maths makes a measurable difference to our code performance, at this stage, is quite low, and so u128 it is (with the intention to profile and benchmark later and verify this assumption.)

The simplest implementation I can think of is giving each piece a u128, which leads to (6 piece types) * (2 colors) = 12 u128's. We can either count the special double-jump state in the free bits in the pawns' bitboards, or we can introduce a new int to store them.


# Edge Cases

# En-Passant

# Move repetition