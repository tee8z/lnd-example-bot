### Small example bot showing how to call a LND lightning node through it's REST API via rust
* Requires:
    * Macaroon with the `/getinfo` permissions (readonly will do fine)
    * Clearnet url of the node to use (normally on port :8080)
    * Folder `.config` at the root of this repo with the macaroon file placed inside
    * `Configuration/local.yaml` will need the following added:
    ```
    lnd:
        url: "https://<node alias>.t.voltageapp.io:8080" #NOTE: `t` is for testnet, `m` would be needed for mainnet
        macaroonpath: ".config/readonly.macaroon"
    ```

* What this bot does:
    - Calls out to the configured lnd node over clearnet and calls the node's `/getinfo` endpoint as a "fake" ping call to the node (lnd does not provide a `/ping` endpoint at this time)
    - This "ping" will repeate until the bot is killed, the wait time between "pings" is configure with the `pingfreqsecs` value (default wait time is 30 seconds)


* NOTE: The interesting peice of code for connecting to lnd can be found in [/src/lnd/lnd_manager.rs](./src/lnd/lnd_manager.rs)
The benfit of using the REST API over grpc comes down to how much of the lightning client do you actually need? If it's only a handful of endpoints, it may be easier to maintain standard REST instead of having to deal with lnd's proto and the issues that can arise with grpc in rust. This is a decision only the developer of a project can make for themselves. If you're looking for a way to connect using grpc in rust, tonic_lnd is a good crate to try: [tonic_lnd](https://crates.io/crates/tonic_lnd).

