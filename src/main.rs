use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashMap;
use std::io::{self, Write};

macro_rules! flushed_print {
    ($($arg:tt)*) => {
        print!(
            "{}",
            format_args!($($arg)*),
        );
        io::stdout().flush().unwrap();
    }
}

fn clear_screen() {
    flushed_print!("\x1B[2J\x1B[1;1H");
}

fn wait_for_enter() {
    read_line();
}

fn read_line() -> String {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();
    // remove newline
    buf.pop();
    buf
}

type PlayerID = u8;

#[derive(Debug)]
struct Player {
    id: PlayerID,
    nickname: String,
    score: u32,
    question_pending_answer: Option<Question>,
}

impl Player {
    fn new(id: PlayerID, nickname: String) -> Self {
        Self {
            id,
            nickname,
            score: 0,
            question_pending_answer: None,
        }
    }
}

#[derive(Debug)]
struct Question {
    author: PlayerID,
    prompt: String,
}

impl Question {
    fn new(author: PlayerID, prompt: String) -> Self {
        Self { author, prompt }
    }

    fn respond(self, answered_by: PlayerID, answer: String) -> AnsweredQuestion {
        AnsweredQuestion {
            question: self,
            answered_by,
            answer
        }
    }
}

#[derive(Debug)]
struct AnsweredQuestion {
    question: Question,
    answered_by: PlayerID,
    answer: String,
}

#[derive(Debug)]
struct Game {
    next_player_id: PlayerID,
    players: HashMap<PlayerID, Player>,
}

impl Game {
    fn new() -> Self {
        Self {
            next_player_id: 0,
            players: HashMap::new(),
        }
    }

    fn player_ids(&self) -> Vec<PlayerID> {
        self.players.keys().cloned().collect()
    }

    fn add_new_player(&mut self, nickname: String) -> PlayerID {
        let id = self.next_player_id;
        self.next_player_id += 1;
        self.players.insert(id, Player::new(id, nickname));

        id
    }

    fn start(&mut self) {
        for round in 1..=3 {
            clear_screen();
            flushed_print!("=> Press <ENTER> to start round {}", round);
            wait_for_enter();
            self.pend_questions();
            flushed_print!("=> Press <ENTER> to start answering");
            wait_for_enter();
            let answers = self.input_answers();
            flushed_print!("=> Press <ENTER> to start guessing");
            wait_for_enter();
            for answered in answers {
                println!("=> Question:\n\t{}", answered.question.prompt);
                println!("=> Response:\n\t{}", answered.answer);
            }
        }
    }

    fn input_answers(&mut self) -> Vec<AnsweredQuestion> {
        let mut answers = Vec::new();
        for p in self.player_ids() {
            self.summon_player(p);
            let pending_q = self.players.get_mut(&p).unwrap()
                .question_pending_answer.take().unwrap();
            println!("=> Answer this question:\n\t{}", pending_q.prompt);
            flushed_print!("Answer: ");
            let response = read_line();
            answers.push(pending_q.respond(p, response));
        }
        clear_screen();
        answers
    }

    fn summon_player(&self, p: PlayerID) {
        clear_screen();
        let player = self.players.get(&p).unwrap();
        flushed_print!("=> {}, press <ENTER>", player.nickname);
        wait_for_enter();
    }

    fn pend_questions(&mut self) {
        let pairs: HashMap<_, _> = self.generate_player_pairs().into_iter().collect();
        for p in self.player_ids() {
            self.summon_player(p);

            let question = self.input_question(p);
            let responder = pairs.get(&p).unwrap();
            self.players.get_mut(&responder).unwrap()
                .question_pending_answer = Some(question);

            clear_screen();
        }
    }

    fn input_question(&self, author: PlayerID) -> Question {
        flushed_print!("Enter a question: ");
        let prompt = read_line();

        Question::new(author, prompt)
    }

    fn generate_player_pairs(&self) -> Vec<(PlayerID, PlayerID)> {
        let mut order: Vec<_> = self.players.keys().cloned().collect();
        order.shuffle(&mut thread_rng());
        let mut pairs = Vec::with_capacity(order.len());
        let mut asker = *order.last().unwrap();
        for responder in order {
            pairs.push((asker, responder));
            asker = responder;
        }
        pairs
    }
}

fn main() {
    let mut game = Game::new();

    let mut player_handles = Vec::new();
    for &name in &["Joe", "Bob", "Fred"] {
        player_handles.push(game.add_new_player(name.into()));
    }
    game.start();
}
