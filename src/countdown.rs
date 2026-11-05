use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::America::Sao_Paulo;

#[derive(Debug, Eq, PartialEq)]
pub struct TimeLeft {
    pub days: u64,
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CountdownState {
    Counting(TimeLeft),
    Launched,
}

pub fn launch_time() -> DateTime<Utc> {
    Sao_Paulo
        .with_ymd_and_hms(2026, 11, 19, 0, 0, 0)
        .single()
        .expect("19/11/2026 00:00 must be a valid São Paulo time")
        .with_timezone(&Utc)
}

pub fn countdown_at(now: DateTime<Utc>) -> CountdownState {
    let seconds = launch_time().signed_duration_since(now).num_seconds();
    if seconds <= 0 {
        return CountdownState::Launched;
    }
    CountdownState::Counting(time_left_from_seconds(seconds as u64))
}

fn time_left_from_seconds(total: u64) -> TimeLeft {
    TimeLeft {
        days: total / 86_400,
        hours: total % 86_400 / 3_600,
        minutes: total % 3_600 / 60,
        seconds: total % 60,
    }
}

#[cfg(test)]
mod tests {
    use super::{CountdownState, TimeLeft, countdown_at, launch_time};
    use chrono::Duration;

    #[test]
    fn counts_the_second_before_launch() {
        assert_eq!(
            countdown_at(launch_time() - Duration::seconds(1)),
            CountdownState::Counting(TimeLeft {
                days: 0,
                hours: 0,
                minutes: 0,
                seconds: 1,
            })
        );
    }

    #[test]
    fn marks_launch_at_the_exact_target() {
        assert_eq!(countdown_at(launch_time()), CountdownState::Launched);
    }

    #[test]
    fn remains_launched_after_the_target() {
        assert_eq!(
            countdown_at(launch_time() + Duration::seconds(1)),
            CountdownState::Launched
        );
    }

    #[test]
    fn splits_multiple_days_into_clock_parts() {
        let state = countdown_at(launch_time() - Duration::seconds(176_461));
        assert_eq!(
            state,
            CountdownState::Counting(TimeLeft {
                days: 2,
                hours: 1,
                minutes: 1,
                seconds: 1,
            })
        );
    }
}
