//! Android JNI initialization for `ndk_context`.
//!
//! iroh's DNS resolver reads `LinkProperties.getDnsServers()` through
//! `ndk_context`, which must be initialized with the process's JavaVM
//! and `Application` context before any `Endpoint` is constructed. Apps
//! call `IrohAndroid.installAndroidContext(applicationContext)` from
//! Kotlin once at process startup; that call lands here and stores the
//! pointers as a JNI global reference for the lifetime of the process.
//!
//! Only one initialization succeeds; subsequent calls are no-ops.

use std::sync::Once;

static INIT: Once = Once::new();

#[unsafe(no_mangle)]
pub extern "system" fn Java_computer_iroh_IrohAndroid_installAndroidContext(
    mut env: jni::JNIEnv,
    _class: jni::objects::JClass,
    context: jni::objects::JObject,
) {
    INIT.call_once(|| {
        let java_vm = match env.get_java_vm() {
            Ok(vm) => vm,
            Err(_) => return,
        };
        let global_ref = match env.new_global_ref(&context) {
            Ok(r) => r,
            Err(_) => return,
        };
        unsafe {
            ndk_context::initialize_android_context(
                java_vm.get_java_vm_pointer() as *mut std::ffi::c_void,
                global_ref.as_obj().as_raw() as *mut std::ffi::c_void,
            );
        }
        // Keep the global ref alive forever; ndk_context holds the raw
        // pointer and expects it to stay valid for the rest of the process.
        std::mem::forget(global_ref);
    });
}
