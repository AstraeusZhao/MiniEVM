use super::Step;

/// Receives a pre-instruction snapshot with frame identity for each executed opcode.
pub trait Tracer {
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
