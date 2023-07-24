import pytest

from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.error import (
    InvalidArgumentException,
    MoveTargetOutOfBoundsException,
    NoSuchElementException,
    NoSuchFrameException,
    NoSuchNodeException,
)
from webdriver.bidi.modules.script import ContextTarget


pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, True, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, value):
    actions = Actions()
    actions.add_key()
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.perform_actions(actions=actions, context=value)


async def test_params_contexts_value_invalid_value(bidi_session):
    actions = Actions()
    actions.add_key()
    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.perform_actions(actions=actions, context="foo")


@pytest.mark.parametrize(
    "value",
    [("fa"), ("\u0BA8\u0BBFb"), ("\u0BA8\u0BBF\u0BA8"), ("\u1100\u1161\u11A8c")],
)
async def test_params_actions_invalid_value_multiple_codepoints(
    bidi_session, top_context, setup_key_test, value
):
    actions = Actions()
    actions.add_key().key_down(value).key_up(value)
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


@pytest.mark.parametrize("missing", ["x", "y"])
async def test_params_actions_missing_coordinates(bidi_session, top_context, missing):
    actions = Actions()
    actions.add_pointer().pointer_move(x=0, y=0)

    json_actions = actions.to_json()
    pointer_input_source_json = json_actions[-1]["actions"]
    del pointer_input_source_json[-1][missing]

    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.perform_actions(
            actions=json_actions, context=top_context["context"]
        )


@pytest.mark.parametrize("missing", ["x", "y", "deltaX", "deltaY"])
async def test_params_actions_missing_wheel_property(
    bidi_session, top_context, missing
):
    actions = Actions()
    actions.add_wheel().scroll(x=0, y=0, delta_x=5, delta_y=10)

    json_actions = actions.to_json()
    wheel_input_actions_json = json_actions[-1]["actions"]
    del wheel_input_actions_json[-1][missing]

    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.perform_actions(
            actions=json_actions, context=top_context["context"]
        )


async def test_params_actions_origin_element_outside_viewport(
    bidi_session, top_context, get_actions_origin_page, get_element
):
    url = get_actions_origin_page(
        """width: 100px; height: 50px; background: green;
           position: relative; left: -200px; top: -100px;"""
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    elem = await get_element("#inner")

    actions = Actions()
    actions.add_pointer().pointer_move(x=0, y=0, origin=get_element_origin(elem))
    with pytest.raises(MoveTargetOutOfBoundsException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


@pytest.mark.parametrize("value", [True, 42, []])
async def test_params_actions_origin_invalid_type(bidi_session, top_context, value):
    actions = Actions()
    actions.add_pointer().pointer_move(x=0, y=0, origin=value)
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


@pytest.mark.parametrize("value", [None, True, 42, {}, [], "foo"])
async def test_params_actions_origin_invalid_value_type(
    bidi_session, top_context, get_actions_origin_page, get_element, value
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_actions_origin_page(""),
        wait="complete",
    )

    elem = await get_element("#inner")
    actions = Actions()
    actions.add_pointer().pointer_move(
        x=0, y=0, origin={"type": value, "element": {"sharedId": elem["sharedId"]}}
    )
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


@pytest.mark.parametrize("value", [None, True, 42, {}, [], "foo"])
async def test_params_actions_origin_invalid_value_element(
    bidi_session, top_context, value
):
    actions = Actions()
    actions.add_pointer().pointer_move(
        x=0, y=0, origin={"type": "element", "element": value}
    )
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


async def test_params_actions_origin_invalid_value_serialized_element(
    bidi_session, top_context, get_actions_origin_page, get_element
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_actions_origin_page(""),
        wait="complete",
    )

    elem = await get_element("#inner")

    actions = Actions()
    actions.add_pointer().pointer_move(x=0, y=0, origin=elem)
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


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
    actions.add_pointer().pointer_move(x=0, y=0, origin=get_element_origin(node))
    with pytest.raises(NoSuchElementException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


async def test_params_actions_origin_no_such_node(bidi_session, top_context):
    actions = Actions()
    actions.add_pointer().pointer_move(
        x=0, y=0, origin={"type": "element", "element": {"sharedId": "foo"}}
    )
    with pytest.raises(NoSuchNodeException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


@pytest.mark.parametrize("origin", ["viewport", "pointer"])
async def test_params_actions_origin_outside_viewport(
    bidi_session, top_context, origin
):
    actions = Actions()
    actions.add_pointer().pointer_move(x=-50, y=-50, origin=origin)
    with pytest.raises(MoveTargetOutOfBoundsException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )
