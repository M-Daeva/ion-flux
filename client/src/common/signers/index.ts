import { l } from "../utils";
import { fromBech32, toBech32 } from "@cosmjs/encoding";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import type {
  Keplr,
  Window as KeplrWindow,
  ChainInfo,
} from "@keplr-wallet/types";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import type { OfflineSigner } from "@cosmjs/launchpad";
import type { OfflineDirectSigner, EncodeObject } from "@cosmjs/proto-signing";
import { stringToPath } from "@cosmjs/crypto";
import {
  INJ_DENOM,
  DEFAULT_GAS_PRICE,
  DEFAULT_GAS_LIMIT,
} from "@injectivelabs/utils";
import type {
  ClientStruct,
  NetworkData,
  ChainResponse,
} from "../helpers/interfaces";
import {
  SigningStargateClient,
  coin,
  type StdFee,
  calculateFee as _calculateFee,
  GasPrice,
  type DeliverTxResponse,
} from "@cosmjs/stargate";

const fee: StdFee = {
  amount: [coin(DEFAULT_GAS_PRICE, INJ_DENOM)],
  gas: `${DEFAULT_GAS_LIMIT}`,
};

declare global {
  interface Window extends KeplrWindow {}
}

function _detectWallet() {
  const { keplr } = window;
  if (!keplr) throw new Error("You need to install Keplr");
  return keplr;
}

function _getChainInfo(
  asset: NetworkData | undefined,
  chainType: "main" | "test"
) {
  if (!asset) throw new Error("Chain registry info is not provided!");

  let network: ChainResponse | undefined;

  if (chainType === "main" && asset.main) {
    network = asset.main;
  }
  if (chainType === "test" && asset.test) {
    network = asset.test;
  }
  if (!network) throw new Error("Chain info is not found!");

  let chainInfo: ChainInfo = {
    chainId: network.chain_id,
    chainName: network.chain_name,
    rpc: network.apis.rpc[0].address,
    rest: network.apis.rest[0].address,
    stakeCurrency: {
      coinDenom: asset.symbol,
      coinMinimalDenom: asset.denomNative,
      coinDecimals: asset.exponent,
      coinGeckoId: asset.coinGeckoId,
    },
    // bip44: { coinType: 118 },
    bip44: { coinType: 60 },
    bech32Config: {
      bech32PrefixAccAddr: `${network.bech32_prefix}`,
      bech32PrefixAccPub: `${network.bech32_prefix}pub`,
      bech32PrefixValAddr: `${network.bech32_prefix}valoper`,
      bech32PrefixValPub: `${network.bech32_prefix}valoperpub`,
      bech32PrefixConsAddr: `${network.bech32_prefix}valcons`,
      bech32PrefixConsPub: `${network.bech32_prefix}valconspub`,
    },
    currencies: [
      {
        coinDenom: asset.symbol,
        coinMinimalDenom: asset.denomNative,
        coinDecimals: asset.exponent,
        coinGeckoId: asset.coinGeckoId,
      },
    ],
    feeCurrencies: [
      {
        coinDenom: asset.symbol,
        coinMinimalDenom: asset.denomNative,
        coinDecimals: asset.exponent,
        coinGeckoId: asset.coinGeckoId,
      },
    ],
  };

  return chainInfo;
}

async function _addChainList(
  wallet: Keplr,
  chainRegistry: NetworkData[],
  chainType: "main" | "test"
) {
  for (let asset of chainRegistry) {
    try {
      const chainInfo = _getChainInfo(asset, chainType);
      await wallet.experimentalSuggestChain(chainInfo);
    } catch (error) {
      l(error);
    }
  }
}

async function _unlockWalletList(
  wallet: Keplr,
  chainRegistry: NetworkData[],
  chainType: "main" | "test"
): Promise<void> {
  let promises: Promise<void>[] = [];

  for (let asset of chainRegistry) {
    try {
      const chainInfo = _getChainInfo(asset, chainType);
      promises.push(wallet.enable(chainInfo.chainId));
    } catch (error) {
      l(error);
    }
  }

  await Promise.all(promises);
}

async function initWalletList(
  chainRegistry: NetworkData[] | undefined,
  chainType: "main" | "test"
) {
  const wallet = _detectWallet();
  if (!chainRegistry || !wallet) return;
  await _addChainList(wallet, chainRegistry, chainType); // add network to Keplr
  await _unlockWalletList(wallet, chainRegistry, chainType); // give permission for the webpage to access Keplr
  return wallet;
}

