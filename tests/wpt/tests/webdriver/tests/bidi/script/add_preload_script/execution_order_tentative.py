import asyncio
import pytest

from webdriver.bidi.modules.script import ContextTarget

CONTEXT_CREATED_EVENT = "browsingContext.contextCreated"
CONTEXT_LOAD_EVENT = "browsingContext.load"

INLINE_SCRIPT_DOCUMENT = "<script>window.baz = window.foo</script>"

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("access_type", [
    "current_context_with_url",
    "current_context_with_non_blank_url",
    "current_context_without_url",
    "opener_context_with_url",
    "opener_context_without_url",
    "inline_script"
])
@pytest.mark.parametrize("create_type", ["popup", "iframe"])
async def test_preload_script_properties_available_immediately(
    bidi_session, add_preload_script, new_tab, subscribe_events, wait_for_event, wait_for_future_safe, inline, create_type, access_type
):
    await add_preload_script(function_declaration="() => { window.foo = 'bar'; }")

    await subscribe_events([CONTEXT_CREATED_EVENT, CONTEXT_LOAD_EVENT])
    on_created = wait_for_event(CONTEXT_CREATED_EVENT)
    if access_type == "inline_script":
        on_loaded = wait_for_event(CONTEXT_LOAD_EVENT)

    url = inline('<div>in window open</div>')
    if create_type == "popup":
        if access_type == "current_context_with_url":
            script = "window.open('about:blank')"
        elif access_type == "current_context_with_non_blank_url":
            script = f"""window.open("{url}")"""
        elif access_type == "current_context_without_url":
            script = "window.open()"
        elif access_type == "opener_context_with_url":
            script = "window.baz = window.open('about:blank').foo"
        elif access_type == "opener_context_without_url":
            script = "window.baz = window.open().foo"
        elif access_type == "inline_script":
            # Browsers may block script-initiated top-level navigations to
            # data: URLs (e.g. Chrome). The specification allows this: Fetch
            # lets user agents intervene on any navigation request, see
            # https://fetch.spec.whatwg.org/#potentially-override-response-for-a-request
            # This test does not check that behavior, so use an HTTP(S)
            # document with the same inline script instead.
            script = f"window.open('{inline(INLINE_SCRIPT_DOCUMENT)}')"
    elif create_type == "iframe":
        script = "const iframe = document.createElement('iframe');"
        if access_type == "current_context_with_url":
            script += "iframe.src='about:blank'; document.body.appendChild(iframe)"
        if access_type == "current_context_with_non_blank_url":
            script += f"iframe.src='{url}'; document.body.appendChild(iframe)"
        elif access_type == "current_context_without_url":
            script += "document.body.appendChild(iframe)"
        elif access_type == "opener_context_with_url":
            script += """iframe.src='about:blank'; document.body.appendChild(iframe);
                window.baz = iframe.contentWindow.foo"""
        elif access_type == "opener_context_without_url":
            script += "document.body.appendChild(iframe); window.baz = iframe.contentWindow.foo"
        elif access_type == "inline_script":
            script += f"""iframe.src='data:text/html,{INLINE_SCRIPT_DOCUMENT}';
                document.body.appendChild(iframe)"""

    asyncio.create_task(
        bidi_session.script.evaluate(
            expression=script,
            target=ContextTarget(new_tab["context"]),
            await_promise=False,
        )
    )

    new_context_info = await wait_for_future_safe(on_created)
    try:
        if access_type == "inline_script":
            # ensure the inline script was executed
            await wait_for_future_safe(on_loaded)

        if access_type == "current_context_with_url" or access_type == "current_context_without_url" or access_type == "current_context_with_non_blank_url":
            result = await bidi_session.script.evaluate(
                expression="window.foo",
                target=ContextTarget(new_context_info["context"]),
                await_promise=False,
            )
        if access_type == "opener_context_with_url" or access_type == "opener_context_without_url":
            result = await bidi_session.script.evaluate(
                expression="window.baz",
                target=ContextTarget(new_tab["context"]),
                await_promise=False,
            )
        if access_type == "inline_script":
            result = await bidi_session.script.evaluate(
                expression="window.baz",
                target=ContextTarget(new_context_info["context"]),
                await_promise=False,
            )

        assert result == {"type": "string", "value": "bar"}

    finally:
        if create_type == "popup":
            await bidi_session.browsing_context.close(context=new_context_info["context"])
