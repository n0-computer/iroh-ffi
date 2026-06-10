# Tests that correspond to the `src/services.rs` rust api.
#
# A well-formed (but fake) API secret: the remote it points at does not exist,
# but the client connects lazily so construction still succeeds. This validates
# the options -> builder -> client plumbing without any network.
import pytest

from iroh import Endpoint, EndpointOptions, ServicesClient, ServicesOptions, preset_minimal

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
