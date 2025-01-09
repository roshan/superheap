# Superheap

Substack blocks kill-the-newsletter.com and I wanted to just run my own version.
Substack has ended RSS feeds and made them into emails.
Superheap is the opposite and takes emails and builds an RSS feed just like KTN.
There's no utility to this over KTN except that you can run it on whatever domain you like and retain everything.

## Installation

1. Run `superheap serve --config /path/to/config.json`

1. Add a cron that runs `superheap generate --config /path/to/config.json` every so often

1. Use `stunnel` to terminate SSL at your host on port 25 and forward to your SMTP server

1. Subscribe to `whatever-email@your.host` on Substack

1. Use some means to host the RSS feeds. I use Cloudflare Pages and just create a new deployment every now and then.

## Configs

An example configuration for `stunnel` is in `configs/stunnel.conf`.
An example configuration for `superheap` is in `configs/superheap.json`.

## License

MIT
