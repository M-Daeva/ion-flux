<script lang="ts">
  import { Router, Link, Route } from "svelte-navigator";
  import Home from "./routes/Home.svelte";
  import Exchange from "./routes/Exchange.svelte";
  import Liquidity from "./routes/Liquidity.svelte";
  import Faucet from "././routes/Faucet.svelte";
  import NotFound from "./routes/NotFound.svelte";
  import { init } from "./services/wallet";
  import Modal from "./components/Modal.svelte";
  import { displayAddress, getImageUrl } from "./services/helpers";
  import {
    isModalActiveStorage,
    initAll,
    CHAIN_TYPE,
  } from "./services/storage";

  const basePath = CHAIN_TYPE === "test" ? "/testnet" : "";

  const paths = {
    home: "/",
    exchange: `${basePath}/exchange`,
    liquidity: `${basePath}/liquidity`,
    faucet: `${basePath}/faucet`,
    other: "/*",
  };

  let checked = false;

  let isModalActive = false;

  function removeOverlay() {
    setTimeout(() => {
      checked = false;
    }, 0);
  }

  isModalActiveStorage.subscribe((value) => {
    isModalActive = value;
  });

  // init storages
  initAll();
</script>

<Router>
  <div
    class="text-amber-200 min-h-screen relative pb-40 sm:pb-14"
    style="background: #090017
  linear-gradient(
    135deg,
    rgba(217, 51, 137, 0.3) 0%,
    rgba(59, 195, 243, 0.3) 50%,
    rgba(49, 46, 129, 0.2) 100%
  );"
  >
    <header
      class="flex justify-between items-center px-4 mb-5 border-white border-b-2"
    >
      <div
        class="hidden sm:flex justify-center align-middle items-center w-3/12 -mt-2"
      >
        <Link
          class="text-center hover:no-underline visited:text-amber-200"
          to={paths.home}
        >
          <img
            class="w-36 -mb-2.5"
            src={getImageUrl("header-logo.png")}
            alt="Starbound-logo"
          />
        </Link>
      </div>
      <!-- hamburger -->
      <div
        class={"flex sm:hidden items-center w-16 h-16 z-50 sm:static " +
          (checked ? "fixed" : "justify-start")}
      >
        <label class="flex flex-col cursor-pointer">
          <input
            type="checkbox"
            id="check"
            class="hidden"
            on:click={() => (checked = !checked)}
          />

          <span
            class={"bg-amber-200 rounded-md h-1 w-10 mb-2 " +
              (checked
                ? "transform -rotate-45 origin-center translate-y-1.5"
                : "")}
          />
          <span
            class={"bg-amber-200 rounded-md h-1 w-10 mb-2 " +
              (checked ? "hidden" : "")}
          />
          <span
            class={"bg-amber-200 rounded-md h-1 w-10 " +
              (checked
                ? "transform rotate-45 origin-center -translate-y-1.5"
                : "")}
          />
        </label>
      </div>
      <!-- navbar -->
      <nav
        class={"font-normal md:font-medium text-xl " +
          (checked
            ? "fixed z-40 w-full h-full bg-gradient-to-br from-fuchsia-800 to-blue-900 text-center top-0 left-0"
            : "hidden sm:block sm:w-6/12")}
      >
        <ul class={"my-5 flex justify-around " + (checked ? "flex-col" : "")}>
          <li on:mousedown={removeOverlay} class={checked ? "mb-14" : "hidden"}>
            <Link
              class="text-center hover:no-underline visited:text-amber-200"
              to={paths.home}>Home</Link
            >
          </li>
          <li on:mousedown={removeOverlay} class={checked ? "mb-14" : ""}>
            <Link
              class="text-center hover:no-underline visited:text-amber-200"
              to={paths.exchange}>Exchange</Link
            >
          </li>
          <li on:mousedown={removeOverlay} class={checked ? "mb-14" : ""}>
            <Link
              class="text-center hover:no-underline visited:text-amber-200"
              to={paths.liquidity}>Liquidity</Link
            >
          </li>
          <li on:mousedown={removeOverlay}>
            <Link
              class="text-center hover:no-underline visited:text-amber-200"
              to={paths.faucet}>Faucet</Link
            >
          </li>
        </ul>
      </nav>
      <div class="w-36 flex flex-row">
        <img class="w-5 mr-1" src={getImageUrl("wallet.png")} alt="wallet" />
        <div class="font-normal">{displayAddress()}</div>
      </div>
      <button
        class="btn btn-primary mt-1.5 w-32 leading-4"
        on:click={async () => {
          const { initCwHandler } = await init();
          initCwHandler();
        }}>Connect Wallet</button
      >
    </header>

    <section>
      <Route primary={false} path={paths.home}><Home /></Route>
      <Route primary={false} path={paths.exchange}><Exchange /></Route>
      <Route primary={false} path={paths.liquidity}><Liquidity /></Route>
      <Route primary={false} path={paths.faucet}><Faucet /></Route>
      <Route primary={false} path={paths.other}><NotFound /></Route>
    </section>

    <footer
      class="absolute bottom-0 flex flex-wrap justify-between items-center w-full p-4 border-white border-t-2"
    >
      <a
        href="https://akash.network"
        class="w-full sm:w-4/12 mb-4 sm:mb-0 flex justify-center sm:justify-start"
      >
        <p class="font-medium">Powered by</p>
        <img
          src={getImageUrl("akash.svg")}
          alt="Powered by Akash"
          class="w-20 fill-lime-500 ml-2"
        />
      </a>

      <div
        class="w-full sm:w-4/12 mb-4 sm:mb-0 flex items-center justify-center"
      >
        <a href="https://github.com/M-Daeva/ion-flux" class="flex ml-2">
          <p class="font-medium">Built by M. Daeva</p>
          <img
            src={getImageUrl("github.png")}
            alt="Github link"
            class="w-6 ml-2"
          />
        </a>
      </div>

      <div class="w-full sm:w-4/12 flex justify-center sm:justify-end">
        <p class="font-medium">Socials:</p>
        <a href="/" class="flex ml-2">
          <img
            src={getImageUrl("discord.svg")}
            alt="Discord link"
            class="w-6 ml-1"
          />
        </a>
        <a href="/" class="flex ml-2">
          <img
            src={getImageUrl("twitter.svg")}
            alt="Twitter link"
            class="w-6 ml-1"
          />
        </a>
      </div>
    </footer>

    {#if isModalActive}
      <div class="fixed right-4 sm:right-5 bottom-5 z-20">
        <Modal />
      </div>
    {/if}
  </div>
</Router>
