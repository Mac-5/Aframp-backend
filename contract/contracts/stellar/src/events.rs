use soroban_sdk::{contractevent, Address};

#[contractevent]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
pub struct Mint {
    pub to: Address,
    pub amount: i128,
}

#[contractevent]
pub struct Burn {
    pub from: Address,
    pub amount: i128,
}

pub fn transfer(env: &soroban_sdk::Env, from: &Address, to: &Address, amount: i128) {
    env.events().publish(Transfer {
        from: from.clone(),
        to: to.clone(),
        amount,
    });
}

pub fn mint(env: &soroban_sdk::Env, to: &Address, amount: i128) {
    env.events().publish(Mint {
        to: to.clone(),
        amount,
    });
}

pub fn burn(env: &soroban_sdk::Env, from: &Address, amount: i128) {
    env.events().publish(Burn {
        from: from.clone(),
        amount,
    });
}
