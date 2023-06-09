<script lang="ts">
  import { l } from "../../../common/utils";
  import { get } from "svelte/store";
  import { displayModal } from "../services/helpers";
  import { init } from "../workers/testnet-strat";
  import {
    addrToSymbol,
    symbolToAddr,
    trimDecimal,
  } from "../../../common/helpers/general";
  import {
    contractProvidersStorage,
    addressStorage,
    contractCw20BalancesStorage,
    contractPricesStorage,
    contractTokensStorage,
    contractTokensWeightStorage,
    contractLiquidityStorage,
    initAll,
  } from "../services/storage";
  import type { Asset } from "../../../common/codegen/IonFlux.types";

  let provider: Asset[] = [];
  let providerToDisplay: {
    token_addr: string;
    bonded: string;
    requested: string;
    unbonded: string;
    actionToExecute: string;
    amountToExecute: string;
  }[] = [];
  let currentRewardsSymbol = "";
  let currentRewardsSwapOutSymbol = "";
  let cw20Balances: [string, number][] = [];
  let priceList: [string, string][] = [];

  let poolsToDisplay: {
    token_addr: string;
    weight: string;
    liquidity: string;
    volume: string;
  }[] = [];

  const handlerList = {
    Deposit: deposit,
    Unbond: unbond,
    Withdraw: withdraw,
  };

  $: currentRewards =
    +(
      provider.find(
        ({ token_addr }) => token_addr === symbolToAddr(currentRewardsSymbol)
      )?.rewards || "0"
    ) / 1e6;

  $: totalRewardsCost =
    provider.reduce((acc, cur) => {
      const { rewards, token_addr } = cur;
      const currentPrice =
        +priceList.find(([addr, price]) => addr === token_addr)?.[1] || 0;
      const currentRewardsAmount = +rewards || 0;

      return acc + currentPrice * currentRewardsAmount;
    }, 0) / 1e6;

  // updatePrices
  contractPricesStorage.subscribe((value) => {
    priceList = value;
  });

  // update rewards
  contractProvidersStorage.subscribe((value) => {
    provider =
      value.find(([addr, asset]) => addr === get(addressStorage))?.[1] || [];

    providerToDisplay = provider.map((item) => ({
      ...item,
      actionToExecute: Object.keys(handlerList)[0],
      amountToExecute: "0",
    }));
  });

  // update symbols
  contractCw20BalancesStorage.subscribe((value) => {
    cw20Balances = value;
    currentRewardsSymbol = value?.[0]?.[0] || "";
    currentRewardsSwapOutSymbol = currentRewardsSymbol;
  });

  // update weights
  contractTokensWeightStorage.subscribe((addrAndWeightList) => {
    poolsToDisplay = addrAndWeightList.map(([addr, weight]) => {
      const pool = poolsToDisplay.find((item) => item.token_addr === addr);
      return pool
        ? { ...pool, weight }
        : { weight, token_addr: addr, liquidity: "0", volume: "0" };
    });
  });

  // update liquidity
  contractLiquidityStorage.subscribe((addrAndLiquidityList) => {
    poolsToDisplay = addrAndLiquidityList.map(([addr, liquidity]) => {
      const pool = poolsToDisplay.find((item) => item.token_addr === addr);
      return pool
        ? { ...pool, liquidity }
        : { liquidity, token_addr: addr, weight: "0", volume: "0" };
    });
  });

  // update volumes
  contractTokensStorage.subscribe((addrAndTokenList) => {
    poolsToDisplay = addrAndTokenList.map(([addr, token]) => {
      const pool = poolsToDisplay.find((item) => item.token_addr === addr);
      return pool
        ? { ...pool, volume: token.swapped_in[1] }
        : {
            volume: token.swapped_in[1],
            token_addr: addr,
            liquidity: "0",
            weight: "0",
          };
    });
  });

  async function claim() {
    try {
      const { cwClaim } = await init();
      const tx = await cwClaim();
      l(tx);
      displayModal(tx);
      await initAll();
    } catch (error) {
      l(error);
    }
  }

  async function swapAndClaim() {
    try {
      const { cwSwapAndClaim } = await init();
      const tx = await cwSwapAndClaim(
        symbolToAddr(currentRewardsSwapOutSymbol)
      );
      l(tx);
      displayModal(tx);
      await initAll();
    } catch (error) {
      l(error);
    }
  }

  async function deposit(tokenAddr: string, amount: number) {
    l({ tokenSymbol: addrToSymbol(tokenAddr), amount });
    try {
      const { cwDeposit } = await init();
      const tx = await cwDeposit(tokenAddr, amount);
      l(tx);
      displayModal(tx);
      await initAll();
    } catch (error) {
      l(error);
    }
  }

  async function unbond(tokenAddr: string, amount: number) {
    try {
      const { cwUnbond } = await init();
      const tx = await cwUnbond(tokenAddr, amount);
      l(tx);
      displayModal(tx);
      await initAll();
    } catch (error) {
      l(error);
    }
  }

  async function withdraw(tokenAddr: string, amount: number) {
    try {
      const { cwWithdraw } = await init();
      const tx = await cwWithdraw(tokenAddr, amount);
      l(tx);
      displayModal(tx);
      await initAll();
    } catch (error) {
      l(error);
    }
  }
