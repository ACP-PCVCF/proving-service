use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
fn main() {
    // TODO: Implement your guest code here

    #[derive(Deserialize)]
    struct Activity {
        process_id: String,
        unit: String,
        consumption: u32,
        e_type: String,
    }

    let activities: Vec<Activity> = env::read();

    let emission_gasoline: u32 = activities
        .iter()
        .filter(|e| e.e_type == "gasoline")
        .map(|e| e.consumption * 2)
        .sum();
    let emission_greenpower: u32 = activities
        .iter()
        .filter(|e| e.e_type == "green power")
        .map(|e| e.consumption * 304)
        .sum();
    let emission_diesel: u32 = activities
        .iter()
        .filter(|e| e.e_type == "diesel")
        .map(|e| e.consumption * 274)
        .sum();

    let pcf_total: u32 = emission_diesel + emission_gasoline + emission_greenpower;

    env::commit(&pcf_total);

    println!("The pcf result is:{} kg CO2e", pcf_total);
}
