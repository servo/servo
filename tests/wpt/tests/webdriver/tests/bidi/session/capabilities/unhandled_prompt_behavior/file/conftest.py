import pytest
import pytest_asyncio
import asyncio

from webdriver.error import TimeoutException
from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def assert_file_dialog_canceled(bidi_session, inline, top_context):
    async def assert_file_dialog_canceled():
        cancel_event = await bidi_session.script.evaluate(
            expression="""
                new Promise(resolve => {
                    const picker = document.createElement('input');
                    picker.type = 'file';
                    picker.addEventListener('cancel', (event) => {
                        resolve(event.isTrusted);
                    });
                    picker.click();
                })""",
            target=ContextTarget(top_context["context"]),
            await_promise=True,
            user_activation=True
        )

        # Assert the `cancel` event is dispatched and the event is trusted.
        assert cancel_event == {
            'type': 'boolean',
            'value': True
        }

    yield assert_file_dialog_canceled


@pytest_asyncio.fixture
async def assert_file_dialog_not_canceled(bidi_session, inline, top_context,
        wait_for_future_safe):
    async def assert_file_dialog_not_canceled():
        cancel_event_future = asyncio.create_task(bidi_session.script.evaluate(
            expression="""
                        new Promise(resolve => {
                            const picker = document.createElement('input');
                            picker.type = 'file';
                            picker.addEventListener('cancel', (event) => {
                                resolve(event.isTrusted);
                            });
                            picker.click();
                        })""",
            target=ContextTarget(top_context["context"]),
            await_promise=True,
            user_activation=True
        ))

        with pytest.raises(TimeoutException):
            await wait_for_future_safe(cancel_event_future, timeout=0.5)

        cancel_event_future.cancel()

    yield assert_file_dialog_not_canceled
