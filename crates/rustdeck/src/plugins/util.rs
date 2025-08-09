use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(thiserror::Error, Debug)]
#[error("Action timed out")]
pub struct TimeoutError {
    pub timeout: Duration,
    pub actual: Duration,
}

/// Block on execution of `func` and return its result.
/// Returns [`TimeoutError`] if `func` took more time than `dur` to complete.
pub fn timeout<F: FnOnce() -> T + Send, T: Send>(
    func: F,
    dur: Duration,
) -> Result<T, TimeoutError> {
    thread::scope(|s| {
        let handle = s.spawn(func);

        let timer = Instant::now();

        while !handle.is_finished() && timer.elapsed() < dur {
            thread::sleep(Duration::from_millis(10));
        }

        if handle.is_finished() {
            Ok(handle.join().unwrap())
        } else {
            Err(TimeoutError {
                timeout: dur,
                actual: timer.elapsed(),
            })
        }
    })
}
