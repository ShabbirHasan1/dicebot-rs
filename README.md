[invite]: https://discord.com/api/oauth2/authorize?client_id=885995691942477864&scope=applications.commands

<img align="right" src="https://raw.githubusercontent.com/twitter/twemoji/master/assets/svg/1f3b2.svg" height="150" width="150" alt="Dice Logo">

# Dice Discord Bot

A simple Discord bot that provides a D&D-like dice rolling feature, running on a Cloudflare worker.

## Usage

Use `/roll` to roll some dice. The `dice` argument specifies what sort of and how many dice to
roll, e.g. `2d6` to roll two six-faced dice, or `d20` to roll a single twenty-faced die. This argument is required.
The `modifier` argument allows you to add a fixed value on top of the result, the `gm` argument allows you to roll as
the game master, only showing the result to you. Both of these arguments are optional.

## Invite

To add the bot to your server, [click here][invite].
