

// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!



#include <stdbool.h>
#include <stdint.h>

// The following structs are used to implement the lowest level
// of the FFI, and thus useful to multiple uniffied crates.
// We ensure they are declared exactly once, with a header guard, UNIFFI_SHARED_H.
#ifdef UNIFFI_SHARED_H
	// We also try to prevent mixing versions of shared uniffi header structs.
	// If you add anything to the #else block, you must increment the version suffix in UNIFFI_SHARED_HEADER_V5
	#ifndef UNIFFI_SHARED_HEADER_V5
		#error Combining helper code from multiple versions of uniffi is not supported
	#endif // ndef UNIFFI_SHARED_HEADER_V5
#else
#define UNIFFI_SHARED_H
#define UNIFFI_SHARED_HEADER_V5
// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V5 in this file.           ⚠️

typedef struct RustBuffer {
	int32_t capacity;
	int32_t len;
	uint8_t *data;
} RustBuffer;

typedef int32_t (*ForeignCallback)(uint64_t, int32_t, uint8_t *, int32_t, RustBuffer *);

// Task defined in Rust that Go executes
typedef void (*RustTaskCallback)(const void *, int8_t);

// Callback to execute Rust tasks using a Go routine
//
// Args:
//   executor: ForeignExecutor lowered into a uint64_t value
//   delay: Delay in MS
//   task: RustTaskCallback to call
//   task_data: data to pass the task callback
typedef int8_t (*ForeignExecutorCallback)(uint64_t, uint32_t, RustTaskCallback, void *);

typedef struct ForeignBytes {
	int32_t len;
	const uint8_t *data;
} ForeignBytes;

// Error definitions
typedef struct RustCallStatus {
	int8_t code;
	RustBuffer errorBuf;
} RustCallStatus;

// ⚠️ Attention: If you change this #else block (ending in `#endif // def UNIFFI_SHARED_H`) you *must* ⚠️
// ⚠️ increment the version suffix in all instances of UNIFFI_SHARED_HEADER_V5 in this file.           ⚠️
#endif // def UNIFFI_SHARED_H

// Needed because we can't execute the callback directly from go.
void cgo_rust_task_callback_bridge_iroh(RustTaskCallback, const void *, int8_t);

int8_t uniffiForeignExecutorCallbackiroh(uint64_t, uint32_t, RustTaskCallback, void*);

// Callbacks for UniFFI Futures
typedef void (*UniFfiFutureCallbackuint8_t)(const void *, uint8_t, RustCallStatus);
typedef void (*UniFfiFutureCallbackint8_t)(const void *, int8_t, RustCallStatus);
typedef void (*UniFfiFutureCallbackuint16_t)(const void *, uint16_t, RustCallStatus);
typedef void (*UniFfiFutureCallbackuint64_t)(const void *, uint64_t, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustArcPtr)(const void *, void*, RustCallStatus);
typedef void (*UniFfiFutureCallbackRustBuffer)(const void *, RustBuffer, RustCallStatus);


