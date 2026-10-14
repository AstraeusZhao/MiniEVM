use super::Step;

/// Receives a pre-instruction snapshot with frame identity for each executed opcode.
pub trait Tracer {
    /// Called once per instruction before constructing its owned stack snapshot.
    /// Return false to skip this event without affecting execution.
    fn capture_next(&mut self) -> bool {
        true
    }

    fn step(&mut self, event: Step);
}

/// In-memory tracer appropriate for tests and local diagnostics.
#[derive(Debug, Default)]
pub struct VecTracer {
    pub steps: Vec<Step>,
}
impl Tracer for VecTracer {
    fn step(&mut self, event: Step) {
        self.steps.push(event);
    }
}

/// Tracer with an explicit record limit for untrusted or long-running programs.
#[derive(Debug)]
pub struct LimitedTracer {
    steps: Vec<Step>,
    max_steps: usize,
    truncated: bool,
}

impl LimitedTracer {
    pub fn new(max_steps: usize) -> Self {
        Self { steps: Vec::new(), max_steps, truncated: false }
    }

    /// The recorded prefix, across all observed frames and transactions.
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Whether at least one instruction was omitted because the limit was reached.
    pub fn truncated(&self) -> bool {
        self.truncated
    }
}

impl Tracer for LimitedTracer {
    fn capture_next(&mut self) -> bool {
        if self.steps.len() < self.max_steps {
            true
        } else {
            self.truncated = true;
            false
        }
    }

    fn step(&mut self, event: Step) {
        if self.steps.len() < self.max_steps {
            self.steps.push(event);
        } else {
            self.truncated = true;
        }
    }
}
