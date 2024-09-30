use iroh::client::blobs::batch::Batch as IrohBatch;

// A batch for write operations
///
/// This serves mostly as a scope for temporary tags.
///
/// It is not a transaction, so things in a batch are not atomic. Also,there is
/// no isolation between batches.
#[derive(uniffi::Object)]
pub struct Batch {
    batch: IrohBatch,
}
