[Skip to main content](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#__docusaurus_skipToContent_fallback)

Reactivate your Stylus contracts to ensure they remain callable - [here’s how to do it.](https://docs.arbitrum.io/stylus/gentle-introduction#activation)

[![Arbitrum Logo](https://docs.arbitrum.io/img/logo.svg)\\
**Arbitrum Docs**](https://docs.arbitrum.io/get-started/overview)

[Get started](https://docs.arbitrum.io/get-started/overview)

[Build apps](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#)

- [Build with Solidity](https://docs.arbitrum.io/build-decentralized-apps/quickstart-solidity-remix)
- [Build with Stylus](https://docs.arbitrum.io/stylus/quickstart)

[Launch a chain](https://docs.arbitrum.io/launch-arbitrum-chain/a-gentle-introduction) [Run a node](https://docs.arbitrum.io/run-arbitrum-node/overview) [Use the bridge](https://docs.arbitrum.io/arbitrum-bridge/quickstart) [How it works](https://docs.arbitrum.io/how-arbitrum-works/inside-arbitrum-nitro) [Notices](https://docs.arbitrum.io/notices/arbos51-upgrade-notice)

Search for anything...

`CtrlK`

- [Run an Arbitrum node](https://docs.arbitrum.io/run-arbitrum-node/overview)

  - [Overview](https://docs.arbitrum.io/run-arbitrum-node/overview)
  - [Run a full node](https://docs.arbitrum.io/run-arbitrum-node/run-full-node)
  - [Run a local full chain simulation](https://docs.arbitrum.io/run-arbitrum-node/run-local-full-chain-simulation)
  - [Run a local dev node](https://docs.arbitrum.io/run-arbitrum-node/run-nitro-dev-node)
  - [L1 Ethereum RPC providers](https://docs.arbitrum.io/run-arbitrum-node/l1-ethereum-beacon-chain-rpc-providers)
  - [Data Availability](https://docs.arbitrum.io/run-arbitrum-node/data-availability)
  - [Run a feed relay](https://docs.arbitrum.io/run-arbitrum-node/run-feed-relay)
  - [Historical blobs](https://docs.arbitrum.io/run-arbitrum-node/beacon-nodes-historical-blobs)
  - [ArbOS software releases](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#)

  - [More node types](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#)

  - [Sequencer](https://docs.arbitrum.io/node-running/sequencer-content-map)

  - [Build Nitro locally](https://docs.arbitrum.io/run-arbitrum-node/nitro/build-nitro-locally)
  - [Migrate to Nitro from Classic](https://docs.arbitrum.io/run-arbitrum-node/nitro/migrate-state-and-history-from-classic)
  - [Database snapshots](https://docs.arbitrum.io/run-arbitrum-node/nitro/nitro-database-snapshots)
  - [Convert node database](https://docs.arbitrum.io/run-arbitrum-node/nitro/how-to-convert-databases-from-leveldb-to-pebble)
  - [Troubleshooting](https://docs.arbitrum.io/run-arbitrum-node/troubleshooting)
  - [FAQ](https://docs.arbitrum.io/node-running/faq)
- [Chain Info↑](https://docs.arbitrum.io/for-devs/dev-tools-and-resources/chain-info)
- [Glossary↑](https://docs.arbitrum.io/intro/glossary)
- [Contribute↑](https://docs.arbitrum.io/for-devs/contribute)

On this page

[✏️Request an update](https://github.com/OffchainLabs/arbitrum-docs/issues/new?title=Docs%20update%20request:%20/run-arbitrum-node/run-full-node&body=Source:%20https://docs.arbitrum.io/run-arbitrum-node/run-full-node%0A%0ARequest:%20(how%20can%20we%20help?)%0A%0APsst,%20this%20issue%20will%20be%20closed%20with%20a%20templated%20response%20if%20it%20isn%27t%20a%20documentation%20update%20request.)

# How to run a full node for an Arbitrum chain

info

If you’re interested in accessing an Arbitrum chain but don’t want to set up your own node, see our [Node Providers](https://docs.arbitrum.io/build-decentralized-apps/reference/node-providers) to get RPC access to fully managed nodes hosted by a third-party provider.

This how-to provides step-by-step instructions for running a full node for Arbitrum on your local machine.

## Prerequisites [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#prerequisites "Direct link to Prerequisites")

In addition to the hardware requirements, the following prerequisites will be necessary when initially setting up your node. It is essential not to skip over these items. You would benefit by copying and pasting them into a notepad or text editor, as you will need to combine them with other commands and configuration/parameter options when you initially run your Arbitrum node.

### Minimum hardware configuration [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#minimum-hardware-configuration "Direct link to Minimum hardware configuration")

The following are the minimum hardware requirements to set up a Nitro full node (not archival):

| Resource | Recommended |
| --- | --- |
| RAM | 64 GB |
| CPU | 8 core CPU (for AWS, a `i4i.2xlarge` instance) |
| Storage type | NVMe SSD drives with locally attached drives strongly recommended |
| Storage size | Depends on the chain and its traffic over time |

Please note that:

- The minimum requirements for RAM and CPU listed here are recommended for nodes that handle a limited number of RPC requests. For nodes that need to process multiple simultaneous requests, both the RAM size and the number of CPU cores should be increased to accommodate higher levels of traffic.
- Single core performance is important. If the node is falling behind and a single core is 100% busy, the recommendation is to upgrade to a faster processor
- The minimum storage requirements will change over time as the chain grows. Using more than the minimum requirements to run a robust full node is recommended.

### Recommended Nitro version [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#recommended-nitro-version "Direct link to Recommended Nitro version")

caution

Although there are beta and release candidate versions of the Arbitrum Nitro software, use only the release version when running your node. Running beta or RC versions is not supported and might lead to unexpected behaviors and/or database corruption.

Latest [Docker image](https://hub.docker.com/r/offchainlabs/nitro-node/tags): `offchainlabs/nitro-node:v3.9.4-7f582c3`

### Database snapshots [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#database-snapshots "Direct link to Database snapshots")

Snapshots availability

Database snapshots are available and located in the [snapshot explorer](https://snapshot-explorer.arbitrum.io/) for Arbitrum One, Arbitrum Nova, and Arbitrum Sepolia. Database snapshots for other Arbitrum chains may be available at the discretion of the team running the chain. Please get in touch with them if you're interested in using a database snapshot for their chains.

Supplying a database snapshot when starting your node for the first time is required for Arbitrum One (to provide information from the Classic era) but is optional for other chains. Supplying a database snapshot on the first run will provide the state and data for that chain up to a specific block, allowing the node to sync faster to the head of the chain.

We provide a summary of the available parameters here, but we recommend reading the [complete guide](https://docs.arbitrum.io/run-arbitrum-node/nitro/nitro-database-snapshots) if you plan to use snapshots.

- Use the parameter `--init.latest <snapshot type>` (accepted values: `archive`, `pruned`, `genesis`) to instruct your node to download the corresponding snapshot from the configured URL
- Optionally, use the parameter `--init.latest-base` to set the base URL when searching for the latest snapshot
- Note that these parameters get ignored if a database already exists
- When running more than one node, it's easier to manually download the different parts of the snapshot, join them into a single archive, and host it locally for your nodes. Please see [Downloading the snapshot manually](https://docs.arbitrum.io/run-arbitrum-node/nitro/nitro-database-snapshots#downloading-the-snapshot-manually) for instructions on how to do that.

Fusaka upgrade: Historical blobs

If running a beacon node, historical data will now be in blobs. To make this transition to using historical blobs refer to the [Historical Blobs for Beacon Nodes](https://docs.arbitrum.io/run-arbitrum-node/beacon-nodes-historical-blobs) guide.

### Required parameters [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#required-parameters "Direct link to Required parameters")

The following list contains all the parameters needed to configure your node. Select the appropriate option depending on the chain you want to run your node for.

- Arbitrum One, Nova, Sepolia
- Arbitrum chains

#### 1\. Parent chain (Ethereum) parameters [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#1-parent-chain-ethereum-parameters "Direct link to 1. Parent chain (Ethereum) parameters")

The `--parent-chain.connection.url` parameter needs to provide a standard RPC endpoint for an Ethereum node, whether self-hosted or obtained from a node service provider:

```shell
--parent-chain.connection.url=<Ethereum RPC URL>
```

Additionally, use the parameter `--parent-chain.blob-client.beacon-url` to provide a beacon chain RPC endpoint:

```shell
--parent-chain.blob-client.beacon-url=<Ethereum beacon chain RPC URL>
```

Try it out

If you choose to self-host an EVM node, the [Prysm client software](https://www.offchainlabs.com/prysm/docs) is a great choice. It's straightforward, efficient, and effective—ensuring your setup runs smoothly!

You can also consult our [list of Ethereum beacon chain RPC providers](https://docs.arbitrum.io/run-arbitrum-node/l1-ethereum-beacon-chain-rpc-providers). Note that historical blob data is required for these chains to properly sync up if they are new or have been offline for more than 18 days. The beacon chain RPC endpoint you use may also need to provide historical blob data. Please see [Special notes on ArbOS 20: Atlas support for EIP-4844](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos20#special-notes-on-arbos-20-atlas-support-for-eip-4844) for more details.

#### 2\. Arbitrum chain parameters [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#2-arbitrum-chain-parameters "Direct link to 2. Arbitrum chain parameters")

Use the parameter `--chain.id` to specify the chain you're running this node for. See [RPC endpoints and providers](https://docs.arbitrum.io/build-decentralized-apps/reference/node-providers) to find the IDs of these chains.

```shell
--chain.id=<Arbitrum chain ID>
```

Alternatively, you can use the parameter `--chain.name` to specify the chain you're running this node for. Use `arb1` for Arbitrum One, `nova` for Arbitrum Nova, or `sepolia-rollup` for Arbitrum Sepolia.

```shell
--chain.name=<Child chain name>
```

#### 1\. Parent chain parameters [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#1-parent-chain-parameters "Direct link to 1. Parent chain parameters")

The `--parent-chain.connection.url` parameter needs to provide a standard RPC endpoint for an EVM node, whether self-hosted or obtained from a node service provider:

```shell
--parent-chain.connection.url=<Parent chain RPC URL>
```

Try it out

If you choose to self-host an EVM node, the [Prysm client software](https://www.offchainlabs.com/prysm/docs) is a great choice. It's straightforward, efficient, and effective—ensuring your setup runs smoothly!

Additionally, if the chain is a Layer-2 (L2) chain on top of Ethereum and uses blobs to post calldata, use the parameter `--parent-chain.blob-client.beacon-url` to provide a beacon chain RPC endpoint:

```shell
--parent-chain.blob-client.beacon-url=<Parent chain beacon chain RPC URL>
```

Public Arbitrum RPC endpoints

[Public Arbitrum RPC endpoints](https://docs.arbitrum.io/build-decentralized-apps/reference/node-providers#arbitrum-public-rpc-endpoints) rate-limit connections. To avoid hitting a bottleneck, you can run a local node for the parent chain or rely on third-party RPC providers.

You can find beacon providers in our [list of Ethereum beacon chain RPC providers](https://docs.arbitrum.io/run-arbitrum-node/l1-ethereum-beacon-chain-rpc-providers). Note that historical blob data is required for these chains to properly sync up if they are new or have been offline for more than 18 days. This means that the beacon chain RPC endpoint you use may also need to provide historical blob data. Please see [Special notes on ArbOS 20: Atlas support for EIP-4844](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos20#special-notes-on-arbos-20-atlas-support-for-eip-4844) for more details.

#### 2\. Child chain parameters [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#2-child-chain-parameters "Direct link to 2. Child chain parameters")

The parameter `--chain.info-json` specifies a JSON string that contains the information about the Arbitrum chain required by the node.

```shell
--chain.info-json=<Orbit chain's info>
```

This information should be provided by the chain owner and will look something like the following:

```shell
--chain.info-json="[{\"chain-id\":94692861356,\"parent-chain-id\":421614,\"chain-name\":\"My Arbitrum L3 Chain\",\"chain-config\":{\"chainId\":94692861356,\"homesteadBlock\":0,\"daoForkBlock\":null,\"daoForkSupport\":true,\"eip150Block\":0,\"eip150Hash\":\"0x0000000000000000000000000000000000000000000000000000000000000000\",\"eip155Block\":0,\"eip158Block\":0,\"byzantiumBlock\":0,\"constantinopleBlock\":0,\"petersburgBlock\":0,\"istanbulBlock\":0,\"muirGlacierBlock\":0,\"berlinBlock\":0,\"londonBlock\":0,\"clique\":{\"period\":0,\"epoch\":0},\"arbitrum\":{\"EnableArbOS\":true,\"AllowDebugPrecompiles\":false,\"DataAvailabilityCommittee\":false,\"InitialArbOSVersion\":10,\"InitialChainOwner\":\"0xAde4000C87923244f0e95b41f0e45aa3C02f1Bb2\",\"GenesisBlockNum\":0}},\"rollup\":{\"bridge\":\"0xde835286442c6446E36992c036EFe261AcD87F6d\",\"inbox\":\"0x0592d3861Ea929B5d108d915c36f64EE69418049\",\"sequencer-inbox\":\"0xf9d77199288f00440Ed0f494Adc0005f362c17b1\",\"rollup\":\"0xF5A42aDA664E7c2dFE9DDa4459B927261BF90E09\",\"validator-utils\":\"0xB11EB62DD2B352886A4530A9106fE427844D515f\",\"validator-wallet-creator\":\"0xEb9885B6c0e117D339F47585cC06a2765AaE2E0b\",\"deployed-at\":1764099}}]"
```

Use the parameter `--chain.name` to specify the chain you're running this node for. The name of the chain should match the name used in the JSON string used in `--chain.info-json`:

```shell
--chain.name=<Orbit chain name>
```

#### 3\. Parameters to connect to the sequencer [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#3-parameters-to-connect-to-the-sequencer "Direct link to 3. Parameters to connect to the sequencer")

Use the parameter `--node.feed.input.url` to point at the sequencer feed endpoint, which should be provided by the chain owner.

```shell
--node.feed.input.url=<Sequencer feed url>
```

Use the parameter `--execution.forwarding-target` to point at the sequencer node of the Arbitrum chain, which should also be provided by the chain owner.

```shell
--execution.forwarding-target=<Sequencer node endpoint url>
```

#### 3\. Additional parameters for AnyTrust chains [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#3-additional-parameters-for-anytrust-chains "Direct link to 3. Additional parameters for AnyTrust chains")

If you're running a node for an Anytrust chain, you need to specify information about the Data Availability Committee (DAC) in the configuration of your node.

First, enable `data-availability` using the following parameters:

```shell
--node.data-availability.enable
--node.data-availability.rest-aggregator.enable
```

Then, choose one of these methods to specify the DAS REST endpoints that your node will read the information from. These endpoints should also be provided by the chain owner.

1. Set the DAS REST endpoints directly:

```shell
--node.data-availability.rest-aggregator.urls=<A list of DAS REST endpoints, separated by commas>
```

2. Set a URL that returns a list of the DAS REST endpoints:

```shell
--node.data-availability.rest-aggregator.online-url-list=<A URL that returns a list of the DAS REST endpoints>
```

Setting a DAS (for chain owners)

If you are a chain owner, please refer to the [DAC setup guide](https://docs.arbitrum.io/launch-arbitrum-chain/configure-your-chain/common/data-availability/data-availability-committees/get-started#if-you-are-a-chain-owner) to set it up.

Additionally, for your batch poster to post data to the DAS, follow [Step 3 of How to configure a DAC](https://docs.arbitrum.io/launch-arbitrum-chain/configure-your-chain/common/data-availability/data-availability-committees/configure-dac#step-3-craft-the-new-configuration-for-the-batch-poster) to configure your batch poster node.

## Putting it into practice: run a node [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#putting-it-into-practice-run-a-node "Direct link to Putting it into practice: run a node")

Caution

If you are running more than on node, you should [run a feed relay](https://docs.arbitrum.io/run-arbitrum-node/run-feed-relay).

- To ensure the database persists across restarts, mount an external volume when running the Docker image. Use the mount point `/home/user/.arbitrum` within the Docker image.

Node config file

If using a `node-config.json` file with Docker to mount, use the following command:

```text
docker run --rm -it -v /Path/to/mount/arbitrum:/home/user/.arbitrum -v /Path/to/node-config.json:/home/user/.arbitrum/node-config.json -p 0.0.0.0:8450:8450  offchainlabs/nitro-node:v3.9.0-cca645a --conf.file /home/user/.arbitrum/node-config.json
```

- Here is an example of how to run `nitro-node`:

- Arbitrum One, Nova, Sepolia
- Arbitrum chains

```shell
docker run --rm -it -v /some/local/dir/arbitrum:/home/user/.arbitrum -p 0.0.0.0:8547:8547 -p 0.0.0.0:8548:8548 offchainlabs/nitro-node:v3.9.4-7f582c3 --parent-chain.connection.url=<Ethereum RPC URL> --parent-chain.blob-client.beacon-url=<Ethereum beacon chain RPC URL> --chain.id=<Arbitrum chain id> --init.latest=pruned --http.api=net,web3,eth --http.corsdomain=* --http.addr=0.0.0.0 --http.vhosts=*
```

```shell
docker run --rm -it -v /some/local/dir/arbitrum:/home/user/.arbitrum -p 0.0.0.0:8547:8547 -p 0.0.0.0:8548:8548 offchainlabs/nitro-node:v3.9.4-7f582c3 --parent-chain.connection.url=<Parent chain RPC URL>  --chain.info-json=<Orbit chain's info> --chain.name=<Orbit chain name> --node.feed.input.url=<Sequencer feed url> --execution.forwarding-target=<Sequencer node endpoint url> --http.api=net,web3,eth --http.corsdomain=* --http.addr=0.0.0.0 --http.vhosts=*
```

- You can see an example of `--chain.info-json` in the section above.

- Note that it is important that `/some/local/dir/arbitrum` already exists; otherwise, the directory might be created with `root` as owner, and the Docker container won't be able to write to it.

- Note that if you are running a node for the parent chain (e.g., Ethereum for Arbitrum One or Nova) on localhost, you may need to add `--network host` right after `docker run` to use Docker host-based networking

- When shutting down the Docker image, it is important to allow a graceful shutdown to save the current state to disk. Here is an example of how to do a graceful shutdown of all Docker images currently running





```shell
docker stop --time=1800 $(docker ps -aq)
```


### Important ports [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#important-ports "Direct link to Important ports")

| Protocol | Default port |
| --- | --- |
| `RPC`/`http` | `8547` |
| `RPC`/`websocket` | `8548` |
| `Sequencer Feed` | `9642` |

- Please note: the `RPC`/`websocket` protocol requires some ports to be enabled, you can use the following flags:
  - `--ws.port=8548`
  - `--ws.addr=0.0.0.0`
  - `--ws.origins=\*`

### Note on permissions [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#note-on-permissions "Direct link to Note on permissions")

- The Docker image is configured to run as non-root UID 1000. This configuration means if you are running in Linux or OSX and you are getting permission errors when trying to run the Docker image, run this command to allow all users to update the persistent folders:





```shell
mkdir /data/arbitrum
chmod -fR 777 /data/arbitrum
```


### Watchtower mode [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#watchtower-mode "Direct link to Watchtower mode")

- By default, the full node runs in Watchtower mode, meaning that it watches the onchain assertions and, if it disagrees with them, logs an error containing the string `found incorrect assertion in watchtower mode`. For a BoLD-enabled chain like Arbitrum One or Arbitrum Nova if you are running Nitro before v3.6.0, the `--node.bold.enable=true` flag should be set to ensure your node can monitor for onchain assertions properly.
- Setting this flag is not required as your node will continue to operate correctly, validate the Arbitrum One/Nova chain, and serve RPC requests as usual, regardless of this flag.
- Note that watchtower mode adds a small amount of execution and memory overhead. You can deactivate this mode using the parameter `--node.staker.enable=false`.

### Pruning [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#pruning "Direct link to Pruning")

- Pruning a full node refers to removing older, unnecessary data from the local copy of the blockchain that the node maintains, thereby saving disk space and slightly improving the node's efficiency. Pruning will remove all states from blocks older than the latest 128.
- If you are using the default setting `--execution.caching.state-scheme=hash` then you can activate pruning by using the parameter:
- `--init.prune <pruning mode>`
  - `minimal`: Only genesis + latest head state is left, takes the least amount of time (several hours for Arbitrum One-sized database)
  - `full` : Genesis + head state + state for latest confirmed block (will take a long time ~50 hours or more)
  - `validator`: All of above + state of latest validated block (will take a little less than twice what `full` will take)

note

This process occurs when the node starts and will not serve RPC requests during pruning.

### Transaction prechecker [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#transaction-prechecker "Direct link to Transaction prechecker")

- Enabling the transaction prechecker will add extra checks before your node forwards `eth_sendRawTransaction` to the Sequencer endpoint.
- Below, we list the flags to set up the prechecker:

| Flag | Description |
| --- | --- |
| `--execution.tx-pre-checker.strictness` | How strict to be when checking transactions before forwarding them. 0 = accept anything, 10 = should never reject anything that'd succeed, 20 = likely won't reject anything that'd succeed, 30 = full validation which may reject transactions that would succeed (default 20) |
| `--execution.tx-pre-checker.required-state-age` | How long ago should the storage conditions from `eth_SendRawTransactionConditional` be true, 0 = don't check old state (default 2) |
| `--execution.tx-pre-checker.required-state-max-blocks` | Maximum number of blocks to look back while looking for the `<required-state-age>` seconds old state, 0 = don't limit the search (default 4) |

### Optional parameters [​](https://docs.arbitrum.io/run-arbitrum-node/run-full-node\#optional-parameters "Direct link to Optional parameters")

Below, we listed the most commonly used parameters when running a node. You can also use the flag `--help` for a comprehensive list of the available parameters.

| Flag | Description |
| --- | --- |
| `--http.api` | Offers APIs over the HTTP-RPC interface. Default: `net,web3,eth,arb`. Add `debug` for tracing. |
| `--http.corsdomain` | Accepts cross-origin requests from these comma-separated domains (browser enforced). |
| `--http.vhosts` | Accepts requests from these comma-separated virtual hostnames (server enforced). Default: `localhost`. Accepts `*`. |
| `--http.addr` | Sets the address to bind RPC to. May require `0.0.0.0` for Docker networking. |
| `--execution.caching.archive` | Retains past block state. For archive nodes. |
| `--node.feed.input.url=<feed address>` | Sets the sequencer feed address to this URL. Default: `wss://<chainName>.arbitrum.io/feed`. ⚠️ One feed relay per datacenter is advised. See [feed relay guide](https://docs.arbitrum.io/run-arbitrum-node/run-feed-relay). |
| `--execution.forwarding-target=<RPC>` | Sets the sequencer endpoint to forward requests to. |
| `--execution.rpc.evm-timeout` | Default: `5s`. Timeout for `eth_call`. (0 == no timeout). |
| `--execution.rpc.gas-cap` | Default: `50000000`. Gas cap for `eth_call`/`estimateGas`. (0 = no cap). |
| `--execution.rpc.tx-fee-cap` | Default: `1`. Transaction fee cap (in ether) for RPC APIs. (0 = no cap). |
| `--execution.tx-lookup-limit` | Default: `126230400`, ~1 year worth of blocks at 250ms/block. Maximum number of blocks from head whose transaction indices are reserved (e.g., `eth_getTransactionReceipt` and `eth_getTransactionByHash` will only return results for indexed transactions). Set to 0 to index transactions for all blocks. Changing this parameter will reindex all missing transactions without the need of resyncing the chain. |
| `--execution.rpc.classic-redirect=<RPC>` | (Arbitrum One only) Redirects archive requests for pre-nitro blocks to this RPC of an Arbitrum Classic node with archive database. |
| `--ipc.path` | Filename for IPC socket/pipe within datadir. 🔉 Not supported on macOS. Note the path is within the Docker container. |
| `--init.prune` | Prunes the database before starting the node. Can be "full" or "validator". |
| `--init.url="<snapshot file>"` | (Required for Arbitrum One) URL to download the genesis database from. Only required for Arbitrum One nodes, when running them for the first time. See [this guide](https://docs.arbitrum.io/run-arbitrum-node/nitro/nitro-database-snapshots) for more information. |
| `--init.download-path="/path/to/dir"` | Temporarily saves the downloaded database snapshot. Defaults to `/tmp/`. Used with `--init.url`. |
| `--init.latest` | Searches for the latest snapshot of the given kind (accepted values: `archive`, `pruned`, `genesis`) |
| `--init.latest-base` | Base url used when searching for the latest snapshot. Default: " [https://snapshot.arbitrum.foundation/](https://snapshot.arbitrum.foundation/)". If you are running an Arbitrum chain, ask the chain owner for this URL. |
| `--init.then-quit` | Allows any `--init.*` parameters to complete, and then the node will automatically quit. It doesn't initiate pruning by itself but works in conjunction with other `--init.*` parameters, making it easier to script tasks like database backups after initialization processes finish. |

[Edit this page](https://github.com/OffchainLabs/arbitrum-docs/edit/master/docs/run-arbitrum-node/02-run-full-node.mdx)

Last updated on **Feb 18, 2026**

[Previous\\
\\
Overview](https://docs.arbitrum.io/run-arbitrum-node/overview) [Next\\
\\
Run a local full chain simulation](https://docs.arbitrum.io/run-arbitrum-node/run-local-full-chain-simulation)

- [Prerequisites](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#prerequisites)
  - [Minimum hardware configuration](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#minimum-hardware-configuration)
  - [Recommended Nitro version](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#recommended-nitro-version)
  - [Database snapshots](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#database-snapshots)
  - [Required parameters](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#required-parameters)
- [Putting it into practice: run a node](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#putting-it-into-practice-run-a-node)
  - [Important ports](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#important-ports)
  - [Note on permissions](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#note-on-permissions)
  - [Watchtower mode](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#watchtower-mode)
  - [Pruning](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#pruning)
  - [Transaction prechecker](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#transaction-prechecker)
  - [Optional parameters](https://docs.arbitrum.io/run-arbitrum-node/run-full-node#optional-parameters)

- [Arbitrum.io](https://arbitrum.io/)
- [Arbitrum Rollup](https://arbitrum.io/rollup)
- [Arbitrum AnyTrust](https://arbitrum.io/anytrust)
- [Arbitrum chains](https://arbitrum.io/launch-chain)
- [Arbitrum Stylus](https://arbitrum.io/stylus)
- [Arbitrum Foundation](https://arbitrum.foundation/)
- [Arbitrum whitepaper](https://docs.arbitrum.io/nitro-whitepaper.pdf)

- [Network status](https://status.arbitrum.io/)
- [Portal](https://portal.arbitrum.io/)
- [Bridge](https://bridge.arbitrum.io/)
- [Governance docs](https://docs.arbitrum.foundation/)
- [Careers](https://offchainlabs.com/careers/)
- [Support](https://support.arbitrum.io/)
- [Bug Bounties](https://immunefi.com/bounty/arbitrum/)

- [Discord](https://discord.gg/ZpZuw7p)
- [Twitter](https://twitter.com/OffchainLabs)
- [Youtube](https://www.youtube.com/@Arbitrum)
- [Medium Blog](https://medium.com/offchainlabs)
- [Research forum](https://research.arbitrum.io/)
- [Privacy Policy](https://arbitrum.io/privacy)
- [Terms of Service](https://arbitrum.io/tos)

© 2026 Offchain Labs

Ask AI
![Chat avatar](https://docs.arbitrum.io/img/logo.svg)