async function _getSigner(clientStruct: ClientStruct, hdPath?: string) {
  let owner: string;
  let signer:
    | (OfflineSigner & OfflineDirectSigner)
    | undefined
    | DirectSecp256k1HdWallet;

  if ("wallet" in clientStruct) {
    const { chainId, wallet } = clientStruct;
    signer = window.getOfflineSigner?.(chainId);
    owner = (await wallet.getKey(chainId)).bech32Address;
  } else if ("seed" in clientStruct) {
    const { seed, prefix } = clientStruct;
    const hdPaths = hdPath ? [stringToPath(hdPath)] : undefined;
    signer = await DirectSecp256k1HdWallet.fromMnemonic(seed, {
      prefix,
      hdPaths,
    });
    owner = (await signer.getAccounts())[0].address;
  } else throw new Error("Wrong arguments!");

  return { signer, owner, RPC: clientStruct.RPC };
}

async function getSgClient(
  clientStruct: ClientStruct,
  hdPath?: string
): Promise<
  | {
      client: SigningStargateClient;
      owner: string;
    }
  | undefined
> {
  try {
    const { signer, owner, RPC } = await _getSigner(clientStruct, hdPath);
    if (!signer) throw new Error("Signer is undefined!");
    const client = await SigningStargateClient.connectWithSigner(RPC, signer);
    return { client, owner };
  } catch (error) {
    l(error);
  }
}

async function getCwClient(
  clientStruct: ClientStruct,
  hdPath?: string
): Promise<
  | {
      client: SigningCosmWasmClient;
      owner: string;
    }
  | undefined
> {
  try {
    const { signer, owner, RPC } = await _getSigner(clientStruct, hdPath);
    if (!signer) throw new Error("Signer is undefined!");
    const client = await SigningCosmWasmClient.connectWithSigner(RPC, signer);
    return { client, owner };
  } catch (error) {
    l(error);
  }
}

function getAddrByPrefix(address: string, prefix: string): string {
  return toBech32(prefix, fromBech32(address).data);
}

async function getAddrByChainPrefix(
  chainRegistry: NetworkData[],
  chainType: "main" | "test",
  prefix: string
) {
  let wallet = _detectWallet();
  if (!wallet) return;

  const chain = chainRegistry.find((item) => item.prefix === prefix);
  let chainId: string | undefined;
  if (chainType === "main") {
    chainId = chain?.main?.chain_id;
  } else {
    chainId = chain?.test?.chain_id;
  }
  if (!chain || !chainId) return;

  await _unlockWalletList(wallet, [chain], chainType); // give permission for the webpage to access Keplr
  return (await wallet.getKey(chainId)).bech32Address;
}

function signAndBroadcastWrapper(
  client: SigningStargateClient | SigningCosmWasmClient,
  signerAddress: string,
  margin: number = 1.2
) {
  return async (
    messages: readonly EncodeObject[],
    gasPrice: string | GasPrice,
    memo?: string
  ): Promise<DeliverTxResponse> => {
    const gasSimulated = await client.simulate(signerAddress, messages, memo);
    const gasWanted = Math.ceil(margin * gasSimulated);
    const fee = _calculateFee(gasWanted, gasPrice);
    return await client.signAndBroadcast(signerAddress, messages, fee, memo);
  };
}

function getGasPriceFromChainRegistryItem(
  chain: NetworkData,
  chainType: "main" | "test"
): string {
  const response = chainType === "main" ? chain.main : chain.test;

  const gasPriceAmountDefault = 0.005;
  let gasPriceAmount = 0;

  const minGasPrice = response?.fees.fee_tokens?.[0]?.fixed_min_gas_price;
  if (minGasPrice) gasPriceAmount = minGasPrice;

  gasPriceAmount = Math.max(gasPriceAmountDefault, gasPriceAmount);
  const gasPrice = `${gasPriceAmount}${chain.denomNative}`;

  return gasPrice;
}

export {
  getSgClient,
  getCwClient,
  getAddrByPrefix,
  initWalletList,
  getAddrByChainPrefix,
  signAndBroadcastWrapper,
  getGasPriceFromChainRegistryItem,
  _calculateFee,
  fee,
};
