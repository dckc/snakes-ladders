# Snakes and Ladders

by Dan Connolly

ref
[CPS506 - Comparative Programming Languages - Winter 2017](http://cps506.sarg.ryerson.ca/)
[Assignment 4 - Rust](http://cps506.sarg.ryerson.ca/w2017/Assignments/a4.html)

Sample input:

    board 3 4
    players 2
    dice 1 2 2 2 2
    ladder 5 11
    snake 8 4
    powerup escalator 6 9
    powerup antivenom 7
    powerup double 4
    turns 10


Resulting output:

    +---+---+---+
    | 12| 11| 10|
    |B  |   |   |
    +---+---+---+
    |  7|  8|  9|
    | a |  S| e |
    +---+---+---+
    |  6|  5|  4|
    | e |  L|Ad |
    +---+---+---+
    |  1|  2|  3|
    |   |   |   |
    +---+---+---+
    Player B won
