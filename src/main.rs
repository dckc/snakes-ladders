// A simple Snakes and Ladders game

// The total number of cells cannot exceed 999.
type CellIx = usize;
// There can be up to 26 players.
type PlayerIx = u8;
type DieDots = u8;


#[derive(PartialEq, Eq, Debug)]
enum Command {

    // 1. board 3 4 command: specifies the number of columns and rows
    // for the board. The total number of cells cannot exceed 999.
    Board { columns: usize, rows: usize },
    
    // 2. players 2 command: specified the number of players, who are
    // named: A, B, ... There can be up to 26 players.
    Players (PlayerIx),
    
    // 3. dice 1 2 2 2 2 command: specifies the sequence of dice
    // rolls. The sequence will repeat indefinitely - e.g. the example
    // above would produce the sequence 1, 2, 2, 2, 2, 1, 2, 2, 2, 2,
    // 1, 2, 2, ...
    Dice (Vec<DieDots>),
    
    // 4. ladder 5 11 command: creates a ladder that starts at the
    // first number and ends at the second number - i.e. if the player
    // lands on the first cell they are transported to land on the
    // second cell.
    Ladder { starts: CellIx, ends: CellIx },
    
    // 5. snake 8 4 command: creates a snake that starts at the first
    // number and ends at the second number - i.e. if the player lands
    // on the first cell they are transported to land on the second
    // cell.
    Snake { starts: CellIx, ends: CellIx },
    
    // 6. powerup type cell list command describes a powerup that is
    // applied to a series of cells. When a player lands on a powerup
    // cell they acquire that powerup and retain it until they use
    // it. Using the powerup removes it from the player. A powerup
    // cell can be triggered any number of times by any player, but
    // they do not accumulate - a player either has the powerup or
    // they don't.
    PowerUps { typ: SubCommand, cells: Vec<CellIx> },
    
    // 7. turns 10 command: plays the specified number of turns (or
    // until a player wins the game). A turn means each player, in
    // order, rolls the dice, and then moves that many cells (or
    // double if they have the powerup). If the (possibly doubled)
    // roll would take them past the end of the board, they don't
    // move, and play proceeds to the next player. As soon as a player
    // wins, the game is over and the turns stop.
    Turns (usize)
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum SubCommand {
    
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

impl SubCommand {
    fn label(&self) -> String {
        match self {
            &SubCommand::Escalator => "e",
            &SubCommand::Antivenom => "a",
            &SubCommand::Double => "d",
        }.into()
    }
}

impl Command {
    // 2. Your module must have a readFrom function that accepts a string.
    // A command is a keyword followed by one or more parameters
    // separated by a single space.
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
                typ: SubCommand::new(ps[0]),
                cells: ps[1..].iter().map(|s| num(*s)).collect() },
            ("turns", ref ps) if ps.len() == 1 => Turns(num(ps[0])),
            _ => panic!("bad keyword")
        }
    }
}

impl SubCommand {
    fn new(name: &str) -> SubCommand {
        use SubCommand::*;

        match name {
            "escalator" => Escalator,
            "antivenom" => Antivenom,
            "double" => Double,
            _ => panic!("bad powerup")
        }
    }
}

/*

2. The last cell is the winning cell. If a player lands on it, by any
means, they win.

 */

// 4. A given cell can only have a single special property: winning,
// snake start, ladder start, or powerup. Note that the end of a snake or
// ladder could have a special property.
#[derive(Clone, Debug, PartialEq, Eq)]
enum CellProperty {
    Plain,
    // TODO: Collapse SnakeStart and LadderStart
    SnakeStart { end: CellIx },
    LadderStart { end: CellIx },
    PowerUp(SubCommand),
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

/*
5. A given cell can only have one player on it. When a player lands on
a cell (including initial positioning), if there is already a player
on the cell, that player gets bumped one cell. When a bumped player
lands on a cell, they get the action associated with that cell,
including winning, powerups, snakes, ladders, or bumping yet another
player.

6. Assume the input is perfectly legal, with no invalid commands,
extra spaces, invalid numbers, etc. Additionally assume the set-up
board will not produce any bump loops during play.
 */


#[derive(PartialEq, Eq, Debug, Default)]
struct BoardState {
    columns: usize,
    rows: usize,
    board: Vec<CellProperty>,
    player_pos: Vec<CellIx>, // 0 = off board at start
    dice: Vec<DieDots>,
    turn: usize,
}

impl BoardState {
    // 3. Your module must have a print function that formats the current
    // state of the board as a string.
    fn print(&self) -> String {
        let mut rev_out: Vec<String> = vec!();
        let sep = BoardState::between_columns();

        for row in 1..(self.rows + 1) {
            let mut first_line = String::from("");
            let mut second_line = String::from("");
            for column in 1..(self.columns + 1) {
                let ix = self.back_and_forth(row, column);

                first_line.push(sep);
                second_line.push(sep);

                first_line.push_str(&BoardState::cell_first_line(ix));
                second_line.push_str(&self.cell_second_line(ix));
            }
            first_line.push(sep);
            second_line.push(sep);

            rev_out.push(self.between_rows());
            rev_out.push(second_line);
            rev_out.push(first_line);
        }
        rev_out.push(self.between_rows());
        //TODO: winner line
        rev_out.reverse();
        return rev_out.join("\n");
    }

