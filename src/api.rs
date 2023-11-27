use anyhow::Result;
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use crate::{Config, Day, PartStatus};

fn url(year: u16, day: u8) -> String {
    format!("https://adventofcode.com/{year}/day/{day}")
}

pub async fn get_input(config: &Config, year: u16, day: u8) -> Result<String> {
    let url = url(year, day);
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{url}/input"))
        .header("Cookie", format!("session={}", config.token.as_ref().unwrap()))
        .send().await?
        .error_for_status()?
        .text().await?;
    Ok(resp)
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SubmitResult {
    Accepted,
    WrongAnswer(WrongAnswerReason),
    TooSoon(String),
    Invalid,
    Unknown(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum WrongAnswerReason {
    TooHigh,
    TooLow,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub timestamp: DateTime<Utc>,
    pub answer: String,
    pub result: SubmitResult,
}

pub async fn submit<'c>(config: &'c mut Config, year: u16, day: u8, part: u8, answer: &str) -> Result<&'c Submission> {
    let data = config.days
        .entry(year).or_default()
        .entry(day).or_insert(Day::new(year, day))
        .part(part);
    if let PartStatus::Solved(ref submission) = data.status {
        return Ok(submission);
    }

    let url = url(year, day);
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{url}/answer"))
        .header("Cookie", format!("session={}", config.token.as_ref().unwrap()))
        .form(&[("level", part.to_string()), ("answer", answer.to_string())])
        .send().await?
        .error_for_status()?
        .text().await?;

    let submission = parse_submit_response(&resp, answer);

    let PartStatus::Active { min, max, incorrect } = &mut data.status else { unreachable!() };
    match submission.result {
        SubmitResult::Accepted => {
            data.status = PartStatus::Solved(submission.clone());
        }
        SubmitResult::WrongAnswer(WrongAnswerReason::TooHigh) => {
            incorrect.push(answer.to_string());
            let answer = answer.parse()?;
            if let Some(max) = max {
                if answer < *max {
                    *max = answer;
                }
            } else {
                *max = Some(answer);
            }
        }
        SubmitResult::WrongAnswer(WrongAnswerReason::TooLow) => {
            incorrect.push(answer.to_string());
            let answer = answer.parse()?;
            if let Some(min) = min {
                if answer > *min {
                    *min = answer;
                }
            } else {
                *min = Some(answer);
            }
        }
        SubmitResult::WrongAnswer(WrongAnswerReason::None) => {
            incorrect.push(answer.to_string());
        }
        _ => {}
    }

    data.submissions.push(submission);

    Ok(data.submissions.last().unwrap())
}

fn parse_submit_response(resp: &str, answer: &str) -> Submission {
    let document = Html::parse_document(resp);
    let message = document.select(&Selector::parse("main > article").unwrap()).next().unwrap();  // TODO: Error handling
    let text = message.text().collect::<Vec<_>>().join("");
    let result = if text.contains("That's the right answer!") {
        SubmitResult::Accepted
    } else if text.contains("That's not the right answer") {
        let reason = if text.contains("too high") {
            WrongAnswerReason::TooHigh
        } else if text.contains("too low") {
            WrongAnswerReason::TooLow
        } else {
            WrongAnswerReason::None
        };
        SubmitResult::WrongAnswer(reason)
    } else if text.contains("You gave an answer too recently") {
        let time = text
            .split("You have ")
            .nth(1).unwrap()
            .split(" left to wait.")
            .next().unwrap();
        SubmitResult::TooSoon(time.into())
    } else if text.contains("You don't seem to be solving the right level") {
        SubmitResult::Invalid
    } else {
        SubmitResult::Unknown(text)
    };
    Submission {
        timestamp: Utc::now(),
        answer: answer.to_string(),
        result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_parse_submit_response {
        ($name:ident, $file:literal, $answer:literal => $result:expr) => {
            #[test]
            fn $name() {
                let submission = parse_submit_response(include_str!($file), $answer);
                assert_eq!(submission.answer, $answer);
                assert_eq!(submission.result, $result);
            }
        };
    }

    #[test]
    fn test_url() {
        assert_eq!(url(2023, 1), "https://adventofcode.com/2023/day/1");
    }

    test_parse_submit_response!(
        test_parse_submit_response_accepted,
        "../test_data/success.html",
        "123"
        => SubmitResult::Accepted
    );
    test_parse_submit_response!(
        test_parse_submit_too_high,
        "../test_data/too_high.html",
        "123"
        => SubmitResult::WrongAnswer(WrongAnswerReason::TooHigh)
    );
    // TODO: Get test data for this
    // test_parse_submit_response!(
    //     test_parse_submit_too_low,
    //     "../test_data/too_low.html",
    //     "123"
    //     => SubmitResult::WrongAnswer(WrongAnswerReason::TooLow)
    // );
    test_parse_submit_response!(
        test_parse_submit_response_incorrect,
        "../test_data/incorrect.html",
        "123"
        => SubmitResult::WrongAnswer(WrongAnswerReason::None)
    );
    test_parse_submit_response!(
        test_parse_submit_too_soon,
        "../test_data/too_soon.html",
        "123"
        => SubmitResult::TooSoon("58s".into())
    );
    test_parse_submit_response!(
        test_parse_submit_response_already_solved,
        "../test_data/already_solved.html",
        "123"
        => SubmitResult::Invalid
    );
}
