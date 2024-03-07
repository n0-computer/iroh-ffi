const { pathToKey, keyToPath } = require('../index');

test('path to key roundtrip', () => {
    const path = "/foo/bar";
    const key = Buffer.from('/foo/bar\0');

    let gotKey = pathToKey(path, null, null);
    expect(gotKey.equals(key)).toBe(true);

    let gotPath = keyToPath(gotKey, null, null);
    expect(gotPath).toBe(path);

    const prefix = "prefix:";
    const prefixedKey = Buffer.from('prefix:/foo/bar\0');

    gotKey = pathToKey(path, prefix, null);
    expect(gotKey.equals(prefixedKey)).toBe(true);

    gotPath = keyToPath(gotKey, prefix, null);
    expect(gotPath).toBe(path);

    const root = "/foo";
    const rootPrefixedKey = Buffer.from('prefix:bar\0');

    gotKey = pathToKey(path, prefix, root);
    expect(gotKey.equals(rootPrefixedKey)).toBe(true);

    gotPath = keyToPath(gotKey, prefix, root);
    expect(gotPath).toBe(path);
});

