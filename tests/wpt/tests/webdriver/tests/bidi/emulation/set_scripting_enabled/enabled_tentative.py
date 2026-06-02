import pytest

from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


async def test_node_set_event_handler_value(bidi_session, inline):
    context = await bidi_session.browsing_context.create(type_hint="tab")
    url = inline("""
        <script>window.onload = ()=>{console.log('onload')}</script>
        """)
    await bidi_session.browsing_context.navigate(context=context["context"],
                                                 url=url, wait="complete")

    result = await bidi_session.script.evaluate(
        expression="window.onload",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    assert result == {'type': 'function'}

    # Disable scripting.
    await bidi_session.emulation.set_scripting_enabled(
        enabled=False,
        contexts=[context["context"]],
    )

    result = await bidi_session.script.evaluate(
        expression="window.onload",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    assert result == {'type': 'function'}


async def test_node_added_event_handlers(bidi_session, inline):
    context = await bidi_session.browsing_context.create(type_hint="tab")
    url = inline("""
        <script>
            addEventListener("message", (event) => { console.log('onmessage')})
        </script>
    """)
    await bidi_session.browsing_context.navigate(context=context["context"],
                                                 url=url, wait="complete")

    result = await bidi_session.script.evaluate(
        expression="window.onmessage",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    assert result == {'type': 'null'}

    await bidi_session.script.evaluate(
        expression="""addEventListener("message", (event) => { console.log('onmessage')})""",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )

    result = await bidi_session.script.evaluate(
        expression="window.onmessage",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    assert result == {'type': 'null'}

    # Disable scripting.
    await bidi_session.emulation.set_scripting_enabled(
        enabled=False,
        contexts=[context["context"]],
    )

    result = await bidi_session.script.evaluate(
        expression="window.onmessage",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    assert result == {'type': 'null'}


async def test_finalization_registry(bidi_session, inline):
    """
    The test relies on `window.gc()` to force GC. In Chrome, it is exposed when
    run with `--js-flags=--expose-gc` flag.
    """
    context = await bidi_session.browsing_context.create(type_hint="tab")

    url = inline("""<script>
            window.finalizationRegistryTriggered = false;
            const registry = new FinalizationRegistry((heldValue) => {
                window.finalizationRegistryTriggered = true;
            });

            function createAndRegister() {
                const myObject = {};
                registry.register(myObject);
            }
            createAndRegister();
        </script>
        """)

    await bidi_session.browsing_context.navigate(context=context["context"],
                                                 url=url, wait="complete")

    result = await bidi_session.script.evaluate(
        expression="window.finalizationRegistryTriggered",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    assert result == {'type': 'boolean', 'value': False}

    # Disable scripting.
    await bidi_session.emulation.set_scripting_enabled(
        enabled=False,
        contexts=[context["context"]],
    )

    result = await bidi_session.script.evaluate(
        expression="window.finalizationRegistryTriggered",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    assert result == {'type': 'boolean', 'value': False}

    # Force GC to collect `myObject`.
    await bidi_session.script.evaluate(
        expression="window.gc()",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )

    result = await bidi_session.script.evaluate(
        expression="window.finalizationRegistryTriggered",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    assert result == {'type': 'boolean', 'value': True}


async def test_enqueue_promise_job(bidi_session, inline):
    context = await bidi_session.browsing_context.create(type_hint="tab")

    # Disable scripting.
    await bidi_session.emulation.set_scripting_enabled(
        enabled=False,
        contexts=[context["context"]],
    )

    result = await bidi_session.script.evaluate(
        expression="Promise.resolve(true).then(x=>x)",
        target=ContextTarget(context["context"]),
        await_promise=True,
    )
    assert result == {'type': 'boolean', 'value': True}
