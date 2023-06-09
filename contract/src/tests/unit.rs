use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128};

use cw20::Cw20Coin;

use crate::{
    actions::{
        instantiate::{PRICE_AGE, SWAP_FEE_RATE, UNBONDING_PERIOD, WINDOW},
        math::{str_to_dec, u128_to_dec},
    },
    messages::response::Balance,
    state::{Asset, Config, Sample, Token},
    tests::helpers::{
        Project, ADDR_ADMIN_INJ, ADDR_ALICE_INJ, ADDR_BOB_INJ, CHAIN_ID_TESTNET,
        PRICE_FEED_ID_STR_ATOM, PRICE_FEED_ID_STR_LUNA, SYMBOL_ATOM, SYMBOL_LUNA,
    },
};

fn default_init() -> (Project, Addr, Cw20Coin) {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    (prj, token, mint_amount)
}

#[test]
fn create_cw20() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);
    let balance = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance, mint_amount.amount);
}

#[test]
fn update_config_default() {
    let (mut prj, ..) = default_init();

    prj.update_config(
        ADDR_ADMIN_INJ,
        None,
        None,
        None,
        None,
        Some(Uint128::from(2 * PRICE_AGE)),
    )
    .unwrap();

    assert_eq!(
        prj.query_config().unwrap(),
        Config::new(
            &Addr::unchecked(ADDR_ADMIN_INJ),
            SWAP_FEE_RATE,
            WINDOW,
            UNBONDING_PERIOD,
            2 * PRICE_AGE
        )
    );
}

#[test]
#[should_panic(expected = "Sender does not have access permissions!")]
fn update_config_unauthorized() {
    let (mut prj, ..) = default_init();

    prj.update_config(
        ADDR_ALICE_INJ,
        None,
        None,
        None,
        None,
        Some(Uint128::from(2 * PRICE_AGE)),
    )
    .unwrap();
}

#[test]
fn deposit() {
    let (mut prj, token, mint_amount) = default_init();

    let balance_contract = prj.get_cw20_balance(token.clone(), prj.address.clone());
    let balance_alice = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance_contract, mint_amount.amount);
    assert_eq!(balance_alice, Uint128::zero());
}

#[test]
#[should_panic(expected = "Token is not included in token list!")]
fn deposit_unsupported_token() {
    let (mut prj, _token, mint_amount) = default_init();

    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone()]);

    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();
}

#[test]
#[should_panic(expected = "There are not enough funds to withdraw!")]
fn withdraw_without_unbond() {
    let (mut prj, token, mint_amount) = default_init();

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
}

