# Redio

A fast and simple command-line client for Radio Browser.

Search internet radio stations and play them directly from your terminal using `mpv`.

## Features

- Search stations by:
  - Name
  - Country
  - Country code
  - Language
  - State
  - Tag
  - Codec
  - Bitrate

- Play stations directly with `mpv`
- Shell completions (Zsh, Bash, Fish, PowerShell)
- Simple CLI interface

## Requirements

- mpv

Arch Linux:

```bash
sudo pacman -S mpv
```

Ubuntu:

```bash
sudo apt install mpv
```

Fedora:

```bash
sudo dnf install mpv
```

## Installation

### From crates.io

```bash
cargo install redio
```

### From source

```bash
git clone https://github.com/YOUR_USERNAME/redio.git

cd redio

cargo install --path .
```

## Usage

Show API status:

```bash
redio status
```

Check dependencies:

```bash
redio doctor
```

Search stations:

```bash
redio search --name anime
```

```bash
redio search --country Japan
```

```bash
redio search --language Japanese
```

Search and play first result:

```bash
redio search --name anime play
```

Play a specific result:

```bash
redio search --name anime play --pick 3
```

## Shell Completions

Generate Zsh completions:

```bash
mkdir -p ~/.zsh/completions

redio completions zsh > ~/.zsh/completions/_redio
```

Add to your `.zshrc`:

```zsh
fpath=(~/.zsh/completions $fpath)

autoload -Uz compinit
compinit
```

Reload Zsh:

```bash
source ~/.zshrc
```

Generate Bash completions:

```bash
redio completions bash
```

Generate Fish completions:

```bash
redio completions fish
```

## Examples

Search by tag:

```bash
redio search --tag jazz
```

Search by country and language:

```bash
redio search --country Japan --language Japanese
```

Play first anime station found:

```bash
redio search --name anime play
```

## License

MIT
