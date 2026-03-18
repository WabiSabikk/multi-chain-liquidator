[Skip to main content](https://docs.arbitrum.io/run-arbitrum-node/overview#__docusaurus_skipToContent_fallback)

Reactivate your Stylus contracts to ensure they remain callable - [here’s how to do it.](https://docs.arbitrum.io/stylus/gentle-introduction#activation)

[![Arbitrum Logo](https://docs.arbitrum.io/img/logo.svg)\\
**Arbitrum Docs**](https://docs.arbitrum.io/get-started/overview)

[Get started](https://docs.arbitrum.io/get-started/overview)

[Build apps](https://docs.arbitrum.io/run-arbitrum-node/overview#)

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
  - [ArbOS software releases](https://docs.arbitrum.io/run-arbitrum-node/overview#)

  - [More node types](https://docs.arbitrum.io/run-arbitrum-node/overview#)

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

[✏️Request an update](https://github.com/OffchainLabs/arbitrum-docs/issues/new?title=Docs%20update%20request:%20/run-arbitrum-node/overview&body=Source:%20https://docs.arbitrum.io/run-arbitrum-node/overview%0A%0ARequest:%20(how%20can%20we%20help?)%0A%0APsst,%20this%20issue%20will%20be%20closed%20with%20a%20templated%20response%20if%20it%20isn%27t%20a%20documentation%20update%20request.)

# Arbitrum nodes: an overview

Note

There is no protocol-level incentive to run an Arbitum full node. If you’re interested in accessing an Arbitrum chain but don’t want to set up a node locally, see our [RPC endpoints and providers](https://docs.arbitrum.io/build-decentralized-apps/reference/node-providers) to get RPC access to fully managed nodes hosted by a third-party provider.

API security disclaimer

When exposing API endpoints to the Internet or any untrusted/hostile network, the following risks may arise:

- **Increased risk of crashes due to Out-of-Memory (OOM)**:
Exposing endpoints increases the risk of OOM crashes.
- **Increased risk of not keeping up with chain progression**:
Resource starvation (IO or CPU) may occur, leading to an inability to keep up with chain progression.

We strongly advise against exposing API endpoints publicly. Users considering such exposure should exercise caution and implement the right measures to enhance resilience.

To be able to _interact with_ or _build applications on_ any of the Arbitrum chains, you need access to the corresponding Arbitrum node. Options are:

- You can use [third party node providers](https://docs.arbitrum.io/build-decentralized-apps/reference/node-providers) to get RPC access to fully-managed nodes
- You can run your own Arbitrum node, especially if you want always to know the state of the Arbitrum chain

The rest of this series focuses on the second approach: running your own Arbitrum node.

:::

To be able to _interact with_ or _build applications on_ any of the Arbitrum chains, you need access to the corresponding Arbitrum node. Options are:

When interacting with the Arbitrum network, users have the option to run either a full node or an archive node. There are distinct advantages to running an Arbitrum full node. In this quick start, we will explore the reasons why a user may prefer to run a full node instead of an archive node. By understanding the benefits and trade-offs of each node type, users can make an informed decision based on their specific requirements and objectives.

## Considerations for running an Arbitrum full node [​](https://docs.arbitrum.io/run-arbitrum-node/overview\#considerations-for-running-an-arbitrum-full-node "Direct link to Considerations for running an Arbitrum full node")

- **Transaction validation and security**: Running a full node allows users to independently validate transactions and verify the state of the Arbitrum blockchain. Users can have complete confidence in the authenticity and integrity of the transactions they interact with.
- **Reduced trust requirements**: By running a full node, users can interact with the Arbitrum network without relying on third-party services or infrastructure. This independence reduces the need to trust external entities and mitigates the risk of potential centralized failures or vulnerabilities.
- **Lower resource requirements**: Compared to archive nodes, full nodes generally require fewer resources such as storage and computational power. These requirements make it more accessible to users with limited hardware capabilities or those operating in resource-constrained environments.

For detailed instructions, read [how to run an Arbitrum full node](https://docs.arbitrum.io/run-arbitrum-node/run-full-node).

## Considerations for running an Arbitrum archive node [​](https://docs.arbitrum.io/run-arbitrum-node/overview\#considerations-for-running-an-arbitrum-archive-node "Direct link to Considerations for running an Arbitrum archive node")

While full nodes offer numerous advantages, there are situations where running an archive node may be more appropriate. Archive nodes store the complete history of the Arbitrum network, making them suitable for users who require access to extensive historical data or advanced analytical purposes. However, it's important to note that archive nodes are more resource-intensive, requiring significant storage capacity and computational power.

For detailed instructions, read [how to run an Arbitrum archive node](https://docs.arbitrum.io/run-arbitrum-node/more-types/run-archive-node).

## Considerations for running an Arbitrum classic node [​](https://docs.arbitrum.io/run-arbitrum-node/overview\#considerations-for-running-an-arbitrum-classic-node "Direct link to Considerations for running an Arbitrum classic node")

The significance of running an Arbitrum classic node is mainly applicable to individuals with specific needs for an archive node and access to classic-related commands.

For detailed instructions, read [how to run an Arbitrum classic node](https://docs.arbitrum.io/run-arbitrum-node/more-types/run-classic-node).

## Considerations for running a feed relay [​](https://docs.arbitrum.io/run-arbitrum-node/overview\#considerations-for-running-a-feed-relay "Direct link to Considerations for running a feed relay")

If you are running a single node, there is no requirement to set up a feed relay. However, if you have multiple nodes, it is highly recommended to have a single feed relay per data center. This setup offers several advantages, including reducing ingress fees and enhancing network stability.

Soon, feed endpoints will mandate compression using a custom dictionary. Therefore, if you plan to connect to a feed using anything other than a standard node, it is strongly advised to run a local feed relay. This local feed relay will ensure that you have access to an uncompressed feed by default, maintaining optimal performance and compatibility.

For detailed instructions, read [how to run an Arbitrum feed relay](https://docs.arbitrum.io/run-arbitrum-node/run-feed-relay).

[Edit this page](https://github.com/OffchainLabs/arbitrum-docs/edit/master/docs/run-arbitrum-node/01-overview.mdx)

Last updated on **Feb 18, 2026**

[Next\\
\\
Overview](https://docs.arbitrum.io/run-arbitrum-node/overview)

- [Considerations for running an Arbitrum full node](https://docs.arbitrum.io/run-arbitrum-node/overview#considerations-for-running-an-arbitrum-full-node)
- [Considerations for running an Arbitrum archive node](https://docs.arbitrum.io/run-arbitrum-node/overview#considerations-for-running-an-arbitrum-archive-node)
- [Considerations for running an Arbitrum classic node](https://docs.arbitrum.io/run-arbitrum-node/overview#considerations-for-running-an-arbitrum-classic-node)
- [Considerations for running a feed relay](https://docs.arbitrum.io/run-arbitrum-node/overview#considerations-for-running-a-feed-relay)

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