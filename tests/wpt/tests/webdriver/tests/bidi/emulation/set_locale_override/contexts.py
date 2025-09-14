import pytest
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


async def test_contexts(bidi_session, new_tab, top_context, get_current_locale,
        default_locale, some_locale):
    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=some_locale
    )

    # Assert locale emulated only in the required context.
    assert await get_current_locale(new_tab) == some_locale
    assert await get_current_locale(top_context) == default_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=None)

    # Assert the locale is restored to the initial one.
    assert await get_current_locale(new_tab) == default_locale
    assert await get_current_locale(top_context) == default_locale


async def test_multiple_contexts(bidi_session, new_tab, get_current_locale,
        default_locale, some_locale):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"], new_context["context"]],
        locale=some_locale
    )

    # Assert locale emulated in all the required contexts.
    assert await get_current_locale(new_tab) == some_locale
    assert await get_current_locale(new_context) == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"], new_context["context"]],
        locale=None)

    # Assert the locale is restored to the initial one.
    assert await get_current_locale(new_tab) == default_locale
    assert await get_current_locale(new_context) == default_locale


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(
    bidi_session,
    new_tab,
    get_current_locale,
    some_locale,
    domain,
    inline,
    another_locale,
):
    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )

    # Assert locale emulated in the required context.
    assert await get_current_locale(new_tab) == some_locale

    iframe_url = inline("<div id='in-iframe'>foo</div>", domain=domain)
    page_url = inline(f"<iframe src='{iframe_url}'></iframe>")

    # Load the page with iframes.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=page_url,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe = contexts[0]["children"][0]

    # Assert locale is emulated in the iframe context.
    assert await get_current_locale(iframe) == some_locale

    sandbox_name = "test"
    # Assert locale is emulated in the newly created sandbox in the iframe context.
    assert await get_current_locale(iframe, sandbox_name) == some_locale

    # Set another locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=another_locale
    )

    # Assert locale is emulated in the iframe context.
    assert await get_current_locale(iframe) == another_locale
    # Assert locale is emulated in the existing sandbox in the iframe context.
    assert await get_current_locale(iframe, sandbox_name) == another_locale


async def test_locale_override_applies_to_new_sandbox(
    bidi_session, new_tab, some_locale, get_current_locale
):
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )

    # Make sure the override got applied to the newly created sandbox.
    assert await get_current_locale(new_tab, "test") == some_locale


async def test_locale_override_applies_to_existing_sandbox(
    bidi_session, new_tab, default_locale, another_locale, get_current_locale
):
    sandbox_name = "test"

    # Create a sandbox.
    assert await get_current_locale(new_tab, sandbox_name) == default_locale

    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=another_locale
    )

    # Make sure the override got applied to the existing sandbox.
    assert await get_current_locale(new_tab, sandbox_name) == another_locale