void uniffi_iroh_fn_free_authorid(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_authorid_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_directaddrinfo(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_doc(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_method_doc_close(
	void* ptr,
	RustCallStatus* out_status
);

uint64_t uniffi_iroh_fn_method_doc_del(
	void* ptr,
	void* author_id,
	RustBuffer prefix,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_doc_get_many(
	void* ptr,
	void* filter,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_doc_get_one(
	void* ptr,
	void* author_id,
	RustBuffer key,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_doc_id(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_method_doc_leave(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_doc_read_to_bytes(
	void* ptr,
	void* entry,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_doc_set_bytes(
	void* ptr,
	void* author,
	RustBuffer key,
	RustBuffer value,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_method_doc_set_hash(
	void* ptr,
	void* author,
	RustBuffer key,
	void* hash,
	uint64_t size,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_doc_share(
	void* ptr,
	RustBuffer mode,
	RustCallStatus* out_status
);

uint64_t uniffi_iroh_fn_method_doc_size(
	void* ptr,
	void* entry,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_method_doc_start_sync(
	void* ptr,
	RustBuffer peers,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_doc_status(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_method_doc_subscribe(
	void* ptr,
	uint64_t cb,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_docticket(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_docticket_from_string(
	RustBuffer content,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_docticket_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_entry(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_entry_author(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_entry_key(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_entry_namespace(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_getfilter(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_getfilter_all(
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_getfilter_author(
	void* author,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_getfilter_author_prefix(
	void* author,
	RustBuffer prefix,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_getfilter_key(
	RustBuffer key,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_getfilter_prefix(
	RustBuffer prefix,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_hash(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_hash_to_bytes(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_hash_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_ipv4addr(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_ipv4addr_from_string(
	RustBuffer str,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_ipv4addr_new(
	uint8_t a,
	uint8_t b,
	uint8_t c,
	uint8_t d,
	RustCallStatus* out_status
);

int8_t uniffi_iroh_fn_method_ipv4addr_equal(
	void* ptr,
	void* other,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_ipv4addr_octets(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_ipv4addr_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_ipv6addr(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_ipv6addr_from_string(
	RustBuffer str,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_ipv6addr_new(
	uint16_t a,
	uint16_t b,
	uint16_t c,
	uint16_t d,
	uint16_t e,
	uint16_t f,
	uint16_t g,
	uint16_t h,
	RustCallStatus* out_status
);

int8_t uniffi_iroh_fn_method_ipv6addr_equal(
	void* ptr,
	void* other,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_ipv6addr_segments(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_ipv6addr_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_irohnode(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_irohnode_new(
	RustBuffer path,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_irohnode_author_list(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_irohnode_author_new(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_irohnode_blob_get(
	void* ptr,
	void* hash,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_irohnode_blob_list_blobs(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_irohnode_connection_info(
	void* ptr,
	void* node_id,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_irohnode_connections(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_irohnode_doc_join(
	void* ptr,
	void* ticket,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_irohnode_doc_list(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_irohnode_doc_new(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_irohnode_node_id(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_irohnode_stats(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_liveevent(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_liveevent_as_content_ready(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_liveevent_as_insert_local(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_liveevent_as_insert_remote(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_liveevent_as_neighbor_down(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_liveevent_as_neighbor_up(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_liveevent_as_sync_finished(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_liveevent_type(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_namespaceid(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_namespaceid_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_peeraddr(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_peeraddr_new(
	void* node_id,
	RustBuffer region_id,
	RustBuffer addresses,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_peeraddr_derp_region(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_peeraddr_direct_addresses(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_publickey(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_publickey_from_bytes(
	RustBuffer bytes,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_publickey_from_string(
	RustBuffer s,
	RustCallStatus* out_status
);

int8_t uniffi_iroh_fn_method_publickey_equal(
	void* ptr,
	void* other,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_publickey_fmt_short(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_publickey_to_bytes(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_publickey_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_socketaddr(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_socketaddr_from_ipv4(
	void* ipv4,
	uint16_t port,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_socketaddr_from_ipv6(
	void* ipv6,
	uint16_t port,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_socketaddr_as_ipv4(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_socketaddr_as_ipv6(
	void* ptr,
	RustCallStatus* out_status
);

int8_t uniffi_iroh_fn_method_socketaddr_equal(
	void* ptr,
	void* other,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_socketaddr_type(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_socketaddrv4(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_socketaddrv4_from_string(
	RustBuffer str,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_socketaddrv4_new(
	void* ipv4,
	uint16_t port,
	RustCallStatus* out_status
);

int8_t uniffi_iroh_fn_method_socketaddrv4_equal(
	void* ptr,
	void* other,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_socketaddrv4_ip(
	void* ptr,
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_fn_method_socketaddrv4_port(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_socketaddrv4_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_socketaddrv6(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_socketaddrv6_from_string(
	RustBuffer str,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_socketaddrv6_new(
	void* ipv6,
	uint16_t port,
	RustCallStatus* out_status
);

int8_t uniffi_iroh_fn_method_socketaddrv6_equal(
	void* ptr,
	void* other,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_socketaddrv6_ip(
	void* ptr,
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_fn_method_socketaddrv6_port(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_socketaddrv6_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_init_callback_subscribecallback(
	ForeignCallback callback_stub,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_func_set_log_level(
	RustBuffer level,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_func_start_metrics_collection(
	RustCallStatus* out_status
);

RustBuffer ffi_iroh_rustbuffer_alloc(
	int32_t size,
	RustCallStatus* out_status
);

RustBuffer ffi_iroh_rustbuffer_from_bytes(
	ForeignBytes bytes,
	RustCallStatus* out_status
);

void ffi_iroh_rustbuffer_free(
	RustBuffer buf,
	RustCallStatus* out_status
);

RustBuffer ffi_iroh_rustbuffer_reserve(
	RustBuffer buf,
	int32_t additional,
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_func_set_log_level(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_func_start_metrics_collection(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_authorid_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_close(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_del(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_get_many(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_get_one(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_id(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_leave(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_read_to_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_set_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_set_hash(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_share(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_size(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_start_sync(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_status(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_subscribe(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_docticket_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_entry_author(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_entry_key(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_entry_namespace(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_hash_to_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_hash_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_ipv4addr_equal(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_ipv4addr_octets(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_ipv4addr_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_ipv6addr_equal(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_ipv6addr_segments(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_ipv6addr_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_author_list(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_author_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_blob_get(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_blob_list_blobs(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_connection_info(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_connections(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_doc_join(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_doc_list(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_doc_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_node_id(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_stats(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_liveevent_as_content_ready(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_liveevent_as_insert_local(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_liveevent_as_insert_remote(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_liveevent_as_neighbor_down(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_liveevent_as_neighbor_up(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_liveevent_as_sync_finished(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_liveevent_type(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_namespaceid_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_peeraddr_derp_region(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_peeraddr_direct_addresses(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_publickey_equal(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_publickey_fmt_short(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_publickey_to_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_publickey_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddr_as_ipv4(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddr_as_ipv6(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddr_equal(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddr_type(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddrv4_equal(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddrv4_ip(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddrv4_port(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddrv4_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddrv6_equal(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddrv6_ip(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddrv6_port(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_socketaddrv6_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_docticket_from_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_getfilter_all(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_getfilter_author(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_getfilter_author_prefix(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_getfilter_key(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_getfilter_prefix(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_ipv4addr_from_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_ipv4addr_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_ipv6addr_from_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_ipv6addr_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_irohnode_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_peeraddr_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_publickey_from_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_publickey_from_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_socketaddr_from_ipv4(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_socketaddr_from_ipv6(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_socketaddrv4_from_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_socketaddrv4_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_socketaddrv6_from_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_socketaddrv6_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_subscribecallback_event(
	RustCallStatus* out_status
);

uint32_t ffi_iroh_uniffi_contract_version(
	RustCallStatus* out_status
);


int32_t iroh_cgo_SubscribeCallback(uint64_t, int32_t, uint8_t *, int32_t, RustBuffer *);
void uniffiFutureCallbackHandlerVoid(void *, uint8_t, RustCallStatus);
void uniffiFutureCallbackHandlerVoidTypeIrohError(void *, uint8_t, RustCallStatus);
void uniffiFutureCallbackHandlerUint16(void *, uint16_t, RustCallStatus);
void uniffiFutureCallbackHandlerUint64TypeIrohError(void *, uint64_t, RustCallStatus);
void uniffiFutureCallbackHandlerBool(void *, int8_t, RustCallStatus);
void uniffiFutureCallbackHandlerString(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerBytes(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerBytesTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerAuthorId(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerAuthorIdTypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerDocTypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerDocTicketTypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerEntry(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerGetFilter(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerHash(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerHashTypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerIpv4Addr(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerIpv4AddrTypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerIpv6Addr(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerIpv6AddrTypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerIrohNodeTypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerNamespaceId(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerPeerAddr(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerPublicKey(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerPublicKeyTypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerSocketAddr(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerSocketAddrV4(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerSocketAddrV4TypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerSocketAddrV6(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerSocketAddrV6TypeIrohError(void *, void*, RustCallStatus);
void uniffiFutureCallbackHandlerTypeInsertRemoteEvent(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerTypeOpenStateTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerTypeSyncEvent(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerTypeLiveEventType(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerTypeSocketAddrType(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerOptionalUint16(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerOptionalEntryTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerOptionalTypeConnectionInfoTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerSequenceUint8(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerSequenceUint16(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerSequenceAuthorIdTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerSequenceEntryTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerSequenceHashTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerSequenceNamespaceIdTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerSequenceSocketAddr(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerSequenceTypeConnectionInfoTypeIrohError(void *, RustBuffer, RustCallStatus);
void uniffiFutureCallbackHandlerMapStringTypeCounterStatsTypeIrohError(void *, RustBuffer, RustCallStatus);