import pytest

pytestmark = pytest.mark.asyncio


async def test_contexts(
    bidi_session,
    new_tab,
    assert_locale_against_default,
    assert_locale_against_value,
    some_locale,
):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )

    # Assert locale emulated only in the required context.
    await assert_locale_against_value(some_locale, new_tab)
    await assert_locale_against_default(new_context)

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=None
    )

    # Assert the locale is restored to the initial one.
    await assert_locale_against_default(new_tab)
    await assert_locale_against_default(new_context)


async def test_multiple_contexts(
    bidi_session,
    new_tab,
    assert_locale_against_default,
    assert_locale_against_value,
    some_locale,
):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"], new_context["context"]], locale=some_locale
    )

    # Assert locale emulated in all the required contexts.
    await assert_locale_against_value(some_locale, new_tab)
    await assert_locale_against_value(some_locale, new_context)

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"], new_context["context"]], locale=None
    )

    # Assert the locale is restored to the initial one.
    await assert_locale_against_default(new_tab)
    await assert_locale_against_default(new_context)


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_iframe(
    bidi_session,
    new_tab,
    inline,
    another_locale,
    assert_locale_against_value,
    some_locale,
    domain,
):
    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )

    # Assert locale emulated in the required context.
    await assert_locale_against_value(some_locale, new_tab)

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
    await assert_locale_against_value(some_locale, iframe)

    sandbox_name = "test"
    # Assert locale is emulated in the newly created sandbox in the iframe context.
    await assert_locale_against_value(some_locale, iframe, sandbox_name)

    # Set another locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=another_locale
    )

    # Assert locale is emulated in the iframe context.
    await assert_locale_against_value(another_locale, iframe)
    # Assert locale is emulated in the existing sandbox in the iframe context.
    await assert_locale_against_value(another_locale, iframe, sandbox_name)


async def test_locale_override_applies_to_new_sandbox(
    bidi_session, new_tab, some_locale, assert_locale_against_value
):
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=some_locale
    )

    # Make sure the override got applied to the newly created sandbox.
    await assert_locale_against_value(some_locale, new_tab, "test")


async def test_locale_override_applies_to_existing_sandbox(
    bidi_session,
    new_tab,
    another_locale,
    assert_locale_against_default,
    assert_locale_against_value,
):
    sandbox_name = "test"

    # Create a sandbox.
    await assert_locale_against_default(new_tab, sandbox_name)

    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]], locale=another_locale
    )

    # Make sure the override got applied to the existing sandbox.
    await assert_locale_against_value(another_locale, new_tab, sandbox_name)
