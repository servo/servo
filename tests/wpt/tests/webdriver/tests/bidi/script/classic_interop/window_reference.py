import pytest

from webdriver import WebFrame, WebWindow
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


async def test_web_window_reference_created_in_classic(
    bidi_session,
    current_session,
    get_test_page,
):
    handle = current_session.new_window(type_hint="tab")
    current_session.window_handle = handle
    current_session.url = get_test_page()

    expected_test_value = "bar"
    window = current_session.execute_script(
        f"window.foo = '{expected_test_value}'; return window;"
    )

    contexts = await bidi_session.browsing_context.get_tree()
    assert len(contexts) == 2

    assert window.id == contexts[1]["context"]

    result = await bidi_session.script.evaluate(
        expression=f"window.foo",
        target=ContextTarget(window.id),
        await_promise=False,
    )

    assert result["value"] == expected_test_value


async def test_web_frame_reference_created_in_classic(
    bidi_session,
    current_session,
    get_test_page,
):
    handle = current_session.new_window(type_hint="tab")
    current_session.window_handle = handle
    current_session.url = get_test_page()

    expected_test_value = "foo"
    frame = current_session.execute_script(
        f"window.frames[0].bar='{expected_test_value}'; return window.frames[0]"
    )

    contexts = await bidi_session.browsing_context.get_tree()
    assert len(contexts) == 2

    assert frame.id == contexts[1]["children"][0]["context"]

    result = await bidi_session.script.evaluate(
        expression=f"window.bar",
        target=ContextTarget(frame.id),
        await_promise=False,
    )

    assert result["value"] == expected_test_value


async def test_web_window_reference_created_in_bidi(
    bidi_session,
    current_session,
    get_test_page,
    new_tab
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=get_test_page(),
        wait="complete"
    )

    expected_test_value = "bar"
    result = await bidi_session.script.evaluate(
        expression=f"window.xyz = '{expected_test_value}'; window;",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    context_id = result["value"]["context"]

    # Use window reference from WebDriver BiDi in WebDriver classic
    current_session.window_handle = new_tab["context"]
    window = WebWindow(current_session, context_id)
    test_value = current_session.execute_script(
        """return arguments[0].xyz""", args=(window,)
    )

    assert test_value == expected_test_value


async def test_web_frame_reference_created_in_bidi(
    bidi_session,
    current_session,
    get_test_page,
    new_tab
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=get_test_page(),
        wait="complete"
    )

    expected_test_value = "foo"
    result = await bidi_session.script.evaluate(
        expression=f"window.frames[0].baz='{expected_test_value}'; window.frames[0];",
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    context_id = result["value"]["context"]

    # Use window reference from WebDriver BiDi in WebDriver classic
    current_session.window_handle = new_tab["context"]
    window = WebFrame(current_session, context_id)
    test_value = current_session.execute_script(
        """return arguments[0].baz""", args=(window,)
    )

    assert test_value == expected_test_value
