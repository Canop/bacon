
**Bacon** is developed by **Denys Séguret**, also known as [Canop](https://github.com/Canop) or [dystroy](https://dystroy.org).

Major updates are announced on Mastodon : [@dystroy@mastodon.dystroy.org](https://mastodon.dystroy.org/@dystroy) and BlueSky: [@dystroy.bsky.social](https://bsky.app/profile/dystroy.bsky.social)

The logo has been designed by [Peter Varo](https://petervaro.com).

# Sponsorship

**Bacon** is free for all uses.

If it helps your company make money, consider helping me find time to add features and to develop new free open-source software.

<div class=sponsorship>
<!-- I don't think anybody even thought about using this
<script src="https://liberapay.com/dystroy/widgets/button.js"></script>
<noscript><a href="https://liberapay.com/dystroy/donate"><img alt="Donate using Liberapay" src="https://liberapay.com/assets/widgets/donate.svg"></a></noscript>
-->
<iframe src="https://github.com/sponsors/Canop/button" title="Sponsor Canop" height="35" width="114" style="border: 0; border-radius: 6px;margin-bottom: 20px;"></iframe>
</div>

I'm also available for consulting or custom development. Head to [https://dystroy.org](https://dystroy.org) for references.

# Chat

The best place to chat about bacon, to talk about features or bugs, is the Miaou chat.

[Bacon room on Miaou](https://miaou.dystroy.org/4683?bacon)

# Issues

We use [GitHub's issue manager](https://github.com/Canop/bacon/issues).

Before posting a new issue, check your problem hasn't already been raised and in case of doubt **please come first discuss it on the chat**.

If bacon didn't understand correctly the output of a cargo tool, it may also be useful to have a look at the log (see [below](#log)) and at the analysis export, which you normally find in a `bacon-analysis.json` file on hitting `ctrl-e`.

# Log

When something looks like a bug, I need both to know the exact configuration (OS, terminal program, mainly) and to have the log.

The log can be obtained this way:

1. start bacon with `BACON_LOG=debug bacon`
2. do the action which seems not to properly work, and nothing else
3. quit bacon
4. go to the [chat](https://miaou.dystroy.org/4683) (or the GitHub issue if you already made one) and paste the content of the `bacon.log` file

# Contribute

If you think you might help, as a tester or coder, you're welcome, but please read [Contributing to my FOSS projects](https://dystroy.org/blog/contributing/) before starting a PR.


**Don't open a PR without discussing the design before**, either in the chat or in an issue, unless you're just fixing a typo. Coding is the easy part. Determining the exact requirement and how we want it to be done is the hard part. This is especially important if you plan to add a dependency or to change the visible parts, eg the launch arguments.

# This documentation...

... needs your help too.

Tell me what seems to be unclear or missing.

Or for simple corrections, head to [the source](https://github.com/Canop/bacon/tree/main/website)
