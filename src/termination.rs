use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use limiga_core::termination::Terminator;

pub struct SignalTerminator {
    exit_signal_received: Arc<AtomicBool>,
    is_registered: bool,
}

impl SignalTerminator {
    /// Register the signal listeners. In case registering fails, this terminator will never cause
    /// the solver to stop running.
    pub fn register() -> SignalTerminator {
        let exit_signal_received = Arc::new(AtomicBool::new(false));

        let result = signal_hook::flag::register(
            signal_hook::consts::SIGINT,
            Arc::clone(&exit_signal_received),
        );

        SignalTerminator {
            exit_signal_received,
            is_registered: result.is_ok(),
        }
    }
}

impl Terminator for SignalTerminator {
    fn should_stop(&self) -> bool {
        if !self.is_registered {
            false
        } else {
            self.exit_signal_received.load(Ordering::Relaxed)
        }
    }
}

pub struct OrTerminator<A, B> {
    a: A,
    b: B,
}

impl<A, B> OrTerminator<A, B> {
    pub fn new(a: A, b: B) -> Self {
        OrTerminator { a, b }
    }
}

impl<A: Terminator, B: Terminator> Terminator for OrTerminator<A, B> {
    fn should_stop(&self) -> bool {
        self.a.should_stop() || self.b.should_stop()
    }
}
