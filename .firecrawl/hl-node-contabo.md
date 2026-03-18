[Sitemap](https://nodescience.medium.com/sitemap/sitemap.xml)

[Open in app](https://play.google.com/store/apps/details?id=com.medium.reader&referrer=utm_source%3DmobileNavBar&source=post_page---top_nav_layout_nav-----------------------------------------)

Sign up

[Sign in](https://medium.com/m/signin?operation=login&redirect=https%3A%2F%2Fnodescience.medium.com%2Fnode-run-your-hyperliquid-node-939a01d27604&source=post_page---top_nav_layout_nav-----------------------global_nav------------------)

[Medium Logo](https://medium.com/?source=post_page---top_nav_layout_nav-----------------------------------------)

Get app

[Write](https://medium.com/m/signin?operation=register&redirect=https%3A%2F%2Fmedium.com%2Fnew-story&source=---top_nav_layout_nav-----------------------new_post_topnav------------------)

[Search](https://medium.com/search?source=post_page---top_nav_layout_nav-----------------------------------------)

Sign up

[Sign in](https://medium.com/m/signin?operation=login&redirect=https%3A%2F%2Fnodescience.medium.com%2Fnode-run-your-hyperliquid-node-939a01d27604&source=post_page---top_nav_layout_nav-----------------------global_nav------------------)

![](https://miro.medium.com/v2/resize:fill:32:32/1*dmbNkD5D-u45r44go_cf0g.png)

# \[Node\] Run your HYPERLIQUID Node

[![Node Science](https://miro.medium.com/v2/resize:fill:32:32/1*CmnZzwlO_SYAXxkLfa1dsw.png)](https://nodescience.medium.com/?source=post_page---byline--939a01d27604---------------------------------------)

[Node Science](https://nodescience.medium.com/?source=post_page---byline--939a01d27604---------------------------------------)

Follow

7 min read

·

Sep 9, 2024

41

3

[Listen](https://medium.com/m/signin?actionUrl=https%3A%2F%2Fmedium.com%2Fplans%3Fdimension%3Dpost_audio_button%26postId%3D939a01d27604&operation=register&redirect=https%3A%2F%2Fnodescience.medium.com%2Fnode-run-your-hyperliquid-node-939a01d27604&source=---header_actions--939a01d27604---------------------post_audio_button------------------)

Share

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*-Xp9_C5En_Hbt6xse2OULQ.png)

[**Hyperliquid**](https://hyperliquid.xyz/) **is, without a doubt, one of my favorite DeFi projects.**

Like Bitcoin or Ethereum, it is a Layer-1 including apps such as a DEX with >100 perps (more information at the end of the guide). The team have announced further developments include spot tarding, permissionless liquidity and above all a **native token**.

Hyperliquid is currently in the _testnet_ phase, and many of us are eagerly awaiting the _mainnet_ launch _:_ I’m sure that running a node on it will be very profitable ! We can actually run node on _testnet_ and according to a message from **ben\_hl** of the Hyperliquid team, the developers are currently hard working on making them available on the _mainnet_.

**So, I decided to run my own node in the meantime and here’s how to do it!**

Welcome to Node Science ! Here, you will learn and understand how to set up your node easily and quickly, without any prior technical knowledge.

**Feel free to follow us on** [**Twitter**](https://x.com/nodescience) **to stay updated on everything related to nodes and join our** [**Discord**](https://discord.gg/XCWThSXzZp) **with more than 800 members for further discussion or any questions with our community !**

## VPS configuration

To run a node you’ll need a VPS (Virtual Private Server) and one of the most reliable and cheapest solutions is [Contabo](https://www.kqzyfj.com/click-101088958-12454592). It’s a cost-effective German VPS solution built in 2003, known for its robust performance and reliability, catering to a wide range of computing needs and budgets.

The following minimum hardware requirements are recommended for running a node, according to their github :

```
Memory: 16 GB RAM
CPU: 4 CPU Cores
Disk: 50 GB SSD Storage
```

You can choose a CLOUD VPS 2 for 3 monthes by clicking [here](https://www.kqzyfj.com/click-101088958-12454592). Note that you can opt for a more powerful server (VPS 3 or 4) to run multiple different nodes on it more economically.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*pW9W8xAr_Kk_l1LmtDgfhw.png)

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*7EOMlAeTZJmAEH7WOveenQ.png)

It is recommended to select **Ubuntu 22.04**(latest version available).

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*Eb6NHv2i64x4-LgOvZ4CcA.png)

Define a password.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*vijwDB_xr0GMfHHts7UN0g.png)

Once payment is complete, you’ll receive an e-mail with your IP address entitled _“Your login data!”_. To connect to your VPS and securely run your node, you must download and install the [Putty](https://www.chiark.greenend.org.uk/~sgtatham/putty/latest.html) software, which enables a secure connection.

![](https://miro.medium.com/v2/resize:fit:450/1*32wRaNFWPKCLT2OYzsARxg.png)

Type your IP address and Open.

Ensure you copy each command line using ( _Ctrl+C_), and then paste it into your terminal by _right-clicking_ your mouse. Execute them sequentially, pressing _Enter_ after each, if required.

### I) Install dependencies

Start by updating and upgrading your OS and confirm by typing “Y”.

```
sudo apt update
sudo apt list --upgradable | more
sudo apt upgrade
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*hmboTgzmAhFQqUEfDug0IA.png)

Then restart your VPS and wait a few seconds.

```
sudo reboot
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*f7XpZQh0qrAEs27fdj0-YA.png)

As you can see in the screenshot above, your VPS is currently running _Ubuntu version 22.04.4_. However, the team announced on their [GitHub](https://github.com/hyperliquid-dex/node) that only Ubuntu version 24.04 is supported. You will need to manually upgrade your version, and it’s very simple !

## Get Node Science’s stories in your inbox

Join Medium for free to get updates from this writer.

Subscribe

Subscribe

You can check your Ubuntu version by typing :

```
lsb_release -a
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*0Uum7x5lIExFhLgRhTNrxw.png)

First, we will start by installing the _Ubuntu Release Upgrader Core_ package. It contains the core functionality needed to upgrade your Ubuntu operating system to a newer release.

```
sudo apt install ubuntu-release-upgrader-core
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*B4YNjcm64dX9Pcfht0a3fA.png)

You can now verify if a new LTS _(Long Term Support)_ release is available.

```
grep 'lts' /etc/update-manager/release-upgrades
cat /etc/update-manager/release-upgrades
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*8XkrJe32GsBn7H_o95qmeg.png)

It’s now time to upgrade Ubuntu to the latest version. This process takes a few minutes.

```
sudo do-release-upgrade -d
```

You will be asked several times to press “y” and then Enter.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*eI5BSFudTpTYUJuMAAajBA.png)

At some point, a window will open. Select the first choice, then press enter.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*W6YC7ADOn7NSm0ut1f0RNg.png)

Same here.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*VNrQ2M68mDhZ9o_aRcbPzw.png)

You will finally need to restart your server by typing “y” and Enter.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*WezKMcJ1R21qAokEmV6ZfA.png)

Your session will be restarted. Wait a little bit, and you will be back on your session again. **Congratulations, you have just upgraded Ubuntu to version 24.04 !**

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*_FgnOYXTFguHVR3Za27x3Q.png)

You can confirm it by typing :

```
lsb_release -a
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*CS_oqBuX3nUwh8VroR3WxQ.png)

Now, you need to open a configuration file using the `nano` text editor and change the prompt to “normal” like on the following screen. Save with CTRL+X, then Y and finally Enter.

```
sudo nano /etc/update-manager/release-upgrades
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*ZTsQP28xNRB6oythjJboDA.png)

Make sure there are no other upgrades to perform and that you have the latest version of Ubuntu.

```
sudo do-release-upgrade
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*npQP130wDTQECHU5FCbd8Q.png)

Install screen and press Y. Careful, when installing screen, I encountered an error message and it didn’t work. As suggested in the message, I resolved it by first typing :

```
apt --fix-broken install
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*edHPsvPLs5CIiCXOuEEBvQ.png)

Then install screen.

```
sudo apt install screen
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*meBMmPCneIGb4xLKchRPEQ.png)

You now need to create a user for your node. Let’s name it `hyperliquid_user`. You will be asked to enter a password. For the rest of the information (full name, room number, etc.), press “ _Enter_” to use the default value.

```
sudo adduser hyperliquid_user
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*WmnUEdkQb1gDFcV88qpK3A.png)

Now you can add the user `hyperliquid_user` to the `sudo` group, which grants them administrative privileges on your Linux system.

```
sudo usermod -aG sudo hyperliquid_user
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*KEtRyl4Y7PP2ARQ_JFCQ1A.png)

Let’s switch to the user account `hyperliquid_user`. _(You can close the_`hyperliquid_user` _session and return to your original user account, by typing_`exit` _or press_`Ctrl + D` _)._

```
su - hyperliquid_user
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*7BJStPRG72psEvdOI2Ar4Q.png)

Now you can download the `initial_peers.json` file which contains a list of peer nodes that the client should initially connect to for syncing with the Hyperliquid blockchain.

```
curl https://binaries.hyperliquid.xyz/Testnet/initial_peers.json > ~/initial_peers.json
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*xkwjH2JidSkkoGoEnyc2jw.png)

We now need to configure the chain to “Testnet”. Mainnet will be available once testing is complete on testnet.

```
echo '{"chain": "Testnet"}' > ~/visor.json
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*SmN94pw3CR9epPQRLMPuDA.png)

Now, download the `non_validator_config.json` file.

```
curl https://binaries.hyperliquid.xyz/Testnet/non_validator_config.json > ~/non_validator_config.json
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*PpXPNi0C3OmWBOpjRYkfBg.png)

Download the visor binary, which will spawn and manage the child node process.

```
curl https://binaries.hyperliquid.xyz/Testnet/hl-visor > ~/hl-visor && chmod a+x ~/hl-visor
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*YMd2zh0GOm6lJUcvIN9cog.png)

Finally, you can open a screen session to keep your node running 24/7, even when you disconnect.

```
screen -S hyperliquid
```

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*iJ1-u41fGvOPzCVxWp_o9A.png)

Start running your node with :

```
~/hl-visor run-non-validator
```

It may take a while as the node navigates the network to find an appropriate peer to stream from ! Make sure your logs display `applied block X`. It means your node should be streaming live data.

Press enter or click to view image in full size

![](https://miro.medium.com/v2/resize:fit:700/1*miROtQNBg6oe2VhIZf0K5A.png)

**Congratulations, you’ve successfully run your Hyperliquid node !** You can now exit the screen session by typing **CTRL+A+D.**

**Feel free to follow us on** [**Twitter**](https://x.com/nodescience) **to stay updated on everything related to nodes and join our** [**Discord**](https://discord.gg/XCWThSXzZp) **for further discussion or any questions with our community!**

Hyperliquid operates an entire ecosystem of permissionless financial applications : every order, cancel, trade, and liquidation happens transparently on-chain with block latency <1 second. The chain currently supports 100,000 orders/second thanks to an optimized consensus algorithm called _HyperBFT_, which is optimized for end-to-end latency (duration between sending request to receiving committed response).

## Official links of Hyperliquid

- [Website](https://hyperliquid.xyz/)
- [DEX](https://app.hyperliquid.xyz/trade)
- [Twitter](https://x.com/HyperliquidX)
- [Discord](https://discord.com/invite/hyperliquid)
- [Telegram](https://t.me/hyperliquid_announcements)
- [Medium](https://medium.com/@hyperliquid)
- [Github](https://github.com/hyperliquid-dex)

[Hyperliquid](https://medium.com/tag/hyperliquid?source=post_page-----939a01d27604---------------------------------------)

[Node](https://medium.com/tag/node?source=post_page-----939a01d27604---------------------------------------)

[Masternodes](https://medium.com/tag/masternodes?source=post_page-----939a01d27604---------------------------------------)

[Blockchain](https://medium.com/tag/blockchain?source=post_page-----939a01d27604---------------------------------------)

[Crypto](https://medium.com/tag/crypto?source=post_page-----939a01d27604---------------------------------------)

41

41

3

[![Node Science](https://miro.medium.com/v2/resize:fill:48:48/1*CmnZzwlO_SYAXxkLfa1dsw.png)](https://nodescience.medium.com/?source=post_page---post_author_info--939a01d27604---------------------------------------)

[![Node Science](https://miro.medium.com/v2/resize:fill:64:64/1*CmnZzwlO_SYAXxkLfa1dsw.png)](https://nodescience.medium.com/?source=post_page---post_author_info--939a01d27604---------------------------------------)

Follow

[**Written by Node Science**](https://nodescience.medium.com/?source=post_page---post_author_info--939a01d27604---------------------------------------)

[2K followers](https://nodescience.medium.com/followers?source=post_page---post_author_info--939a01d27604---------------------------------------)

· [3 following](https://nodescience.medium.com/following?source=post_page---post_author_info--939a01d27604---------------------------------------)

Learn how to easily set up your node with Node Science ! [https://discord.gg/XCWThSXzZp](https://discord.gg/XCWThSXzZp)

Follow

## Responses (3)

![](https://miro.medium.com/v2/resize:fill:32:32/1*dmbNkD5D-u45r44go_cf0g.png)

Write a response

[What are your thoughts?](https://medium.com/m/signin?operation=register&redirect=https%3A%2F%2Fnodescience.medium.com%2Fnode-run-your-hyperliquid-node-939a01d27604&source=---post_responses--939a01d27604---------------------respond_sidebar------------------)

Cancel

Respond

[![Benjamin Aurelius](https://miro.medium.com/v2/resize:fill:32:32/0*_ARyCC1X0v0gHa1k)](https://medium.com/@benjamin.aurelius?source=post_page---post_responses--939a01d27604----0-----------------------------------)

[Benjamin Aurelius](https://medium.com/@benjamin.aurelius?source=post_page---post_responses--939a01d27604----0-----------------------------------)

[Nov 8, 2024](https://medium.com/@benjamin.aurelius/could-i-run-a-node-on-a-thin-client-with-a-ssd-and-similar-specs-as-the-contabo-vps-94f83525444d?source=post_page---post_responses--939a01d27604----0-----------------------------------)

```
Could I run a node on a thin client with a ssd and similar specs as the Contabo vps? Costs around 20€ on eBay instead of having to pay a similar amount every month?
```

--

Reply

[![Jol Pendres](https://miro.medium.com/v2/resize:fill:32:32/1*kF9ye3fDs1VxkDDVy2GQMg.png)](https://medium.com/@jjsnews?source=post_page---post_responses--939a01d27604----1-----------------------------------)

[Jol Pendres](https://medium.com/@jjsnews?source=post_page---post_responses--939a01d27604----1-----------------------------------)

[Oct 11, 2024](https://medium.com/@jjsnews/any-reward-for-testnet-8db8d1bc93e3?source=post_page---post_responses--939a01d27604----1-----------------------------------)

```
Any reward for testnet ?
```

--

Reply

[![Ajit Singh Duhoon](https://miro.medium.com/v2/resize:fill:32:32/0*-8qY38bsE1fEXBWc)](https://mavericksdilettante.medium.com/?source=post_page---post_responses--939a01d27604----2-----------------------------------)

[Ajit Singh Duhoon](https://mavericksdilettante.medium.com/?source=post_page---post_responses--939a01d27604----2-----------------------------------)

[Sep 11, 2024](https://mavericksdilettante.medium.com/hi-1a41f3492e9d?source=post_page---post_responses--939a01d27604----2-----------------------------------)

```
hi

Thanks a lot for such a detailed guide . if putty closes for some reason, how we can reconnect to node to check the running status
```

--

Reply

## More from Node Science

![[Farming Strategy] How to farm grass 24/7 with a VPS](https://miro.medium.com/v2/resize:fit:679/format:webp/0*bXJiVbKRVrF1Ha9A.png)

[![Node Science](https://miro.medium.com/v2/resize:fill:20:20/1*CmnZzwlO_SYAXxkLfa1dsw.png)](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604----0---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[Node Science](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604----0---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[**\[Farming Strategy\] How to farm grass 24/7 with a VPS**\\
\\
**In a guide I posted previously, I explained how to farm Grass with multiple accounts using proxies and anti-detect browser. It seems that…**](https://nodescience.medium.com/farming-strategy-how-to-farm-grass-24-7-with-a-vps-345372d39b6b?source=post_page---author_recirc--939a01d27604----0---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

Mar 30, 2024

[A response icon10](https://nodescience.medium.com/farming-strategy-how-to-farm-grass-24-7-with-a-vps-345372d39b6b?source=post_page---author_recirc--939a01d27604----0---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

![[Node] Run your CELESTIA Light Node](https://miro.medium.com/v2/resize:fit:679/format:webp/1*HE-hfUN5V-FKTZSP-ZaJBw.png)

[![Node Science](https://miro.medium.com/v2/resize:fill:20:20/1*CmnZzwlO_SYAXxkLfa1dsw.png)](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604----1---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[Node Science](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604----1---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[**\[Node\] Run your CELESTIA Light Node**\\
\\
**Celestia is the first modular blockchain network designed to address the scalability, security, and decentralization challenges of…**](https://nodescience.medium.com/node-run-your-celestia-light-node-part-1-451f43cda382?source=post_page---author_recirc--939a01d27604----1---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

Aug 25, 2024

[A response icon2](https://nodescience.medium.com/node-run-your-celestia-light-node-part-1-451f43cda382?source=post_page---author_recirc--939a01d27604----1---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

![[Farming Strategy] How to earn several hundred dollars a day with Grass ?](https://miro.medium.com/v2/resize:fit:679/format:webp/0*pa3qkmPvH7qY0y9u.png)

[![Node Science](https://miro.medium.com/v2/resize:fill:20:20/1*CmnZzwlO_SYAXxkLfa1dsw.png)](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604----2---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[Node Science](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604----2---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[**\[Farming Strategy\] How to earn several hundred dollars a day with Grass ?**\\
\\
**Grass.io positions itself as an innovative solution for earning passive income by selling unused internet bandwidth to companies and AI…**](https://nodescience.medium.com/farming-strategy-how-to-earn-several-hundred-dollars-a-day-with-grass-fca267b19313?source=post_page---author_recirc--939a01d27604----2---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

Mar 21, 2024

[A response icon4](https://nodescience.medium.com/farming-strategy-how-to-earn-several-hundred-dollars-a-day-with-grass-fca267b19313?source=post_page---author_recirc--939a01d27604----2---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

![[Node] Run your DUSK node](https://miro.medium.com/v2/resize:fit:679/format:webp/1*QvqTfn5MJgaxiLhYb8CEAQ.png)

[![Node Science](https://miro.medium.com/v2/resize:fill:20:20/1*CmnZzwlO_SYAXxkLfa1dsw.png)](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604----3---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[Node Science](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604----3---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[**\[Node\] Run your DUSK node**\\
\\
**09/03 UPDATE : Dusk has definitely closed its faucet. It continues to process previous requests, but no longer accepts new ones. The…**](https://nodescience.medium.com/node-run-your-dusk-node-2cbb3dbb5a82?source=post_page---author_recirc--939a01d27604----3---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

Mar 6, 2024

[A response icon2](https://nodescience.medium.com/node-run-your-dusk-node-2cbb3dbb5a82?source=post_page---author_recirc--939a01d27604----3---------------------b07bb6e3_9edc_4a5a_bd4f_a009dd150343--------------)

[See all from Node Science](https://nodescience.medium.com/?source=post_page---author_recirc--939a01d27604---------------------------------------)

## Recommended from Medium

![Local LLMs That Can Replace Claude Code](https://miro.medium.com/v2/resize:fit:679/format:webp/0*JpX_vOrpLzFhJfJM.png)

[![Agent Native](https://miro.medium.com/v2/resize:fill:20:20/1*dt5tcaKMBhB6JboQ9lIEAA.jpeg)](https://agentnativedev.medium.com/?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[Agent Native](https://agentnativedev.medium.com/?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[**Local LLMs That Can Replace Claude Code**\\
\\
**Small team of engineers can easily burn >$2K/mo on Anthropic’s Claude Code (Sonnet/Opus 4.5). As budgets are tight, you might be wondering…**](https://agentnativedev.medium.com/local-llms-that-can-replace-claude-code-6f5b6cac93bf?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

Jan 20

[A response icon44](https://agentnativedev.medium.com/local-llms-that-can-replace-claude-code-6f5b6cac93bf?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

![6 brain images](https://miro.medium.com/v2/resize:fit:679/format:webp/1*Q-mzQNzJSVYkVGgsmHVjfw.png)

[![Write A Catalyst](https://miro.medium.com/v2/resize:fill:20:20/1*KCHN5TM3Ga2PqZHA4hNbaw.png)](https://medium.com/write-a-catalyst?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

In

[Write A Catalyst](https://medium.com/write-a-catalyst?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

by

[Dr. Patricia Schmidt](https://medium.com/@creatorschmidt?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[**As a Neuroscientist, I Quit These 5 Morning Habits That Destroy Your Brain**\\
\\
**Most people do \#1 within 10 minutes of waking (and it sabotages your entire day)**](https://medium.com/@creatorschmidt/as-a-neuroscientist-i-quit-these-5-morning-habits-that-destroy-your-brain-3efe1f410226?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

Jan 14

[A response icon633](https://medium.com/@creatorschmidt/as-a-neuroscientist-i-quit-these-5-morning-habits-that-destroy-your-brain-3efe1f410226?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

![Anthropic Just Released Claude Code Course (And I Earned My Certificate)](https://miro.medium.com/v2/resize:fit:679/format:webp/1*03JPjS5mc0CIGl80kS2nUQ.png)

[![AI Software Engineer](https://miro.medium.com/v2/resize:fill:20:20/1*RZVWENvZRwVijHDlg5hw7w.png)](https://medium.com/ai-software-engineer?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

In

[AI Software Engineer](https://medium.com/ai-software-engineer?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

by

[Joe Njenga](https://medium.com/@joe.njenga?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[**Anthropic Just Released Claude Code Course (And I Earned My Certificate)**\\
\\
**Anthropic just launched their Claude Code in Action course, and I’ve just passed — how about you?**](https://medium.com/@joe.njenga/anthropic-just-released-claude-code-course-and-i-earned-my-certificate-ad68745d46de?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

Jan 21

[A response icon47](https://medium.com/@joe.njenga/anthropic-just-released-claude-code-course-and-i-earned-my-certificate-ad68745d46de?source=post_page---read_next_recirc--939a01d27604----0---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

![Why Thousands Are Buying Mac Minis to Escape Issues with Big Tech AI Subscriptions Forever |…](https://miro.medium.com/v2/resize:fit:679/format:webp/1*YZcveDctIOQ2Zsf2z2_Ztg.png)

[![CodeX](https://miro.medium.com/v2/resize:fill:20:20/1*VqH0bOrfjeUkznphIC7KBg.png)](https://medium.com/codex?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

In

[CodeX](https://medium.com/codex?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

by

[MayhemCode](https://medium.com/@mayhemcode?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[**Why Thousands Are Buying Mac Minis to Escape Issues with Big Tech AI Subscriptions Forever \|…**\\
\\
**Something strange happened in early 2026. Apple stores started running low on Mac Minis. Tech forums exploded with setup guides. Developers…**](https://medium.com/@mayhemcode/why-thousands-are-buying-mac-minis-to-escape-big-tech-ai-subscriptions-forever-clawdbot-10c970c72404?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

Feb 15

[A response icon74](https://medium.com/@mayhemcode/why-thousands-are-buying-mac-minis-to-escape-big-tech-ai-subscriptions-forever-clawdbot-10c970c72404?source=post_page---read_next_recirc--939a01d27604----1---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

![Why the Smartest People in Tech Are Quietly Panicking Right Now](https://miro.medium.com/v2/resize:fit:679/format:webp/1*W96wtREHKtBU9qvqSJkovw.png)

[![Activated Thinker](https://miro.medium.com/v2/resize:fill:20:20/1*I0dmd2-TIrUdjo5eUTjtvw.png)](https://medium.com/activated-thinker?source=post_page---read_next_recirc--939a01d27604----2---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

In

[Activated Thinker](https://medium.com/activated-thinker?source=post_page---read_next_recirc--939a01d27604----2---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

by

[Shane Collins](https://medium.com/@intellizab?source=post_page---read_next_recirc--939a01d27604----2---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[**Why the Smartest People in Tech Are Quietly Panicking Right Now**\\
\\
**The water is rising fast, and your free version of ChatGPT is hiding the terrifying, exhilarating truth**](https://medium.com/@intellizab/why-the-smartest-people-in-tech-are-quietly-panicking-right-now-d2feb86e7e4b?source=post_page---read_next_recirc--939a01d27604----2---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

Feb 13

[A response icon525](https://medium.com/@intellizab/why-the-smartest-people-in-tech-are-quietly-panicking-right-now-d2feb86e7e4b?source=post_page---read_next_recirc--939a01d27604----2---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

![Screenshot of a desktop with the Cursor application open](https://miro.medium.com/v2/resize:fit:679/format:webp/0*7x-LQAg1xBmi-L1p)

[![Jacob Bennett](https://miro.medium.com/v2/resize:fill:20:20/1*abnkL8PKTea5iO2Cm5H-Zg.png)](https://jacob.blog/?source=post_page---read_next_recirc--939a01d27604----3---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[Jacob Bennett](https://jacob.blog/?source=post_page---read_next_recirc--939a01d27604----3---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[**The 5 paid subscriptions I actually use in 2026 as a Staff Software Engineer**\\
\\
**Tools I use that are (usually) cheaper than Netflix**](https://jacob.blog/the-5-paid-subscriptions-i-actually-use-in-2026-as-a-staff-software-engineer-b4261c2e1012?source=post_page---read_next_recirc--939a01d27604----3---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

Jan 18

[A response icon86](https://jacob.blog/the-5-paid-subscriptions-i-actually-use-in-2026-as-a-staff-software-engineer-b4261c2e1012?source=post_page---read_next_recirc--939a01d27604----3---------------------bb2131de_9dc0_412d_9a4f_46ac604022c8--------------)

[See more recommendations](https://medium.com/?source=post_page---read_next_recirc--939a01d27604---------------------------------------)

[Help](https://help.medium.com/hc/en-us?source=post_page-----939a01d27604---------------------------------------)

[Status](https://status.medium.com/?source=post_page-----939a01d27604---------------------------------------)

[About](https://medium.com/about?autoplay=1&source=post_page-----939a01d27604---------------------------------------)

[Careers](https://medium.com/jobs-at-medium/work-at-medium-959d1a85284e?source=post_page-----939a01d27604---------------------------------------)

[Press](mailto:pressinquiries@medium.com)

[Blog](https://blog.medium.com/?source=post_page-----939a01d27604---------------------------------------)

[Privacy](https://policy.medium.com/medium-privacy-policy-f03bf92035c9?source=post_page-----939a01d27604---------------------------------------)

[Rules](https://policy.medium.com/medium-rules-30e5502c4eb4?source=post_page-----939a01d27604---------------------------------------)

[Terms](https://policy.medium.com/medium-terms-of-service-9db0094a1e0f?source=post_page-----939a01d27604---------------------------------------)

[Text to speech](https://speechify.com/medium?source=post_page-----939a01d27604---------------------------------------)

reCAPTCHA

Recaptcha requires verification.

[Privacy](https://www.google.com/intl/en/policies/privacy/) \- [Terms](https://www.google.com/intl/en/policies/terms/)

protected by **reCAPTCHA**

[Privacy](https://www.google.com/intl/en/policies/privacy/) \- [Terms](https://www.google.com/intl/en/policies/terms/)