#[test]
#[should_panic(expected = "There are not enough funds to withdraw!")]
fn withdraw_with_unbond_but_too_early() {
    let (mut prj, token, mint_amount) = default_init();

    prj.unbond(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
}

#[test]
fn withdraw() {
    let (mut prj, token, mint_amount) = default_init();

    prj.unbond(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    prj.wait(UNBONDING_PERIOD as u64);

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    let balance_contract = prj.get_cw20_balance(token.clone(), prj.address.clone());
    let balance_alice = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance_contract, Uint128::zero());
    assert_eq!(balance_alice, mint_amount.amount);
}

#[test]
fn deposit_unbond_withdraw_loop() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(9u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    // deposit 9 ATOM
    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    // unbond 3 ATOM
    prj.unbond(ADDR_ALICE_INJ, &token, Uint128::from(3u128))
        .unwrap();

    // wait UNBONDING_PERIOD
    prj.wait(UNBONDING_PERIOD as u64);

    // withdraw 1 ATOM
    prj.withdraw(ADDR_ALICE_INJ, &token, Uint128::from(1u128))
        .unwrap();

    // unbond 1 ATOM
    prj.unbond(ADDR_ALICE_INJ, &token, Uint128::from(1u128))
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![Asset {
                token_addr: token.clone(),
                bonded: Uint128::from(5u128),
                unbonded: Uint128::from(2u128),
                requested: Uint128::from(1u128),
                counter: Timestamp::from_nanos(1571804619879305533u64),
                rewards: Uint128::from(0u128),
            }]
        )
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    // unbond 5 ATOM
    prj.unbond(ADDR_ALICE_INJ, &token, Uint128::from(5u128))
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![Asset {
                token_addr: token.clone(),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(2u128),
                requested: Uint128::from(6u128),
                counter: Timestamp::from_nanos(1571806419879305533u64),
                rewards: Uint128::from(0u128),
            }]
        )
    );

    // deposit 9 LUNA
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![
                Asset {
                    token_addr: token.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(2u128),
                    requested: Uint128::from(6u128),
                    counter: Timestamp::from_nanos(1571806419879305533u64),
                    rewards: Uint128::from(0u128),
                },
                Asset {
                    token_addr: token2.clone(),
                    bonded: Uint128::from(9u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(0u128),
                    counter: Timestamp::from_nanos(1571802819879305533u64),
                    rewards: Uint128::from(0u128),
                },
            ]
        )
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    // unbond 3 LUNA
    prj.unbond(ADDR_ALICE_INJ, &token2, Uint128::from(3u128))
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![
                Asset {
                    token_addr: token.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(2u128),
                    requested: Uint128::from(6u128),
                    counter: Timestamp::from_nanos(1571806419879305533u64),
                    rewards: Uint128::from(0u128),
                },
                Asset {
                    token_addr: token2.clone(),
                    bonded: Uint128::from(6u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(3u128),
                    counter: Timestamp::from_nanos(1571808219879305533u64),
                    rewards: Uint128::from(0u128),
                },
            ]
        )
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    // unbond 3 LUNA
    prj.unbond(ADDR_ALICE_INJ, &token2, Uint128::from(3u128))
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![
                Asset {
                    token_addr: token.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(8u128),
                    requested: Uint128::from(0u128),
                    counter: Timestamp::from_nanos(1571806419879305533u64),
                    rewards: Uint128::from(0u128),
                },
                Asset {
                    token_addr: token2.clone(),
                    bonded: Uint128::from(3u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(6u128),
                    counter: Timestamp::from_nanos(1571810019879305533u64),
                    rewards: Uint128::from(0u128),
                },
            ]
        )
    );

    // withdraw 8 ATOM
    prj.withdraw(ADDR_ALICE_INJ, &token, Uint128::from(8u128))
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![Asset {
                token_addr: token2.clone(),
                bonded: Uint128::from(3u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(6u128),
                counter: Timestamp::from_nanos(1571810019879305533u64),
                rewards: Uint128::from(0u128),
            },]
        )
    );

    // unbond 3 LUNA
    prj.unbond(ADDR_ALICE_INJ, &token2, Uint128::from(3u128))
        .unwrap();

    // wait UNBONDING_PERIOD
    prj.wait((UNBONDING_PERIOD) as u64);

    // withdraw 9 LUNA
    prj.withdraw(ADDR_ALICE_INJ, &token2, Uint128::from(9u128))
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap(),
        vec![(Addr::unchecked(ADDR_ALICE_INJ), vec![])]
    );
}

