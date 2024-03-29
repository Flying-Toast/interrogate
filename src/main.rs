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

    fn add_new_player(&mut self, nickname: String) {
        let id = self.next_player_id;
        self.next_player_id += 1;
        self.players.insert(id, Player::new(id, nickname));
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
            clear_screen();
            self.do_guesses(&answers);
        }
        clear_screen();
        self.show_final_scores();
    }

    fn show_final_scores(&self) {
        let mut players: Vec<_> = self.players.values().collect();
        players.sort_by(|a, b| b.score.cmp(&a.score));
        let max_nickname_len = players.iter().max_by(|a, b| a.nickname.len().cmp(&b.nickname.len())).unwrap().nickname.len();
        println!("=> Final scores:");
        for player in &players {
            let padded_nickname = format!("{}{}", player.nickname, " ".repeat(max_nickname_len - player.nickname.len()));
            println!("{}: {} points", padded_nickname, player.score);
        }
    }

    fn do_guesses(&mut self, answers: &[AnsweredQuestion]) {
        for answered_q in answers {
            // tuple is (guesser, guess)
            let mut guesses: Vec<(PlayerID, PlayerID)> = Vec::new();
            let mut ps = self.players.values().collect::<Vec<_>>();
            clear_screen();
            ps.shuffle(&mut thread_rng());
            for player in ps {
                flushed_print!("=> {}, press enter to start guessing", player.nickname);
                wait_for_enter();
                clear_screen();
                if player.id == answered_q.answered_by {
                    println!("=> {}, your question is being guessed this round. Type a random number and press enter, so that people don't realize this is your question. (your answer will be ignored)", player.nickname);
                    wait_for_enter();
                } else {
                    println!("=> question:\n\t{}", answered_q.question.prompt);
                    println!("=> response:\n\t{}", answered_q.answer);
                    println!();
                    println!("=> ids:");
                    let mut sorted_players: Vec<_> = self.players.values().collect();
                    sorted_players.sort_by(|a, b| a.id.cmp(&b.id));
                    for player in sorted_players {
                        println!("{:2}: {}", player.id, player.nickname);
                    }
                    println!("=> {}, who do you think wrote this answer? Enter an ID from above.", player.nickname);
                    let guess: PlayerID;
                    loop {
                        flushed_print!("{}'s guess: ", player.nickname);
                        let input = read_line();
                        match input.parse() {
                            Ok(id) if self.players.contains_key(&id) => {
                                if id != player.id {
                                    guess = id;
                                    break;
                                } else {
                                    println!("=> You can't guess yourself!");
                                }
                            },
                            _ => {
                                println!("=> You need to enter an ID from the list.");
                            },
                        }
                    }
                    guesses.push((player.id, guess));
                }
                clear_screen();
            }
            flushed_print!("=> Guessing done. Press <ENTER> to see the results.");
            wait_for_enter();
            let answerer = self.players.get(&answered_q.answered_by).unwrap();
            println!("=> **{}** was the one who answered the question.", answerer.nickname);
            for (guesser_id, guessed_id) in guesses {
                let guesser = self.players.get_mut(&guesser_id).unwrap();
                if guessed_id == answered_q.answered_by {
                    let points = 1;
                    println!("=> {} was CORRECT. +{} points.", guesser.nickname, points);
                    println!("TODO: bonus if the asker is the only one who guessed correctly");
                    guesser.score += points;
                } else {
                    println!("=> {} was INCORRECT.", guesser.nickname);
                }
            }
            flushed_print!("=> Press <ENTER> continue.");
            wait_for_enter();
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
    loop {
        flushed_print!("Who's playing? Enter your name then press enter (or press enter if there are no more players to add): ");
        let line = read_line();
        if line == "" {
            break;
        } else {
            game.add_new_player(line);
        }
    }
    game.start();
}
