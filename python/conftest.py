import asyncio

import pytest
import iroh


@pytest.fixture(autouse=True)
async def _uniffi_event_loop():
    """uniffi's async runtime needs the *running* asyncio loop for each test."""
    iroh.iroh_ffi.uniffi_set_event_loop(asyncio.get_running_loop())
    yield
