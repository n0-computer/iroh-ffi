const os = require('os');
const fs = require('fs');
const path = require('path');
const { Hash, IrohNode, BlobFormat, AddProgressType } = require('../index');

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

test('blob read and write path', async () => {
    const irohDir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
    const node = await IrohNode.withPath(irohDir);

    const blobSize = 100;
    const bytes = Buffer.alloc(blobSize).map(() => Math.floor(Math.random() * 256));

    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'read_and_write'));
    const filePath = path.join(dir, "in");
    fs.writeFileSync(filePath, bytes);

    const progress = await node.blobsAddFromPath(filePath, false, null, false);

    let hash = null;
    let format = null;
    for (const val of progress) {
        if (val && val.hasOwnProperty('AllDone')) {
            hash = Hash.fromString(val.AllDone.hash);
            format = val.AllDone.format
            break;
        }
        if (val && val.hasOwnProperty('Abort')) {
          throw new Error(val.Abort.error);
        }
    }

    expect(hash).not.toBeNull();
    expect(format).toBe(BlobFormat.Raw);

    const gotSize = await node.blobsSize(hash);
    expect(gotSize).toBe(blobSize);

    const gotBytes = await node.blobsReadToBytes(hash);
    expect(gotBytes.length).toBe(blobSize);
    expect(gotBytes).toEqual(bytes);

    const outPath = path.join(dir, "out");
    await node.blobsWriteToPath(hash, outPath);

    const gotBytesFromFile = fs.readFileSync(outPath);
    expect(gotBytesFromFile.length).toBe(blobSize);
    expect(gotBytesFromFile).toEqual(bytes);
});

test('blob collections', async () => {
    const collectionDir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
    const numFiles = 3;
    const blobSize = 100;

    for (let i = 0; i < numFiles; i++) {
        const bytes = Buffer.alloc(blobSize).map(() => Math.floor(Math.random() * 256));
        const filePath = path.join(collectionDir, `${i}`);
        fs.writeFileSync(filePath, bytes);
    }

    const irohDir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
    const node = await IrohNode.withPath(irohDir);

    const addProgress = await node.blobsAddFromPath(collectionDir, false, null, false);

    let collectionHash = null;
    let format = null;
    let blobHashes = [];
    for (const progressEvent of addProgress) {
        if (!progressEvent) { throw new Error("unexpected empty progress event")}
        if (progressEvent.hasOwnProperty('AllDone')) {
            collectionHash = progressEvent.AllDone.hash;
            format = progressEvent.AllDone.format;
        }
        if (progressEvent.hasOwnProperty('Abort')) {
            throw new Error(progressEvent.Abort.error);
        }
        if (progressEvent.hasOwnProperty('Done')) {
            blobHashes.push(progressEvent.Done.hash);
        }
    }

    expect(collectionHash).not.toBeNull();
    expect(format).toBe(BlobFormat.HashSeq);

    const collections = await node.blobsListCollections();
    expect(collections.length).toBe(1);
    expect(collections[0].hash).toBe(collectionHash);
    expect(collections[0].total_blobs_count).toBe(4);

    const collectionHashes = [...blobHashes, collectionHash];
    const gotHashes = await node.blobsList();
    for (const hash of gotHashes) {
        const blobBytes = await node.blobsReadToBytes(Hash.fromString(hash));
        console.log("hash", hash.toString(), "has size", blobBytes.length)
    }
    expect(collectionHashes.length + 1).toBe(gotHashes.length);

    for (const expectHash of collectionHashes) {
        let found = false;
        for (const gotHash of gotHashes) {
            if (expectHash == gotHash) {
                found = true;
                break;
            }
        }
        if (!found) {
            throw new Error(`Could not find ${expectHash} in list`);
        }
    }
});

test('list and delete', async () => {
    const irohDir = fs.mkdtempSync(path.join(os.tmpdir(), ''));
    const node = await IrohNode.withPath(irohDir);

    const blobSize = 100;
    const numBlobs = 3;
    const blobs = Array.from({length: numBlobs }, () => {
        return Buffer.alloc(blobSize).map(() => Math.floor(Math.random() * 256));
    });

    const hashes = [];
    for (const blob of blobs) {
        const output = await node.blobsAddBytes(blob, null);
        hashes.push(output.hash);
    }

    let gotHashes = await node.blobsList();
    expect(gotHashes.length).toBe(numBlobs);
    for (const expectHash of hashes) {
        let found = false;
        for (const gotHash of gotHashes) {
            if (expectHash == gotHash) {
                found = true;
                break;
            }
        }
        if (!found) {
            throw new Error(`Could not find ${expectHash} in list`);
        }
    }

    const removeHash = hashes.shift();
    await node.blobsDeleteBlob(Hash.fromString(removeHash));

    gotHashes = await node.blobsList();
    expect(gotHashes.length).toBe(numBlobs - 1);
    for (const gotHash of gotHashes) {
        if (removeHash == gotHash) {
            throw new Error(`Blob ${removeHash} should have been removed`);
        }
    }
});

