# SettingSpec Specification

**Version:** 0.4  
**Status:** Released  
**Authors:** Arijit Basu and SettingSpec Contributors  
**Repository:** [https://github.com/sayanarijit/settingspec](https://github.com/sayanarijit/settingspec)

---

## 1. Overview and Goals

### 1.1 Purpose

Modern software development requires managing configuration across multiple environments (such as development, staging, testing, and production), various programming languages (such as Python, Node.js, Rust, Go, Lua), and nested modules. Traditional approaches frequently split configurations across multiple files (`common.toml`, `dev.toml`, `prod.toml`), leading to:

1. **Mental overhead:** Developers must mentally merge configurations and guess override precedence.
2. **Silent configuration drift:** Missing keys in production often remain undetected until deployment failure or runtime crashes.
3. **Language lock-in:** Configurations written for one language (e.g., Python config or YAML) require duplication or custom converters for other tools in polyglot projects.
4. **Secret handling friction:** Secrets are either inadvertently committed to source control or detached from configuration schemas altogether.

**SettingSpec** establishes a unified, single-source-of-truth configuration format (`settingspec.toml`). It provides declarative profile definitions, compile-time/load-time strict environment validation, language-agnostic export capabilities, and seamless secret sourcing.

### 1.2 Conformance and Key Words

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

### 1.3 Terminology

- **Configuration File:** The primary TOML file declaring the specification and settings (by default `settingspec.toml`).
- **Profile:** A named runtime environment or operational context (e.g., `dev`, `stage`, `prod`).
- **Default Profile (`default`):** A fallback profile selector denoting settings applicable across all profiles unless explicitly overridden.
- **Setting Key:** A hierarchical dot-delimited identifier (e.g., `key1`, `database.host`, `group1.auth.timeout`).
- **Directive:** A terminal attribute defining the origin or property of a setting (`val`, `env`, `null`, `tags`).
- **Active Profile:** The single profile selected for the current execution context.
- **Export Target:** A file or standard output stream where resolved settings are serialized.

---

## 2. File Format and Document Structure

### 2.1 File Location and Syntax

1. A SettingSpec configuration MUST be a valid [TOML v1.0.0](https://toml.io/en/v1.0.0) document.
2. The configuration file name is `settingspec.toml`, located in the root of the project directory. If `settingspec.toml` is not found in the current directory, SettingSpec traverses parent directories upwards until it finds one; that directory is the project root.
3. A SettingSpec document consists of two top-level tables:
   - `[spec]`: Metadata, profile declarations, environment file sources, decryption key declarations, and export definitions (OPTIONAL).
   - `[settings]`: Setting definitions and per-profile value declarations (REQUIRED).

### 2.2 TOML Key Representations

SettingSpec uses dotted keys to represent hierarchical settings, target profiles, and directives.

```toml
[settings]
key1.default.val = "val1"
```

---

## 3. The `[spec]` Section

The `[spec]` section configures profile resolution, environment variable file loading, decryption key declarations and export definitions.

```toml
[spec]
profile.key = "SETTINGSPEC_PROFILE"
profile.options = ["dev", "stage", "prod"]
profile.default = "dev"

envfile.default = ".env"
envfile.stage = ".env.age"  # age-encrypted env file
envfile.prod = "-"

decryption.key.env = "SETTINGSPEC_DECRYPTION_KEY"
decryption.key.path = "~/.ssh/"

[spec.export]
mode = 0x600
keep = false
stdout = "toml"

[spec.export.file]
"settings.toml" = true
"settings.json".group1 = true
```

### 3.1 Profile Configuration (`spec.profile`)

The `profile` sub-table configures active profile selection and validation rules:

| Field             | Type             | Default                 | Description                                                                         |
| ----------------- | ---------------- | ----------------------- | ----------------------------------------------------------------------------------- |
| `profile.key`     | String           | `"SETTINGSPEC_PROFILE"` | The name of the environment variable inspected to determine the active profile.     |
| `profile.options` | Array of Strings | `[]` (empty / unset)    | The closed list of valid profile names. Enables strict coverage validation.         |
| `profile.default` | String           | `None` (unset)          | Fallback profile to activate when the environment variable is not defined or empty. |

#### 3.1.1 Profile Validation Rules

- If `profile.options` is specified:
  1. The active profile MUST be an element of `profile.options`. If the resolved active profile is not in `profile.options`, execution MUST fail immediately with an error.
  2. Any profile name used within the `[settings]` table MUST either be an element of `profile.options` or the reserved identifier `default`. Any unknown profile name MUST cause configuration loading to fail.
  3. The reserved name `default` MUST NOT be included as an element in `profile.options`.
  4. **Strict Coverage Rule:** For every setting declared in `[settings]`, if a `default` declaration is absent, a declaration MUST exist for **every** profile listed in `profile.options`. If any profile is unmapped, loading MUST fail.
  5. **Reserved Word Conflict:** No element of `profile.options` MUST be one of the reserved directive keywords `val`, `env`, `null`, or `tags` (see Section 4.1.1). A profile name colliding with a directive keyword MUST cause configuration loading to fail.

### 3.2 Environment Files (`spec.envfile`)

SettingSpec can load environment variable definition files (dotenv files) prior to resolving settings. This is declared under `spec.envfile`:

- `envfile.default`: Environment file loaded when the active profile has no specific override.
- `envfile.<profile>`: Environment file loaded specifically when `<profile>` is active.

```toml
[spec]
envfile.default = ".env"
envfile.stage = ".env.age"
envfile.prod = "-"
```

An environment file MAY be [age](https://age-encryption.org/)-encrypted. SettingSpec identifies an encrypted environment file either by its `.age` extension or by detecting the age binary/armored payload header when reading the file's content, and transparently decrypts it (per Section 3.2.2) before it is parsed as dotenv content.

#### 3.2.1 Sourcing Rules

1. If an environment file is declared for the active profile (`envfile.<profile>`), it is loaded directly. If declared but the referenced file is missing, SettingSpec MUST terminate with an error (it MUST NOT fall back to `envfile.default`). SettingSpec MUST NOT overwrite or merge `envfile.default` with an active profile's environment file. If no profile-specific environment file is declared for the active profile, `envfile.default` is loaded as a fallback.
2. Sourcing occurs **before** setting values are resolved.
3. The special value `"-"` denotes reading environment variables in dotenv format (`KEY=VALUE`) from standard input (`stdin`).
4. Sourced environment variables populate the execution environment for the duration of setting resolution and child command execution.
5. If the environment file is age-encrypted, SettingSpec MUST decrypt it (Section 3.2.2) prior to applying Rules 1–4. The special value `"-"` (stdin) MAY also carry age-encrypted content; the same detection and decryption applies before dotenv parsing.

#### 3.2.2 Encrypted Environment Files (`spec.decryption`)

SettingSpec supports transparent decryption of [age](https://age-encryption.org/)-encrypted `envfile` sources, allowing encrypted secrets to be committed to source control alongside `settingspec.toml`.

- Detection: a file referenced by `spec.envfile.*` is treated as age-encrypted if its name ends in `.age`, or its content matches the age binary or armored (`-----BEGIN AGE ENCRYPTED FILE-----`) format.
- Decryption keys are resolved via the `decryption.key` sub-table:

  | Field                 | Type                       | Default                        | Description                                                                                         |
  | --------------------- | -------------------------- | ------------------------------ | --------------------------------------------------------------------------------------------------- |
  | `decryption.key.env`  | String                     | `"SETTINGSPEC_DECRYPTION_KEY"` | Name of the environment variable inspected for age decryption identities (keys).                    |
  | `decryption.key.path` | String or Array of Strings | unset                          | Fallback filesystem path(s) searched for age identity/key files when the env var is unset or empty. |

  ```toml
  [spec]
  decryption.key.env = "SETTINGSPEC_DECRYPTION_KEY"
  decryption.key.path = "~/.ssh/"
  ```

- `decryption.key.path` MAY be a single path or an array of paths; each path MAY reference a specific key file or a directory. Directories are searched recursively for candidate identity files (see Section 3.2.3, Path Lookup). Passphrase-protected identity files are not accepted (see Section 3.2.3, Passphrase Restriction).

#### 3.2.3 Decryption Key Resolution Order

1. **Environment Variable:** SettingSpec reads the environment variable named by `decryption.key.env` (default `SETTINGSPEC_DECRYPTION_KEY`). If set and non-empty, its value is interpreted per the **Value Format** rules below, and `decryption.key.path` is NOT consulted.
2. **Fallback Path(s):** If the environment variable is unset or empty, SettingSpec falls back to `decryption.key.path`, resolved using the same **Path Lookup** rules below.
3. **Resolution Failure:** If an `envfile.*` entry is age-encrypted and no decryption identity resolves via steps 1–2, decryption MUST fail and SettingSpec MUST terminate with an error identifying the affected profile and file.

**Value Format (`decryption.key.env`):** The value of the environment variable named by `decryption.key.env` MUST be exactly one of the following two forms:

- **A single literal age identity string** (e.g. `AGE-SECRET-KEY-1...`), used directly as the decryption identity; or
- **One or more filesystem paths, delimited by colons (`:`)**, `PATH`-style (e.g. `~/.age/keys:/etc/settingspec/keys`), resolved per the **Path Lookup** rule below.

A value is treated as a literal identity if it matches the age identity string format; otherwise it is treated as a colon-delimited path list.

**Path Lookup:** For each path supplied — whether via `decryption.key.env` (when it holds paths) or via `decryption.key.path` — SettingSpec MUST:

1. If the path is a file, consider that file directly.
2. If the path is a directory, recursively scan the directory tree for files, and consider every file that matches a supported age identity file format.
3. Attempt decryption against each considered candidate, in the order encountered, and use the first identity that successfully decrypts the file. If none succeed across all supplied paths, resolution fails per Rule 3 above.

**Passphrase Restriction:** Age identity files protected by a passphrase (i.e., requiring interactive passphrase entry to unlock the identity itself) are NOT supported. SettingSpec MUST reject/skip passphrase-protected identity files and MUST only consider plain, unencrypted age identities as decryption candidates.

### 3.3 Export Configuration (`spec.export`)

The `export` sub-table specifies how resolved settings are output to files, stdout, stdin, or environment variables.

#### 3.3.1 File Mode (`export.mode`)

- **Type:** Integer (POSIX permission bits).
- **Default:** `0x600` (octal `0600`, read/write by owner only).
- **Description:** All files created by `settingspec export` or `settingspec run` MUST be created or chmod-ed to this mode to protect configuration secrets against unauthorized local access.

#### 3.3.2 File Retention (`export.keep`)

- **Type:** Boolean.
- **Default:** `false`.
- **Description:** Controls whether exported target files are retained after `settingspec run` finishes execution.
  - When `false` (default), SettingSpec automatically cleans up and deletes all generated export files after the child command finishes execution.
  - When `true`, SettingSpec retains all exported files on disk after the child command finishes execution.

#### 3.3.3 Standard Output (`export.stdout`)

- **Type:** Boolean or String (`"toml"`, `"json"`, `"yaml"`, `"env"` etc. supported formats).
- **Default:** None.
- **Description:** Emits the resolved settings directly to standard output in the specified serialization format.
  - Setting `export.stdout = true` or `export.stdout = ""` emits resolved settings formatted as TOML (default format).
  - Setting `export.stdout = "<format>"` (e.g., `"json"`, `"yaml"`, `"toml"`, `"env"`) emits resolved settings formatted in the specified serialization format.
  - Setting `export.stdout = false` (or omitting `export.stdout`) disables stdout export.

#### 3.3.4 Target Files (`export.file`)

The `export.file` table maps target file paths to export filter expressions:

```toml
[spec.export.file]
"settings.toml" = true
"settings.yaml" = { key1 = true, group1 = true, group2.subgroup = true, group3.subgroup = false, "#tag1" = true }
```

- Setting `export.file = true` defaults to exporting all settings to `settings.toml`.
- Setting `export.file = false` disables exporting to files.
- If no export configuration is declared in `[spec]`, the default behavior MUST be:
  ```toml
  [spec.export.file]
  "settings.toml" = true
  ```
- Environment variable files (`.env`, `.env.*`, `env.*`, `*.env`) are exported in `env` format.
- If a target file's format is unrecognizable (e.g., `"aaa"`, `"output.xyz"`), SettingSpec MUST fail with an unsupported export format error.

#### 3.3.5 Environment Variables (`export.env`)

- **Type:** Boolean or String.
- **Default:** None (disabled).
- **Description:** Exports resolved settings directly as environment variables into the child process when executing `settingspec run`.
  - Setting `export.env = true` or `export.env = ""` exports resolved settings using their uppercase, underscore-delimited key names (e.g., `database.host` -> `DATABASE_HOST`).
  - Setting `export.env = "PREFIX_"` exports resolved settings with the specified prefix prepended to the key names (e.g., `database.host` -> `PREFIX_DATABASE_HOST`).
  - Setting `export.env = false` (or omitting `export.env`) disables exporting settings as environment variables.
  - Setting keys with `null` values are omitted from exported environment variables.

#### 3.3.6 Standard Input (`export.stdin`)

- **Type:** Boolean or String (`"toml"`, `"json"`, `"yaml"`, `"env"` etc. supported formats).
- **Default:** None (disabled).
- **Description:** Passes resolved settings directly to standard input (stdin) of the child process when executing `settingspec run`.
  - Setting `export.stdin = true` or `export.stdin = ""` passes resolved settings formatted as TOML.
  - Setting `export.stdin = "<format>"` (e.g., `"json"`, `"yaml"`, `"toml"`, `"env"`) passes resolved settings formatted in the specified serialization format.
  - Setting `export.stdin = false` (or omitting `export.stdin`) disables passing settings via stdin.

#### 3.3.7 Git Integration (`export.skip_gitignore`)

- **Type:** Boolean.
- **Default:** `false`.
- **Description:** When executing inside a git repository, SettingSpec automatically appends all exported file targets to `.gitignore` (adjacent to the git root) before each `run` or `export` execution if not already present. Setting `spec.export.skip_gitignore = true` disables this automatic behavior.

---

## 4. The `[settings]` Section

The `[settings]` section defines configuration keys, their values, environment bindings, tags, and profile-specific overrides.

### 4.1 Declaration Grammar

Each declaration in `[settings]` follows the canonical syntax:

```
{key}.{profile}.{directive} = {value}
```

Where:

- `{key}`: A dot-delimited identifier representing the setting name or namespace path (e.g., `key1`, `database.host`, `app.server.port`).
- `{profile}`: A valid profile name defined in `spec.profile.options` or the reserved identifier `default`.
- `{directive}`: One of the reserved terminal keywords: `val`, `env`, `null`, or `tags`.
- `{value}`: The directive value.

Because TOML treats dot-delimited keys as table hierarchies, the declaration parser identifies:

1. The **terminal segment** as the directive (`val`, `env`, `null`, or `tags`).
2. The **penultimate segment** as the profile identifier (`default` or profile name).
3. All **preceding segments** as the hierarchical setting key path.

#### 4.1.1 Reserved Directive Keywords

The identifiers `val`, `env`, `null`, and `tags` are **reserved directive keywords**. They MUST NOT be used as:

1. A **setting key segment** (any segment in the `{key}` path, at any depth — top-level or nested), including as a standalone key name or as an intermediate group/namespace name (e.g., `env.default.val = "x"`, `group1.tags.prod.val = "y"`, and `val.default.env = "Z"` are all INVALID).
2. A **profile name**, whether or not `spec.profile.options` is declared (e.g., a profile literally named `env`, `val`, `null`, or `tags` is INVALID; see also Section 3.1.1, Rule 5).

**Rationale:** The declaration grammar (Section 4.1) disambiguates a dotted path purely by _position_ — the terminal segment is always parsed as the directive, and the penultimate segment is always parsed as the profile. If a key segment or profile name reuses one of the four reserved words, the resulting path becomes structurally ambiguous or silently misparsed (e.g., it may be unclear whether `env` in a given position denotes the reserved `env` directive/profile or a user-defined namespace/profile called "env"). Reserving these four identifiers eliminates this ambiguity entirely, independent of position.

**Validation:** SettingSpec implementations MUST reject, at configuration load time, any setting declaration whose key path or profile segment matches a reserved directive keyword. This validation MUST occur alongside the Static Validation Rules in Section 5.3 (Directive Validity), and MUST cause configuration loading to fail with a clear diagnostic identifying the offending segment.

### 4.2 Setting Directives

#### 4.2.1 `val`

Supplies a static, literal value for the setting.

- **Supported Types:** String, Integer, Float, Boolean, Datetime (RFC 3339), Array, Inline Table.
- **Example:**
  ```toml
  app.name.default.val = "MyApp"
  app.port.dev.val = 8080
  app.port.prod.val = 80
  ```

#### 4.2.2 `env`

Binds the setting to an environment variable name.

- **Type:** String (the name of an environment variable).
- **Semantics:** At resolution time, SettingSpec attempts to read the named environment variable from the runtime environment (including variables loaded via `spec.envfile`).
- **Example:**
  ```toml
  database.password.default.env = "DB_PASSWORD"
  ```

#### 4.2.3 Coexistence of `val/null` and `env` (Default with Env Override)

A setting profile declaration MAY specify both `val/null` and `env`. In such cases, the following resolution rules apply:

```toml
secret2.default.val = "defaultvalue"
secret2.default.env = "SECRET2"
secret2.prod.env = "PRODSECRET"
```

- If the environment variable specified by `env` exists, its value MUST be used.
- If the environment variable does not exist, the value specified by `val/null` MUST be used as the fallback default.
- If a setting has an `env` directive without a `val` or `null` directive, and the environment variable is absent at runtime, resolution MUST fail with a missing variable error.

#### 4.2.4 `null`

Explicitly declares that a setting has a null or unset value for the specified profile.

- **Type:** Boolean (`true`).
- **Example:**
  ```toml
  debug_banner.default.val = "My App BETA 0.0.1"
  debug_banner.prod.null = true
  ```
- **Target Export Semantics:**
  - For formats with native null representations (JSON, YAML, Python `None`, Lua `nil`), the setting MUST be exported as null/None/nil.
  - For formats without native null representations (TOML 1.0, shell `.env`), the setting key MUST be omitted from the exported target.
  - `null` can only be `true` and cannot co-exist with `val` for the same profile declaration. If both are present, configuration loading MUST fail.

#### 4.2.5 `tags`

Associates a list of string tags with a setting.

- **Type:** Array of Strings.
- **Example:**
  ```toml
  api_key.default.tags = ["sensitive", "auth"]
  ```
- **Semantics:** Tags are metadata and are NOT exported as values of the setting. They are used by export filters to include or exclude tagged settings (e.g., `#auth`).

---

## 5. Resolution and Validation Semantics

### 5.1 Active Profile Resolution Order

When executing SettingSpec, the active profile MUST be determined using the following precedence (highest priority first):

1. **Environment Variable:** The value of the environment variable specified by `spec.profile.key` (default: `SETTINGSPEC_PROFILE`).
2. **Specification Default:** The value specified by `spec.profile.default`.
3. **Resolution Failure:** If none of the above are set:
   - If `spec.profile.options` is defined, execution MUST terminate with an error stating that no profile was selected.
   - If `spec.profile.options` is not defined, execution MAY continue using default-only resolution.

### 5.2 Setting Resolution Precedence

For a given setting key `K` and active profile `P`, the resolved value `V(K, P)` MUST be resolved using the following order:

```mermaid
flowchart TD
    Start["Resolve Key K for Profile P"] --> HasProfileDecl{"Has declaration for Profile P?"}

    HasProfileDecl -- Yes --> CheckProfileEnv{"Profile P has env declared?"}
    CheckProfileEnv -- Yes --> CheckEnvExists{"Is env var set?"}
    CheckEnvExists -- Yes --> ReturnEnv["Return env var value"]
    CheckEnvExists -- No --> CheckProfileVal{"Profile P has val/null declared?"}

    CheckProfileEnv -- No --> CheckProfileVal
    CheckProfileVal -- Yes --> ReturnProfileVal["Return Profile P val/null"]
    CheckProfileVal -- No --> CheckDefault

    HasProfileDecl -- No --> CheckDefault{"Has default declaration?"}

    CheckDefault -- Yes --> CheckDefaultEnv{"Default has env declared?"}
    CheckDefaultEnv -- Yes --> CheckDefaultEnvExists{"Is default env var set?"}
    CheckDefaultEnvExists -- Yes --> ReturnDefaultEnv["Return default env var value"]
    CheckDefaultEnvExists -- No --> CheckDefaultVal{"Default has val/null declared?"}

    CheckDefaultEnv -- No --> CheckDefaultVal
    CheckDefaultVal -- Yes --> ReturnDefaultVal["Return default val/null"]
    CheckDefaultVal -- No --> ResolutionError["Error: Required value missing"]

    CheckDefault -- No --> StrictCheck{"spec.profile.options defined?"}
    StrictCheck -- Yes --> MissingCoverError["Error: Key missing coverage for Profile P"]
    StrictCheck -- No --> KeyOmitted["Key omitted"]
```

> **Note:** This flowchart assumes the key path `K`, profile `P`, and directive have already been unambiguously decomposed from the raw TOML declaration per Section 4.1's positional parsing rule, and that no segment of `K` or `P` collides with a reserved directive keyword (Section 4.1.1). Declarations violating that constraint MUST be rejected during static validation (Section 5.3) before resolution begins. Environment file sourcing (including decryption of age-encrypted `envfile` sources per Section 3.2.2) MUST complete before this resolution process begins, so that variables loaded from a decrypted envfile are visible to `CheckEnvExists`/`CheckDefaultEnvExists`.

### 5.3 Static Validation Rules

During initialization, SettingSpec MUST validate the configuration document prior to performing any export or execution:

1. **Option Membership:** If `spec.profile.options` is set, every profile referenced in `[settings]` MUST be present in `spec.profile.options` or be `default`.
2. **Exhaustive Profile Coverage:** If `spec.profile.options` is set, every setting key `K` defined in `[settings]` MUST satisfy at least one of:
   - A `default` declaration exists for `K`.
   - An explicit declaration exists for `K` across **all** profiles listed in `spec.profile.options`.
3. **Directive Validity:** A declaration MUST contain `env` or at least one of `val` or `null = true`, along with optional `tags`. Unknown directives MUST be rejected with an error.
4. **Reserved Keyword Conflict:** No setting key segment and no profile name MUST match a reserved directive keyword (`val`, `env`, `null`, `tags`), per Section 4.1.1. Any such collision MUST be rejected with an error identifying the offending segment.

---

## 6. Export Target Filtering and Generation

### 6.1 Filter Expressions and Types

Export target specifications under `spec.export.file` select subsets of resolved settings for each target file. SettingSpec supports two filter representations:

#### 6.1.1 Boolean Filter (`true` / `false`)

- **`true`:** Exports all resolved settings into the target file.
- **`false`:** Disables exporting to this target file (no file is created).

```toml
[spec.export.file]
"settings.toml" = true
```

#### 6.1.2 Table Filter (Fine-Grained Selector Map)

When finer control over inclusion and exclusion is required, an export target can be defined as an inline table or sub-table mapping selectors (exact keys, namespace prefix paths, or tags) to boolean values (`true` to include, `false` to exclude):

```toml
[spec.export.file]
"settings.yaml" = { key1 = true, group1 = true, group2.subgroup = true, group3.subgroup = false, "#tag1" = true }
```

Or using standard TOML table syntax:

```toml
[spec.export.file."settings.yaml"]
key1 = true
group1 = true
group2.subgroup = true
group3.subgroup.key1 = false
"#tag1" = true
```

Exclusion rules take precedence over inclusion rules.

### 6.2 Table Filter Semantics and Resolution Rules

In a table filter, each key is a selector (exact key, group path, or tag) and each value is a boolean flag (`true` or `false`).

#### 6.2.1 TOML Syntax and Non-Conflicting Rules

SettingSpec configurations MUST be valid TOML v1.0.0 documents. In TOML:

1. **No scalar/table conflicts:** A key cannot be assigned a scalar value and subsequently reopened as a table. Syntax such as:
   ```toml
   # INVALID TOML: Fails with parse error
   group2 = true
   group2.subgroup = false
   ```
   is syntactically invalid because `group2` cannot simultaneously be a boolean scalar and a table containing `subgroup`.
2. **No duplicate keys:** Defining duplicate keys in the same scope (e.g., `key = true` and `key = false`) is forbidden by TOML.

Consequently, table filters do NOT allow or require conflicting hierarchical rules on the exact same key path. Instead, SettingSpec interprets scoped rules hierarchically.

#### 6.2.2 Filter Evaluation and Precedence Rules

SettingSpec MUST evaluate table filters using the following precedence and scoping rules:

1. **Exclusion Preferred Over Inclusion:**
   Exclusion (`false`) always takes precedence over inclusion (`true`). If a setting matches an applicable exclusion rule (whether by exact key, namespace prefix, or tag), it MUST NOT be exported, even if it also matches an inclusion rule.

2. **Group and Subgroup Inclusion (`= true`):**
   Setting a group or subgroup selector to `true` includes all settings under that namespace:
   - If `group1 = true`, all settings under `group1` (e.g., `group1.key1`, `group1.sub.key2`) are included.
   - If `group1.subgroup1 = true`, all settings under `group1.subgroup1` (e.g., `group1.subgroup1.key1`) are included.

3. **Exact Key Inclusion (`= true`):**
   Setting an individual key to `true` (e.g., `key1 = true` or `group2.subgroup2.key = true`) includes only that specific setting:
   - If `group2.subgroup2.key = true`, only `group2.subgroup2.key` is included. Sibling keys within `group2.subgroup2` are NOT included unless explicitly selected.

4. **Implicit Parent Inclusion via Child Exclusion (`= false`):**
   Because TOML syntax does not permit writing `group = true` alongside `group.subgroup = false`, setting a child path or key to `false` in the absence of sibling inclusion rules implicitly includes the enclosing parent namespace while excluding that specified child:
   - If `group3.subgroup = false`, all settings from `group3` are included, **excluding** those under `group3.subgroup`.
   - If `group2.subgroup2.key = false` (or `group1.subgroup2.key = false`), all settings from the parent namespace are included, **excluding** `group2.subgroup2.key`.

5. **Mixed Inclusion and Exclusion in the Same Scope:**
   If a namespace specifies both inclusion (`true`) and exclusion (`false`) rules:
   - For example, if `group2.subgroup2.key1 = true` and `group2.subgroup2.key2 = false`:
     - Only `group2.subgroup2.key1` is included.
     - `group2.subgroup2.key2` is explicitly excluded.
     - Unmentioned sibling keys (e.g., `group2.subgroup2.key3`) are NOT included. Explicit inclusion rules (`key1 = true`) establish the candidate set for that scope, so the presence of an exclusion rule (`key2 = false`) does not implicitly include unselected siblings. Exclusion is preferred over inclusion.

6. **Tag Selectors (`"#tag"`):**
   Tag selectors cross-cut hierarchy:
   - If `"#tag1" = true`, any setting declaring `tags = ["tag1"]` is included (unless excluded by an exclusion rule).
   - If `"#tag2" = false`, any setting declaring `tags = ["tag2"]` is excluded, even if its parent group was set to `true`.

7. **Base Inclusion Scope:**
   - **Explicit Inclusion Mode:** If the table filter contains one or more inclusion rules (or implicit parent inclusions from child exclusion rules), only settings matching those inclusions are considered candidates for export. Settings in other unmentioned namespaces are excluded.
   - **Pure Exclusion Mode:** If the table filter contains **only** top-level exclusion rules (e.g., `group1 = false`, `"#draft" = false`), all resolved settings are candidates by default, except those matching any exclusion rule.

#### 6.2.3 Summary of Filter Patterns

| Pattern                | Example                                                           | Resolved Behavior                                                |
| ---------------------- | ----------------------------------------------------------------- | ---------------------------------------------------------------- |
| Group Inclusion        | `group1 = true`                                                   | Include all settings from `group1`.                              |
| Subgroup Inclusion     | `group1.subgroup1 = true`                                         | Include all settings from `group1.subgroup1`.                    |
| Exact Key Inclusion    | `group2.subgroup2.key = true`                                     | Include only `group2.subgroup2.key`.                             |
| Subgroup Exclusion     | `group3.subgroup = false`                                         | Include all from `group3` except `group3.subgroup`.              |
| Key Exclusion          | `group2.subgroup2.key = false`                                    | Include all from parent namespace except `group2.subgroup2.key`. |
| Mixed in Same Scope    | `group2.subgroup2.key1 = true`<br>`group2.subgroup2.key2 = false` | Include only `group2.subgroup2.key1`.                            |
| Tag Exclusion Override | `group1 = true`<br>`"#sensitive" = false`                         | Include all from `group1` except settings tagged `#sensitive`.   |

### 6.3 Supported File Formats and Code Generators

Export formats are inferred from the destination file extension:

| Extension                          | Format            | Description                                                                               |
| ---------------------------------- | ----------------- | ----------------------------------------------------------------------------------------- |
| `.toml`                            | TOML 1.0          | Standard TOML format. Keys with `null` values are omitted.                                |
| `.json`                            | JSON              | Standard JSON. Keys with `null` values are serialized as `null`.                          |
| `.yaml`, `.yml`                    | YAML              | Standard YAML. Keys with `null` values are serialized as `null` or `~`.                   |
| `.py`                              | Python Module     | Python source file defining nested classes or top-level variables. `null` maps to `None`. |
| `.js`, `.mjs`                      | JavaScript Module | ECMAScript module (`export default { ... }`). `null` maps to `null`.                      |
| `.ts`                              | TypeScript Module | TypeScript module with typed interfaces and `export default { ... }`.                     |
| `.lua`                             | Lua Table         | Lua module returning a table (`return { ... }`). `null` maps to `nil`.                    |
| `.env`, `.env.*`, `env.*`, `*.env` | Shell Environment | Key-value pairs (`KEY=VALUE`). Keys with `null` values are omitted.                       |

---

## 7. Command-Line Interface (CLI) Specification

SettingSpec provides a unified command-line tool named `settingspec`.

### 7.1 Common Options

All subcommands accept the following common options:

- `-h, --help`: Print help information.
- `-V, --version`: Print version information.

### 7.2 Subcommand: `export`

Generates all configured export files and/or writes formatted output to stdout.

```bash
settingspec export [OPTIONS]
```

### 7.3 Subcommand: `run`

Executes a command with resolved configuration available as exported files.

```bash
settingspec run [OPTIONS] -- <COMMAND> [ARGS...]
```

#### 7.3.1 Execution Lifecycle

```mermaid
sequenceDiagram
    autonumber
    participant CLI as settingspec run
    participant Disk as File System
    participant Child as Target Program

    CLI->>Disk: Write to stdout and/or all target export files with mode spec.export.mode (0x600)
    CLI->>Child: Spawn child process
    Child-->>CLI: Child process completes (exit code or signal)

    alt If spec.export.keep is false (default)
        CLI->>Disk: Delete exported files listed in spec.export.file
    else If spec.export.keep is true
        CLI->>Disk: Retain exported files on disk
    end

    CLI->>CLI: Exit with child process exit code
```

1. **File Generation:** SettingSpec resolves settings for the active profile and writes all target export files declared in `spec.export.file` using permission mode `spec.export.mode` (default `0x600`).
2. **Execution:** The child process `<COMMAND> [ARGS...]` is spawned. Sourced environment variables from `spec.envfile` are set (decrypting any age-encrypted envfile per Section 3.2.2 first), if `spec.export.env` is configured, resolved settings are exported as environment variables into the child process, and if `spec.export.stdin` is configured, resolved settings are passed to the child process via standard input (stdin). Signals (e.g., `SIGINT`, `SIGTERM`) MUST be forwarded to the child process.
3. **Cleanup:**
   - If `spec.export.keep` is `false` (default): All exported target files are deleted upon completion.
   - If `spec.export.keep` is `true`: Exported target files are retained on disk.
4. **Exit Code Propagation:** SettingSpec MUST exit with the same exit status code as the child process.

### 7.4 Subcommand: `check`

Validates configuration syntax, profile completeness, and environment variable requirements without writing any files to disk.

```bash
settingspec check [OPTIONS]
```

- Returns exit code `0` on validation success.
- Returns non-zero exit code with diagnostic errors if syntax is invalid, profiles are incomplete, or required environment variables are missing. Validation MUST also verify that any age-encrypted `envfile` entries have a working decryption identity for the active profile per Section 3.2.3.

### 7.5 Subcommand: `init`

Initializes a starter `settingspec.toml` in the current directory if one does not already exist.

```bash
settingspec init [OPTIONS]
```

- If `settingspec.toml` already exists in the current directory, execution skips without overwriting or error.
- Initializes with default profile options (`local`, `prod`), default profile (`local`), and sample settings.

---

## 8. Security Considerations

1. **File Permission Enforcement:** All generated export files containing configuration or secrets MUST be created with restricted file permissions (`spec.export.mode`, default `0x600`). On POSIX systems, this prevents reading by other unprivileged users on the same host.
2. **Ephemeral Secrets:** Applications that utilize `settingspec run` benefit from ephemeral file lifetimes. By default (`spec.export.keep = false`), SettingSpec guarantees cleanup of exported files upon process exit, even in the event of child errors or termination signals.
3. **Source Control Hygiene:** Configuration authors SHOULD add generated targets (e.g., `settings.toml`, `settings.json` etc.) to `.gitignore` to prevent inadvertent commits of decrypted secrets. When inside a git repository, SettingSpec automatically appends exported target files to `.gitignore` (adjacent to the git root) before each `run` or `export` execution if not already present, unless disabled via `spec.export.skip_gitignore = true`.
4. **Standard Input for Secrets:** When pairing with secret managers (e.g., SecretSpec, 1Password CLI, Vault), users SHOULD pass secrets via environment variables or stdin (`envfile.prod = "-"`) to avoid writing raw secrets to persistent disk.
5. **Age-Encrypted Environment Files:** Unlike plaintext `envfile` sources, age-encrypted `envfile` sources (Section 3.2.2) MAY be safely committed to source control, since their contents are unreadable without a resolvable decryption identity. Configuration authors SHOULD NOT rely on this to relax the `.gitignore` guidance in Rule 3 for any _decrypted output_ — only the encrypted source file itself is safe to commit. Decryption identities supplied via `decryption.key.env` (including multi-key, colon-delimited values, Section 3.2.3) MUST NOT be committed to source control or logged in plaintext by SettingSpec or downstream tooling.
