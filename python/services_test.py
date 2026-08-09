# Tests that correspond to the `src/services.rs` rust api.
#
# A well-formed (but fake) API secret: the remote it points at does not exist,
# but the client connects lazily so construction still succeeds. This validates
# the options -> builder -> client plumbing without any network.
import pytest

from iroh import (
    Endpoint,
    EndpointOptions,
    ServicesClient,
    ServicesOptions,
    ServicesPresetOptions,
    preset_iroh_services,
    preset_minimal,
)

FAKE_API_SECRET = (
    "servicesaaqaobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqobyha4dqob"
    "75c4sdqwvay5nwj63yzvqc7iozsh66x53lcpcy5vyc5ledl2pwdaaa"
)


async def _endpoint():
    return await Endpoint.bind(EndpointOptions(preset=preset_minimal()))


async def test_services_client_boots_with_fake_secret():
    ep = await _endpoint()
    client = await ServicesClient.create(ep, ServicesOptions(api_secret=FAKE_API_SECRET))
    assert client is not None
    await ep.close()


async def test_services_client_rejects_no_credentials():
    ep = await _endpoint()
    with pytest.raises(Exception):
        await ServicesClient.create(ep, ServicesOptions())
    await ep.close()


async def test_services_client_rejects_two_credentials():
    ep = await _endpoint()
    with pytest.raises(Exception):
        await ServicesClient.create(
            ep,
            ServicesOptions(api_secret=FAKE_API_SECRET, api_secret_from_env=True),
        )
    await ep.close()


async def test_services_client_rejects_malformed_secret():
    ep = await _endpoint()
    with pytest.raises(Exception):
        await ServicesClient.create(ep, ServicesOptions(api_secret="not-a-valid-ticket"))
    await ep.close()


async def test_services_preset_binds_with_custom_relays():
    # The relay is unreachable, but bind does not connect, so this checks the
    # mint -> relay map -> builder plumbing.
    preset = preset_iroh_services(
        ServicesPresetOptions(
            relays=["https://relay.example.org/"],
            api_secret=FAKE_API_SECRET,
        )
    )
    ep = await Endpoint.bind(EndpointOptions(preset=preset))
    assert ep.bound_sockets()
    await ep.close()


def test_services_preset_rejects_bad_options():
    relays = ["https://relay.example.org/"]
    with pytest.raises(Exception):
        preset_iroh_services(ServicesPresetOptions(relays=relays))
    with pytest.raises(Exception):
        preset_iroh_services(
            ServicesPresetOptions(
                relays=relays, api_secret=FAKE_API_SECRET, api_secret_from_env=True
            )
        )
    with pytest.raises(Exception):
        preset_iroh_services(
            ServicesPresetOptions(relays=relays, api_secret="not-a-valid-ticket")
        )
    # An explicitly empty list errors; omitting it falls back to the n0 relays.
    with pytest.raises(Exception):
        preset_iroh_services(ServicesPresetOptions(relays=[], api_secret=FAKE_API_SECRET))
    with pytest.raises(Exception):
        preset_iroh_services(
            ServicesPresetOptions(relays=["not a url"], api_secret=FAKE_API_SECRET)
        )


def test_services_preset_defaults_to_n0_relays():
    assert preset_iroh_services(ServicesPresetOptions(api_secret=FAKE_API_SECRET))


async def test_services_preset_pins_the_endpoint_key():
    # The token is scoped to the preset's key, so an EndpointOptions key must
    # fail loudly rather than break relay auth at connect time.
    preset = preset_iroh_services(
        ServicesPresetOptions(
            relays=["https://relay.example.org/"],
            api_secret=FAKE_API_SECRET,
        )
    )
    with pytest.raises(Exception):
        await Endpoint.bind(EndpointOptions(preset=preset, secret_key=bytes(32)))
