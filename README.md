<p align="center">
  <picture>
    <a href="https://youtu.be/JGBSFYzBN2U"><img alt="The SettingSpec promo" width="100%" src="https://s15.gifyu.com/images/bWlSx.webp"></a>
  </picture>
</p>

[![SettingSpec Check](https://github.com/sayanarijit/settingspec/actions/workflows/settingspec-check.yml/badge.svg)](https://github.com/sayanarijit/settingspec/actions/workflows/settingspec-check.yml)
[![Crates.io Version](https://img.shields.io/crates/v/settingspec)](https://crates.io/crates/settingspec)
[![PyPI Version](https://img.shields.io/pypi/v/settingspec)](https://pypi.org/project/settingspec)
[![NPM Version](https://img.shields.io/npm/v/%40sayanarijit%2Fsettingspec)](https://www.npmjs.com/package/@sayanarijit/settingspec)

# SettingSpec

Don't program your settings. Specify • Validate • Export - in any format.

1. [Install](#install)
2. [Basic usage](#basic-usage)
   1. [Specify settings in a single file](#specify-settings-in-a-single-file)
   2. [Validate the specification](#validate-the-specification)
   3. [Export the settings](#export-the-settings)
   4. [Use the exported settings in your program](#use-the-exported-settings-in-your-program)
3. [Concepts](#concepts)
   1. [Single file convenience](#single-file-convenience)
   2. [Strict validation of per profiles settings](#strict-validation-of-per-profiles-settings)
   3. [Language independent](#language-independent)
   4. [Easy secrets](#easy-secrets)
      1. [SecretSpec](#secretspec)
      2. [Dotenv](#dotenv)
      3. [Cottage or agebox](#cottage-or-agebox)
4. [Learn more](#learn-more)

## Install

[Directly download the binary](https://github.com/sayanarijit/settingspec/releases).

Or install from a package registry:

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

## Basic usage

#### Specify settings in a single file

Create a file named `settingspec.toml` at the project root.

```toml
# settingspec.toml

[spec]
profile.key = "ENV"
profile.options = ["local", "prod"]
profile.default = "local"

[settings]
HOST._ = {
    local.val = "127.0.0.1",
    prod.val = "0.0.0.0",
}
PORT._.default.val = 8080
```

The `[spec]` section is optional. If not present, the default specification is used.

The `[settings]` section contains the settings in `{key}._.{profile}.{directive} = {value}` syntax.

#### Validate the specification

```bash
settingspec check
```

Ensures that the declared settings are valid and consistent with the specification, no key is missing declaration for any of the declared profiles, among other checks.

#### Export the settings

```bash
# export ENV=prod
settingspec export
```

#### Use the exported settings in your program

```bash
my-app -c settings.toml
```

`my-app` will use the following settings:

```toml
HOST = "127.0.0.1"
PORT = 8080
```

As per our expectation, `HOST` will be `0.0.0.0` when `ENV=prod` is set.

## Concepts

### Single file convenience

Are you tired of mentally merging settings from multiple files and debugging runtime errors due to missing or misconfigured settings?

```python
# Fragile boilerplate code to load settings from multiple files
def load_settings():
    with open("common.toml") as f:
        settings = toml.load(f)
    if os.environ.get("ENV") == "staging":
        with open("dev.toml") as f:
            settings.update(toml.load(f))
    elif os.environ.get("ENV") == "production":
        with open("prod.toml") as f:
            settings.update(toml.load(f))
    resolve_environment_variables(settings)
    return settings
```

With a single source of truth, i.e., `settingspec.toml`, everything is declarative. Everything is visible. No surprise value overrides, or missed overrides.

### Strict validation of per profiles settings

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

The command-line tool `settingspec` can export the final settings into well-known formats such as `toml`, `json`, `yaml`, `tf`, etc., or hard-coded modules such as `.py`, `.js`, `.ts`, etc., write them to disk, print, or pipe them via stdin or even export as environment variables, allowing you to use a single `settingspec.toml` without worrying about the target language.

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
secret2._.prod.val = "prodvalue"        # Fallback for prod only, else fail if $PRODSECRET is not set
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
envfile = ".env.cott.age"
# envfile = ".env.agebox"

decryption.key.path = ["~/.config/cottage/identity", "~/.ssh"]
decryption.key.env = "COTTAGE_IDENTITY"
# decryption.key.env = "AGEBOX_PRIVATE_KEYS"

[settings]
secret._.default.env = "SECRET"
```

## Learn more

Check out the [settingspec.toml](./settingspec.toml) example file and [full specification](./SPECIFICATION.md) for more details.
