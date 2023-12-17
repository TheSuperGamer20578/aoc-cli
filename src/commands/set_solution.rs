use tracing::{info, warn};
use chrono::Utc;
use crate::{Config, PartStatus};
use crate::api::{Submission, SubmitResult};

pub fn set_solution(config: &mut Config, year: u16, day: u8, part: u8, solution: Option<String>) {
    let data = config.day(year, day).part(part);
    if let PartStatus::Solved(Submission {answer, ..}) = &data.status {
        warn!("Part is already solved!");
        info!("Previous solution: '{answer}'");
        if solution.is_some() {
            info!("Run without a solution to clear");
            return;
        }
    }
    match solution {
        Some(answer) => {
            data.status = PartStatus::Solved(Submission {
                timestamp: Utc::now(),
                result: SubmitResult::Accepted,
                answer,
            });
        }
        None => {
            data.status = PartStatus::default();
        }
    }
    info!("Successfully set solution");
}

