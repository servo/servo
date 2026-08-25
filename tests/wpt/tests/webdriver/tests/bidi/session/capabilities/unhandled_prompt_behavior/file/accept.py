# META: timeout=long
import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.capabilities({"unhandledPromptBehavior": 'accept'})
async def test_string_accept(assert_file_dialog_canceled):
    await assert_file_dialog_canceled()


@pytest.mark.capabilities({"unhandledPromptBehavior": 'accept and notify'})
async def test_string_accept_and_notify(assert_file_dialog_canceled):
    await assert_file_dialog_canceled()


@pytest.mark.capabilities({"unhandledPromptBehavior": {'default': 'accept'}})
async def test_default_accept(assert_file_dialog_canceled):
    await assert_file_dialog_canceled()


@pytest.mark.capabilities(
    {"unhandledPromptBehavior": {'file': 'accept', 'default': 'ignore'}})
async def test_file_accept(assert_file_dialog_canceled):
    await assert_file_dialog_canceled()
