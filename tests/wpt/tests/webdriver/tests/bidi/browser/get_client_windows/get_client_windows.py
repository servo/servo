import pytest

pytestmark = pytest.mark.asyncio

async def test_open_and_close(bidi_session):
    initial_windows = await bidi_session.browser.get_client_windows()
    assert len(initial_windows) == 1

    new_browsing_context = await bidi_session.browsing_context.create(type_hint="window")
    try:
        updated_windows = await bidi_session.browser.get_client_windows()
        assert isinstance(updated_windows, list)
        assert len(updated_windows) == 2
        assert updated_windows[0]["clientWindow"] != updated_windows[1]["clientWindow"]
    finally:
        await bidi_session.browsing_context.close(context=new_browsing_context["context"])

    final_windows = await bidi_session.browser.get_client_windows()
    assert final_windows == initial_windows


async def test_activate_client_windows(bidi_session):
    initial_windows = await bidi_session.browser.get_client_windows()
    assert len(initial_windows) == 1
    initial_window = initial_windows[0]
    initial_window_id = initial_window["clientWindow"]

    initial_contexts = await bidi_session.browsing_context.get_tree()
    assert len(initial_contexts) == 1
    initial_context_id = initial_contexts[0]["context"]

    try:
        new_browsing_context = await bidi_session.browsing_context.create(type_hint="window")
        all_windows = await bidi_session.browser.get_client_windows()
        assert len(all_windows) == 2

        first_window = next(window for window in all_windows if window["clientWindow"] == initial_window_id)
        second_window = next(window for window in all_windows if window["clientWindow"] != initial_window_id)

        assert second_window["active"]
        assert not first_window["active"]

        await bidi_session.browsing_context.activate(context=initial_context_id)

        all_windows = await bidi_session.browser.get_client_windows()

        first_window = next(window for window in all_windows if window["clientWindow"] == initial_window_id)
        second_window = next(window for window in all_windows if window["clientWindow"] != initial_window_id)

        assert first_window["active"]
        assert not second_window["active"]
    finally:
        await bidi_session.browsing_context.close(context=new_browsing_context["context"])

    final_windows = await bidi_session.browser.get_client_windows()
    assert(final_windows[0]["active"]) == True
    assert final_windows[0]["clientWindow"] == initial_window_id