</script>

<div class="flex flex-col px-4 -mt-3 pb-4">
  <!-- Rewards -->
  <h2 class="text-xl">Rewards</h2>
  <div
    class="flex flex-col sm:flex-row justify-around items-center pt-2 pb-4 text-amber-200 font-medium my-2"
    style="background-color: rgba(0, 200, 180, 0.1);"
  >
    <div class="flex flex-col justify-start">
      <div class="flex flex-row justify-start mt-2 sm:mt-5">
        <select
          id="symbol-selector"
          class="w-28 m-0 bg-stone-700"
          bind:value={currentRewardsSymbol}
        >
          {#each provider as { token_addr }}
            <option value={addrToSymbol(token_addr)}>
              {addrToSymbol(token_addr)}
            </option>
          {/each}
        </select>
        <span class="mx-1 my-auto">:</span>
        <span class="mx-1 my-auto">{trimDecimal(`${currentRewards}`)}</span>
      </div>

      <div class="mt-5">
        <span class="text-start">Sum in USD:</span>
        <span class="mx-1 my-auto">{trimDecimal(`${totalRewardsCost}`)}</span>
      </div>
    </div>

    <div class="flex flex-col mt-5 sm:mt-0">
      <div>
        <button class="btn btn-secondary m-0 w-28 mb-5" on:click={claim}
          >Claim</button
        >
      </div>
      <div class="flex flex-row">
        <button class="btn btn-secondary m-0 w-28" on:click={swapAndClaim}
          >Swap And Claim</button
        >
        <label for="symbol-selector" class="mx-2 my-auto">to</label>
        <select
          id="symbol-selector"
          class="w-28 m-0 bg-stone-700"
          bind:value={currentRewardsSwapOutSymbol}
        >
          {#each cw20Balances as [tokenSymbol, _]}
            <option value={tokenSymbol}>
              {tokenSymbol}
            </option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <!-- My pools -->
  <h2 class="text-xl mt-5">My Pools</h2>
  <div class="w-full overflow-x-auto mt-2">
    <table class="table table-compact w-full overflow-x-scroll">
      <thead class="bg-black flex text-white w-full">
        <tr class="flex w-full mb-1 justify-around">
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            POOL
          </th>
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            BONDED
          </th>
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            REQUESTED
          </th>
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            UNBONDED
          </th>
          <th
            class="flex flex-row justify-center items-center bg-black w-4/12 text-center"
          >
            CONTROL
          </th>
          <th class="w-12 bg-black"><div /></th>
        </tr>
      </thead>

      <tbody
        class="bg-grey-light flex flex-col items-center justify-start overflow-y-scroll w-full"
        style="max-height: 63vh; min-height: fit-content;"
      >
        {#each providerToDisplay as { token_addr, bonded, requested, unbonded, actionToExecute, amountToExecute }}
          <tr
            class="flex justify-around w-full mt-4 first:mt-0"
            style="background-color: rgb(42 48 60);"
          >
            <td
              class="flex flex-row justify-center items-center w-2/12 bg-inherit border-b-0"
              >{addrToSymbol(token_addr)}</td
            >
            <td
              class="flex justify-center items-center w-2/12 bg-inherit border-b-0"
              >{+bonded / 1e6}</td
            >
            <td
              class="flex justify-center items-center w-2/12 bg-inherit border-b-0"
              >{+requested / 1e6}</td
            >
            <td
              class="flex justify-center items-center w-2/12 bg-inherit border-b-0"
              >{+unbonded / 1e6}</td
            >
            <td
              class="flex justify-around content-center w-4/12 bg-opacity-90 bg-slate-800 border-b-0"
            >
              <div class="flex flex-row justify-center w-9/12">
                <select
                  id="symbol-selector"
                  class="w-28 mx-0 bg-stone-700 my-auto"
                  bind:value={actionToExecute}
                >
                  {#each Object.keys(handlerList) as actionToExecute}
                    <option value={actionToExecute}>
                      {actionToExecute}
                    </option>
                  {/each}
                </select>
                <input
                  type="number"
                  min="1"
                  max="100"
                  class="w-28 ml-2 my-auto text-center bg-stone-700"
                  bind:value={amountToExecute}
                />
              </div>

              <button
                class="btn btn-secondary m-0 w-3/12"
                on:click={() => {
                  handlerList[actionToExecute](
                    token_addr,
                    +amountToExecute * 1e6
                  );
                }}>Execute</button
              >
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  <!-- All pools -->
  <h2 class="text-xl mt-5">All Pools</h2>
  <div class="w-full overflow-x-auto mt-2">
    <table class="table table-compact w-full overflow-x-scroll">
      <thead class="bg-black flex text-white w-full">
        <tr class="flex w-full mb-1 justify-around">
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            POOL
          </th>
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            WEIGHT
          </th>
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            LIQUIDITY
          </th>
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            VOLUME
          </th>
          <th
            class="flex flex-row justify-center items-center bg-black w-2/12 text-center"
          >
            CONTROL
          </th>
        </tr>
      </thead>

      <tbody
        class="bg-grey-light flex flex-col items-center justify-start overflow-y-scroll w-full"
        style="max-height: 63vh; min-height: fit-content;"
      >
        {#each poolsToDisplay as { token_addr, weight, liquidity, volume }}
          <tr
            class="flex justify-around w-full mt-4 first:mt-0"
            style="background-color: rgb(42 48 60);"
          >
            <td
              class="flex flex-row justify-center items-center w-2/12 bg-inherit border-b-0"
              >{addrToSymbol(token_addr)}</td
            >
            <td
              class="flex justify-center items-center w-2/12 bg-inherit border-b-0"
              >{trimDecimal(weight)}</td
            >
            <td
              class="flex justify-center items-center w-2/12 bg-inherit border-b-0"
              >{trimDecimal(`${+liquidity / 1e6}`)}</td
            >
            <td
              class="flex justify-center items-center w-2/12 bg-inherit border-b-0"
              >{trimDecimal(`${+volume / 1e6}`)}</td
            >
            <td
              class="flex justify-around content-center w-2/12 bg-opacity-90 bg-slate-800 border-b-0"
            >
              <button
                class="btn btn-secondary m-0 w-28"
                on:click={() => {
                  const asset = providerToDisplay.find(
                    (item) => item.token_addr === token_addr
                  );
                  if (asset) return;

                  providerToDisplay = [
                    ...providerToDisplay,
                    {
                      token_addr,
                      bonded: "0",
                      requested: "0",
                      unbonded: "0",
                      actionToExecute: Object.keys(handlerList)[0],
                      amountToExecute: "0",
                    },
                  ];
                }}>Apply</button
              >
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
