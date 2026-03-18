[Skip to main content](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#__docusaurus_skipToContent_fallback)

Reactivate your Stylus contracts to ensure they remain callable - [here’s how to do it.](https://docs.arbitrum.io/stylus/gentle-introduction#activation)

[![Arbitrum Logo](https://docs.arbitrum.io/img/logo.svg)\\
**Arbitrum Docs**](https://docs.arbitrum.io/get-started/overview)

[Get started](https://docs.arbitrum.io/get-started/overview)

[Build apps](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#)

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
  - [ArbOS software releases](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#)

    - [Overview](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview)
    - [ArbOS 51 Dia](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos51)
    - [ArbOS 40 Callisto](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos40)
    - [ArbOS 32 Bianca](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos32)
    - [ArbOS 20 Atlas](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos20)
    - [ArbOS 11](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos11)
  - [More node types](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#)

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

[✏️Request an update](https://github.com/OffchainLabs/arbitrum-docs/issues/new?title=Docs%20update%20request:%20/run-arbitrum-node/arbos-releases/overview&body=Source:%20https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview%0A%0ARequest:%20(how%20can%20we%20help?)%0A%0APsst,%20this%20issue%20will%20be%20closed%20with%20a%20templated%20response%20if%20it%20isn%27t%20a%20documentation%20update%20request.)

# ArbOS software releases: Overview

info

This document provides an overview of Nitro node software releases that upgrade ArbOS. Visit the [Nitro Github repository](https://github.com/OffchainLabs/nitro/releases) for a detailed index of Nitro releases.

Arbitrum chains are powered by Arbitrum nodes running the Nitro software stack. The Nitro software stack includes [ArbOS](https://forum.arbitrum.foundation/t/arbitrum-arbos-upgrades/19695), the child chain EVM hypervisor that facilitates the execution environment of an Arbitrum chain.

Although new Nitro releases are shipped regularly, only a subset of Nitro releases carry ArbOS upgrades. These special Nitro releases are significant because ArbOS upgrades are Arbitrum's equivalent to a ["hard fork"](https://ethereum.org/en/history/)—an upgrade that alters a node's ability to produce valid Arbitrum blocks. This is why validator nodes supporting a public Arbitrum chain (One, Nova) **must update Nitro** whenever a new ArbOS version is released and voted for adoption by the ArbitrumDAO.

note

Every Nitro release is backwards compatible. In other words, the latest version of Nitro will support all previous ArbOS releases. This means that your validator's Nitro version must be greater than or equal to the version that includes the latest ArbOS upgrade.

How often should I be upgrading my ArbOS version?

It is strongly recommended to keep your Nitro's node software up-to-date as best you can to ensure you are benefting from the latest improvements to the Arbitrum technology stack. ArbOS version bumps are especially important because these upgrades change how Arbitrum nodes produce and validate assertions on a rollup's state.

ArbOS upgrades are carried out by the chain's owner; in the case of Arbitrum One and Nova, the owner is the Arbitrum DAO and so an upgrade will require a governance proposal and vote to pass to complete the upgrade. [This is an example of a Nitro release that contains an ArbOS version bump, specifically to ArbOS 11](https://github.com/OffchainLabs/nitro/releases/tag/v2.2.0).

Visit [How Arbitrum works](https://docs.arbitrum.io/how-arbitrum-works/inside-arbitrum-nitro) to learn more about Nitro's architecture; more information about ArbOS software releases is available on [the Arbitrum DAO forum](https://forum.arbitrum.foundation/t/arbitrum-arbos-upgrades/19695).

## List of available ArbOS releases [​](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview\#list-of-available-arbos-releases "Direct link to List of available ArbOS releases")

- [Dia (ArbOS 51)](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos51)
- [Callisto (ArbOS 40)](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos40)
- [Bianca (ArbOS 32)](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos32)
- [Atlas (ArbOS 20)](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos20)
- [ArbOS 11](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos11)

## Naming and numbering scheme [​](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview\#naming-and-numbering-scheme "Direct link to Naming and numbering scheme")

Beginning with ArbOS 20, ArbOS releases use the name of planetary moons in our solar system, ascending in alphabetical order (i.e., the next ArbOS upgrade after ArbOS 20 "Atlas" will be a planetary moon that begins with the letter "B").

The number used to denote each upgrade will increment by 10, starting from ArbOS 20 (i.e., the next ArbOS upgrade after ArbOS 20 will be ArbOS 31). This was done because there are teams who have customized their Arbitrum chain's [behavior](https://docs.arbitrum.io/launch-arbitrum-chain/customize-your-chain/customize-stf) or [precompiles](https://docs.arbitrum.io/launch-arbitrum-chain/customize-your-chain/customize-precompile) and who may wish to use ArbOS's naming schema between official ArbOS version bumps (e.g., ArbOS 12 could be the name of a customized version of ArbOS for a project's L3 Arbitrum chain).

Note that there may be cases where special optimizations or critical fixes are needed for a specific family of ArbOS releases that will diverge from the standard numbering scheme described above. For example, ArbOS 32 will be the canonical ArbOS version for the “Bianca” family of releases. Node operators and chain owners are expected to upgrade from ArbOS 20 directly to ArbOS 32 (instead of ArbOS 30 or ArbOS 31).

## Network status [​](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview\#network-status "Direct link to Network status")

To view the status and timeline of network upgrades on Arbitrum One and Nova, [please visit this page](https://docs.arbitrum.foundation/network-upgrades).

## Expectations for Arbitrum chain owners [​](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview\#expectations-for-arbitrum-chain-owners "Direct link to Expectations for Arbitrum chain owners")

For Arbitrum chain owners or maintainers: it is important to note that _before_ upgrading your Arbitrum chain(s) to the newest ArbOS release, we strongly encourage waiting at least four weeks after the new ArbOS release becomes active on Arbitrum One and Nova before attempting the upgrade yourself. The rationale behind this short time buffer is to allow the Offchain Labs team to address any upgrade issues or stability concerns that may arise with the initial rollout so that we can minimize the chances of your chain(s) hitting the same or similar issues and to maximize the likelihood of an eventual smooth, seamless upgrade. Arbitrum chains, as always, can pick up new features & enable new customizations as they see fit. However, this delay ensures a consistent user experience (UX) across all Arbitrum chain owners and managers for these critical upgrades.

Note that enabling an ArbOS upgrade is not as simple as bumping your chain’s Nitro node version. Instead, there are other steps required that are outlined in our docs on [How to upgrade ArbOS on your Arbitrum chain](https://docs.arbitrum.io/launch-arbitrum-chain/configure-your-chain/common/validation-and-security/arbos-upgrade). Please be sure to follow them and let us know if you encounter any issues.

## Stay up to date [​](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview\#stay-up-to-date "Direct link to Stay up to date")

To stay up to date with proposals, timelines, and statuses of network upgrades to Arbitrum One and Nova:

- Subscribe to the [Arbitrum Node Upgrade Announcement channel on Telegram](https://t.me/arbitrumnodeupgrade)
- Join both the `#dev-announcements` and `#node-runners` Discord channels in the [Arbitrum Discord server](https://discord.gg/arbitrum)
- Follow the official Arbitrum ( [`@Arbitrum`](https://twitter.com/arbitrum)) and Arbitrum Developers ( [`@ArbitrumDevs`](https://twitter.com/ArbitrumDevs)) X accounts, formerly Twitter.

[Edit this page](https://github.com/OffchainLabs/arbitrum-docs/edit/master/docs/run-arbitrum-node/arbos-releases/01-overview.mdx)

Last updated on **Feb 18, 2026**

[Previous\\
\\
Historical blobs](https://docs.arbitrum.io/run-arbitrum-node/beacon-nodes-historical-blobs) [Next\\
\\
ArbOS 51 Dia](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/arbos51)

- [List of available ArbOS releases](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#list-of-available-arbos-releases)
- [Naming and numbering scheme](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#naming-and-numbering-scheme)
- [Network status](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#network-status)
- [Expectations for Arbitrum chain owners](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#expectations-for-arbitrum-chain-owners)
- [Stay up to date](https://docs.arbitrum.io/run-arbitrum-node/arbos-releases/overview#stay-up-to-date)

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