// A simple Snakes and Ladders game

// The total number of cells cannot exceed 999.
type CellIx = usize;
const MAX_CELLS: usize = 999;
// There can be up to 26 players.
type PlayerIx = usize;
const MAX_PLAYERS: PlayerIx = 26;

type DieDots = u8;

// 1. The input is a series of lines, each containing one command.
#[cfg(test)]
const SAMPLE_INPUT: &'static str = "
board 3 4
players 2
dice 1 2 2 2 2
ladder 5 11
snake 8 4
powerup escalator 6 9
powerup antivenom 7
powerup double 4
turns 10
";

// 6. The output must be exactly as shown, as automatic comparison
// will be part of marking.
#[cfg(test)]
const RESULTING_OUTPUT: &'static str = "
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
";


// When your `main` program is run, it must read commands from
// standard input, passing each line to the `readFrom` function. At
// the end of the input, it must print the state of the board on
// standard output.
fn main() {
    use std::io;
    use std::io::BufRead;

    let mut game = GameState::default();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.expect("stdin failed?!");
        game.apply(Command::readFrom(line))
    }

    println!("{}", game.print());
}


#[derive(PartialEq, Eq, Debug, Default)]
struct GameState {
    board: BoardConfig,
    players: Vec<PlayerState>,
    dice: Vec<DieDots>,
    turn: usize,
}

impl GameState {
    // 3. Your module must have a print function that formats the current
    // state of the board as a string.
    pub fn print(&self) -> String {
        let mut rev_out: Vec<String> = vec!();
        let sep = BoardConfig::between_columns();

        if let Some(which) = self.winner() {
            rev_out.push(format!("Player {} won", which))
        }

        for row in 1..(self.board.rows + 1) {
            let mut first_line = String::from("");
            let mut second_line = String::from("");
            for column in 1..(self.board.columns + 1) {
                let ix = self.board.back_and_forth(row, column);

                first_line.push(sep);
                second_line.push(sep);

                first_line.push_str(&BoardConfig::cell_first_line(ix));
                second_line.push_str(&self.cell_second_line(ix));
            }
            first_line.push(sep);
            second_line.push(sep);

            rev_out.push(self.board.between_rows());
            rev_out.push(second_line);
            rev_out.push(first_line);
        }
        rev_out.push(self.board.between_rows());
        rev_out.reverse();
        return rev_out.join("\n");
    }

    // The second line has: the player or blank, followed by the first
    // letter of the powerup or blank, followed by the start of a
    // snake or ladder, or blank.
    fn cell_second_line(&self, ix: CellIx) -> String {
        let look = self.players.iter().enumerate().find(|&(_, p)| p.loc == ix);
        let player_spot = match look {
            Some((which, _)) => player_name(which),
            None => ' '
        };
        format!("{}{}", player_spot, self.board.cells[ix - 1].label())
    }

    // 1. If no player has won, that last line would not appear at all.
    // 2. The last cell is the winning cell. If a player lands on it, by any
    // means, they win.
    pub fn winner(&self) -> Option<char> {
        self.players.iter().enumerate()
            .find(|&(_, p)| p.loc >= self.board.size())
            .map(|(ix, _)| player_name(ix))
    }

    // 3. There can be any number of snake, ladder, or powerup
    // commands, and hence any number of the resulting game
    // features. There can also be any number of turn commands, each
    // of which will run in turn.
    pub fn apply(&mut self, command: Command) {
        use Command::*;
        use CellProperty::*;

        match command {
            Board { columns: y, rows: x } => self.board.set_size(y, x),
            Players(n) => self.set_player_count(n),
            Dice(ds) => { self.dice = ds },
            Ladder { starts: s, ends: e } =>
                self.board.set_cell_prop(s, LadderStart { end: e }),
            Snake { starts: s, ends: e } =>
                self.board.set_cell_prop(s, SnakeStart { end: e }),
            PowerUps { typ: pt, cells: cs } =>
                for c in cs {
                    self.board.set_cell_prop(c, PowerUp(pt.clone()))
                },
            Turns(qty) =>
                for _ in 0..qty {
                    // println!("turn {} of {}", turn + 1, qty);
                    for which_player in 0..self.players.len() {
                        if self.player_turn_wins(which_player) {
                            return
                        }
                    }
                },
        }
    }

