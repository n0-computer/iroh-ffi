const iroh = require('../index');

test('public key', () => {
  const key_str = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva";
  const fmt_str = "ki6htfv2252cj2lh";
  const bytes = [0x52, 0x3c, 0x79, 0x96, 0xba, 0xd7, 0x74, 0x24, 0xe9, 0x67, 0x86, 0xcf, 0x7a, 0x72, 0x05, 0x11, 0x53, 0x37, 0xa5, 0xb4, 0x56, 0x5c, 0xd2, 0x55, 0x06, 0xa0, 0xf2, 0x97, 0xb1, 0x91, 0xa5, 0xea];

  // create key from string
  const key = iroh.PublicKey.fromString(key_str);

  // test methods are as expected
  expect(key.toString()).toBe(key_str);
  expect(key.toBytes()).toEqual(bytes);
  expect(key.fmtShort()).toBe(fmt_str);

  // create key from bytes
  const key_0 = iroh.PublicKey.fromBytes(bytes);

  // test methods are as expected
  expect(key_0.toString()).toBe(key_str);
  expect(key_0.toBytes()).toEqual(bytes);
  expect(key_0.fmtShort()).toBe(fmt_str);

  // test that the equal function works
  expect(key.equal(key_0)).toBe(true);
  expect(key_0.equal(key)).toBe(true);
})
