const { IrohNode, PublicKey, NodeAddr, AuthorId, Query, SortBy, SortDirection, QueryOptions, pathToKey, keyToPath } = require( '../index');
const path = require('path');

test('node address', () => {
    const keyStr = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva";
    const nodeID = PublicKey.fromString(keyStr);
    const ipv4 = "127.0.0.1:3000";
    const ipv6 = "::1:3000";
    const derpURL = "https://example.com";
    const expectAddrs = [ipv4, ipv6];
    const nodeAddr = new NodeAddr(nodeID, derpURL, expectAddrs);
    const gotAddrs = nodeAddr.directAddresses;
    expect(gotAddrs).toEqual(expectAddrs);
    expect(nodeAddr.derpUrl).toBe(derpURL);
});

test('author ID', () => {
    const authorStr = "mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q";
    const author = AuthorId.fromString(authorStr);
    expect(author.toString()).toBe(authorStr);
    const author0 = AuthorId.fromString(authorStr);
    expect(author.equal(author0)).toBe(true);
});

// test('query', () => {
//     const opts = new QueryOptions(SortBy.KEY_AUTHOR, SortDirection.ASC, 10, 10);
//     const all = Query.all(opts);
//     expect(all.offset()).toBe(10);
//     expect(all.limit()).toBe(10);

//     opts.direction = SortDirection.DESC;
//     opts.limit = 0;
//     opts.offset = 0;
//     const singleLatestPerKey = Query.singleLatestPerKey(opts);
//     expect(singleLatestPerKey.offset()).toBe(0);
//     expect(singleLatestPerKey.limit()).toBe(null);

//     opts.direction = SortDirection.ASC;
//     opts.offset = 100;
//     const author = Query.author(AuthorId.fromString("mqtlzayyv4pb4xvnqnw5wxb2meivzq5ze6jihpa7fv5lfwdoya4q"), opts);
//     expect(author.offset()).toBe(100);
//     expect(author.limit()).toBe(null);

//     opts.sortBy = SortBy.KEY_AUTHOR;
//     opts.direction = SortDirection.DESC;
//     opts.offset = 0;
//     opts.limit = 100;
//     const keyExact = Query.keyExact(Buffer.from('key'), opts);
//     expect(keyExact.offset()).toBe(0);
//     expect(keyExact.limit()).toBe(100);

//     const keyPrefix = Query.keyPrefix(Buffer.from('prefix'), opts);
//     expect(keyPrefix.offset()).toBe(0);
//     expect(keyPrefix.limit()).toBe(100);
// });

test('document entry basics', async () => {
    const dir = path.join(require('os').tmpdir(), 'document_entry_basics');
    const node = await IrohNode.withPath(dir);
    const author = await node.authorCreate();
    const doc = await node.docCreate();
    const val = Buffer.from('hello world!');
    const key = Buffer.from('foo');
    const hash = await doc.setBytes(author, key, val);
//     const query = Query.authorKeyExact(author, key);
//     const entry = doc.getOne(query);
//     expect(hash.equal(entry.contentHash())).toBe(true);
//     expect(val.length).toBe(entry.contentLen());
//     const gotVal = entry.contentBytes(doc);
//     expect(gotVal.equals(val)).toBe(true);
});

// test('document import export', async () => {
//     const tmp = require('os').tmpdir();
//     const inRoot = require('path').join(tmp, "in");
//     const outRoot = require('path').join(tmp, "out");
//     require('fs').mkdirSync(inRoot, { recursive: true });
//     require('fs').mkdirSync(outRoot, { recursive: true });
//     const path = require('path').join(inRoot, "test");
//     const size = 100;
//     const bytes = Buffer.alloc(size).map(() => Math.floor(Math.random() * 256));
//     require('fs').writeFileSync(path, bytes);
//     const irohDir = require('os').tmpdir();
//     const node = await new IrohNode.withPath(irohDir);
//     const doc = await node.docCreate();
//     const author = await node.authorCreate();
//     const key = pathToKey(path, null, inRoot);
//     await doc.importFile(author, key, path, true, null);
    // const query = Query.authorKeyExact(author, key);
    // const entry = doc.getOne(query);
    // const outPath = keyToPath(key, null, outRoot);
    // doc.exportFile(entry, outPath, null);
    // const gotBytes = require('fs').readFileSync(outPath);
    // expect(gotBytes.equals(bytes)).toBe(true);
// });
