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
typedef void (*UniFfiFutureCallbackInt8)(const void * _Nonnull, int8_t, RustCallStatus);
typedef void (*UniFfiFutureCallbackUInt16)(const void * _Nonnull, uint16_t, RustCallStatus);
typedef void (*UniFfiFutureCallbackUInt64)(const void * _Nonnull, uint64_t, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
typedef void (*UniFfiFutureCallbackUnsafeMutableRawPointer)(const void * _Nonnull, void*_Nonnull, RustCallStatus);
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
void*_Nonnull uniffi_iroh_fn_constructor_authorid_from_string(RustBuffer str, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_authorid_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_authorid_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_directaddrinfo(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_doc(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_method_doc_close(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
uint64_t uniffi_iroh_fn_method_doc_del(void*_Nonnull ptr, void*_Nonnull author_id, RustBuffer prefix, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_doc_get_many(void*_Nonnull ptr, void*_Nonnull query, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_doc_get_one(void*_Nonnull ptr, void*_Nonnull query, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_doc_id(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_method_doc_leave(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_doc_read_to_bytes(void*_Nonnull ptr, void*_Nonnull entry, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_doc_set_bytes(void*_Nonnull ptr, void*_Nonnull author, RustBuffer key, RustBuffer value, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_method_doc_set_hash(void*_Nonnull ptr, void*_Nonnull author, RustBuffer key, void*_Nonnull hash, uint64_t size, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_doc_share(void*_Nonnull ptr, RustBuffer mode, RustCallStatus *_Nonnull out_status
);
uint64_t uniffi_iroh_fn_method_doc_size(void*_Nonnull ptr, void*_Nonnull entry, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_method_doc_start_sync(void*_Nonnull ptr, RustBuffer peers, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_doc_status(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_method_doc_subscribe(void*_Nonnull ptr, uint64_t cb, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_docticket(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_docticket_from_string(RustBuffer content, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_docticket_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_docticket_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_entry(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_entry_author(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_entry_key(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_entry_namespace(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_hash(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_hash_from_bytes(RustBuffer bytes, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_hash_from_cid_bytes(RustBuffer bytes, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_hash_from_string(RustBuffer str, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_hash_new(RustBuffer buf, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_hash_as_cid_bytes(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_hash_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_hash_to_bytes(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_hash_to_hex(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_hash_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_ipv4addr(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_ipv4addr_from_string(RustBuffer str, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_ipv4addr_new(uint8_t a, uint8_t b, uint8_t c, uint8_t d, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_ipv4addr_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_ipv4addr_octets(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_ipv4addr_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_ipv6addr(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_ipv6addr_from_string(RustBuffer str, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_ipv6addr_new(uint16_t a, uint16_t b, uint16_t c, uint16_t d, uint16_t e, uint16_t f, uint16_t g, uint16_t h, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_ipv6addr_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_ipv6addr_segments(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_ipv6addr_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
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
RustBuffer uniffi_iroh_fn_method_irohnode_doc_list(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
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
void*_Nonnull uniffi_iroh_fn_constructor_namespaceid_from_string(RustBuffer str, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_namespaceid_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_namespaceid_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_nodeaddr(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_nodeaddr_new(void*_Nonnull node_id, RustBuffer region_id, RustBuffer addresses, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_nodeaddr_derp_region(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_nodeaddr_direct_addresses(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_nodeaddr_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_publickey(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_publickey_from_bytes(RustBuffer bytes, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_publickey_from_string(RustBuffer s, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_publickey_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_publickey_fmt_short(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_publickey_to_bytes(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_publickey_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_query(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_query_all(RustBuffer sort_by, RustBuffer direction, RustBuffer offset, RustBuffer limit, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_query_author(void*_Nonnull author, RustBuffer sort_by, RustBuffer direction, RustBuffer offset, RustBuffer limit, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_query_key_exact(RustBuffer key, RustBuffer sort_by, RustBuffer direction, RustBuffer offset, RustBuffer limit, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_query_key_prefix(RustBuffer prefix, RustBuffer sort_by, RustBuffer direction, RustBuffer offset, RustBuffer limit, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_query_single_latest_per_key(RustBuffer direction, RustBuffer offset, RustBuffer limit, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_query_limit(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
uint64_t uniffi_iroh_fn_method_query_offset(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_socketaddr(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_socketaddr_from_ipv4(void*_Nonnull ipv4, uint16_t port, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_socketaddr_from_ipv6(void*_Nonnull ipv6, uint16_t port, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_socketaddr_as_ipv4(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_socketaddr_as_ipv6(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_socketaddr_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_socketaddr_type(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_socketaddrv4(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_socketaddrv4_from_string(RustBuffer str, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_socketaddrv4_new(void*_Nonnull ipv4, uint16_t port, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_socketaddrv4_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_socketaddrv4_ip(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
uint16_t uniffi_iroh_fn_method_socketaddrv4_port(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_socketaddrv4_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_socketaddrv6(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_socketaddrv6_from_string(RustBuffer str, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_socketaddrv6_new(void*_Nonnull ipv6, uint16_t port, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_socketaddrv6_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_method_socketaddrv6_ip(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
uint16_t uniffi_iroh_fn_method_socketaddrv6_port(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_socketaddrv6_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void uniffi_iroh_fn_free_tag(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_tag_from_bytes(RustBuffer bytes, RustCallStatus *_Nonnull out_status
);
void*_Nonnull uniffi_iroh_fn_constructor_tag_from_string(RustBuffer s, RustCallStatus *_Nonnull out_status
);
int8_t uniffi_iroh_fn_method_tag_equal(void*_Nonnull ptr, void*_Nonnull other, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_tag_to_bytes(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
);
RustBuffer uniffi_iroh_fn_method_tag_to_string(void*_Nonnull ptr, RustCallStatus *_Nonnull out_status
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
uint16_t uniffi_iroh_checksum_method_authorid_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_authorid_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_close(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_del(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_get_many(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_get_one(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_id(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_leave(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_read_to_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_set_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_set_hash(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_share(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_size(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_start_sync(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_status(void
    
);
uint16_t uniffi_iroh_checksum_method_doc_subscribe(void
    
);
uint16_t uniffi_iroh_checksum_method_docticket_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_docticket_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_entry_author(void
    
);
uint16_t uniffi_iroh_checksum_method_entry_key(void
    
);
uint16_t uniffi_iroh_checksum_method_entry_namespace(void
    
);
uint16_t uniffi_iroh_checksum_method_hash_as_cid_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_hash_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_hash_to_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_hash_to_hex(void
    
);
uint16_t uniffi_iroh_checksum_method_hash_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_ipv4addr_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_ipv4addr_octets(void
    
);
uint16_t uniffi_iroh_checksum_method_ipv4addr_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_ipv6addr_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_ipv6addr_segments(void
    
);
uint16_t uniffi_iroh_checksum_method_ipv6addr_to_string(void
    
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
uint16_t uniffi_iroh_checksum_method_irohnode_doc_list(void
    
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
uint16_t uniffi_iroh_checksum_method_namespaceid_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_namespaceid_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_nodeaddr_derp_region(void
    
);
uint16_t uniffi_iroh_checksum_method_nodeaddr_direct_addresses(void
    
);
uint16_t uniffi_iroh_checksum_method_nodeaddr_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_publickey_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_publickey_fmt_short(void
    
);
uint16_t uniffi_iroh_checksum_method_publickey_to_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_publickey_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_query_limit(void
    
);
uint16_t uniffi_iroh_checksum_method_query_offset(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddr_as_ipv4(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddr_as_ipv6(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddr_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddr_type(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddrv4_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddrv4_ip(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddrv4_port(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddrv4_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddrv6_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddrv6_ip(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddrv6_port(void
    
);
uint16_t uniffi_iroh_checksum_method_socketaddrv6_to_string(void
    
);
uint16_t uniffi_iroh_checksum_method_tag_equal(void
    
);
uint16_t uniffi_iroh_checksum_method_tag_to_bytes(void
    
);
uint16_t uniffi_iroh_checksum_method_tag_to_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_authorid_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_docticket_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_hash_from_bytes(void
    
);
uint16_t uniffi_iroh_checksum_constructor_hash_from_cid_bytes(void
    
);
uint16_t uniffi_iroh_checksum_constructor_hash_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_hash_new(void
    
);
uint16_t uniffi_iroh_checksum_constructor_ipv4addr_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_ipv4addr_new(void
    
);
uint16_t uniffi_iroh_checksum_constructor_ipv6addr_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_ipv6addr_new(void
    
);
uint16_t uniffi_iroh_checksum_constructor_irohnode_new(void
    
);
uint16_t uniffi_iroh_checksum_constructor_namespaceid_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_nodeaddr_new(void
    
);
uint16_t uniffi_iroh_checksum_constructor_publickey_from_bytes(void
    
);
uint16_t uniffi_iroh_checksum_constructor_publickey_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_query_all(void
    
);
uint16_t uniffi_iroh_checksum_constructor_query_author(void
    
);
uint16_t uniffi_iroh_checksum_constructor_query_key_exact(void
    
);
uint16_t uniffi_iroh_checksum_constructor_query_key_prefix(void
    
);
uint16_t uniffi_iroh_checksum_constructor_query_single_latest_per_key(void
    
);
uint16_t uniffi_iroh_checksum_constructor_socketaddr_from_ipv4(void
    
);
uint16_t uniffi_iroh_checksum_constructor_socketaddr_from_ipv6(void
    
);
uint16_t uniffi_iroh_checksum_constructor_socketaddrv4_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_socketaddrv4_new(void
    
);
uint16_t uniffi_iroh_checksum_constructor_socketaddrv6_from_string(void
    
);
uint16_t uniffi_iroh_checksum_constructor_socketaddrv6_new(void
    
);
uint16_t uniffi_iroh_checksum_constructor_tag_from_bytes(void
    
);
uint16_t uniffi_iroh_checksum_constructor_tag_from_string(void
    
);
uint16_t uniffi_iroh_checksum_method_subscribecallback_event(void
    
);
uint32_t ffi_iroh_uniffi_contract_version(void
    
);

