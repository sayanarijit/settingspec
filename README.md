<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://s15.gifyu.com/images/buXVO.png">
    <source media="(prefers-color-scheme: light)" srcset="https://s15.gifyu.com/images/buXZJ.png">
    <img alt="The settingspec cover" width="100%" src="https://s15.gifyu.com/images/buXZJ.png">
  </picture>
</p>

[![SettingSpec Check](https://github.com/sayanarijit/settingspec/actions/workflows/settingspec-check.yml/badge.svg)](https://github.com/sayanarijit/settingspec/actions/workflows/settingspec-check.yml)
[![Crates.io Version](https://img.shields.io/crates/v/settingspec)](https://crates.io/crates/settingspec)
[![PyPI Version](https://img.shields.io/pypi/v/settingspec)](https://pypi.org/project/settingspec)
[![NPM Version](https://img.shields.io/npm/v/%40sayanarijit%2Fsettingspec)](https://www.npmjs.com/package/@sayanarijit/settingspec)

# SettingSpec

1. [Install](#install)
2. [Concepts](#concepts)
   1. [Single file convenience](#single-file-convenience)
   2. [Declarative profiles as environments](#declarative-profiles-as-environments)
   3. [Language independent](#language-independent)
   4. [Easy secrets](#easy-secrets)
      1. [SecretSpec](#secretspec)
      2. [Dotenv](#dotenv)
      3. [Cottage or agebox](#cottage-or-agebox)
3. [Learn more](#learn-more)

## Install

```bash
# rust: cargo-binstall/cargo
cargo binstall --locked settingspec
cargo install --locked settingspec

# python: pip/uv/uvx
pip install settingspec
uv pip install settingspec
uvx settingspec --version

# node: yarn/pnpm/npx
yarn global add @sayanarijit/settingspec
pnpm add -g @sayanarijit/settingspec
npx @sayanarijit/settingspec --version

# container: docker/podman
docker run --rm -v $PWD:/app ghcr.io/sayanarijit/settingspec --version
podman run --rm -v $PWD:/app ghcr.io/sayanarijit/settingspec --version
```

## Concepts

### Single file convenience

With a single source of truth, i.e., `settingspec.toml`, you don't need to mentally merge settings from multiple files, say `common.toml`, `dev.toml`, `prod.toml`, etc. No surprise overwrites. One file for all environments, all languages, all submodules.

### Declarative profiles as environments

```toml
[spec]                               # Optional: Declare the specification here
profile.key = "SETTINGSPEC_PROFILE"  # Default: Environment variable name to switch between profiles
profile.options = [                  # Optional: Enables strict checking of per-profile declarations
  "dev",
  "stage",
  "prod",
]
profile.default = "dev"  # Optional: Default profile when the switch is not set

[settings]                   # Declare the settings here with syntax: `<key>._.<profile>.<directive> = <value>`
key1._.default.val = "val1"  # Default value for all profiles
key1._.prod.val = "prod1"    # Override the default value when SETTINGSPEC_PROFILE=prod

key2._ = {  # Strictly define different values for different profiles
  dev.val = "dev2",
  stage.val = "stage2",
  prod.val = "prod2",
}
```

Overrides are intentional and kept in plain sight (see `key1`).

By not defining a default value, you can ensure that you never miss adding a value for a specific profile (e.g. this file will refuse to load if `key2._.prod` declaration is missing).

### Language independent

The command-line tool `settingspec` can export the final settings into well-known formats such as `toml`, `json`, `yaml`, `tf`, etc., or hard-coded modules such as `.py`, `.js`, `.ts`, `.lua`, `.rs`, `.go`, `.zig`, `.c`, `.cpp`, `.java`, `.elm`, `.rb`, `.scala`, `.hs`, etc., write them to disk, print, or pipe them via stdin or even export as environment variables, allowing you to use a single `settingspec.toml` without worrying about the target language.

```toml
[spec]
export.file = true         # Default: Export all settings into settings.toml
# export.file = {          # Optional: Fine-grained control over what to export
#   "settings.py" = true,  # Get auto-completion of settings in your editor/IDE for free
#   "settings.yaml" = {    # Select which keys to export
#     key1 = true,
#     group1 = true,
#     group2.subgroup = true,
#     group3.subgroup.key1 = false,
#     "#tag1" = true,
#   },
# }
export.skip_gitignore = false  # Default: Auto-append exported files to .gitignore if inside a git repo
export.mode = 0x600     # Default: File permission mode for the exported files
export.keep = false     # Default: Cleanup the exported files after the program exits
export.stdout = "toml"  # Optional: Print in toml format
export.env = "PREFIX_"  # Optional: Export as environment variables prefixed with this string
export.stdin = "toml"   # Optional: Pass settings to program stdin in specified format

[settings]
key1._.default.val = "val1"
group1.key1._.default.val = "group1val1"
group2.subgroup.key1._.default.val = "group2val1"
group3.subgroup.key1._.default.val = "group3val1"
group3.subgroup.key1._.tags = ["tag1", "tag2"]  # Syntax: `<key>._.tags = [...]`
```

Now export the final settings as declared.

```bash
settingspec export
```

Or keep watching to export when `settingspec.toml` changes.

```bash
settingspec watch
```

Or directly run the program with exported settings.

```bash
settingspec run -- [your program]...
```

If no export option is specified, the default behavior is to export all settings into `settings.toml`.

> [!NOTE]
> If the target programming language supports null values, you can declare it as `key._.default.null = true` in the settings section.

### Easy secrets

Unlike regular settings, secrets aren't supposed to be easily visible. So, they are best declared inside hidden/encrypted files and passed via environment variables or via stdin.

However, with SettingSpec, we can ensure that they are actually set and also include them in the final exported settings.

#### SecretSpec

You can pair up SettingSpec with a declarative secrets manager such as [SecretSpec](https://secretspec.dev).

secretspec.toml

```toml
[project]
name = "my-app"
revision = "1.0"

[profiles.default]
SECRET1 = { description = "Secret One", required = true }
SECRET2 = { description = "Secret Two", required = false }
```

settingspec.toml

```toml
[spec]
profile.options = ["dev", "stage", "prod"]

[settings]
secret1._.default.env = "SECRET1"  # Load value from $SECRET1

# Override the default value from environment variables
secret2._.default.val = "defaultvalue"  # Default value if $SECRET2 is not set
secret2._.default.env = "SECRET2"       # Load value from $SECRET2 if set
secret2._.prod.env = "PRODSECRET"       # Load value from $PRODSECRET if SETTINGSPEC_PROFILE=prod
```

With these set up, run:

```bash
secretspec run -- settingspec run -- [your program]...
```

#### Dotenv

Or keep things simple with [Dotenv](https://www.dotenv.org).

```toml
[spec]
profile.options = ["dev", "stage", "prod"]

envfile.default = ".env"    # Default environment file
envfile.stage = ".env.age"  # Also supports age-encrypted env files
envfile.prod = "-"          # Read from stdin, encrypted or not

# decryption.key.env = "SETTINGSPEC_DECRYPTION_KEY"  # Default
# decryption.key.path = "~/.ssh/"                    # Default

[settings]
secret1._.default.env = "SECRET1"     # Load value from $SECRET1
```

#### Cottage or agebox

Too simple? Use [Cottage](https://cottage-cli.pages.dev) or [agebox](https://github.com/slok/agebox).

```toml
[spec]
envfile.default = ".env.cott.age"
# envfile.default = ".env.agebox"

decryption.key.path = "~/.config/cottage/identity:~/.ssh"
decryption.key.env = "COTTAGE_IDENTITY"
# decryption.key.env = "AGEBOX_PRIVATE_KEYS"

[settings]
secret._.default.env = "SECRET"
```

## Learn more

Check out the [settingspec.toml](./settingspec.toml) example file and [full specification](./SPECIFICATION.md) for more details.