#[test]
fn deposit_2_providers() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(50u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    prj.unbond(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.unbond(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![
                Asset {
                    token_addr: token.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(5u128),
                    counter: Timestamp::from_nanos(1571801019879305533u64),
                    rewards: Uint128::from(0u128),
                },
                Asset {
                    token_addr: token2.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(5u128),
                    counter: Timestamp::from_nanos(1571801019879305533u64),
                    rewards: Uint128::from(0u128),
                },
            ]
        )
    );

    prj.wait((UNBONDING_PERIOD / 2) as u64);

    prj.deposit(ADDR_BOB_INJ, &token, mint_amount2.amount)
        .unwrap();
    prj.deposit(ADDR_BOB_INJ, &token2, mint_amount2.amount)
        .unwrap();

    prj.unbond(ADDR_BOB_INJ, &token, mint_amount2.amount)
        .unwrap();
    prj.unbond(ADDR_BOB_INJ, &token2, mint_amount2.amount)
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![
                Asset {
                    token_addr: token.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(5u128),
                    counter: Timestamp::from_nanos(1571801019879305533u64),
                    rewards: Uint128::from(0u128),
                },
                Asset {
                    token_addr: token2.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(5u128),
                    counter: Timestamp::from_nanos(1571801019879305533u64),
                    rewards: Uint128::from(0u128),
                },
            ]
        )
    );
    assert_eq!(
        prj.query_providers(vec![ADDR_BOB_INJ]).unwrap()[0],
        (
            Addr::unchecked(ADDR_BOB_INJ),
            vec![
                Asset {
                    token_addr: token.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(50u128),
                    counter: Timestamp::from_nanos(1571802819879305533u64),
                    rewards: Uint128::from(0u128),
                },
                Asset {
                    token_addr: token2.clone(),
                    bonded: Uint128::from(0u128),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(50u128),
                    counter: Timestamp::from_nanos(1571802819879305533u64),
                    rewards: Uint128::from(0u128),
                },
            ]
        )
    );

    prj.wait((UNBONDING_PERIOD / 2) as u64);

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.withdraw(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![ADDR_ALICE_INJ]).unwrap(),
        vec![(Addr::unchecked(ADDR_ALICE_INJ), vec![])]
    );
    assert_eq!(
        prj.query_providers(vec![]).unwrap(),
        vec![
            (Addr::unchecked(ADDR_ALICE_INJ), vec![]),
            (
                Addr::unchecked(ADDR_BOB_INJ),
                vec![
                    Asset {
                        token_addr: Addr::unchecked("contract1"),
                        bonded: Uint128::from(0u128),
                        unbonded: Uint128::from(0u128),
                        requested: Uint128::from(50u128),
                        counter: Timestamp::from_nanos(1571802819879305533u64),
                        rewards: Uint128::from(0u128),
                    },
                    Asset {
                        token_addr: Addr::unchecked("contract2"),
                        bonded: Uint128::from(0u128),
                        unbonded: Uint128::from(0u128),
                        requested: Uint128::from(50u128),
                        counter: Timestamp::from_nanos(1571802819879305533u64),
                        rewards: Uint128::from(0u128),
                    },
                ]
            ),
        ]
    );

    prj.wait((UNBONDING_PERIOD / 2) as u64);

    prj.withdraw(ADDR_BOB_INJ, &token, mint_amount2.amount)
        .unwrap();
    prj.withdraw(ADDR_BOB_INJ, &token2, mint_amount2.amount)
        .unwrap();

    assert_eq!(
        prj.query_providers(vec![]).unwrap(),
        vec![
            (Addr::unchecked(ADDR_ALICE_INJ), vec![]),
            (Addr::unchecked(ADDR_BOB_INJ), vec![]),
        ]
    );
}

