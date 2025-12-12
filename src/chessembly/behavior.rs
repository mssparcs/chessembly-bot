use super::{Color, DeltaPosition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Behavior<'a> {
    TakeMove(DeltaPosition),
    Take(DeltaPosition),
    Repeat(i8),
    Move(DeltaPosition),
    Catch(DeltaPosition),
    Shift(DeltaPosition),
    Anchor(DeltaPosition),
    Peek(DeltaPosition),
    Observe(DeltaPosition),
    While,
    Jump(DeltaPosition),
    Do,
    Bound(DeltaPosition),
    Edge(DeltaPosition),
    EdgeTop(DeltaPosition),
    EdgeLeft(DeltaPosition),
    EdgeRight(DeltaPosition),
    EdgeBottom(DeltaPosition),
    Corner(DeltaPosition),
    CornerTopLeft(DeltaPosition),
    CornerTopRight(DeltaPosition),
    CornerBottomLeft(DeltaPosition),
    CornerBottomRight(DeltaPosition),
    Not,
    Jmp(u8),
    Jne(u8),
    BlockOpen,
    BlockClose,
    Label(u8),
    End,
    Danger(DeltaPosition),
    Check,
    Enemy(DeltaPosition),
    Friendly(DeltaPosition),
    PieceOn((&'a str, DeltaPosition)),
    SetState((&'a str, u8)),
    IfState((&'a str, u8)),
    Transition(&'a str),
    Piece(String),
}

pub type BehaviorChain<'a> = Vec<Behavior<'a>>;

impl<'a> Behavior<'a> {
    pub fn from_str(fragment: &'a str) -> Behavior<'a> {
        if fragment.starts_with("end") {
            return Behavior::End;
        } else if fragment.starts_with("while") {
            return Behavior::While;
        } else if fragment.starts_with("do") {
            return Behavior::Do;
        } else if fragment.starts_with("not") {
            return Behavior::Not;
        } else if fragment.starts_with("check") {
            return Behavior::Check;
        } else if fragment == "transition" {
            return Behavior::Transition("");
        } else if fragment.starts_with("}") {
            return Behavior::BlockClose;
        } else if fragment.starts_with("{") {
            return Behavior::BlockOpen;
        }
        let fs1 = fragment.split_once('(');
        if fs1.is_none() {
            return Behavior::End;
        }
        let (cmd, pwr) = fs1.unwrap();
        let fs2 = pwr.split_once(')');
        if fs2.is_none() {
            return Behavior::End;
        }
        let (params, _) = fs2.unwrap();
        let params_vec: Vec<&str> = params.split(',').map(|x| x.trim()).collect();

        if cmd == "label" {
            return Behavior::Label(
                params_vec
                    .get(0)
                    .map(|s| s.parse::<u8>().unwrap_or(0))
                    .unwrap_or(0),
            );
        } else if cmd == "jmp" {
            return Behavior::Jmp(
                params_vec
                    .get(0)
                    .map(|s| s.parse::<u8>().unwrap_or(0))
                    .unwrap_or(0),
            );
        } else if cmd == "jne" {
            return Behavior::Jne(
                params_vec
                    .get(0)
                    .map(|s| s.parse::<u8>().unwrap_or(0))
                    .unwrap_or(0),
            );
        } else if cmd == "repeat" {
            return Behavior::Repeat(
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            );
        } else if cmd == "transition" {
            return Behavior::Transition(params_vec.get(0).unwrap_or(&""));
        } else if cmd == "piece" {
            return Behavior::Piece(
                params_vec
                    .get(0)
                    .map(|s| String::from(*s))
                    .unwrap_or(String::new()),
            );
        } else if cmd == "set-state" {
            return Behavior::SetState((
                params_vec.get(0).unwrap_or(&""),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<u8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "if-state" {
            return Behavior::IfState((
                params_vec.get(0).unwrap_or(&""),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<u8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "piece-on" {
            return Behavior::PieceOn((
                params_vec.get(0).unwrap_or(&""),
                (
                    params_vec
                        .get(1)
                        .map(|s| s.parse::<i8>().unwrap_or(0))
                        .unwrap_or(0),
                    params_vec
                        .get(2)
                        .map(|s| s.parse::<i8>().unwrap_or(0))
                        .unwrap_or(0),
                ),
            ));
        } else if cmd == "take-move" {
            return Behavior::TakeMove((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "take" {
            return Behavior::Take((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "jump" {
            return Behavior::Jump((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "move" {
            return Behavior::Move((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "catch" {
            return Behavior::Catch((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "shift" {
            return Behavior::Shift((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "danger" {
            return Behavior::Danger((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "enemy" {
            return Behavior::Enemy((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "friendly" {
            return Behavior::Friendly((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "peek" {
            return Behavior::Peek((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "anchor" {
            return Behavior::Anchor((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "observe" {
            return Behavior::Observe((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "bound" {
            return Behavior::Bound((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "edge" {
            return Behavior::Edge((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "corner" {
            return Behavior::Corner((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "edge-left" {
            return Behavior::EdgeLeft((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "edge-right" {
            return Behavior::EdgeRight((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "edge-top" {
            return Behavior::EdgeTop((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "edge-bottom" {
            return Behavior::EdgeBottom((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "corner-top-left" {
            return Behavior::CornerTopLeft((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "corner-top-right" {
            return Behavior::CornerTopRight((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "corner-bottom-left" {
            return Behavior::CornerBottomLeft((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        } else if cmd == "corner-bottom-right" {
            return Behavior::CornerBottomRight((
                params_vec
                    .get(0)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
                params_vec
                    .get(1)
                    .map(|s| s.parse::<i8>().unwrap_or(0))
                    .unwrap_or(0),
            ));
        }

        Behavior::End
    }

    fn reflect_turn_vector(position: &DeltaPosition, turn: Color) -> DeltaPosition {
        if turn == Color::Black {
            return (-position.0, -position.1);
        } else {
            return position.clone();
        }
    }

    pub fn reflect_turn(&'a self, turn: Color) -> Behavior<'a> {
        match self {
            Behavior::Bound(delta) => Behavior::Bound(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Edge(delta) => Behavior::Edge(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Corner(delta) => Behavior::Corner(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::EdgeTop(delta) => {
                Behavior::EdgeTop(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::EdgeBottom(delta) => {
                Behavior::EdgeBottom(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::EdgeLeft(delta) => {
                Behavior::EdgeLeft(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::EdgeRight(delta) => {
                Behavior::EdgeRight(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::CornerTopLeft(delta) => {
                Behavior::CornerTopLeft(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::CornerTopRight(delta) => {
                Behavior::CornerTopLeft(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::CornerBottomLeft(delta) => {
                Behavior::CornerBottomLeft(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::CornerBottomRight(delta) => {
                Behavior::CornerBottomRight(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::Enemy(delta) => Behavior::Enemy(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Friendly(delta) => {
                Behavior::Friendly(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::Danger(delta) => Behavior::Danger(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Take(delta) => Behavior::Take(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Jump(delta) => Behavior::Jump(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::TakeMove(delta) => {
                Behavior::TakeMove(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::Move(delta) => Behavior::Move(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Catch(delta) => Behavior::Catch(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::Observe(delta) => {
                Behavior::Observe(Behavior::reflect_turn_vector(delta, turn))
            }
            Behavior::Peek(delta) => Behavior::Peek(Behavior::reflect_turn_vector(delta, turn)),
            Behavior::PieceOn((piece, delta)) => {
                Behavior::PieceOn((piece, Behavior::reflect_turn_vector(delta, turn)))
            }
            _ => self.clone(),
        }
    }
}
