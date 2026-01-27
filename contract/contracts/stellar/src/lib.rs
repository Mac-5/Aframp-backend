#[contractimpl]
impl AfrIContract {
    pub fn init(env: Env, admin: Address) {
        storage::set_admin(&env, &admin);
    }

    pub fn admin(env: &Env) -> Address {
        storage::get_admin(env)
    }

    pub fn balance_of(env: &Env, user: &Address) -> i128 {
        storage::get_balance(env, user)
    }

    pub fn set_balance(env: &Env, user: &Address, amount: i128) {
        storage::set_balance(env, user, amount)
    }

    pub fn total_supply(env: &Env) -> i128 {
        storage::get_total_supply(env)
    }

    pub fn set_total_supply(env: &Env, amount: i128) {
        storage::set_total_supply(env, amount)
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin = Self::admin(&env);
        admin.require_auth();

        let balance = Self::balance_of(&env, &to);
        Self::set_balance(&env, &to, balance + amount);

        let total_supply = Self::total_supply(&env);
        Self::set_total_supply(&env, total_supply + amount);
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        contract::burn(env, from, amount);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        contract::transfer(env, from, to, amount);
    }

    pub fn balance(env: Env, user: Address) -> i128 {
        storage::get_balance(&env, &user)
    }
}

mod test;