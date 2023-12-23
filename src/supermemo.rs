use std::collections::HashMap;

use chrono::{prelude::*, Duration};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
pub struct Sm {
    /// the number of times the card has been successfully recalled in a row since the last time it was not.
    n: u32,
    /// how "easy" the card is (more precisely, it determines how quickly the inter-repetition interval grows)
    ef: f32,
    ///  is the length of time (in days) SuperMemo will wait after the previous review before asking the user to review the card again.
    interval: u32,

    last_reviewed: DateTime<Local>,
}

impl Default for Sm {
    // for init (new word)
    fn default() -> Self {
        Self {
            n: 0,
            ef: 2.5,
            interval: 1,
            last_reviewed: Local::now(),
        }
    }
}

impl Sm {
    fn next_review_time(&self) -> DateTime<Local> {
        self.last_reviewed + Duration::days(self.interval.try_into().unwrap())
    }

    // requires{0 <= user_grade <= 5}
    fn sm2(&self, user_grade: u8) -> Self {
        let n: u32;
        let I: u32;

        if user_grade >= 3 {
            if self.n == 0 {
                I = 1;
            } else if self.n == 1 {
                I = 6;
            } else {
                I = (self.interval as f32 * self.ef).round() as u32;
            }
            n = self.n + 1;
        } else {
            I = 1;
            n = 0;
        }

        let mut ef =
            self.ef + (0.1 - (5 - user_grade) as f32 * (0.08 + ((5 - user_grade) as f32) * 0.02));
        if ef < 1.3 {
            ef = 1.3;
        }
        Self {
            n,
            ef,
            interval: I,
            last_reviewed: Local::now(),
        }
    }
}

#[derive(Default)]
pub struct Deck(HashMap<String, Sm>);

impl Deck {
    pub fn search_reviewable(&self) -> Option<String> {
        for (k, v) in &self.0 {
            if v.next_review_time() <= Local::now() {
                return Some(k.to_owned());
            }
        }
        None
    }

    // requires{0 <= q <= 5}
    pub fn update(&mut self, question: String, q: u8) {
        let sm = self.0[&question].sm2(q);
        self.0.insert(question, sm);
    }

    pub fn fake_data() -> Self {
        Self(
            [(
                "hello".to_owned(),
                Sm {
                    n: 1,
                    ef: 2.5,
                    interval: 1,
                    last_reviewed: "2014-11-28T12:00:09Z".parse::<DateTime<Local>>().unwrap(),
                },
            )]
            .into_iter()
            .collect(),
        )
    }
}

/*
fn main() {
    let mut my_map = HashMap::new();
    my_map.insert("name", Sm::default());
    my_map.insert("city", Sm::default());

    // Using serde_json::to_string():
    let json_string = serde_json::to_string(&my_map).unwrap();
    println!("{}", json_string); // Output: {"name":"Alice","age":30,"city":"New York"}
}

fn main() {
    let json_string = r#"{"name":"Bob","age":"25"}"#;
    let deserialized_map: HashMap<String, String> = serde_json::from_str(json_string).unwrap();
    println!("{:?}", deserialized_map); // Output: {"name": "Bob", "age": "25"}
}
*/
