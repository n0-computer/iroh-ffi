const os = require('os');
const fs = require('fs');
const path = require('path');
const { Hash, IrohNode, SetTagOption, BlobFormat, WrapOption, AddProgressType } = require('../index');

test('hash', () => {
    const hashStr = "2kbxxbofqx5rau77wzafrj4yntjb4gn4olfpwxmv26js6dvhgjhq";
    const hexStr = "d2837b85c585fb1053ffb64058a7986cd21e19bc72cafb5d95d7932f0ea7324f";
    const bytes = [0xd2, 0x83, 0x7b, 0x85, 0xc5, 0x85, 0xfb, 0x10, 0x53, 0xff, 0xb6, 0x40, 0x58, 0xa7, 0x98, 0x6c, 0xd2, 0x1e, 0x19, 0xbc, 0x72, 0xca, 0xfb, 0x5d, 0x95, 0xd7, 0x93, 0x2f, 0x0e, 0xa7, 0x32, 0x4f];

    const hash = Hash.fromString(hashStr);

    expect(hash.toString()).toBe(hashStr);
    expect(hash.toBytes()).toEqual(bytes);
    expect(hash.toHex()).toBe(hexStr);

    const hash0 = Hash.fromBytes(bytes);

    expect(hash0.toString()).toBe(hashStr);
    expect(hash0.toBytes()).toEqual(bytes);
    expect(hash0.toHex()).toBe(hexStr);

    expect(hash.equal(hash0)).toBe(true);
    expect(hash0.equal(hash)).toBe(true);
});

test('blob add and get bytes', async () => {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'add_and_get_bytes'));
    const node = await IrohNode.withPath(dir);

    const blobSize = 100;
    const bytes = Buffer.alloc(blobSize).map(() => Math.floor(Math.random() * 256));

    const addOutcome = await node.blobsAddBytes(bytes);

    expect(addOutcome.format).toBe(BlobFormat.Raw);
    expect(addOutcome.size).toBe(blobSize);

    const hash = Hash.fromString(addOutcome.hash);
    const gotSize = await node.blobsSize(hash);
    expect(gotSize).toBe(blobSize);

    const gotBytes = await node.blobsReadToBytes(hash);
    expect(gotBytes.length).toBe(blobSize);
    expect(gotBytes).toEqual(bytes);
});

// test('blob read and write path', () => {
//     const irohDir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
//     const node = new IrohNode(irohDir);

//     const blobSize = 100;
//     const bytes = Buffer.alloc(blobSize).map(() => Math.floor(Math.random() * 256));

//     const dir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
//     const filePath = path.join(dir, "in");
//     fs.writeFileSync(filePath, bytes);

//     const tag = SetTagOption.auto();
//     const wrap = WrapOption.noWrap();

//     class AddCallback {
//         constructor() {
//             this.hash = null;
//             this.format = null;
//         }

//         progress(progressEvent) {
//             if (progressEvent.type() === AddProgressType.ALL_DONE) {
//                 const allDoneEvent = progressEvent.asAllDone();
//                 this.hash = allDoneEvent.hash;
//                 this.format = allDoneEvent.format;
//             }
//             if (progressEvent.type() === AddProgressType.ABORT) {
//                 const abortEvent = progressEvent.asAbort();
//                 throw new Error(abortEvent.error);
//             }
//         }
//     }

//     const cb = new AddCallback();
//     node.blobsAddFromPath(filePath, false, tag, wrap, cb);

//     expect(cb.format).toBe(BlobFormat.RAW);
//     expect(cb.hash).not.toBeNull();

//     const gotSize = node.blobsSize(cb.hash);
//     expect(gotSize).toBe(blobSize);

//     const gotBytes = node.blobsReadToBytes(cb.hash);
//     expect(gotBytes.length).toBe(blobSize);
//     expect(gotBytes).toEqual(bytes);

//     const outPath = path.join(dir, "out");
//     node.blobsWriteToPath(cb.hash, outPath);