    fn set_player_count(&mut self, n: PlayerIx) {
        assert!(n <= MAX_PLAYERS);
        self.players = vec![
            PlayerState { loc: 1, ..PlayerState::default()};
            n as usize];
    }

    // A turn means each player, in order, rolls the dice, and then
    // moves that many cells (or double if they have the powerup). If
    // the (possibly doubled) roll would take them past the end of the
    // board, they don't move, and play proceeds to the next
    // player. As soon as a player wins, the game is over and the
    // turns stop.

    // 5. A given cell can only have one player on it. When a player
    // lands on a cell (including initial positioning), if there is
    // already a player on the cell, that player gets bumped one
    // cell. When a bumped player lands on a cell, they get the action
    // associated with that cell, including winning, powerups, snakes,
    // ladders, or bumping yet another player.
    pub fn player_turn_wins(&mut self, who: usize) -> bool {
        let (mut delta, start_loc) = {
            let die = self.roll_dice();
            // println!("start turn: player {} rolls {}",
            //          player_name(who), die);
            let player = &mut self.players[who];
            (player.use_double(die), player.loc)
        };
        if start_loc + delta > self.board.size() {
            // println!("cannot move");
            return false
        }

        let mut current_player = who;

        loop {
            let land_loc = {
                let player = &mut self.players[current_player];

                if self.board.move_wins(player, delta) { return true }
                player.loc
            };

            let already = self.players.iter().enumerate()
                .find(|&(ix, p)| p.loc == land_loc && ix != current_player);
            if let Some((which, _)) = already {
                current_player = which;
                delta = 1;
            } else {
                break;
            }
        }

        // println!("board:\n{}\n{:?}", self.print(), self);
        false
    }

    // The sequence will repeat indefinitely - e.g. the example above
    // would produce the sequence 1, 2, 2, 2, 2, 1, 2, 2, 2, 2, 1, 2,
    // 2, ...
    fn roll_dice(&mut self) -> DieDots {
        let die = self.dice[self.turn % self.dice.len()];
        self.turn += 1;
        die
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Default)]
struct PlayerState {
    loc: CellIx,
    // A powerup cell can be triggered any number of
    // times by any player, but they do not accumulate - a player
    // either has the powerup or they don't.
    // @@ can a player have one of each at the same time?
    powerup: Option<PowerType>
}

impl PlayerState {
    fn go(&mut self, loc: CellIx) {
        self.loc = loc
    }

    fn set_powerup(&mut self, pt: PowerType) {
        self.powerup = Some(pt)
    }

    fn use_double(&mut self, die: u8) -> usize {
        if self.powerup == Some(PowerType::Double) {
            self.powerup = None;
            die as usize * 2
        } else {
            die as usize
        }
    }

    fn use_escalator(&mut self, there: CellIx) {
        self.loc += 2 * (there - self.loc);
        self.powerup = None
    }

    fn use_antivenom(&mut self) {
        self.powerup = None
    }
}


#[derive(PartialEq, Eq, Debug, Default)]
struct BoardConfig {
    columns: usize,
    rows: usize,
    cells: Vec<CellProperty>,
}

impl BoardConfig {
    fn set_size(&mut self, columns: CellIx, rows: CellIx) {
        use CellProperty::*;

        assert!(columns * rows <= MAX_CELLS);
        self.columns = columns;
        self.rows = rows;
        self.cells = vec![Plain; columns * rows];
        self.cells[columns * rows - 1] = Winning;
    }

    fn size(&self) -> CellIx {
        self.columns * self.rows
    }

    fn set_cell_prop(&mut self, ix: CellIx, cp: CellProperty) {
        self.cells[ix - 1] = cp
    }

    // 2. The cell numbering starts at the bottom left, and loops back
    // and forth.
    fn back_and_forth(&self, row: usize, column: usize) -> CellIx {
        // ideally we'd make an iterator, but that's a pain.
        (row - 1) * self.columns + (
            if row % 2 == 1 { column }
            else { self.columns - column + 1 }
        )
    }

