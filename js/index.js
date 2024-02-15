const iroh = require('../index');

console.log('hello', iroh);

const node = iroh.IrohNode.withPath("./iroh-node");
console.log("hello iroh", node.nodeId());

const blobs = node.blobsList();
console.log("any blobs?", blobs);
