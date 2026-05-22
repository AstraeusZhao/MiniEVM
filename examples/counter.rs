use minievm::{Address, Evm, Transaction, WorldState};
fn main() {
    let caller = Address::from_last_byte(1);
    let counter = Address::from_last_byte(2);
    let mut state = WorldState::default();
    state.account_mut(caller).balance = 1;
    // storage[0] = storage[0] + 1
    state.set_code(counter, vec![0x60, 0x00, 0x54, 0x60, 0x01, 0x01, 0x60, 0x00, 0x55, 0x00]);
    let mut evm = Evm::new(state);
    let result = evm.transact(Transaction {
        caller,
        target: counter,
        value: 0,
        input: vec![],
        gas_limit: 10_000,
        nonce: 0,
    });
    println!("{result:?}");
}
