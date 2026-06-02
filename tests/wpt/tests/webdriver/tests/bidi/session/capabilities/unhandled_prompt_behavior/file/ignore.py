# META: timeout=long
import pytest

pytestmark = pytest.mark.asyncio


async def test_no_capabilities(assert_file_dialog_not_canceled):
    await assert_file_dialog_not_canceled()


@pytest.mark.capabilities({"unhandledPromptBehavior": 'ignore'})
async def test_string_ignore(assert_file_dialog_not_canceled):
    await assert_file_dialog_not_canceled()


@pytest.mark.capabilities({"unhandledPromptBehavior": {'default': 'ignore'}})
async def test_default_ignore(assert_file_dialog_not_canceled):
    await assert_file_dialog_not_canceled()


@pytest.mark.capabilities(
    {"unhandledPromptBehavior": {'file': 'ignore', 'default': 'accept'}})
async def test_file_ignore(assert_file_dialog_not_canceled):
    await assert_file_dialog_not_canceled()
