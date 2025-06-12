/*
use serde::{Serialize, Deserialize};
use risc0_zkvm::guest::env; // 


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
*/
include!(concat!(env!("OUT_DIR"), "/methods.rs"));