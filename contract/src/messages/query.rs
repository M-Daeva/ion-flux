use cosmwasm_schema::{cw_serde, QueryResponses};

#[allow(unused_imports)] // preventing optimizer warning message
use cosmwasm_std::{Addr, Decimal, Uint128};

#[allow(unused_imports)] // preventing optimizer warning message
use crate::{
    messages::response::Balance,
    state::{Asset, Config, Token},
};

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    QueryConfig {},
    #[returns(Vec<(Addr, Decimal)>)]
    QueryTokensWeight { address_list: Vec<String> },
    #[returns(Vec<(Addr, Uint128)>)]
    QueryLiquidity { address_list: Vec<String> },
    #[returns(Vec<(Addr, Vec<Asset>)>)]
    QueryProviders { address_list: Vec<String> },
    #[returns(Vec<(Addr, Token)>)]
    QueryTokens { address_list: Vec<String> },
    #[returns(Vec<Balance>)]
    QueryBalances { address_list: Vec<String> },
    #[returns(Vec<(Addr, Decimal)>)]
    QueryPrices { address_list: Vec<String> },
    #[returns(Vec<(Addr, Decimal)>)]
    QueryPricesMocked { address_list: Vec<String> },
}
