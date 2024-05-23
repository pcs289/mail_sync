# Mail Sync

CLI tool to synchronize emails between **Source** and **Destination** servers over secure port __tcp/993 (IMAP over TLS)__.

## Parameters

- Config File (`-c` or `--config`): Path to Configuration File
- Verbosity (`-v` or `--verbose`): Output `DEBUG` Logs
- Quiet (`-q` or `--quiet`): No Output
- Help (`-h` or `--help`): Show Help Menu
- Version (`-V` or `--version`): Show Version

## Commands

- `list`: List Source Mailboxes
- `sync`: Synchronize Mailboxes
- `help`: Show Help Message

## Usage

1. Create a basic `creds.conf` file
```toml
[src]
host="mail.example.com"
user="test@example.com"
password="MyMailPassword"

[dst]
host="mail2.example.com"
user="test2@example.com"
password="MyMail2Password"
```

2. Execute `list` command to list source mailboxes
```shell
mail_sync -c creds.conf list
```

3. (Optional) Modify `src` section on `creds.conf` to filter certain mailboxes with `include` & `exclude` keywords. This specific example syncs all (`*`) mailboxes but `INBOX`,`Sent` & `Trash`, which are ignored.
```toml
[src]
host="mail.example.com"
user="test@example.com"
password="MyMailPassword"
include="*"
exclude="INBOX,Sent,Trash"

[dst]
host="mail2.example.com"
user="test2@example.com"
password="MyMail2Password"
```

4. Execute `sync` command to synchronize mailboxes
```shell
mail_sync -c creds.conf sync
```

## Installation

__Requirements__
- [Cargo - Rust Build System](https://doc.rust-lang.org/cargo/getting-started/installation.html)

__Steps__
1. Clone repository
```shell
git clone https://github.com/pcs289/mail_sync.git
```
2. Compile source code
```shell
cd mail_sync
cargo build --release
```
