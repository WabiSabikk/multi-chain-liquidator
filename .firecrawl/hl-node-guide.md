[Sitemap](https://medium.com/sitemap/sitemap.xml)

[Open in app](https://play.google.com/store/apps/details?id=com.medium.reader&referrer=utm_source%3DmobileNavBar&source=post_page---top_nav_layout_nav-----------------------------------------)

Sign up

[Sign in](https://medium.com/m/signin?operation=login&redirect=https%3A%2F%2Fmedium.com%2F%40manuelbagoole%2Fdemystifying-hyperliquid-how-to-run-a-node-without-losing-your-mind-06e1c12b216c&source=post_page---top_nav_layout_nav-----------------------global_nav------------------)

[Medium Logo](https://medium.com/?source=post_page---top_nav_layout_nav-----------------------------------------)

Get app

[Write](https://medium.com/m/signin?operation=register&redirect=https%3A%2F%2Fmedium.com%2Fnew-story&source=---top_nav_layout_nav-----------------------new_post_topnav------------------)

[Search](https://medium.com/search?source=post_page---top_nav_layout_nav-----------------------------------------)

Sign up

[Sign in](https://medium.com/m/signin?operation=login&redirect=https%3A%2F%2Fmedium.com%2F%40manuelbagoole%2Fdemystifying-hyperliquid-how-to-run-a-node-without-losing-your-mind-06e1c12b216c&source=post_page---top_nav_layout_nav-----------------------global_nav------------------)

![](https://miro.medium.com/v2/resize:fill:32:32/1*dmbNkD5D-u45r44go_cf0g.png)

# Demystifying Hyperliquid: How to Run a Node Without Losing Your Mind

## Hyperliquid is pushing the boundaries of decentralized finance with its high-performance DEX and HyperBFT consensus. But for many, the idea of running a node — validator or non-validator — feels like stepping into a maze of binaries, ports, and cryptic flags. Let’s break it down.

[![Manuel](https://miro.medium.com/v2/da:true/resize:fill:32:32/0*jSW1xizdBE87DcR9)](https://medium.com/@manuelbagoole?source=post_page---byline--06e1c12b216c---------------------------------------)

[Manuel](https://medium.com/@manuelbagoole?source=post_page---byline--06e1c12b216c---------------------------------------)

Follow

3 min read

·

Aug 25, 2025

1

[Listen](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2Fplans%3Fdimension%3Dpost_audio_button%26postId%3D06e1c12b216c&operation=register&redirect=https%3A%2F%2Fmedium.com%2F%40manuelbagoole%2Fdemystifying-hyperliquid-how-to-run-a-node-without-losing-your-mind-06e1c12b216c&source=---header_actions--06e1c12b216c---------------------post_audio_button------------------)

Share

Whether you’re a curious developer, a validator candidate, or just someone who wants to stream real-time trading data, this guide will walk you through the essentials of running a Hyperliquid node — without the guesswork.

### Node Roles & Hardware Requirements

Hyperliquid nodes come in two flavors

**Validator Node**

vCPUs — 32

RAM — 128 GB

Storage — 1 TB SSD

**Non-Validator**

vCPUs — 16

RAM — 64 GB

Storage — 500 GB SSD

OS: Ubuntu 24.04 only.

Latency Tip: Tokyo is the recommended region for lowest latency.

### **Network & Chain Configuration**

**Ports to open**

- Gossip: `4001`, `4002` (public)
- Validators: `4000–4010` (peer-to-peer)

**Chain setup**

```

echo ‘ {“chain”: “Testnet”} ‘ > ~/visor.json
# or
echo ‘ {“chain”: “Mainnet”} ‘ > ~/visor.json
```

**Download & verify visor binary**

```

curl https://binaries.hyperliquid.xyz/Mainnet/hl-visor > ~/hl-visor && chmod a+x ~/hl-visor
gpg — import pub_key.asc
gpg — verify hl-visor.asc hl-visor
```

The `hl-visor` binary manages the node lifecycle and ensures integrity by verifying `hl-node`.

## Running a Non-Validator Node

Start with

```
~/hl-visor run-non-validator
```

Data is streamed live and stored in `~/hl/data`. Expect ~100 GB/day—so plan for archiving.

🔍 Key Data Outputs

Transaction Blocks — `replica_cmds/{start_time}/{date}/{height}`

State Snapshots — `periodic_abci_states/{date}/{height}.rmp`

L4 Book Snapshots — Computed from state snapshots

**Convert snapshots to JSON**

```
hl-node — chain Mainnet translate-abci-state /tmp/out.json
```

**Optional Flags for Custom Streaming**

## Get Manuel’s stories in your inbox

Join Medium for free to get updates from this writer.

Subscribe

Subscribe

Want granular control? Use flags like:

```
~/hl-visor run-non-validator \ — write-trades \ — write-order-statuses \ — serve-eth-rpc
```

Other useful flags:

- `--write-fills`, `--write-raw-book-diffs`, `--write-misc-events`
- `--serve-info`: Enables local HTTP server at `http://localhost:3001/info`
- `--disable-output-file-buffering`: Reduces latency, increases disk I/O

## **Running a Validator Node**

First, run a non-validator node. Then

### **Wallet Setup**

- **Validator Wallet**: Holds funds, receives rewards.
- **Signer Wallet**: Signs consensus messages.

**Create config**

```
echo ‘ {“key”: “”} ‘ > ~/hl/hyperliquid_data/node_config.json
```

**Register & Delegate**

```
hl-node — chain Mainnet — key send-signed-action ‘{ “type”: “CValidatorAction”, “register”: { “profile”: { “node_ip”: {“Ip”: “1.2.3.4”}, “signer”: “”, “name”: “ValidatorName”, “description”: “My Hyperliquid Node” }, “initial_wei”: 1000000000000 } }’
```

**Jailing & Unjailing**

Validators are jailed on registration or IP change. To unjail:

```
hl-node — chain Mainnet — key send-signed-action ‘{“type”: “CSignerAction”, “unjailSelf”: null}’
```



Jailing ensures performance — validators must maintain

**_Sentry Nodes & Alerts_**

- Run up to 2 sentry nodes for redundancy.
- Add IPs to `node_config.json` under `"sentry_ips": [...]`
- Set up Slack alerts via `~/hl/api_secrets.json` to monitor uptime.

**_Delegation: Let Others Stake With You_**

- Deposit tokens:
- Delegate:
- Withdraw (5-min unbonding):

```
hl-node — chain Mainnet — key staking-withdrawal
```

**_Troubleshooting_**

- **Crash Logs**: `visor_child_stderr/{date}/{index}`
- **Consensus Logs**: `node_logs/consensus` (search for “suspect”)
- **Status Logs**: `node_logs/status` (latency, connectivity)

**_Seed Peers for Mainnet_**

Use community-run root peers (Japan, Korea, France, Germany, UK, Singapore) by adding them to:

```
~/override_gossip_config.json
```

**Final Thoughts**

Running a Hyperliquid node isn’t just for protocol insiders — it’s for builders, educators, and anyone who wants to contribute to a high-performance, decentralized future. With the right setup and a bit of curiosity, you can stream real-time data, validate blocks, and even earn rewards through delegation.

Want help setting up your node or optimizing your validator performance? Drop a comment or reach out — I’m always down to troubleshoot and share reproducible workflows.

— Manuel Prhyme

Blockchain Developer \| Developer Advocate \| Security and Node Running Aficionado

[Node Running](https://medium.com/tag/node-running?source=post_page-----06e1c12b216c---------------------------------------)

[Hyperliquid](https://medium.com/tag/hyperliquid?source=post_page-----06e1c12b216c---------------------------------------)

[Blockchain](https://medium.com/tag/blockchain?source=post_page-----06e1c12b216c---------------------------------------)

[Validator](https://medium.com/tag/validator?source=post_page-----06e1c12b216c---------------------------------------)

1

1

[![Manuel](https://miro.medium.com/v2/resize:fill:48:48/0*jSW1xizdBE87DcR9)](https://medium.com/@manuelbagoole?source=post_page---post_author_info--06e1c12b216c---------------------------------------)

[![Manuel](https://miro.medium.com/v2/resize:fill:64:64/0*jSW1xizdBE87DcR9)](https://medium.com/@manuelbagoole?source=post_page---post_author_info--06e1c12b216c---------------------------------------)

Follow

[**Written by Manuel**](https://medium.com/@manuelbagoole?source=post_page---post_author_info--06e1c12b216c---------------------------------------)

[5 followers](https://medium.com/@manuelbagoole/followers?source=post_page---post_author_info--06e1c12b216c---------------------------------------)

· [62 following](https://medium.com/@manuelbagoole/following?source=post_page---post_author_info--06e1c12b216c---------------------------------------)

Coding Therapist

Follow

## No responses yet

![](https://miro.medium.com/v2/resize:fill:32:32/1*dmbNkD5D-u45r44go_cf0g.png)

Write a response

[What are your thoughts?](https://medium.com/m/signin?operation=register&redirect=https%3A%2F%2Fmedium.com%2F%40manuelbagoole%2Fdemystifying-hyperliquid-how-to-run-a-node-without-losing-your-mind-06e1c12b216c&source=---post_responses--06e1c12b216c---------------------respond_sidebar------------------)

Cancel

Respond

## More from Manuel

![So I have an occasional nostalgia for “Not so old movies” and recently I decided to rewatch…](https://miro.medium.com/v2/resize:fit:679/format:webp/56439e068a17200d733d148d8933d55501ac9dcc329a6f86272aeffc8ceb50a8)

[![Manuel](https://miro.medium.com/v2/resize:fill:20:20/0*jSW1xizdBE87DcR9)](https://medium.com/@manuelbagoole?source=post_page---author_recirc--06e1c12b216c----0---------------------fa18e718_cb05_48cf_9b2e_d5ba44b4b236--------------)

[Manuel](https://medium.com/@manuelbagoole?source=post_page---author_recirc--06e1c12b216c----0---------------------fa18e718_cb05_48cf_9b2e_d5ba44b4b236--------------)

[**So I have an occasional nostalgia for “Not so old movies” and recently I decided to rewatch…**\\
\\
**Right now it seems totally impossible to get to a level of technology that lofty and sophisticated. When machine learning advances and…**](https://medium.com/@manuelbagoole/so-i-have-an-occasional-nostalgia-for-not-so-old-movies-and-recently-i-decided-to-rewatch-ff3007178ba9?source=post_page---author_recirc--06e1c12b216c----0---------------------fa18e718_cb05_48cf_9b2e_d5ba44b4b236--------------)

Oct 5, 2020

[A clap icon6](https://medium.com/@manuelbagoole/so-i-have-an-occasional-nostalgia-for-not-so-old-movies-and-recently-i-decided-to-rewatch-ff3007178ba9?source=post_page---author_recirc--06e1c12b216c----0---------------------fa18e718_cb05_48cf_9b2e_d5ba44b4b236--------------)

[![Manuel](https://miro.medium.com/v2/resize:fill:20:20/0*jSW1xizdBE87DcR9)](https://medium.com/@manuelbagoole?source=post_page---author_recirc--06e1c12b216c----0-----------------------------------)

[Manuel](https://medium.com/@manuelbagoole?source=post_page---author_recirc--06e1c12b216c----0-----------------------------------)

[**So I have an occasional nostalgia for “Not so old movies” and recently I decided to rewatch…** \\
**Right now it seems totally impossible to get to a level of technology that lofty and sophisticated. When machine learning advances and…**](https://medium.com/@manuelbagoole/so-i-have-an-occasional-nostalgia-for-not-so-old-movies-and-recently-i-decided-to-rewatch-ff3007178ba9?source=post_page---author_recirc--06e1c12b216c----0-----------------------------------)

Oct 5, 2020

[A clap icon6](https://medium.com/@manuelbagoole/so-i-have-an-occasional-nostalgia-for-not-so-old-movies-and-recently-i-decided-to-rewatch-ff3007178ba9?source=post_page---author_recirc--06e1c12b216c----0-----------------------------------)

Oct 5, 2020

[A clap icon6](https://medium.com/@manuelbagoole/so-i-have-an-occasional-nostalgia-for-not-so-old-movies-and-recently-i-decided-to-rewatch-ff3007178ba9?source=post_page---author_recirc--06e1c12b216c----0-----------------------------------)

[See all from Manuel](https://medium.com/@manuelbagoole?source=post_page---author_recirc--06e1c12b216c---------------------------------------)

## Recommended from Medium

![Why Thousands Are Buying Mac Minis to Escape Issues with Big Tech AI Subscriptions Forever |…](https://miro.medium.com/v2/resize:fit:679/format:webp/1*YZcveDctIOQ2Zsf2z2_Ztg.png)

[![CodeX](https://miro.medium.com/v2/resize:fill:20:20/1*VqH0bOrfjeUkznphIC7KBg.png)](https://medium.com/codex?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

In

[CodeX](https://medium.com/codex?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

by

[MayhemCode](https://medium.com/@mayhemcode?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

[**Why Thousands Are Buying Mac Minis to Escape Issues with Big Tech AI Subscriptions Forever \|…**\\
\\
**Something strange happened in early 2026. Apple stores started running low on Mac Minis. Tech forums exploded with setup guides. Developers…**](https://medium.com/codex/why-thousands-are-buying-mac-minis-to-escape-big-tech-ai-subscriptions-forever-clawdbot-10c970c72404?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

Feb 15

[A clap icon4.3K\\
\\
A response icon74](https://medium.com/codex/why-thousands-are-buying-mac-minis-to-escape-big-tech-ai-subscriptions-forever-clawdbot-10c970c72404?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

![Google Is Quietly Dismantling Everything OpenAI Built](https://miro.medium.com/v2/resize:fit:679/format:webp/1*S6MjGmQYT-jeK8W2RlkSJw.png)

[![Level Up Coding](https://miro.medium.com/v2/resize:fill:20:20/1*5D9oYBd58pyjMkV_5-zXXQ.jpeg)](https://medium.com/gitconnected?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

In

[Level Up Coding](https://medium.com/gitconnected?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

by

[Teja Kusireddy](https://medium.com/@teja.kusireddy23?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

[**Google Is Quietly Dismantling Everything OpenAI Built**\\
\\
**The most dangerous failure in Silicon Valley isn’t bankruptcy. It’s becoming the engine inside someone else’s car.**](https://medium.com/gitconnected/google-is-quietly-dismantling-everything-openai-built-4edc406f572d?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

Feb 17

[A clap icon4.8K\\
\\
A response icon144](https://medium.com/gitconnected/google-is-quietly-dismantling-everything-openai-built-4edc406f572d?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

![6 brain images](https://miro.medium.com/v2/resize:fit:679/format:webp/1*Q-mzQNzJSVYkVGgsmHVjfw.png)

[![Write A Catalyst](https://miro.medium.com/v2/resize:fill:20:20/1*KCHN5TM3Ga2PqZHA4hNbaw.png)](https://medium.com/write-a-catalyst?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

In

[Write A Catalyst](https://medium.com/write-a-catalyst?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

by

[Dr. Patricia Schmidt](https://medium.com/@creatorschmidt?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

[**As a Neuroscientist, I Quit These 5 Morning Habits That Destroy Your Brain**\\
\\
**Most people do \#1 within 10 minutes of waking (and it sabotages your entire day)**](https://medium.com/write-a-catalyst/as-a-neuroscientist-i-quit-these-5-morning-habits-that-destroy-your-brain-3efe1f410226?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

Jan 14

[A clap icon35K\\
\\
A response icon633](https://medium.com/write-a-catalyst/as-a-neuroscientist-i-quit-these-5-morning-habits-that-destroy-your-brain-3efe1f410226?source=post_page---read_next_recirc--06e1c12b216c----0---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

![Stanford Just Killed Prompt Engineering With 8 Words (And I Can’t Believe It Worked)](https://miro.medium.com/v2/resize:fit:679/format:webp/1*va3sFwIm26snbj5ly9ZsgA.jpeg)

[![Generative AI](https://miro.medium.com/v2/resize:fill:20:20/1*M4RBhIRaSSZB7lXfrGlatA.png)](https://medium.com/generative-ai?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

In

[Generative AI](https://medium.com/generative-ai?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

by

[Adham Khaled](https://medium.com/@adham__khaled__?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

[**Stanford Just Killed Prompt Engineering With 8 Words (And I Can’t Believe It Worked)**\\
\\
**ChatGPT keeps giving you the same boring response? This new technique unlocks 2× more creativity from ANY AI model — no training required…**](https://medium.com/generative-ai/stanford-just-killed-prompt-engineering-with-8-words-and-i-cant-believe-it-worked-8349d6524d2b?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

Oct 19, 2025

[A clap icon24K\\
\\
A response icon651](https://medium.com/generative-ai/stanford-just-killed-prompt-engineering-with-8-words-and-i-cant-believe-it-worked-8349d6524d2b?source=post_page---read_next_recirc--06e1c12b216c----1---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

![cover image](https://miro.medium.com/v2/resize:fit:679/format:webp/1*aJ__IAMofCHG1KbEBq4Tng.gif)

[![AI Advances](https://miro.medium.com/v2/resize:fill:20:20/1*R8zEd59FDf0l8Re94ImV0Q.png)](https://medium.com/ai-advances?source=post_page---read_next_recirc--06e1c12b216c----2---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

In

[AI Advances](https://medium.com/ai-advances?source=post_page---read_next_recirc--06e1c12b216c----2---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

by

[Jose Crespo, PhD](https://medium.com/@pepitoscrespo?source=post_page---read_next_recirc--06e1c12b216c----2---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

[**Anthropic is Killing Bitcoin**\\
\\
**The AI-native currency already exists — hiding in plain sight, outperforming crypto by six orders of magnitude.**](https://medium.com/ai-advances/anthropic-is-killing-bitcoin-088288759706?source=post_page---read_next_recirc--06e1c12b216c----2---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

Feb 17

[A clap icon3.2K\\
\\
A response icon137](https://medium.com/ai-advances/anthropic-is-killing-bitcoin-088288759706?source=post_page---read_next_recirc--06e1c12b216c----2---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

![Why the Smartest People in Tech Are Quietly Panicking Right Now](https://miro.medium.com/v2/resize:fit:679/format:webp/1*W96wtREHKtBU9qvqSJkovw.png)

[![Activated Thinker](https://miro.medium.com/v2/resize:fill:20:20/1*I0dmd2-TIrUdjo5eUTjtvw.png)](https://medium.com/activated-thinker?source=post_page---read_next_recirc--06e1c12b216c----3---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

In

[Activated Thinker](https://medium.com/activated-thinker?source=post_page---read_next_recirc--06e1c12b216c----3---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

by

[Shane Collins](https://medium.com/@intellizab?source=post_page---read_next_recirc--06e1c12b216c----3---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

[**Why the Smartest People in Tech Are Quietly Panicking Right Now**\\
\\
**The water is rising fast, and your free version of ChatGPT is hiding the terrifying, exhilarating truth**](https://medium.com/activated-thinker/why-the-smartest-people-in-tech-are-quietly-panicking-right-now-d2feb86e7e4b?source=post_page---read_next_recirc--06e1c12b216c----3---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

Feb 13

[A clap icon12K\\
\\
A response icon525](https://medium.com/activated-thinker/why-the-smartest-people-in-tech-are-quietly-panicking-right-now-d2feb86e7e4b?source=post_page---read_next_recirc--06e1c12b216c----3---------------------55e7f9ca_57e6_4e34_a199_af45801ca942--------------)

[See more recommendations](https://medium.com/?source=post_page---read_next_recirc--06e1c12b216c---------------------------------------)

[Help](https://help.medium.com/hc/en-us?source=post_page-----06e1c12b216c---------------------------------------)

[Status](https://status.medium.com/?source=post_page-----06e1c12b216c---------------------------------------)

[About](https://medium.com/about?autoplay=1&source=post_page-----06e1c12b216c---------------------------------------)

[Careers](https://medium.com/jobs-at-medium/work-at-medium-959d1a85284e?source=post_page-----06e1c12b216c---------------------------------------)

[Press](mailto:pressinquiries@medium.com)

[Blog](https://blog.medium.com/?source=post_page-----06e1c12b216c---------------------------------------)

[Privacy](https://policy.medium.com/medium-privacy-policy-f03bf92035c9?source=post_page-----06e1c12b216c---------------------------------------)

[Rules](https://policy.medium.com/medium-rules-30e5502c4eb4?source=post_page-----06e1c12b216c---------------------------------------)

[Terms](https://policy.medium.com/medium-terms-of-service-9db0094a1e0f?source=post_page-----06e1c12b216c---------------------------------------)

[Text to speech](https://speechify.com/medium?source=post_page-----06e1c12b216c---------------------------------------)

reCAPTCHA

Recaptcha requires verification.

[Privacy](https://www.google.com/intl/en/policies/privacy/) \- [Terms](https://www.google.com/intl/en/policies/terms/)

protected by **reCAPTCHA**

[Privacy](https://www.google.com/intl/en/policies/privacy/) \- [Terms](https://www.google.com/intl/en/policies/terms/)