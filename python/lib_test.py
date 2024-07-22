# tests that correspond to the `src/lib.rs` rust api functions
from iroh import path_to_key, key_to_path

def test_path_to_key_roundtrip():
    path = "/foo/bar"
    key = b'/foo/bar\0'
    #
    got_key = path_to_key(path, None, None)
    assert key == got_key
    #
    got_path = key_to_path(got_key, None, None)
    assert path == got_path
    #
    # including prefix
    prefix = "prefix:"
    key = b'prefix:/foo/bar\0'
    #
    got_key = path_to_key(path, prefix, None)
    assert key == got_key
    #
    got_path = key_to_path(got_key, prefix, None)
    assert path == got_path
    #
    # including root
    root = "/foo"
    key = b'prefix:bar\0'
    #
    got_key = path_to_key(path, prefix, root)
    assert key == got_key
    #
    got_path = key_to_path(got_key, prefix, root)
    assert path == got_path
