//! The single shipped cdylib. Contains no API of its own — it just pulls both
//! uniffi components into one native library.
//!
//! `uniffi_reexport_scaffolding!` is uniffi's supported way to do this: it
//! forces the dependency's `#[no_mangle]` scaffolding symbols to be re-exported
//! from this cdylib instead of being dropped by the linker.

core_lib::uniffi_reexport_scaffolding!();
ext_lib::uniffi_reexport_scaffolding!();
