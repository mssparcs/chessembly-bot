pub const GAME_SCRIPT: &'static str = 
"piece(knight)
    { take-move(1, 2) }
    { take-move(1, -2) }
    { take-move(-1, 2) }
    { take-move(-1, -2) }
    { take-move(2, 1) }
    { take-move(2, -1) }
    { take-move(-2, 1) }
    take-move(-2, -1);
piece(bishop)
    { take-move(1, 1) repeat(1) }
    { take-move(1, -1) repeat(1) }
    { take-move(-1, 1) repeat(1) }
    take-move(-1, -1) repeat(1);
piece(queen)
    { take-move(1, 1) repeat(1) }
    { take-move(1, -1) repeat(1) }
    { take-move(-1, 1) repeat(1) }
    { take-move(-1, -1) repeat(1) }
    { take-move(1, 0) repeat(1) }
    { take-move(-1, 0) repeat(1) }
    { take-move(0, 1) repeat(1) }
    take-move(0, -1) repeat(1);
piece(tempest-rook)
    {
        take-move(1, 1)
        { take-move(1, 0) repeat(1) }
        take-move(0, 1) repeat(1)
    } {
        take-move(1, -1)
        { take-move(1, 0) repeat(1) }
        take-move(0, -1) repeat(1)
    } {
        take-move(-1, 1)
        { take-move(-1, 0) repeat(1) }
        take-move(0, 1) repeat(1)
    }
    take-move(-1, -1)
    { take-move(-1, 0) repeat(1) }
    take-move(0, -1) repeat(1);
piece(bouncing-bishop)
    {
        do take-move(1, 1) while
        edge(1, 1)
        { take-move(-1, 1) repeat(1) }
        take-move(1, -1) repeat(1)
    } {
        do take-move(-1, 1) while
        edge(-1, 1)
        { take-move(1, 1) repeat(1) }
        take-move(-1, -1) repeat(1)
    } {
        do take-move(1, -1) while
        edge(1, -1)
        { take-move(1, 1) repeat(1) }
        take-move(-1, -1) repeat(1)
    }
    do take-move(-1, -1) while
    edge(-1, -1)
    { take-move(1, -1) repeat(1) }
    take-move(-1, 1) repeat(1);
piece(dozer)
    { take-move(-2, 1) }
    { take-move(-1, 1) }
    { take-move(0, 1) }
    { take-move(1, 1) }
    take-move(2, 1);
piece(alfil)
    { take-move(2, 2) }
    { take-move(2, -2) }
    { take-move(-2, 2) }
    take-move(-2, -2);
piece(bard)
    { take-move(2, 0) }
    { take-move(-2, 0) }
    { take-move(0, 2) }
    { take-move(0, -2) }
    { take-move(2, 2) }
    { take-move(2, -2) }
    { take-move(-2, 2) }
    take-move(-2, -2);
piece(wasp)
    { move(1, -1) repeat(1) }
    { move(-1, -1) repeat(1) }
    take-move(0, 1) repeat(1);
piece(amazon)
    { take-move(1, 2) }
    { take-move(1, -2) }
    { take-move(-1, 2) }
    { take-move(-1, -2) }
    { take-move(2, 1) }
    { take-move(2, -1) }
    { take-move(-2, 1) }
    { take-move(-2, -1) }
    { take-move(1, 1) repeat(1) }
    { take-move(1, -1) repeat(1) }
    { take-move(-1, 1) repeat(1) }
    { take-move(-1, -1) repeat(1) }
    { take-move(1, 0) repeat(1) }
    { take-move(-1, 0) repeat(1) }
    { take-move(0, 1) repeat(1) }
    take-move(0, -1) repeat(1);
piece(chancellor)
    { take-move(1, 2) }
    { take-move(1, -2) }
    { take-move(-1, 2) }
    { take-move(-1, -2) }
    { take-move(2, 1) }
    { take-move(2, -1) }
    { take-move(-2, 1) }
    { take-move(-2, -1) }
    { take-move(1, 0) repeat(1) }
    { take-move(-1, 0) repeat(1) }
    { take-move(0, 1) repeat(1) }
    take-move(0, -1) repeat(1);
piece(archbishop)
    { take-move(1, 2) }
    { take-move(1, -2) }
    { take-move(-1, 2) }
    { take-move(-1, -2) }
    { take-move(2, 1) }
    { take-move(2, -1) }
    { take-move(-2, 1) }
    { take-move(-2, -1) }
    { take-move(1, 1) repeat(1) }
    { take-move(1, -1) repeat(1) }
    { take-move(-1, 1) repeat(1) }
    take-move(-1, -1) repeat(1);
piece(cannon)
    { do take(1, 0) enemy(0, 0) not while jump(1, 0) repeat(1) }
    { do take(-1, 0) enemy(0, 0) not while jump(-1, 0) repeat(1) }
    { do take(0, 1) enemy(0, 0) not while jump(0, 1) repeat(1) }
    { do take(0, -1) enemy(0, 0) not while jump(0, -1) repeat(1) }
    { do peek(1, 0) while friendly(0, 0) move(1, 0) repeat(1) }
    { do peek(-1, 0) while friendly(0, 0) move(-1, 0) repeat(1) }
    { do peek(0, 1) while friendly(0, 0) move(0, 1) repeat(1) }
    do peek(0, -1) while friendly(0, 0) move(0, -1) repeat(1);
piece(centaur)
    { take-move(1, 2) }
    { take-move(1, -2) }
    { take-move(-1, 2) }
    { take-move(-1, -2) }
    { take-move(2, 1) }
    { take-move(2, -1) }
    { take-move(-2, 1) }
    { take-move(-2, -1) }
    { take-move(1, 1) }
    { take-move(1, -1) }
    { take-move(-1, 1) }
    { take-move(-1, -1) }
    { take-move(1, 0) }
    { take-move(-1, 0) }
    { take-move(0, 1) }
    take-move(0, -1);
piece(zebra)
    { take-move(2, 3) }
    { take-move(2, -3) }
    { take-move(-2, 3) }
    { take-move(-2, -3) }
    { take-move(3, 2) }
    { take-move(3, -2) }
    { take-move(-3, 2) }
    take-move(-3, -2);
piece(giraffe)
    { take-move(1, 4) }
    { take-move(1, -4) }
    { take-move(-1, 4) }
    { take-move(-1, -4) }
    { take-move(4, 1) }
    { take-move(4, -1) }
    { take-move(-4, 1) }
    take-move(-4, -1);
piece(camel)
    { take-move(1, 3) }
    { take-move(1, -3) }
    { take-move(-1, 3) }
    { take-move(-1, -3) }
    { take-move(3, 1) }
    { take-move(3, -1) }
    { take-move(-3, 1) }
    take-move(-3, -1);
piece(windmill-bishop) transition(windmill-rook)
    { take-move(1, 0) repeat(1) }
    { take-move(-1, 0) repeat(1) }
    { take-move(0, 1) repeat(1) }
    take-move(0, -1) repeat(1);
piece(windmill-rook) transition(windmill-bishop)
    { take-move(1, 1) repeat(1) }
    { take-move(1, -1) repeat(1) }
    { take-move(-1, 1) repeat(1) }
    take-move(-1, -1) repeat(1);";
