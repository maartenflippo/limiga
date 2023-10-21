use std::time::{Duration, Instant};

/// A terminator indicates when the solver should stop.
pub trait Terminator {
    /// Indicates whether the solver should stop or may continue.
    fn should_stop(&self) -> bool;
}

/// A time budget can be used to stop the solver after some duration.
pub struct TimeBudget {
    end_time: Option<Instant>,
}

impl TimeBudget {
    /// Create a time budget that starts now and gives the solver `duration` time to find a
    /// solution.
    pub fn starting_now(duration: Duration) -> TimeBudget {
        let end_time = Instant::now() + duration;

        TimeBudget {
            end_time: Some(end_time),
        }
    }

    /// Create an infinite time budget. The solver will not terminate before it finds a solution or
    /// concludes unsat.
    pub fn infinite() -> TimeBudget {
        TimeBudget { end_time: None }
    }
}

impl Terminator for TimeBudget {
    fn should_stop(&self) -> bool {
        self.end_time
            .map(|end_time| Instant::now() > end_time)
            .unwrap_or(false)
    }
}
