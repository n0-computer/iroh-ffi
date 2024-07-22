# tests that correspond to the `src/key.rs` rust api
from iroh import PublicKey

def test_public_key():
    key_str = "ki6htfv2252cj2lhq3hxu4qfcfjtpjnukzonevigudzjpmmruxva"
    fmt_str = "ki6htfv2252cj2lh"
    bytes = b'\x52\x3c\x79\x96\xba\xd7\x74\x24\xe9\x67\x86\xcf\x7a\x72\x05\x11\x53\x37\xa5\xb4\x56\x5c\xd2\x55\x06\xa0\xf2\x97\xb1\x91\xa5\xea'
    #
    # create key from string
    key = PublicKey.from_string(key_str)
    #
    # test methods are as expected
    assert str(key) == key_str
    assert key.to_bytes() == bytes
    assert key.fmt_short() == fmt_str
    #
    # create key from bytes
    key_0 = PublicKey.from_bytes(bytes)
    #
    # test methods are as expected
    assert str(key_0) == key_str
    assert key_0.to_bytes() == bytes
    assert key_0.fmt_short() == fmt_str
    #
    # test that the eq function works
    assert key.equal(key_0)
    assert key_0.equal(key)
