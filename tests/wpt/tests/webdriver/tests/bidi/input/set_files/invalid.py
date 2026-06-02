import pytest

from webdriver.bidi.error import (
    InvalidArgumentException,
    NoSuchElementException,
    NoSuchFrameException,
    NoSuchNodeException,
    UnableToSetFileInputException,
)
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, True, 42, {}, []])
async def test_params_context_invalid_type(
    bidi_session, load_static_test_page, get_element, value
):
    await load_static_test_page(page="files.html")
    element = await get_element("#input")
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.set_files(
            context=value,
            element=element,
            files=[],
        )


async def test_params_context_invalid_value(
    bidi_session, load_static_test_page, get_element
):
    await load_static_test_page(page="files.html")
    element = await get_element("#input")
    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.set_files(
            context="foo",
            element=element,
            files=[],
        )


@pytest.mark.parametrize("value", [None, True, 42, []])
async def test_params_element_invalid_type(bidi_session, top_context, value):
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=value,
            files=[],
        )


@pytest.mark.parametrize("value", [None, True, 42, {}])
async def test_params_files_invalid_type(
    top_context, bidi_session, load_static_test_page, get_element, value
):
    await load_static_test_page(page="files.html")
    element = await get_element("#input")
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=value,
        )


@pytest.mark.parametrize("value", [None, True, 42, {}, []])
async def test_params_files_file_invalid_type(
    top_context, bidi_session, load_static_test_page, get_element, value
):
    await load_static_test_page(page="files.html")
    element = await get_element("#input")
    with pytest.raises(InvalidArgumentException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=[value],
        )


async def test_params_element_invalid_shared_reference_value(
    bidi_session,
    top_context,
    load_static_test_page,
    create_files,
):
    await load_static_test_page(page="files.html")

    with pytest.raises(NoSuchNodeException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element={"sharedId": "invalid"},
            files=create_files(["path/to/noop.txt"]),
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
async def test_params_element_invalid_element(
    bidi_session,
    top_context,
    get_test_page,
    create_files,
    expression
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

    with pytest.raises(NoSuchElementException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=node,
            files=create_files(["path/to/noop.txt"]),
        )


async def test_params_element_disabled(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input-disabled")

    with pytest.raises(UnableToSetFileInputException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=create_files(["path/to/noop.txt"]),
        )


@pytest.mark.parametrize(
    "html",
    [
        "<div id='test'>foo</div>",
        "<a id='test' href='#'>foo</a>",
        "<span id='test' href='#'>foo</span>",
    ],
)
async def test_params_element_non_input(
    bidi_session,
    top_context,
    inline,
    get_element,
    create_files,
    html,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(html),
        wait="complete",
    )

    element = await get_element("#test")

    with pytest.raises(UnableToSetFileInputException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=create_files(["path/to/noop.txt"]),
        )


async def test_params_element_non_file_input(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files,
):
    await load_static_test_page(page="files.html")

    element = await get_element("#text-input")

    with pytest.raises(UnableToSetFileInputException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=create_files(["path/to/noop.txt"]),
        )


async def test_params_element_not_multiple(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input")

    with pytest.raises(UnableToSetFileInputException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=create_files(["path/to/noop.txt", "path/to/noop-2.txt"]),
        )
