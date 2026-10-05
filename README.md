```
▀█▀  ▄▄▄ ▄▄▄  ▀ ▄█▄  ▀  ▄▄  ▄▄▄  
 █  █  █ █  █ █  █   █ █  █ █  █ 
▀▀▀  ▀▀█ ▀  ▀ ▀   ▀▀ ▀  ▀▀  ▀  ▀ 
     ▀▀                 v1.0.0 💥       
```

Build-time retrieval of precompiled dependencies.

Ignition is a build dependency that, during its own compilation, downloads target-specific
asset archives from a remote bucket, extracts them, and exports their paths as cargo metadata.

Dependent build scripts then read that metadata and set the environment variables their own
`-sys` crates expect (`OPENCV_LINK_PATHS`, `ORT_LIB_LOCATION`, and so on).

## How it works

```
IGNITION_BUCKET_URL -> ignition/build.rs -> download/extract -> cargo::metadata
                                                                      |
                                                DEP_IGNITION_SYS_* ---|
                                                                      |
                                                      [dependent build.rs sets env vars]
```

Assets are declared once in [`config/registry.yaml`](config/registry.yaml). The
`register_assets!` macro reads that file at compile time and generates a type per entry.

## Usage

Add Ignition as a **build dependency** of the crate that needs the assets:

```toml
[build-dependencies]
ignition = { git = "https://github.com/diego-a-orellana/ignition", version = "1.0" }
```

Then, in that crate's `build.rs`, set the environment variables for the assets you use:

```rust
use ignition::{OpenCv, environment_variables};

fn main() {
    // reads DEP_IGNITION_SYS_* metadata and sets OPENCV_LINK_PATHS, OPENCV_INCLUDE_PATHS
    environment_variables(&OpenCv::new(), None).expect("failed to set OpenCV environment");
}
```

Passing `None` retrieves metadata and sets the variables. Ignition itself passes `Some(&config)`
during its own build, which is what exports them in the first place.

NOTE: Cargo only passes `links` metadata to the build scripts of **immediate** dependents, so each
crate that needs these variables must depend on Ignition directly.

## Configuration

`IGNITION_BUCKET_URL` is required; the rest have defaults.

| Variable | Default | Purpose |
| --- | --- | --- |
| `IGNITION_BUCKET_URL` | — | Root url of the asset bucket |
| `IGNITION_ASSET_DIR` | `assets/dependencies` | Asset path within the bucket and the build directory |
| `IGNITION_CACHE_DIR` | `cache` | Archive cache, relative to the build directory |

NOTE: `OUT_DIR` and `TARGET` are supplied by cargo. The build directory is derived from `OUT_DIR`,
and `TARGET` selects which build of an asset to fetch.

### Features

Retrieval is per asset, so a consumer only downloads what it uses:

```toml
ignition = { version = "1.0", default-features = false, features = ["download-opencv"] }
```

Both `download-opencv` and `download-onnxruntime` are enabled by default and each implies the
internal `download` feature, which pulls in the HTTP and archive dependencies.

## Layout

Remote archives are addressed by asset key and full Rust target triplet:

```
<bucket-url>/<asset-dir>/<asset>/<target-triplet>/<asset>.tar.gz
```

The triplet is used verbatim as a single folder name, e.g. `aarch64-unknown-linux-gnu`.

Locally, archives are cached and extracted under the build directory:

```
<build-dir>/<cache-dir>/<asset-dir>/<asset>/<target-triplet>/<asset>.tar.gz   # archive
<build-dir>/<asset-dir>/<asset>/                                              # extracted
```

A cached archive is only ever a complete one: downloads are written to a `.part` file, checked
against `Content-Length`, and renamed into place on success.

## Adding an asset

1. Upload `<asset>.tar.gz` to the bucket for every target you support.
2. Add an entry to [`config/registry.yaml`](config/registry.yaml):

   ```yaml
   my-crate:            # identifier of the consuming crate
     name: MyAsset      # generated Rust type
     key: myasset       # bucket folder, archive name and metadata prefix
     contents:          # paths expected after extraction, relative to <asset-dir>
       - myasset/lib
     environment:       # environment variable to content path
       - MYASSET_LIB_DIR: myasset/lib
   ```

3. Add the matching feature to `Cargo.toml`:

   ```toml
   download-myasset = ["download"]
   ```

NOTE: the feature name must be `download-<key>`.

## Exported metadata

Each asset exports one variable per `environment` entry, plus the archive path for consumers
that want to verify a download. Cargo prefixes every key with `DEP_IGNITION_SYS_`:

| Metadata key | Read by dependents as |
| --- | --- |
| `OPENCV_LINK_PATHS` | `DEP_IGNITION_SYS_OPENCV_LINK_PATHS` |
| `OPENCV_ASSET_PATH` | `DEP_IGNITION_SYS_OPENCV_ASSET_PATH` |

Missing contents are reported as `cargo::error`, failing the build with the path that was
expected.

## Development

```
make            # format, clippy and test
make format     # cargo +nightly fmt
make clippy
make test
```

The workspace is this crate plus [`macros/`](macros), which holds the `register_assets!`
procedural macro.

## License

MIT. See [LICENSE](LICENSE).
