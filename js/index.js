const iroh = require('../index');

(async () => {
  console.log('hello', iroh);

  const node = await iroh.IrohNode.withPath("./iroh-node");
  console.log("hello iroh", node.nodeId());

  const blobs = await node.blobsList();
  console.log("any blobs?", blobs);

  const author = await node.authorCreate();
  console.log("author created", author.toString());

  await node.blobsAddFromPath(
    "/Users/dignifiedquire/opensource/iroh-ffi/js/index.js",
    false, // not in place
    null, // auto tag
    true, // wrap
    (err, progress) => {
      if (err != null) {
        throw err;
      }
      console.log("progress", progress);
    }
  );
  console.log("done adding blob");
})()