#[test]
fn query_tokens() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(50u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_BOB_INJ, &token, mint_amount2.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    prj.wait(UNBONDING_PERIOD as u64);

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_BOB_INJ, &token2, mint_amount2.amount)
        .unwrap();

    assert_eq!(
        prj.query_tokens(vec![]).unwrap(),
        vec![
            (
                token,
                Token {
                    symbol: SYMBOL_ATOM.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_ATOM.to_string(),
                    bonded: (
                        vec![
                            Sample::new(
                                Uint128::from(28u128),
                                Timestamp::from_nanos(1571799219879305533u64),
                            ),
                            Sample::new(
                                Uint128::from(5u128),
                                Timestamp::from_nanos(1571801019879305533u64),
                            ),
                        ],
                        Uint128::from(16u128),
                    ),
                    requested: (vec![], Uint128::from(0u128)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
            (
                token2,
                Token {
                    symbol: SYMBOL_LUNA.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_LUNA.to_string(),
                    bonded: (
                        vec![
                            Sample::new(
                                Uint128::from(27u128),
                                Timestamp::from_nanos(1571799219879305533u64),
                            ),
                            Sample::new(
                                Uint128::from(50u128),
                                Timestamp::from_nanos(1571801019879305533u64),
                            ),
                        ],
                        Uint128::from(38u128),
                    ),
                    requested: (vec![], Uint128::from(0u128)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
        ]
    );
}

#[test]
fn swap_default() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mint_amount3 = Cw20Coin {
        address: ADDR_ADMIN_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(
        SYMBOL_ATOM,
        vec![
            mint_amount.clone(),
            mint_amount2.clone(),
            mint_amount3.clone(),
        ],
    );
    let token2 = prj.create_cw20(
        SYMBOL_LUNA,
        vec![mint_amount.clone(), mint_amount2, mint_amount3.clone()],
    );

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    let price_list = prj
        .query_prices_mocked(vec![token.as_str(), token2.as_str()])
        .unwrap();
    let (token_in_price, token_out_price) = (price_list[0].1, price_list[1].1);

    let amount_in = mint_amount3.amount / Uint128::from(10u128);
    let amount_out =
        ((Decimal::one() - str_to_dec(SWAP_FEE_RATE)) * u128_to_dec(amount_in) * token_in_price
            / token_out_price)
            .to_uint_floor();

    prj.swap(ADDR_ADMIN_INJ, amount_in, &token, &token2)
        .unwrap();

    assert_eq!(
        prj.get_cw20_balance(token, ADDR_ADMIN_INJ),
        mint_amount3.amount - amount_in
    );
    assert_eq!(
        prj.get_cw20_balance(token2, ADDR_ADMIN_INJ),
        mint_amount3.amount + amount_out
    );
}

#[test]
#[should_panic(expected = "Can not swap same tokens!")]
fn swap_same_tokens() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    let amount_in = mint_amount.amount / Uint128::from(10u128);

    prj.swap(ADDR_BOB_INJ, amount_in, &token, &token).unwrap();
}

#[test]
fn swap_updates_unbonded() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(
        ADDR_ALICE_INJ,
        &token,
        mint_amount.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.deposit(
        ADDR_ALICE_INJ,
        &token2,
        mint_amount.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // deposit x/2 of ATOM total liquidity
    prj.deposit(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // unbond x/2 of ATOM total liquidity
    prj.unbond(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait(UNBONDING_PERIOD as u64);

    let requested = prj.query_providers(vec![ADDR_BOB_INJ]).unwrap()[0].1[0].requested;

    // make some swaps to create trading volume
    // swap x/20 of ATOM total liquidity to LUNA
    prj.swap(
        ADDR_ALICE_INJ,
        mint_amount2.amount / Uint128::from(20u128),
        &token,
        &token2,
    )
    .unwrap();

    let unbonded = prj.query_providers(vec![ADDR_BOB_INJ]).unwrap()[0].1[0].unbonded;

    assert_eq!(requested, unbonded);
}

#[test]
fn claim_default() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    let price_list = prj
        .query_prices_mocked(vec![token.as_str(), token2.as_str()])
        .unwrap();
    let (token_in_price, token_out_price) = (price_list[0].1, price_list[1].1);

    let amount_in = mint_amount.amount / Uint128::from(10u128);
    let rewards = (str_to_dec(SWAP_FEE_RATE) * u128_to_dec(amount_in) * token_in_price
        / token_out_price)
        .to_uint_floor();

    prj.swap(ADDR_BOB_INJ, amount_in, &token, &token2).unwrap();

    prj.claim(ADDR_ALICE_INJ).unwrap();

    assert_eq!(
        prj.get_cw20_balance(token, ADDR_ALICE_INJ),
        mint_amount.amount + rewards
    );
}

#[test]
#[should_panic(expected = "There is nothing to claim!")]
fn claim_no_rewards() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    prj.claim(ADDR_ALICE_INJ).unwrap();
}

#[test]
fn swap_and_claim_default() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    let price_list = prj
        .query_prices_mocked(vec![token.as_str(), token2.as_str()])
        .unwrap();
    let (_token_in_price, token_out_price) = (price_list[0].1, price_list[1].1);

    let amount_in = mint_amount.amount / Uint128::from(4u128);
    let rewards = (u128_to_dec(
        (str_to_dec(SWAP_FEE_RATE) * u128_to_dec(amount_in) / token_out_price).to_uint_floor(),
    ) * token_out_price)
        .to_uint_floor();

    prj.swap(ADDR_BOB_INJ, amount_in, &token2, &token).unwrap();

    prj.swap_and_claim(ADDR_ALICE_INJ, &token).unwrap();

    assert_eq!(prj.get_cw20_balance(token, ADDR_ALICE_INJ), rewards);
}

#[test]
#[should_panic(expected = "There is nothing to claim!")]
fn swap_and_claim_no_rewards() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    prj.swap_and_claim(ADDR_ALICE_INJ, &token2).unwrap();
}

#[test]
fn query_config_default() {
    let (prj, ..) = default_init();

    assert_eq!(
        prj.query_config().unwrap(),
        Config::new(
            &Addr::unchecked(ADDR_ADMIN_INJ),
            SWAP_FEE_RATE,
            WINDOW,
            UNBONDING_PERIOD,
            PRICE_AGE
        )
    );
}

#[test]
fn query_tokens_weight_zero_volume() {
    let (prj, ..) = default_init();

    assert_eq!(
        prj.query_tokens_weight(vec![]).unwrap()[0].1,
        Decimal::one()
    );
}

#[test]
fn tokens_weight_decreases_on_swap_in() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res = prj.query_tokens_weight(vec![]).unwrap();

    // swap x/2 of ATOM total liquidity to LUNA
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(2u128),
        &token,
        &token2,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res2 = prj.query_tokens_weight(vec![]).unwrap();

    // as ATOM liquidity increases then its weight decreases
    assert!(res2[0] < res[0]);
}

#[test]
fn tokens_weight_increases_on_swap_out() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res = prj.query_tokens_weight(vec![]).unwrap();

    // swap x/4 of ATOM total liquidity to LUNA
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(4u128),
        &token,
        &token2,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // swap x/2 of LUNA total liquidity to ATOM
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(2u128),
        &token2,
        &token,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res2 = prj.query_tokens_weight(vec![]).unwrap();

    // as ATOM liquidity decreases then its weight increases
    assert!(res2[0] > res[0]);
}

#[test]
fn tokens_weight_decreases_on_deposit() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // make some swaps to create trading volume
    // swap x/20 of ATOM total liquidity to LUNA
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(20u128),
        &token,
        &token2,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // swap x/20 of LUNA total liquidity to ATOM
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(20u128),
        &token2,
        &token,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res = prj.query_tokens_weight(vec![]).unwrap();

    // deposit x/2 of ATOM total liquidity
    prj.deposit(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res2 = prj.query_tokens_weight(vec![]).unwrap();

    // as ATOM liquidity increases then its weight decreases
    assert!(res2[0] < res[0]);
}

#[test]
fn tokens_weight_increases_on_unbond() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // deposit x/2 of ATOM total liquidity
    prj.deposit(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res = prj.query_tokens_weight(vec![]).unwrap();

    // make some swaps to create trading volume
    // swap x/20 of ATOM total liquidity to LUNA
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(20u128),
        &token,
        &token2,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // swap x/20 of LUNA total liquidity to ATOM
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(20u128),
        &token2,
        &token,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // unbond x/2 of ATOM total liquidity
    prj.unbond(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res2 = prj.query_tokens_weight(vec![]).unwrap();

    // as ATOM liquidity decreases then its weight increases
    assert!(res2[0] > res[0]);
}

#[test]
fn tokens_weight_doesnt_change_on_withdraw() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // deposit x/2 of ATOM total liquidity
    prj.deposit(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // make some swaps to create trading volume
    // swap x/20 of ATOM total liquidity to LUNA
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(20u128),
        &token,
        &token2,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // swap x/20 of LUNA total liquidity to ATOM
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(20u128),
        &token2,
        &token,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // unbond x/2 of ATOM total liquidity
    prj.unbond(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD) as u64);

    let res = prj.query_tokens_weight(vec![]).unwrap();

    // unbond x/2 of ATOM total liquidity
    prj.withdraw(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    let res2 = prj.query_tokens_weight(vec![]).unwrap();

    // as ATOM liquidity doesn't change then its weight will be the same
    assert_eq!(res2[0], res[0]);
}

#[test]
fn query_liquidity_default() {
    let (mut prj, token, mint_amount) = default_init();

    prj.unbond(
        ADDR_ALICE_INJ,
        &token,
        mint_amount.amount / Uint128::from(5u128),
    )
    .unwrap();

    prj.wait(UNBONDING_PERIOD as u64);

    prj.unbond(
        ADDR_ALICE_INJ,
        &token,
        mint_amount.amount / Uint128::from(2u128),
    )
    .unwrap();

    assert_eq!(
        prj.query_liquidity(vec![]).unwrap()[0].1,
        Uint128::from(4u128) * mint_amount.amount / Uint128::from(5u128)
    );
}

#[test]
fn liquidity_manipulations() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(20_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // deposit x/2 of ATOM total liquidity
    prj.deposit(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // swap x/2 of LUNA total liquidity to ATOM
    prj.swap(
        ADDR_BOB_INJ,
        mint_amount2.amount / Uint128::from(2u128),
        &token2,
        &token,
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // unbond x/2 of ATOM total liquidity
    prj.unbond(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD) as u64);

    // withdraw x/2 of ATOM total liquidity
    prj.withdraw(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // deposit x/2 of ATOM total liquidity
    prj.deposit(
        ADDR_BOB_INJ,
        &token,
        mint_amount2.amount / Uint128::from(2u128),
    )
    .unwrap();
    prj.wait((UNBONDING_PERIOD / 100) as u64);

    // unbond x of ATOM total liquidity
    prj.unbond(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.wait((UNBONDING_PERIOD) as u64);

    let balances = prj.query_balances(vec![]).unwrap();
    let liquidity = prj.query_liquidity(vec![]).unwrap();
    let providers = prj.query_providers(vec![]).unwrap();

    let balance_atom = balances
        .iter()
        .find(|x| x.token_addr == token)
        .unwrap()
        .amount;
    let balance_luna = balances
        .iter()
        .find(|x| x.token_addr == token2)
        .unwrap()
        .amount;

    let liquidity_atom = liquidity
        .iter()
        .find(|(addr, _val)| addr == &token)
        .unwrap()
        .1;
    let liquidity_luna = liquidity
        .iter()
        .find(|(addr, _val)| addr == &token2)
        .unwrap()
        .1;

    let rewards_atom = providers.iter().fold(Uint128::zero(), |acc, cur| {
        let (_addr, asset_list) = cur;

        let rewards = match asset_list.iter().find(|x| x.token_addr == token) {
            Some(y) => y.rewards,
            _ => Uint128::zero(),
        };

        acc + rewards
    });

    let rewards_luna = providers.iter().fold(Uint128::zero(), |acc, cur| {
        let (_addr, asset_list) = cur;

        let rewards = match asset_list.iter().find(|x| x.token_addr == token2) {
            Some(y) => y.rewards,
            _ => Uint128::zero(),
        };

        acc + rewards
    });

    assert_eq!(balance_atom - rewards_atom, liquidity_atom);
    assert_eq!(balance_luna - rewards_luna, liquidity_luna);
}

#[test]
fn query_provider_not_found() {
    let (prj, ..) = default_init();

    assert_eq!(prj.query_providers(vec![ADDR_BOB_INJ]).unwrap(), vec![]);
}

#[test]
fn query_balances() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(50u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_BOB_INJ, &token, mint_amount2.amount)
        .unwrap();
    prj.deposit(ADDR_BOB_INJ, &token2, mint_amount2.amount)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    assert_eq!(
        prj.query_balances(vec![]).unwrap(),
        vec![
            Balance {
                token_addr: token,
                amount: Uint128::from(55u128),
            },
            Balance {
                token_addr: token2,
                amount: Uint128::from(55u128),
            },
        ]
    );
}

#[test]
fn query_prices_mocked_default() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mut prj = Project::new(None);

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount]);

    assert_eq!(
        prj.query_prices_mocked(vec![token.as_str(), token2.as_str()])
            .unwrap(),
        vec![(token, u128_to_dec(1u128)), (token2, u128_to_dec(2u128))]
    );
}

#[test]
#[should_panic(expected = "Mocked actions are disabled on real networks!")]
fn query_prices_mocked_real_network() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(100_000u128),
    };

    let mut prj = Project::new(Some(CHAIN_ID_TESTNET));

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount]);

    prj.query_prices_mocked(vec![token.as_str(), token2.as_str()])
        .unwrap();
}
