//! SPIKE: bindgen that merges uniffi metadata from a *dependency* library with the
//! metadata of the library being generated.
//!
//! Usage: spike-bindgen <plugin-lib> <out-dir> <crate-filter> [dep-lib ...]
//!
//! The stock CLI reads metadata from one library only, so external types that live in a
//! separately-compiled dylib cannot be resolved. Merging the `MetadataGroupMap`s fixes it.

use anyhow::{Result, bail};
use camino::Utf8PathBuf;
use uniffi_bindgen::{BindgenLoader, BindgenPaths, bindings::python};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        bail!("usage: spike-bindgen <plugin-lib> <out-dir> <crate-filter> [dep-lib ...]");
    }
    let plugin_lib = Utf8PathBuf::from(&args[0]);
    let out_dir = Utf8PathBuf::from(&args[1]);
    let crate_filter = args[2].clone();
    let dep_libs: Vec<Utf8PathBuf> = args[3..].iter().map(Utf8PathBuf::from).collect();

    // Per-crate `uniffi.toml` lookup, so core and plugin each get their own config
    // (notably each namespace's `cdylib_name` and the plugin's `external_packages`).
    let mut paths = BindgenPaths::default();
    paths.add_cargo_metadata_layer(false)?;
    let loader = BindgenLoader::new(paths);

    // Dependency metadata first, then the plugin's own.
    let mut metadata = std::collections::HashMap::new();
    for dep in &dep_libs {
        let dep_meta = loader.load_metadata(dep)?;
        println!("  + {} namespace(s) from {}", dep_meta.len(), dep);
        metadata.extend(dep_meta);
    }
    let own = loader.load_metadata(&plugin_lib)?;
    println!("  + {} namespace(s) from {}", own.len(), plugin_lib);
    metadata.extend(own);

    println!(
        "  merged namespaces: {:?}",
        metadata
            .values()
            .map(|g| (g.namespace.crate_name.clone(), g.namespace.name.clone()))
            .collect::<Vec<_>>()
    );

    // `source_path` sets `root.cdylib`; pass the plugin so its namespace defaults to the
    // plugin library. Core's own uniffi.toml pins `cdylib_name = "iroh_ffi"` regardless.
    let root = loader.load_pipeline_initial_root(&plugin_lib, metadata)?;
    python::run_pipeline(root, &out_dir, Some(&crate_filter))?;
    Ok(())
}