//     const gotBytesFromFile = fs.readFileSync(outPath);
//     expect(gotBytesFromFile.length).toBe(blobSize);
//     expect(gotBytesFromFile).toEqual(bytes);
// });

// test('blob collections', () => {
//     const collectionDir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
//     const numFiles = 3;
//     const blobSize = 100;
//     const blobBytes = Array.from({ length: blobSize }, () => Math.floor(Math.random() * 256));

//     for (let i = 0; i < numFiles; i++) {
//         const filePath = path.join(collectionDir, `${i}`);
//         fs.writeFileSync(filePath, Buffer.from(blobBytes));
//     }

//     const irohDir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
//     const node = new IrohNode(irohDir);

//     const class AddCallback {
//         constructor() {
//             this.collectionHash = null;
//             this.format = null;
//             this.blobHashes = [];
//         }

//         progress(progressEvent) {
//             if (progressEvent.type() === AddProgressType.ALL_DONE) {
//                 const allDoneEvent = progressEvent.asAllDone();
//                 this.collectionHash = allDoneEvent.hash;
//                 this.format = allDoneEvent.format;
//             }
//             if (progressEvent.type() === AddProgressType.ABORT) {
//                 const abortEvent = progressEvent.asAbort();
//                 throw new Error(abortEvent.error);
//             }
//             if (progressEvent.type() === AddProgressType.DONE) {
//                 const doneEvent = progressEvent.asDone();
//                 this.blobHashes.push(doneEvent.hash);
//             }
//         }
//     }

//     const cb = new AddCallback();
//     const tag = SetTagOption.auto();
//     const wrap = WrapOption.noWrap();
//     node.blobsAddFromPath(collectionDir, false, tag, wrap, cb);

//     expect(cb.collectionHash).not.toBeNull();
//     expect(cb.format).toBe(BlobFormat.HASH_SEQ);

//     const collections = node.blobsListCollections();
//     expect(collections.length).toBe(1);
//     expect(collections[0].hash.equal(cb.collectionHash)).toBe(true);
//     expect(collections[0].totalBlobsCount).toBe(numFiles);

//     const collectionHashes = [...cb.blobHashes, cb.collectionHash];
//     const gotHashes = node.blobsList();
//     for (const hash of gotHashes) {
//         const blobBytes = node.blobsReadToBytes(hash);
//         expect(blobBytes.length).toBe(blobSize);
//     }
//     expect(collectionHashes.length + 1).toBe(gotHashes.length);

//     for (const expectHash of collectionHashes) {
//         let found = false;
//         for (const gotHash of gotHashes) {
//             if (expectHash.equal(gotHash)) {
//                 found = true;
//                 break;
//             }
//         }
//         if (!found) {
//             throw new Error(`Could not find ${expectHash} in list`);
//         }
//     }
// });

// test('list and delete', () => {
//     const irohDir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
//     const node = new IrohNode(irohDir);

//     const blobSize = 100;
//     const numBlobs = 3;
//     const blobs = Array.from({ length: numBlobs }, () => {
//         return Buffer.alloc(blobSize).map(() => Math.floor(Math.random() * 256));
//     });

//     const hashes = [];
//     for (const blob of blobs) {
//         const output = node.blobsAddBytes(blob, SetTagOption.auto());
//         hashes.push(output.hash);
//     }

//     let gotHashes = node.blobsList();
//     expect(gotHashes.length).toBe(numBlobs);
//     for (const expectHash of hashes) {
//         let found = false;
//         for (const gotHash of gotHashes) {
//             if (expectHash.equal(gotHash)) {
//                 found = true;
//                 break;
//             }
//         }
//         if (!found) {
//             throw new Error(`Could not find ${expectHash} in list`);
//         }
//     }

//     const removeHash = hashes.shift();
//     node.blobsDeleteBlob(removeHash);

//     gotHashes = node.blobsList();
//     expect(gotHashes.length).toBe(numBlobs - 1);
//     for (const gotHash of gotHashes) {
//         if (removeHash.equal(gotHash)) {
//             throw new Error(`Blob ${removeHash} should have been removed`);
//         }
//     }
// });

