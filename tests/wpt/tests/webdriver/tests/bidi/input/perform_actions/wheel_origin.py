import pytest

from webdriver.bidi.error import NoSuchElementException
from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget


pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "expression",
    [
        "document.querySelector('input#button').attributes[0]",
        "document.querySelector('#with-text-node').childNodes[0]",
        """document.createProcessingInstruction("xml-stylesheet", "href='foo.css'")""",
        "document.querySelector('#with-comment').childNodes[0]",
        "document",
        "document.doctype",
        "document.createDocumentFragment()",
        "document.querySelector('#custom-element').shadowRoot",
    ],
    ids=[
        "attribute",
        "text node",
        "processing instruction",
        "comment",
        "document",
        "doctype",
        "document fragment",
        "shadow root",
    ]
)
async def test_params_actions_origin_no_such_element(
    bidi_session, top_context, get_test_page, expression
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_test_page(),
        wait="complete",
    )

    node = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    actions = Actions()
    actions.add_wheel().scroll(x=0, y=0, delta_x=5, delta_y=10, origin=get_element_origin(node))

    with pytest.raises(NoSuchElementException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )
