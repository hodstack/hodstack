# Hodstack

Hodstack makes coding agents more productive.

This package installs `hod`, the Hodstack command line program. The install step downloads the binary for your platform and verifies its checksum.

```sh
npm install --global hodstack@edge
```

Then write the two files that a coding agent reads:

```sh
hod init
```

Run `hod update` to install the newest build. The command writes over the binary that this package holds, so you do not install the package again.

Hodstack is before its first tag. Every push to `0.x` builds the binary again, and this package gives you that build. Run `hod --version` to see the commit you have.

This package carries three names: `hodstack`, `@hodstack/cli` and `@hodstack/hod`. They hold the same files. Install any one of them.

The source is at [github.com/hodstack/hodstack](https://github.com/hodstack/hodstack), under the MIT license.
