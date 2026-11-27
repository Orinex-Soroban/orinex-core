use soroban_sdk::{contractevent, Address, Env};

/// Emitted once when the contract is initialized. Topics: `("init",)`.
#[contractevent(topics = ["init"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialized {
    pub admin: Address,
    pub token: Address,
    pub fee_recipient: Address,
    pub fee_bps: u32,
}

/// Emitted when the protocol fee changes. Topics: `("cfg", "fee")`.
#[contractevent(topics = ["cfg", "fee"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeBpsUpdated {
    pub fee_bps: u32,
}

/// Emitted when the fee recipient changes. Topics: `("cfg", "payout")`.
#[contractevent(topics = ["cfg", "payout"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeeRecipientUpdated {
    pub fee_recipient: Address,
}

/// Emitted when a deal is created. Topics: `("deal", "created", id)`.
#[contractevent(topics = ["deal", "created"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DealCreated {
    #[topic]
    pub deal_id: u64,
    pub payer: Address,
    pub payee: Address,
    pub arbiter: Address,
    pub amount: i128,
    pub deadline: u64,
}

/// Emitted when a payer funds a deal. Topics: `("deal", "funded", id)`.
#[contractevent(topics = ["deal", "funded"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DealFunded {
    #[topic]
    pub deal_id: u64,
    pub payer: Address,
    pub amount: i128,
}

/// Emitted when funds are released to the payee. Topics: `("deal", "released", id)`.
#[contractevent(topics = ["deal", "released"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DealReleased {
    #[topic]
    pub deal_id: u64,
    pub payee: Address,
    pub net: i128,
    pub fee: i128,
}

/// Emitted when funds are returned to the payer. Topics: `("deal", "refunded", id)`.
#[contractevent(topics = ["deal", "refunded"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DealRefunded {
    #[topic]
    pub deal_id: u64,
    pub payer: Address,
    pub amount: i128,
}

/// Emitted when a party escalates a funded deal. Topics: `("deal", "disputed", id)`.
#[contractevent(topics = ["deal", "disputed"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DealDisputed {
    #[topic]
    pub deal_id: u64,
    pub caller: Address,
}

/// Emitted when the arbiter settles a dispute. Topics: `("deal", "resolved", id)`.
#[contractevent(topics = ["deal", "resolved"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeResolved {
    #[topic]
    pub deal_id: u64,
    pub released_to_payee: bool,
    pub net: i128,
    pub fee: i128,
}

/// Publish the initialization event.
pub fn initialized(
    env: &Env,
    admin: &Address,
    token: &Address,
    fee_recipient: &Address,
    fee_bps: u32,
) {
    Initialized {
        admin: admin.clone(),
        token: token.clone(),
        fee_recipient: fee_recipient.clone(),
        fee_bps,
    }
    .publish(env);
}

/// Publish the fee change event.
pub fn fee_bps_updated(env: &Env, fee_bps: u32) {
    FeeBpsUpdated { fee_bps }.publish(env);
}

/// Publish the fee recipient change event.
pub fn fee_recipient_updated(env: &Env, fee_recipient: &Address) {
    FeeRecipientUpdated {
        fee_recipient: fee_recipient.clone(),
    }
    .publish(env);
}

/// Publish the deal creation event.
pub fn deal_created(env: &Env, deal: &crate::storage::Deal) {
    DealCreated {
        deal_id: deal.id,
        payer: deal.payer.clone(),
        payee: deal.payee.clone(),
        arbiter: deal.arbiter.clone(),
        amount: deal.amount,
        deadline: deal.deadline,
    }
    .publish(env);
}

/// Publish the deal funded event.
pub fn deal_funded(env: &Env, deal_id: u64, payer: &Address, amount: i128) {
    DealFunded {
        deal_id,
        payer: payer.clone(),
        amount,
    }
    .publish(env);
}

/// Publish the funds released event.
pub fn deal_released(env: &Env, deal_id: u64, payee: &Address, net: i128, fee: i128) {
    DealReleased {
        deal_id,
        payee: payee.clone(),
        net,
        fee,
    }
    .publish(env);
}

/// Publish the funds refunded event.
pub fn deal_refunded(env: &Env, deal_id: u64, payer: &Address, amount: i128) {
    DealRefunded {
        deal_id,
        payer: payer.clone(),
        amount,
    }
    .publish(env);
}

/// Publish the deal disputed event.
pub fn deal_disputed(env: &Env, deal_id: u64, caller: &Address) {
    DealDisputed {
        deal_id,
        caller: caller.clone(),
    }
    .publish(env);
}

/// Publish the dispute resolved event.
pub fn dispute_resolved(env: &Env, deal_id: u64, released_to_payee: bool, net: i128, fee: i128) {
    DisputeResolved {
        deal_id,
        released_to_payee,
        net,
        fee,
    }
    .publish(env);
}
