# Tests that correspond to the `src/relay.rs` rust api.
from iroh import RelayMap, RelayMode, RelayConfig


def test_relay_map_crud():
    m = RelayMap.empty()
    assert m.is_empty()
    assert m.len() == 0

    cfg = RelayConfig(
        url="https://relay.example.org/",
        quic_port=7842,
        auth_token="hunter2",
    )
    m.insert(cfg)
    assert m.len() == 1
    assert m.contains("https://relay.example.org/")

    got = m.get("https://relay.example.org/")
    assert got is not None
    assert got.url == "https://relay.example.org/"
    assert got.quic_port == 7842
    assert got.auth_token == "hunter2"

    assert "https://relay.example.org/" in m.urls()

    assert m.remove("https://relay.example.org/")
    assert m.is_empty()


def test_relay_map_from_urls():
    m = RelayMap.from_urls(["https://r1.example.org/", "https://r2.example.org/"])
    assert m.len() == 2


def test_relay_mode_constructors():
    RelayMode.disabled()
    RelayMode.default_mode()
    RelayMode.staging()
    m = RelayMap.from_urls(["https://r1.example.org/"])
    custom = RelayMode.custom(m)
    assert custom.relay_map().len() == 1
    RelayMode.custom_from_urls(["https://r2.example.org/"])
