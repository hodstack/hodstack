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

Hodstack is before its first tag. Every push to `0.x` builds the binary again, and this package gives you that build. Run `hod --version` to see the commit you have.

The source is at [github.com/hodstack/hodstack](https://github.com/hodstack/hodstack), under the MIT license.