    // 1. If no player has won, that last line would not appear at all.
    //TODO: fn winner_line

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
    
    // The second line has: the player or blank, followed by the first
    // letter of the powerup or blank, followed by the start of a
    // snake or ladder, or blank.
    fn cell_second_line(&self, ix: CellIx) -> String {
        let which = (0..(self.player_pos.len())).find(
            |p| self.player_pos[*p] == ix);
        let player_spot = match which {
            Some(p) => player_name(p as u8),
            None => ' '
        };
        format!("{}{}", player_spot, self.board[ix - 1].label())
    }
    
    // 6. The output must be exactly as shown, as automatic comparison
    // will be part of marking.

    // 3. There can be any number of snake, ladder, or powerup
    // commands, and hence any number of the resulting game
    // features. There can also be any number of turn commands, each
    // of which will run in turn.
    fn apply(&mut self, command: Command) {
        use Command::*;
        use CellProperty::*;

        match command {
            Board { columns: y, rows: x } => {
                self.columns = y;
                self.rows = x;
                self.board = vec![Plain; x * y];
            },
            Players(n) => {
                self.player_pos = vec![0; n as usize]
            },
            Dice(ds) => { self.dice = ds },
            Ladder { starts: s, ends: e } => {
                self.board[s - 1] = LadderStart { end: e }
            },
            Snake { starts: s, ends: e } => {
                self.board[s - 1] = SnakeStart { end: e }
            },
            PowerUps { typ: pt, cells: cs } => {
                for c in cs {
                    self.board[c - 1] = PowerUp(pt.clone())
                }
            },
            Turns(n) => {
                for _ in 0..n {
                    let player = (self.turn % self.player_pos.len()) as usize;
                    let die = self.dice[self.turn % self.dice.len()];
                    println!("@@start turn: player {} rolls {}",
                             player_name(player as u8), die);
                    self.player_pos[player] += die as usize;

                    /*
                    Handle snakes, ladders, powerups
                    loop {
                        match self.board[target] {
                            SnakeStart { end: t2 } => target = t2,
                            LadderStart { end: t2 } => target = t2,
                            PowerUp(Escalator)
                        }
                     */
                    self.turn += 1;
                    println!("@@board:\n{}\n{:?}", self.print(), self);
                }
            },
        }
    }
}


// Players are named: A, B, ... There can be up to 26 players.
fn player_name(pix: PlayerIx) -> char {
    (('A' as u8) + (pix as u8)) as char
}

// When your `main` program is run, it must read commands from
// standard input, passing each line to the `readFrom` function. At
// the end of the input, it must print the state of the board on
// standard output.
fn main() {
    use std::io;
    use std::io::BufRead;

    let mut game = BoardState::default();
    let stdin = io::stdin();

    // 1. The input is a series of lines, each containing one command.
    for line in stdin.lock().lines() {
        println!("@@board:\n{}", game.print());
        let cmd = Command::readFrom(line.expect("lines failed?!"));
        game.apply(cmd)
    }

    let result = game.print();
    println!("@@result:\n{}", result);
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

    #[test]
    fn command_parse1() {
        use Command::*;
        assert!(Command::readFrom("board 3 4".into()) ==
                Board { columns: 3, rows: 4});
    }

    #[test]
    fn parse_sample_input() {
        use Command::*;
        use SubCommand::*;
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

    #[test]
    fn make_sample_output() {
        let mut game = BoardState::default();
        
        for line in SAMPLE_INPUT.trim().lines() {
            let cmd = Command::readFrom(line.into());
            game.apply(cmd);
        }

        assert!(game.print() == RESULTING_OUTPUT.trim());
    }

}
