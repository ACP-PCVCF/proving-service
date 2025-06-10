use risc0_zkvm::env;
use serde::{Serialize, Deserialize}; 

#[derive(Serialize, Deserialize, Debug)]
pub struct GuestMetrics {
    pub start_cycles: u64,
    pub end_cycles: u64,
    pub risc_v_cycles: u64,
}

impl GuestMetrics {
    pub fn new() -> Self {
        Self {
            start_cycles: 0,
            end_cycles: 0,
            risc_v_cycles: 0,
        }
    }

    pub fn start_riscv_cyc_count(&mut self) {
        self.start_cycles = env::cycle_count();
    }

    pub fn end_riscv_cyc_count(&mut self) {
        self.end_cycles = env::cycle_count();
        self.risc_v_cycles = self.end_cycles - self.start_cycles;
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use risc0_zkvm::guest::env;

    #[test]
    fn test_guest_metrics_cycle_count() {
        let mut gm = GuestMetrics::new();

        // Simulate start
        gm.start_riscv_cyc_count();
        // Simulated work: no-op
        gm.end_riscv_cyc_count();

        // Check that end > start and cycles computed
        assert!(gm.end_cycles >= gm.start_cycles);
        assert_eq!(gm.risc_v_cycles, gm.end_cycles - gm.start_cycles);
    }
}
