"""SPIKE: the composition case.

Four independently built native libraries in a diamond — `iroh_docs` consumes objects
created by `iroh` (Endpoint), `iroh_blobs` (Store) and `iroh_gossip` (Gossip), all of which
must be the same monomorphizations, with one copy of iroh in the process.
"""

import asyncio

import iroh
import iroh_blobs
import iroh_docs
import iroh_gossip


async def main() -> None:
    for mod in (iroh.iroh_ffi, iroh_blobs.iroh_ffi_blobs,
                iroh_gossip.iroh_ffi_gossip, iroh_docs.iroh_ffi_docs):
        mod.uniffi_set_event_loop(asyncio.get_running_loop())

    endpoint = await iroh.Endpoint.bind(iroh.EndpointOptions(preset=iroh.preset_minimal()))
    print(f"  endpoint bound: {endpoint.addr()}")

    # From the blobs package — and prove it actually works standalone.
    store = await iroh_blobs.Store.memory()
    digest = await store.add_bytes(b"composition works")
    echoed = await store.get_bytes(digest)
    print(f"  blobs: add_bytes -> {digest[:16]}... | get_bytes -> {echoed!r}")
    await store.serve(endpoint)

    # From the gossip package.
    gossip = await iroh_gossip.Gossip.spawn(endpoint)
    print(f"  gossip: spawned, max_message_size = {gossip.max_message_size()}")

    # THE COMPOSITION: docs takes an Endpoint, a Store and a Gossip, each minted by a
    # different native library, and spawns a real iroh-docs engine over them.
    docs = await iroh_docs.Docs.memory(endpoint, store, gossip)
    author = await docs.author_create()
    doc_id = await docs.doc_create()
    print(f"  docs: author = {author[:16]}...")
    print(f"  docs: doc    = {doc_id[:16]}...")

    await endpoint.close()
    print("  OK: docs composed Endpoint + Store + Gossip across four native libraries")


if __name__ == "__main__":
    asyncio.run(main())
