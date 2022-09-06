# just-a-blockchain

<p align="center">~ Just me implementing a blockchain in Rust ~</p>

<p align="center">Developed by <a href="https://veeso.github.io/" target="_blank">@veeso</a></p>
<p align="center">Current version: 0.1.0 (??/08/2022)</p>

<p align="center">
  <a href="https://opensource.org/licenses/MIT"
    ><img
      src="https://img.shields.io/badge/License-MIT-teal.svg"
      alt="License-MIT"
  /></a>
  <a href="https://github.com/veeso/just-a-blockchain/stargazers"
    ><img
      src="https://img.shields.io/github/stars/veeso/just-a-blockchain.svg"
      alt="Repo stars"
  /></a>
  <a href="https://ko-fi.com/veeso">
    <img
      src="https://img.shields.io/badge/donate-ko--fi-red"
      alt="Ko-fi"
  /></a>
</p>
<p align="center">
  <a href="https://github.com/veeso/just-a-blockchain/actions"
    ><img
      src="https://github.com/veeso/just-a-blockchain/workflows/Build/badge.svg"
      alt="Build CI"
  /></a>
</p>

---

- [just-a-blockchain](#just-a-blockchain)
  - [About just-a-blockchain 💸](#about-just-a-blockchain-)
  - [Get started 🏁](#get-started-)
  - [Support the developer ☕](#support-the-developer-)
  - [Changelog ⏳](#changelog-)
  - [License 📃](#license-)

---

## About just-a-blockchain 💸

Just-a-blockchain or JAB is just a blockchain I developed in Rust to learn how blockchains work. It is somehow inspired by Bitcoin.
The repository provides two binaries and the jab library. The first binary is `jab` which runs a node of the jab blockchain, while the other is `jab-wallet`, which can be used to interact with the blockchain nodes in order to check your balance and to spend your JABs.

> ⚠️ This blockchain IS SUPPOSED just to be used as a reference. DON'T USE IT for any real purpose, especially which involves money, since this blockchain just won't work. There's no proof of work of any kind of protection against double spending.

## Get started 🏁

1. Install dependencies

    ```sh
    sudo apt install -y libleveldb-dev
    # or on macos
    brew install leveldb
    ```

2. Setup environment

    ```sh
    cp .env.{PROFILE} .env
    ```

3. Create your wallet

    ```sh
    jab-wallet -w <YOUR_WALLET_DIR> -g
    ```

    > ❗ a node must be running to perform this command. You can run a node with the existing genesis key

4. Configure your environment

    ```env
    DATABASE_DIRECTORY="./db"
    WALLET_SECRET_KEY="<YOUR_WALLET_DIR>/.jab.key"
    ```

5. Run a node

    ```sh
    export RUST_LOG=debug
    jab
    ```

---

## Support the developer ☕

If you like just-a-blockchain and you're grateful for the work I've done, please consider a little donation 🥳

You can make a donation with one of these platforms:

[![ko-fi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/veeso)
[![PayPal](https://img.shields.io/badge/PayPal-00457C?style=for-the-badge&logo=paypal&logoColor=white)](https://www.paypal.me/chrisintin)
[![bitcoin](https://img.shields.io/badge/Bitcoin-ff9416?style=for-the-badge&logo=bitcoin&logoColor=white)](https://btc.com/bc1qvlmykjn7htz0vuprmjrlkwtv9m9pan6kylsr8w)

---

## Changelog ⏳

View just-a-blockchain's changelog [HERE](CHANGELOG.md)

---

## License 📃

just-a-blockchain is licensed under The Unlicense license.

You can read the entire license [HERE](LICENSE)
