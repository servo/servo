import pytest
from webdriver.bidi.modules.script import ContextTarget, RealmTarget

from ... import recursive_compare


@pytest.mark.asyncio
async def test_evaluate(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="1 + 2",
        target=ContextTarget(top_context["context"]),
        await_promise=True)

    assert result == {
        "type": "number",
        "value": 3}


@pytest.mark.asyncio
async def test_interact_with_dom(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="'window.location.href: ' + window.location.href",
        target=ContextTarget(top_context["context"]),
        await_promise=True)

    assert result == {
        "type": "string",
        "value": "window.location.href: about:blank"}


@pytest.mark.asyncio
async def test_target_realm(bidi_session, default_realm):
    result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="window.foo = 3",
        target=RealmTarget(default_realm),
        await_promise=True,
    )

    recursive_compare({"realm": default_realm, "result": {"type": "number", "value": 3}}, result)

    result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="window.foo",
        target=RealmTarget(default_realm),
        await_promise=True,
    )

    recursive_compare(
        {"realm": default_realm, "result": {"type": "number", "value": 3}}, result
    )


@pytest.mark.asyncio
async def test_different_target_realm(bidi_session):
    await bidi_session.browsing_context.create(type_hint="tab")

    realms = await bidi_session.script.get_realms()
    first_tab_default_realm = realms[0]["realm"]
    second_tab_default_realm = realms[1]["realm"]

    assert first_tab_default_realm != second_tab_default_realm

    await bidi_session.script.evaluate(
        raw_result=True,
        expression="window.foo = 3",
        target=RealmTarget(first_tab_default_realm),
        await_promise=True,
    )
    await bidi_session.script.evaluate(
        raw_result=True,
        expression="window.foo = 5",
        target=RealmTarget(second_tab_default_realm),
        await_promise=True,
    )

    top_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="window.foo",
        target=RealmTarget(first_tab_default_realm),
        await_promise=True,
    )
    recursive_compare(
        {"realm": first_tab_default_realm, "result": {"type": "number", "value": 3}}, top_context_result
    )

    new_context_result = await bidi_session.script.evaluate(
        raw_result=True,
        expression="window.foo",
        target=RealmTarget(second_tab_default_realm),
        await_promise=True,
    )
    recursive_compare(
        {"realm": second_tab_default_realm, "result": {"type": "number", "value": 5}}, new_context_result
    )