    // 3. Lines between rows are drawn as sequences
    // of + and - characters, as shown.
    fn between_rows(&self) -> String {
        "+---".repeat(self.columns as usize) + "+".into()
    }

    // 4. Lines between columns are drawn with | characters, as shown.
    fn between_columns() -> char { '|' }

    // 5. Cells are printed in 2 lines. The first line is the cell
    // number, right justified with spaces.
    fn cell_first_line(ix: CellIx) -> String{
        format!("{:>3}", ix)
    }

    // When a player lands on a powerup cell they acquire that powerup
    // and retain it until they use it. Using the powerup removes it
    // from the player.
    // If the player lands on the first cell of a ladder or snake,
    // they are transported to land on the second cell.
    fn move_wins(&self, player: &mut PlayerState, delta: CellIx) -> bool {
        use CellProperty::*;
        use PowerType::*;

        let cell_qty = self.cells.len();
        player.loc += delta;
        let cell = &self.cells[player.loc - 1];
        
        let mut win = false;
        match *cell {
            Plain => {},
            Winning => win = true,
            SnakeStart {..} if player.powerup == Some(Antivenom) =>
                player.use_antivenom(),
            SnakeStart { end: there } => player.go(there),
            LadderStart { end: there } if player.powerup == Some(Escalator) => {
                player.use_escalator(there);
                if player.loc >= cell_qty {
                    win = true;
                    player.go(cell_qty);
                }
            }
            LadderStart { end: there } => player.go(there),
            PowerUp(ref pt) => player.set_powerup(pt.clone()),
        }
        win
    }
}


// 4. A given cell can only have a single special property: winning,
// snake start, ladder start, or powerup. Note that the end of a snake or
// ladder could have a special property.
#[derive(Clone, Debug, PartialEq, Eq)]
enum CellProperty {
    Plain,
    SnakeStart { end: CellIx },
    LadderStart { end: CellIx },
    PowerUp(PowerType),
    Winning,
}

impl CellProperty {
    fn label(&self) -> String {
        use CellProperty::*;
        match self {
            &Plain => "  ".into(),
            &SnakeStart{..} => " S".into(),
            &LadderStart{..} => " L".into(),
            &PowerUp(ref pt) => pt.label() + " ",
            &Winning => "  ".into(),
        }
    }
}


// Players are named: A, B, ... There can be up to 26 players.
fn player_name(pix: PlayerIx) -> char {
    (('A' as u8) + (pix as u8)) as char
}


#[derive(PartialEq, Eq, Debug, Clone)]
enum PowerType {
    // 1. powerup escalator 6 9 sub-command: makes the next ladder
    // cell a player steps onto twice as boosting - i.e. they move
    // twice as far up the board. If the boost takes them past the end
    // of the board, they get moved to the last cell, and hence win.
    Escalator,

    // 2. powerup antivenom 7 sub-command: make the player immune to
    // the next snake cell they step onto - i.e. they don't slide down
    // the snake.
    Antivenom,

    // 3. powerup double 5 sub-command:
    // doubles the next dice roll.
    Double
}

impl PowerType {
    fn new(name: &str) -> PowerType {
        use PowerType::*;

        match name {
            "escalator" => Escalator,
            "antivenom" => Antivenom,
            "double" => Double,
            _ => panic!("bad powerup: {}", name)
        }
    }

    fn label(&self) -> String {
        match self {
            &PowerType::Escalator => "e",
            &PowerType::Antivenom => "a",
            &PowerType::Double => "d",
        }.into()
    }
}


#[derive(PartialEq, Eq, Debug)]
enum Command {

    // 1. board 3 4 command: specifies the number of columns and rows
    // for the board.
    Board { columns: usize, rows: usize },

    // 2. players 2 command: specified the number of players, who are
    // named: A, B, ... There can be up to 26 players.
    Players (PlayerIx),

    // 3. dice 1 2 2 2 2 command: specifies the sequence of dice
    // rolls.
    Dice (Vec<DieDots>),

    // 4. ladder 5 11 command: creates a ladder that starts at the
    // first number and ends at the second number
    Ladder { starts: CellIx, ends: CellIx },

