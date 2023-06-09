/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.25.2.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { InstantiateMsg, ExecuteMsg, Uint128, Binary, Decimal, Cw20ReceiveMsg, QueryMsg, MigrateMsg, Addr, ArrayOfBalance, Balance, Config, ArrayOfTupleOfAddrAndUint128, ArrayOfTupleOfAddrAndDecimal, Timestamp, Uint64, ArrayOfTupleOfAddrAndArrayOfAsset, Asset, ArrayOfTupleOfAddrAndToken, Token, Sample } from "./IonFlux.types";
export interface IonFluxReadOnlyInterface {
  contractAddress: string;
  queryConfig: () => Promise<Config>;
  queryTokensWeight: ({
    addressList
  }: {
    addressList: string[];
  }) => Promise<ArrayOfTupleOfAddrAndDecimal>;
  queryLiquidity: ({
    addressList
  }: {
    addressList: string[];
  }) => Promise<ArrayOfTupleOfAddrAndUint128>;
  queryProviders: ({
    addressList
  }: {
    addressList: string[];
  }) => Promise<ArrayOfTupleOfAddrAndArrayOfAsset>;
  queryTokens: ({
    addressList
  }: {
    addressList: string[];
  }) => Promise<ArrayOfTupleOfAddrAndToken>;
  queryBalances: ({
    addressList
  }: {
    addressList: string[];
  }) => Promise<ArrayOfBalance>;
  queryPrices: ({
    addressList
  }: {
    addressList: string[];
  }) => Promise<ArrayOfTupleOfAddrAndDecimal>;
  queryPricesMocked: ({
    addressList
  }: {
    addressList: string[];
  }) => Promise<ArrayOfTupleOfAddrAndDecimal>;
}
export class IonFluxQueryClient implements IonFluxReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.queryConfig = this.queryConfig.bind(this);
    this.queryTokensWeight = this.queryTokensWeight.bind(this);
    this.queryLiquidity = this.queryLiquidity.bind(this);
    this.queryProviders = this.queryProviders.bind(this);
    this.queryTokens = this.queryTokens.bind(this);
    this.queryBalances = this.queryBalances.bind(this);
    this.queryPrices = this.queryPrices.bind(this);
    this.queryPricesMocked = this.queryPricesMocked.bind(this);
  }

  queryConfig = async (): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, {
      query_config: {}
    });
  };
  queryTokensWeight = async ({
    addressList
  }: {
    addressList: string[];
  }): Promise<ArrayOfTupleOfAddrAndDecimal> => {
    return this.client.queryContractSmart(this.contractAddress, {
      query_tokens_weight: {
        address_list: addressList
      }
    });
  };
  queryLiquidity = async ({
    addressList
  }: {
    addressList: string[];
  }): Promise<ArrayOfTupleOfAddrAndUint128> => {
    return this.client.queryContractSmart(this.contractAddress, {
      query_liquidity: {
        address_list: addressList
      }
    });
  };
  queryProviders = async ({
    addressList
  }: {
    addressList: string[];
  }): Promise<ArrayOfTupleOfAddrAndArrayOfAsset> => {
    return this.client.queryContractSmart(this.contractAddress, {
      query_providers: {
        address_list: addressList
      }
    });
  };
  queryTokens = async ({
    addressList
  }: {
    addressList: string[];
  }): Promise<ArrayOfTupleOfAddrAndToken> => {
    return this.client.queryContractSmart(this.contractAddress, {
      query_tokens: {
        address_list: addressList
      }
    });
  };
  queryBalances = async ({
    addressList
  }: {
    addressList: string[];
  }): Promise<ArrayOfBalance> => {
    return this.client.queryContractSmart(this.contractAddress, {
      query_balances: {
        address_list: addressList
      }
    });
  };
  queryPrices = async ({
    addressList
  }: {
    addressList: string[];
  }): Promise<ArrayOfTupleOfAddrAndDecimal> => {
    return this.client.queryContractSmart(this.contractAddress, {
      query_prices: {
        address_list: addressList
      }
    });
  };
  queryPricesMocked = async ({
    addressList
  }: {
    addressList: string[];
  }): Promise<ArrayOfTupleOfAddrAndDecimal> => {
    return this.client.queryContractSmart(this.contractAddress, {
      query_prices_mocked: {
        address_list: addressList
      }
    });
  };
}
export interface IonFluxInterface extends IonFluxReadOnlyInterface {
  contractAddress: string;
  sender: string;
  receive: ({
    amount,
    msg,
    sender
  }: {
    amount: Uint128;
    msg: Binary;
    sender: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  updateConfig: ({
    admin,
    priceAge,
    swapFeeRate,
    unbondingPeriod,
    window
  }: {
    admin?: string;
    priceAge?: Uint128;
    swapFeeRate?: Decimal;
    unbondingPeriod?: Uint128;
    window?: Uint128;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  updateToken: ({
    priceFeedIdStr,
    symbol,
    tokenAddr
  }: {
    priceFeedIdStr: string;
    symbol: string;
    tokenAddr: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  unbond: ({
    amount,
    tokenAddr
  }: {
    amount: Uint128;
    tokenAddr: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  withdraw: ({
    amount,
    tokenAddr
  }: {
    amount: Uint128;
    tokenAddr: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  claim: (fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  swapAndClaim: ({
    tokenOutAddr
  }: {
    tokenOutAddr: string;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
export class IonFluxClient extends IonFluxQueryClient implements IonFluxInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.receive = this.receive.bind(this);
    this.updateConfig = this.updateConfig.bind(this);
    this.updateToken = this.updateToken.bind(this);
    this.unbond = this.unbond.bind(this);
    this.withdraw = this.withdraw.bind(this);
    this.claim = this.claim.bind(this);
    this.swapAndClaim = this.swapAndClaim.bind(this);
  }

  receive = async ({
    amount,
    msg,
    sender
  }: {
    amount: Uint128;
    msg: Binary;
    sender: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      receive: {
        amount,
        msg,
        sender
      }
    }, fee, memo, funds);
  };
  updateConfig = async ({
    admin,
    priceAge,
    swapFeeRate,
    unbondingPeriod,
    window
  }: {
    admin?: string;
    priceAge?: Uint128;
    swapFeeRate?: Decimal;
    unbondingPeriod?: Uint128;
    window?: Uint128;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_config: {
        admin,
        price_age: priceAge,
        swap_fee_rate: swapFeeRate,
        unbonding_period: unbondingPeriod,
        window
      }
    }, fee, memo, funds);
  };
  updateToken = async ({
    priceFeedIdStr,
    symbol,
    tokenAddr
  }: {
    priceFeedIdStr: string;
    symbol: string;
    tokenAddr: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_token: {
        price_feed_id_str: priceFeedIdStr,
        symbol,
        token_addr: tokenAddr
      }
    }, fee, memo, funds);
  };
  unbond = async ({
    amount,
    tokenAddr
  }: {
    amount: Uint128;
    tokenAddr: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      unbond: {
        amount,
        token_addr: tokenAddr
      }
    }, fee, memo, funds);
  };
  withdraw = async ({
    amount,
    tokenAddr
  }: {
    amount: Uint128;
    tokenAddr: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      withdraw: {
        amount,
        token_addr: tokenAddr
      }
    }, fee, memo, funds);
  };
  claim = async (fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      claim: {}
    }, fee, memo, funds);
  };
  swapAndClaim = async ({
    tokenOutAddr
  }: {
    tokenOutAddr: string;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      swap_and_claim: {
        token_out_addr: tokenOutAddr
      }
    }, fee, memo, funds);
  };
}