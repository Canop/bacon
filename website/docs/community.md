
**Bacon** is developed by **Denys Séguret**, also known as [Canop](https://github.com/Canop) or [dystroy](https://dystroy.org).

Major updates are announced on my Mastodon account: [@dystroy@mastodon.dystroy.org](https://mastodon.dystroy.org/@dystroy).

The logo has been designed by [Peter Varo](https://petervaro.com).

# Sponsorship

**Bacon** is free for all uses.

If it helps your company make money, consider helping me find time to add features and to develop new free open-source software.

<div class=sponsorship>
<script src="https://liberapay.com/dystroy/widgets/button.js"></script>
<noscript><a href="https://liberapay.com/dystroy/donate"><img alt="Donate using Liberapay" src="https://liberapay.com/assets/widgets/donate.svg"></a></noscript>

<iframe src="https://github.com/sponsors/Canop/button" title="Sponsor Canop" height="35" width="114" style="border: 0; border-radius: 6px;"></iframe>
</div>

# Chat

The best place to chat about bacon, to talk about features or bugs, is the Miaou chat.

* [Rust & Bacon room on Miaou](https://miaou.dystroy.org/3490?rust)
* [French Programmers room on Miaou](https://miaou.dystroy.org/3)

# Issues

We use [GitHub's issue manager](https://github.com/Canop/bacon/issues).

Before posting a new issue, check your problem hasn't already been raised and in case of doubt **please come first discuss it on the chat**.

# Contribute

If you think you might help, as a tester or coder, you're welcome, but please read [Contributing to my FOSS projects](https://dystroy.org/blog/contributing/) before starting a PR.

# FAQ

## What does it exactly do ?

Bacon watches the content of your source directories and launches `cargo check` or other commands on changes.

Watching and computations are done on background threads to prevent any blocking.

The screen isn't cleaned until the compilation is finished to prevent useful information from being replaced by the lines of an unfinished computation.

Errors and test failures are displayed before warnings because you usually want to fix them first.

Rendering is adapted to the dimensions of the terminal to ensure you get a proper usable report. And bacon manages rewrapping on resize.

## Parallel bacon ?

It's perfectly OK to have several bacon running in paralle, and can be useful to check several compilation targets.

Similarly you don't have to stop bacon when you want to use cargo to build the application, or when you're just working on something else. You may have a dozen bacon running without problem.

Bacon is efficient and doesn't work when there's no notification.

## Supported platforms

It works on all decent terminals on Linux, Max OSX and Windows.

## Vim & Neovim support

(Neo)Vim is perfectly supported but you may have had a problem, depending on your installation, with bacon sometimes not recomputing on file changes.

The default write strategy of vim makes successive savings of the same file not always detectable by inotify.

A solution is to add this to your init.vim file:

	set nowritebackup

This doesn't prevent vim from keeping copies during editions, it just changes the behavior of the write operation and has no practical downside.

## Licences

Bacon is licenced under [AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html).
You're free to use it to compile the Rust projects of your choice, even commercial.

The logo is licensed under a [Creative Commons Attribution-ShareAlike 4.0 International License](https://creativecommons.org/licenses/by-sa/4.0).

## Why "bacon" ?

* It's a **bac**kground **con**piler.
* It comes from France and, as you know, France is bacon.

It's just a name. You don't have to eat meat to use the software.
