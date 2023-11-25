use anyhow::Result;
use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubmitResult {
    Accepted,
    WrongAnswer(WrongAnswerReason),
    TooFast,
    AlreadySolved,
    ManualSubmissionRequired,
    Locked,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WrongAnswerReason {
    TooHigh,
    TooLow,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    timestamp: DateTime<Utc>,
    answer: String,
    response: String,
    result: SubmitResult,
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
    let result = match resp.as_str() {
        "That's the right answer!" => SubmitResult::Accepted,
        "That's not the right answer." => SubmitResult::WrongAnswer(WrongAnswerReason::None),
        "That's not the right answer; your answer is too low." => SubmitResult::WrongAnswer(WrongAnswerReason::TooLow),
        "That's not the right answer; your answer is too high." => SubmitResult::WrongAnswer(WrongAnswerReason::TooHigh),
        "You gave an answer too recently; you have to wait before trying again." => SubmitResult::TooFast,
        "You don't seem to be solving the right level; did you already complete it?" => SubmitResult::AlreadySolved,
        _ => SubmitResult::Unknown,
    };

    let submission = Submission {
        timestamp: Utc::now(),
        answer: answer.to_string(),
        response: resp,
        result,
    };

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
