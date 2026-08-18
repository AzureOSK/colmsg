[日本語](README.md) | English

> [!WARNING]
> It is currently not possible to obtain a `refresh_token` on iOS because of an update to the message apps, rather than a problem with `colmsg`. For Sakurazaka46, Hinatazaka46, and Nogizaka46, you can use the experimental [official web client login](#web-client-login). Android users can still follow the [refresh-token instructions](https://github.com/proshunsuke/colmsg/blob/main/doc/how_to_get_refresh_token.md#android%E3%82%A2%E3%83%97%E3%83%AA%E3%81%AE%E5%A0%B4%E5%90%88) (in Japanese).

<div align="center">
  <h1><strong>colmsg</strong></h1>
  <img src="https://user-images.githubusercontent.com/3148511/158018437-09822a33-8767-4e03-ba90-e0f69594c493.jpeg" width="32px" alt="Sakurazaka46 Message logo"><img src="https://user-images.githubusercontent.com/3148511/158018441-dd7cb9eb-bf31-4938-830d-1ef293a2afba.jpg" width="32px" alt="Hinatazaka46 Message logo"><img src="https://user-images.githubusercontent.com/3148511/158018442-ae54e926-760d-4b47-b0a0-7255485e1f28.jpg" width="32px" alt="Nogizaka46 Message logo">

  Save messages from the Sakurazaka46 Message, Hinatazaka46 Message, Nogizaka46 Message, Asuka Saito Message, Mai Shiraishi Message, and yodel apps to your computer.

  ![Demo](https://user-images.githubusercontent.com/3148511/158026220-90735546-2401-40ca-a9e6-89d2176ad3b4.gif)
</div>

## Overview

See [Installation](#installation) to install `colmsg`.

First, obtain the `refresh_token` for each app you subscribe to. See the [instructions](doc/how_to_get_refresh_token.md) (in Japanese).

Then run the following command, replacing each placeholder with the corresponding token. You only need to specify apps to which you subscribe.

```shell
colmsg --s_refresh_token <s_refresh_token> --h_refresh_token <h_refresh_token> --n_refresh_token <n_refresh_token> --a_refresh_token <a_refresh_token> --m_refresh_token <m_refresh_token> --y_refresh_token <y_refresh_token>
```

On Windows, use `colmsg.exe` in place of `colmsg`.

This downloads all available messages from every member you subscribe to.

## Web client login

Sakurazaka46, Hinatazaka46, and Nogizaka46 can use their official web clients instead of a `refresh_token`. Google Chrome is required, but Node.js is not.

For the initial setup, run:

```shell
colmsg --web-login-setup --group nogizaka
```

Sign in using the Chrome window that opens, confirm that the web client works, and then close Chrome. Afterwards, run:

```shell
colmsg --web-login --group nogizaka
```

Replace `nogizaka` with `sakurazaka` or `hinatazaka` as needed. Other download options can be used at the same time:

```shell
colmsg --web-login --group hinatazaka -k picture -k video
```

`colmsg` keeps the signed-in state in a dedicated Chrome profile. Do not share this directory. Set `COLMSG_CHROME` if Chrome cannot be detected automatically, or `COLMSG_WEB_PROFILE` to choose a different profile location.

Web access tokens expire quickly. If one expires during a download, `colmsg` obtains a new token from Chrome and resumes automatically. If renewal fails, run the same command again; messages already saved will not be downloaded again.

## Features

- No rooted device required
- Supports both Android and iOS apps
- Runs on Windows, macOS, and Linux
- Several download filters and storage options
- Supports these app versions:
  - Sakurazaka46 Message: 1.12.01.169
  - Hinatazaka46 Message: 2.13.01.169
  - Nogizaka46 Message: 1.8.01.169
  - Asuka Saito Message: 1.1.01.169
  - Mai Shiraishi Message: 3.4.3.426
  - yodel: 4.1.1.455

## Usage

Because refresh tokens are sensitive, storing them in the [configuration file](#configuration-file) is preferable to entering them directly in the terminal. The examples below assume that your tokens are already configured.

Download messages from particular members:

```shell
colmsg -n 菅井友香 -n 佐々木久美
```

Download messages from a particular group:

```shell
colmsg -g sakurazaka
```

Download particular message types:

```shell
colmsg -k picture -k video
```

Download messages sent after a particular date and time:

```shell
colmsg -F '2020/01/01 00:00:00'
```

Options can be combined. Run the following command for the full list:

```shell
colmsg --help
```

## File layout and download behavior

- When messages have already been saved, subsequent runs download only newer messages.
- Messages are stored using the following directory structure:

  ```text
  colmsg/
  ├── 日向坂46 一期生
  │   └── 佐々木久美
  │       └── 1_0_20191231235959.txt
  ├── 乃木坂46
  │   └── 秋元真夏
  │       └── 2_1_20200101000000.jpg
  └── 櫻坂46 一期生
      └── 菅井友香
          ├── 3_2_20200101000001.mp4
          └── 4_3_20200101000002.mp4
  ```

- File names use the format `<sequence>_<type>_<date>.<extension>`.
  - The sequence number represents chronological order. Sorting by file name places messages in order from oldest to newest.
  - Message type numbers are:
    - `0`: text
    - `1`: picture
    - `2`: video
    - `3`: voice
    - `4`: link
- To display the default download location, run:

  ```shell
  colmsg --download-dir
  ```

- Existing files are not overwritten.

## Configuration file

`colmsg` can read default command-line options from a configuration file. Display its default location with:

```shell
colmsg --config-dir
```

Alternatively, set `COLMSG_CONFIG_PATH` to the configuration file's path:

```shell
export COLMSG_CONFIG_PATH="/path/to/colmsg.conf"
```

### Format

The configuration file is a plain list of command-line arguments. Run `colmsg --help` to see the available options and values. Lines beginning with `#` are comments.

Example:

```text
# Sakurazaka46 refresh token
--s_refresh_token s_refresh_token

# Hinatazaka46 refresh token
--h_refresh_token h_refresh_token

# Nogizaka46 refresh token
--n_refresh_token n_refresh_token

# Asuka Saito refresh token
--a_refresh_token a_refresh_token

# Mai Shiraishi refresh token
--m_refresh_token m_refresh_token

# yodel refresh token
--y_refresh_token y_refresh_token

# Download media files only
-k picture -k video -k voice
```

## Installation

### Windows

Prebuilt Windows executables are available as ZIP archives on the [releases page](https://github.com/proshunsuke/colmsg/releases). Extract the archive using Windows Explorer or a tool such as [7-Zip](https://www.7-zip.org/), then run `colmsg.exe` from PowerShell.

### macOS

Install using Homebrew:

```shell
brew tap proshunsuke/colmsg
brew install colmsg
```

To install without Homebrew, download the macOS archive for your system from the [releases page](https://github.com/proshunsuke/colmsg/releases):

- Apple Silicon: `colmsg-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- Intel: `colmsg-vX.Y.Z-x86_64-apple-darwin.tar.gz`

Then extract and install the binary, replacing `vX.Y.Z` with the downloaded version:

```shell
cd ~/Downloads
tar -xzf colmsg-vX.Y.Z-aarch64-apple-darwin.tar.gz
chmod +x colmsg
mkdir -p ~/.local/bin
mv colmsg ~/.local/bin/
```

Intel users should use the `x86_64` archive name in the `tar` command. If `~/.local/bin` is not already on your `PATH`, add it:

```shell
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

Confirm the installation with `colmsg --version`. If macOS blocks the binary because the developer cannot be verified, confirm that it came from the release page and allow it under **System Settings > Privacy & Security**.

### Arch Linux

Install from the [AUR](https://aur.archlinux.org/packages/colmsg/):

```shell
yay -S colmsg
```

### Other platforms

Prebuilt executables for other architectures are available on the [releases page](https://github.com/proshunsuke/colmsg/releases).

## Development

`colmsg` calls external APIs. Mock servers based on the OpenAPI definitions can be started during development:

```shell
make server/kh
make server/n46
```

Set `S_BASE_URL`, `H_BASE_URL`, or `N_BASE_URL` to use a mock server:

```shell
S_BASE_URL=http://localhost:8003 H_BASE_URL=http://localhost:8003 N_BASE_URL=http://localhost:8006 cargo run -- -d ~/Downloads/temp/ --help
```

## TODO

- [ ] Automated testing in CI
- [ ] Add examples
- [ ] Download messages in parallel
- [ ] Extract the API client into its own crate

## License

`colmsg` is distributed under the MIT License. See [LICENSE.txt](LICENSE.txt) for details.

## Important notice

Please note that Article 8 (Prohibited Conduct) of the apps' terms of service includes the following restrictions:

- Accessing or attempting to access the service by means other than those specified by the operator
- Accessing or attempting to access the service using automated means, including crawlers and similar technologies
