import pytest
import webdriver.bidi.error as error


pytestmark = pytest.mark.asyncio


async def test_when_browsingcontext_recreated(
    bidi_session, wait_for_future_safe, inline
):
    new_tab = await bidi_session.browsing_context.create(type_hint="tab")

    page = inline("""<div style="margin-top: 10000px;">foo</div>""")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page, wait="complete"
    )

    printed_future = await bidi_session.send_command(
        "browsingContext.print",
        {
            "context": new_tab["context"],
        },
    )

    await bidi_session.browsing_context.close(context=new_tab["context"])

    # Closing the browsing context for printing may leave the print-preview
    # window open. Make sure that creating a new tab doesn’t reference this
    # window, which could cause failures.
    await bidi_session.browsing_context.create(type_hint="tab")

    with pytest.raises(error.UnknownErrorException):
        await wait_for_future_safe(printed_future)
