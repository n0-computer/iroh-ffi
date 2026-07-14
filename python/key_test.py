# Tests that correspond to the `src/key.rs` rust api.
import pytest

from iroh import EndpointId, IrohError, IrohErrorKind, SecretKey, Signature


def test_endpoint_id():
    key_str = "523c7996bad77424e96786cf7a7205115337a5b4565cd25506a0f297b191a5ea"
    fmt_str = "523c7996ba"
    bytes_value = (
        b"\x52\x3c\x79\x96\xba\xd7\x74\x24\xe9\x67\x86\xcf\x7a\x72\x05\x11"
        b"\x53\x37\xa5\xb4\x56\x5c\xd2\x55\x06\xa0\xf2\x97\xb1\x91\xa5\xea"
    )

    id_ = EndpointId.from_string(key_str)
    assert str(id_) == key_str
    assert id_.to_bytes() == bytes_value
    assert id_.fmt_short() == fmt_str

    id_2 = EndpointId.from_bytes(bytes_value)
    assert str(id_2) == key_str
    assert id_2.to_bytes() == bytes_value
    assert id_2.fmt_short() == fmt_str

    assert id_ == id_2
    assert id_2 == id_


def test_endpoint_id_invalid():
    with pytest.raises(IrohError) as exc_info:
        EndpointId.from_bytes(b"too short")
    err = exc_info.value
    assert err.kind() == IrohErrorKind.INVALID_INPUT
    assert err.is_kind(IrohErrorKind.INVALID_INPUT)
    assert "32 bytes" in err.message()
    assert err.debug_message() == err.message()


def test_endpoint_id_parse_error_kind():
    with pytest.raises(IrohError) as exc_info:
        EndpointId.from_string("not-an-endpoint-id")
    err = exc_info.value
    assert err.kind() == IrohErrorKind.KEY_PARSING
    assert err.is_kind(IrohErrorKind.KEY_PARSING)
    assert err.message()
    assert err.debug_message()


def test_secret_key_roundtrip():
    secret = SecretKey.generate()
    raw = secret.to_bytes()
    assert len(raw) == 32
    secret2 = SecretKey.from_bytes(raw)
    assert secret.to_bytes() == secret2.to_bytes()
    assert secret.public().to_bytes() == secret2.public().to_bytes()


def test_sign_verify_roundtrip():
    secret = SecretKey.generate()
    public = secret.public()
    msg = b"hello iroh"
    sig = secret.sign(msg)

    # signature serializes to 64 bytes and round-trips
    raw = sig.to_bytes()
    assert len(raw) == 64
    sig2 = Signature.from_bytes(raw)
    assert sig2.to_bytes() == raw

    public.verify(msg, sig)
    public.verify(msg, sig2)


def test_verify_rejects_tampered():
    secret = SecretKey.generate()
    public = secret.public()
    sig = secret.sign(b"original")
    try:
        public.verify(b"tampered", sig)
        assert False, "expected verification failure"
    except Exception:
        pass
