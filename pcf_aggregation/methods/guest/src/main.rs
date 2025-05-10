use risc0_zkvm::guest::env;
use risc0_zkvm::Receipt;

fn main() {
    // TODO: Implement your guest code here

    // read the input
    let input: u32 = env::read();
    let previous_id: [u32; 8] = env::read();
    let previous_receipt: Receipt = env::read();

    previous_receipt.verify(previous_id).unwrap();

    let pcf_previous: u32 = previous_receipt.journal.decode().unwrap();

    let pcf_new = pcf_previous + input;

    println!(
        "Generate proof that the new pcf (verified old + new pcf_activity) is:{}",
        pcf_new
    );

    // TODO: do something with the input

    // write public output to the journal
    env::commit(&pcf_new);
}
