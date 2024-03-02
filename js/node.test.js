const os = require('os');
const fs = require('fs');
const path = require('path');
const { IrohNode, ShareMode, Hash } = require('../index'); 

test('basic sync', async () => {
    // Create node_0
    const irohDir0 = fs.mkdtempSync(path.join(os.tmpdir(), 'dir0'));
    const node0 = await IrohNode.withPath(irohDir0);

    // Create node_1
    const irohDir1 = fs.mkdtempSync(path.join(os.tmpdir(), 'dir1'));
    const node1 = await IrohNode.withPath(irohDir1);

    // Create doc on node_0
    const doc0 = await node0.docCreate();
    const ticket = await doc0.share(ShareMode.Write);

    // Subscribe to sync events
    let events = await doc0.subscribe();

    // Join the same doc from node_1
    const doc1 = await node1.docJoin(ticket);

    // Create author on node_1
    const author = await node1.authorCreate();
    const key = Buffer.from("hello");
    const val = Buffer.from("world");
    await doc1.setBytes(author, key, val);

    let hash = null;
    for (const val of events) {
      if (val && val.hasOwnProperty('ContentReady')) {
        hash = val.ContentReady.hash
        break;
      }
    }

    // Get content from hash
    const got = await node1.blobsReadToBytes(Hash.fromString(hash));
    expect(got).toStrictEqual(val);

    await doc1.close();
    await doc0.close();
});

// test('basic sync', async () => {
//     // Create node_0
//     const irohDir0 = fs.mkdtempSync(path.join(os.tmpdir(), 'dir0'));
//     const node0 = await IrohNode.withPath(irohDir0);

//     // Create node_1
//     const irohDir1 = fs.mkdtempSync(path.join(os.tmpdir(), 'dir1'));
//     const node1 = await IrohNode.withPath(irohDir1);

//     // Create doc on node_0
//     const doc0 = await node0.docCreate();
//     const ticket = await doc0.share(ShareMode.Write);

//     class SubscribeCallback {
//         constructor(found) {
//             this.found = found;
//             this.event = this.event.bind(this);
//         }

//         async event(_, event) {
//             if (event && event.hasOwnProperty('ContentReady')) {
//                 this.found.push(event.ContentReady.hash);
//             }
//         }
//     }

//     // Subscribe to sync events
//     const foundS = [];
//     const cb = new SubscribeCallback(foundS);
//     await doc0.subscribe(cb.event);

//     // Join the same doc from node_1
//     const doc1 = await node1.docJoin(ticket);

//     // Create author on node_1
//     const author = await node1.authorCreate();
//     const key = Buffer.from("hello");
//     const val = Buffer.from("world");
//     await doc1.setBytes(author, key, val);

//     // Wait for the content ready event
//     // TODO(ramfox): this is horrible
//     await new Promise(resolve => setTimeout(resolve, 1000));

//     // Get content from hash
//     const hash = foundS.shift();
//     const got = await node1.blobsReadToBytes(Hash.fromString(hash));
//     expect(got).toStrictEqual(val);

//     await doc1.close();
//     await doc0.close();
// });

