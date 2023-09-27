

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

// Task defined in Rust that Swift executes
typedef void (*RustTaskCallback)(const void *, int8_t);

// Callback to execute Rust tasks using a Go routine
//
// Args:
//   executor: ForeignExecutor lowered into a uint64_t value
//   delay: Delay in MS
//   task: RustTaskCallback to call
//   task_data: data to pass the task callback
typedef int8_t (*ForeignExecutorCallback)(uint64_t, uint32_t, RustTaskCallback, const void *);


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

void uniffi_iroh_fn_free_authorid(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_authorid_to_string(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_free_doc(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_doc_get_content_bytes(
	void* ptr,
	void* entry,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_doc_id(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_doc_keys(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_doc_set_bytes(
	void* ptr,
	void* author,
	RustBuffer key,
	RustBuffer value,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_doc_share_read(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_method_doc_share_write(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_doc_status(
	void* ptr,
	RustCallStatus* out_status
);

void uniffi_iroh_fn_method_doc_stop_sync(
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

void* uniffi_iroh_fn_method_entry_hash(
	void* ptr,
	RustCallStatus* out_status
);

RustBuffer uniffi_iroh_fn_method_entry_key(
	void* ptr,
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

void uniffi_iroh_fn_free_irohnode(
	void* ptr,
	RustCallStatus* out_status
);

void* uniffi_iroh_fn_constructor_irohnode_new(
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

void uniffi_iroh_fn_free_publickey(
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

uint16_t uniffi_iroh_checksum_method_doc_get_content_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_id(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_keys(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_set_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_share_read(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_share_write(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_status(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_doc_stop_sync(
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

uint16_t uniffi_iroh_checksum_method_entry_hash(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_entry_key(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_hash_to_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_hash_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_author_list(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_author_new(
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

uint16_t uniffi_iroh_checksum_method_irohnode_doc_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_node_id(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_irohnode_stats(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_publickey_to_bytes(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_publickey_to_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_docticket_from_string(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_constructor_irohnode_new(
	RustCallStatus* out_status
);

uint16_t uniffi_iroh_checksum_method_subscribecallback_event(
	RustCallStatus* out_status
);

uint32_t ffi_iroh_uniffi_contract_version(
	RustCallStatus* out_status
);


int32_t iroh_cgo_SubscribeCallback(uint64_t, int32_t, uint8_t *, int32_t, RustBuffer *);