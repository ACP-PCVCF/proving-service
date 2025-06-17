use risc0_zkvm::guest::env;
use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
        self.risc_v_cycles = self.end_cycles.saturating_sub(self.start_cycles);
    }
}

fn main() {
    // TODO: Implement your guest code here

    // Metriken initialisieren
    let mut guest_metrics = GuestMetrics::new();

    // read the input
    let input: u32 = env::read();
    let previous_id: [u32; 8] = env::read();
    let previous_receipt: Receipt = env::read();

    // Start der Zykluszählung
    guest_metrics.start_riscv_cyc_count();

    previous_receipt.verify(previous_id).unwrap();

    let pcf_previous: u32 = previous_receipt.journal.decode().unwrap();

    let pcf_new = pcf_previous + input;

    println!(
        "Generate proof that the new pcf (verified old + new pcf_activity) is:{}",
        pcf_new
    );

    guest_metrics.end_riscv_cyc_count();

    // TODO: do something with the input

    // write public output to the journal
    env::commit(&(&pcf_new, guest_metrics));
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