    // 5. snake 8 4 command: creates a snake that starts at the first
    // number and ends at the second number
    Snake { starts: CellIx, ends: CellIx },

    // 6. powerup type cell list command describes a powerup that is
    // applied to a series of cells.
    PowerUps { typ: PowerType, cells: Vec<CellIx> },

    // 7. turns 10 command: plays the specified number of turns (or
    // until a player wins the game).
    Turns (usize)
}


impl Command {
    // 2. Your module must have a readFrom function that accepts a string.
    // A command is a keyword followed by one or more parameters
    // separated by a single space.

    // 6. Assume the input is perfectly legal, with no invalid
    // commands, extra spaces, invalid numbers, etc. Additionally
    // assume the set-up board will not produce any bump loops during
    // play.
    #[allow(non_snake_case)]
    fn readFrom(line: String) -> Command {
        use Command::*;

        let num8 = |txt: &str| txt
            .parse::<u8>().expect("bad number");
        let num = |txt: &str| txt
            .parse::<CellIx>().expect("bad number");

        let mut parameters = line.split(' ');
        let keyword = parameters.next().expect("no keyword");
        match (keyword, parameters.collect::<Vec<_>>()) {
            ("board", ref ps) if ps.len() == 2 =>
                Board{ columns: num(ps[0]),
                       rows: num(ps[1]) },

            ("players", ref ps) if ps.len() == 1 =>
                Players(num8(ps[0]) as PlayerIx),

            ("dice", ps) => Dice(ps.iter().map(|s| num8(*s)).collect()),

            ("ladder", ref ps) if ps.len() == 2 =>
                Ladder { starts: num(ps[0]),
                         ends: num(ps[1]) },

            ("snake", ref ps) if ps.len() == 2 =>
                Snake { starts: num(ps[0]),
                        ends: num(ps[1]) },

            ("powerup", ref ps) => PowerUps {
                typ: PowerType::new(ps[0]),
                cells: ps[1..].iter().map(|s| num(*s)).collect() },

            ("turns", ref ps) if ps.len() == 1 => Turns(num(ps[0])),

            _ => panic!("bad command: {}", line)
        }
    }
}


/*
5. Put your ownership information (see the assignment page) in
the assign4/README.md file.

6. The marker should be able to run your program by entering the following code:
cargo run
board 3 4
players 2
turns 5
*/

// 7. In addition to any tests or test data we provide, you must have
// unit tests for the components of your program, verifying that they
// perform correctly. This will be worth 20% of the mark.
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn command_parse1() {
        use Command::*;
        assert!(Command::readFrom("board 3 4".into()) ==
                Board { columns: 3, rows: 4});
    }

    #[test]
    fn parse_sample_input() {
        use Command::*;
        use PowerType::*;
        let lines: Vec<_> = SAMPLE_INPUT.trim().lines().collect();

        assert!(Command::readFrom(lines[0].into()) ==
                Board { columns: 3, rows: 4});
        assert!(Command::readFrom(lines[1].into()) ==
                Players(2));
        assert!(Command::readFrom(lines[2].into()) ==
                Dice(vec!(1, 2, 2, 2, 2)));
        assert!(Command::readFrom(lines[3].into()) ==
                Ladder{ starts: 5, ends: 11});
        assert!(Command::readFrom(lines[4].into()) ==
                Snake{ starts: 8, ends: 4});
        assert!(Command::readFrom(lines[5].into()) ==
                PowerUps{ typ: Escalator, cells: vec!(6, 9) });
        assert!(Command::readFrom(lines[6].into()) ==
                PowerUps{ typ: Antivenom, cells: vec!(7) });
        assert!(Command::readFrom(lines[7].into()) ==
                PowerUps{ typ: Double, cells: vec!(4) });
        assert!(Command::readFrom(lines[8].into()) ==
                Turns(10));

    }

    #[test]
    fn make_sample_output() {
        let mut game = GameState::default();

        for line in SAMPLE_INPUT.trim().lines() {
            let cmd = Command::readFrom(line.into());
            game.apply(cmd);
        }

        println!("{}", game.print());
        assert!(game.print() == RESULTING_OUTPUT.trim());
    }

}
