// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// The following structs are used to implement the lowest level
// of the FFI, and thus useful to multiple uniffied crates.
// We ensure they are declared exactly once, with a header guard, UNIFFI_SHARED_H.
#ifdef UNIFFI_SHARED_H
    // We also try to prevent mixing versions of shared uniffi header structs.
    // If you add anything to the #else block, you must increment the version suffix in UNIFFI_SHARED_HEADER_V4
    #ifndef UNIFFI_SHARED_HEADER_V4
        #error Combining helper code from multiple versions of uniffi is not supported
    #endif // ndef UNIFFI_SHARED_HEADER_V4
#else
#define UNIFFI_SHARED_H
#define UNIFFI_SHARED_HEADER_V4
// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V4 in this file.           ⚠️

typedef struct RustBuffer
{
    int32_t capacity;
    int32_t len;
    uint8_t *_Nullable data;
} RustBuffer;

typedef int32_t (*ForeignCallback)(uint64_t, int32_t, const uint8_t *_Nonnull, int32_t, RustBuffer *_Nonnull);

// Task defined in Rust that Swift executes
typedef void (*UniFfiRustTaskCallback)(const void * _Nullable, int8_t);

// Callback to execute Rust tasks using a Swift Task
//
// Args:
//   executor: ForeignExecutor lowered into a size_t value
//   delay: Delay in MS
//   task: UniFfiRustTaskCallback to call
//   task_data: data to pass the task callback
typedef int8_t (*UniFfiForeignExecutorCallback)(size_t, uint32_t, UniFfiRustTaskCallback _Nullable, const void * _Nullable);

typedef struct ForeignBytes
{
    int32_t len;
    const uint8_t *_Nullable data;
} ForeignBytes;

// Error definitions
typedef struct RustCallStatus {
    int8_t code;
    RustBuffer errorBuf;
} RustCallStatus;

// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V4 in this file.           ⚠️
#endif // def UNIFFI_SHARED_H

// Callbacks for UniFFI Futures
typedef void (*UniFfiFutureCallbackUInt8)(const void * _Nonnull, uint8_t, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustBuffer)(const void * _Nonnull, RustBuffer, RustCallStatus);

// Scaffolding functions
void uniffi_iroh_fn_free_authorid(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_authorid_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_doc(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_doc_get_content_bytes(void*_Nonnull ptr, void*_Nonnull entry, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_doc_id(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_doc_keys(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_doc_set_bytes(void*_Nonnull ptr, void*_Nonnull author, RustBuffer key, RustBuffer value, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_doc_share_read(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_doc_share_write(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_doc_status(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_method_doc_stop_sync(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_method_doc_subscribe(void*_Nonnull ptr, uint64_t cb, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_docticket(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_docticket_from_string(RustBuffer content, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_docticket_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_entry(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_entry_author(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_entry_hash(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_entry_key(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_hash(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_hash_to_bytes(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_hash_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_irohnode(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_irohnode_new(RustBuffer path, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_irohnode_author_list(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_irohnode_author_new(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_irohnode_blob_get(void*_Nonnull ptr, void*_Nonnull hash, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_irohnode_blob_list_blobs(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_irohnode_connection_info(void*_Nonnull ptr, void*_Nonnull node_id, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_irohnode_connections(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_irohnode_doc_join(void*_Nonnull ptr, void*_Nonnull ticket, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_irohnode_doc_new(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_irohnode_node_id(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_irohnode_stats(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_liveevent(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_liveevent_as_content_ready(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_liveevent_as_insert_local(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_liveevent_as_insert_remote(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_liveevent_as_neighbor_down(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_liveevent_as_neighbor_up(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_liveevent_as_sync_finished(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_liveevent_type(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_namespaceid(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_namespaceid_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_publickey(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_publickey_to_bytes(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_publickey_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_init_callback_subscribecallback(ForeignCallback _Nonnull callback_stub, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_func_set_log_level(RustBuffer level, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_func_start_metrics_collection(RustCallStatus *_Nonnull out_status
    
);
RustBuffer ffi_iroh_rustbuffer_alloc(int32_t size, RustCallStatus *_Nonnull out_status
);
RustBuffer ffi_iroh_rustbuffer_from_bytes(ForeignBytes bytes, RustCallStatus *_Nonnull out_status
);
void ffi_iroh_rustbuffer_free(RustBuffer buf, RustCallStatus *_Nonnull out_status
);
RustBuffer ffi_iroh_rustbuffer_reserve(RustBuffer buf, int32_t additional, RustCallStatus *_Nonnull out_status
);
uint16_t uniffi_iroh_checksum_func_set_log_level(void
    
);
uint16_t uniffi_iroh_checksum_func_start_metrics_collection(void
    
);
uint16_t uniffi_iroh_checksum_method_authorid_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_get_content_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_id(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_keys(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_set_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_share_read(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_share_write(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_status(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_stop_sync(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_subscribe(void
    
);
uint16_t uniffi_iroh_checksum_method_docticket_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_entry_author(void
    
);
uint16_t uniffi_iroh_checksum_method_entry_hash(void
    
);
uint16_t uniffi_iroh_checksum_method_entry_key(void
    
);
uint16_t uniffi_iroh_checksum_method_hash_to_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_hash_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_author_list(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_author_new(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_blob_get(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_blob_list_blobs(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_connection_info(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_connections(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_doc_join(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_doc_new(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_node_id(void
    
);
uint16_t uniffi_iroh_checksum_method_irohnode_stats(void
    
);
uint16_t uniffi_iroh_checksum_method_liveevent_as_content_ready(void
    
);
uint16_t uniffi_iroh_checksum_method_liveevent_as_insert_local(void
    
);
uint16_t uniffi_iroh_checksum_method_liveevent_as_insert_remote(void
    
);
uint16_t uniffi_iroh_checksum_method_liveevent_as_neighbor_down(void
    
);
uint16_t uniffi_iroh_checksum_method_liveevent_as_neighbor_up(void
    
);
uint16_t uniffi_iroh_checksum_method_liveevent_as_sync_finished(void
    
);
uint16_t uniffi_iroh_checksum_method_liveevent_type(void
    
);
uint16_t uniffi_iroh_checksum_method_namespaceid_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_publickey_to_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_publickey_to_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_docticket_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_irohnode_new(void
    
);
uint16_t uniffi_iroh_checksum_method_subscribecallback_event(void
    
);
uint32_t ffi_iroh_uniffi_contract_version(void
    
);

