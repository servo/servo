# META: timeout=long
import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.capabilities({"unhandledPromptBehavior": 'dismiss'})
async def test_string_dismiss(assert_file_dialog_canceled):
    await assert_file_dialog_canceled()


@pytest.mark.capabilities({"unhandledPromptBehavior": 'dismiss and notify'})
async def test_string_dismiss_and_notify(assert_file_dialog_canceled):
    await assert_file_dialog_canceled()


@pytest.mark.capabilities({"unhandledPromptBehavior": {'default': 'dismiss'}})
async def test_default_dismiss(assert_file_dialog_canceled):
    await assert_file_dialog_canceled()


@pytest.mark.capabilities(
    {"unhandledPromptBehavior": {'file': 'dismiss', 'default': 'ignore'}})
async def test_file_dismiss(assert_file_dialog_canceled):
    await assert_file_dialog_canceled()
