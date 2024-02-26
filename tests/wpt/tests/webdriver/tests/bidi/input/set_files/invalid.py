import pytest

from webdriver.bidi.error import (
    InvalidArgumentException,
    NoSuchElementException,
    NoSuchFrameException,
    UnableToSetFileInputException,
)

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


@pytest.mark.parametrize(
    "files",
    [
        ["path/to/noop.txt"],
        [],
    ],
)
async def test_params_element_invalid_value(
    bidi_session,
    top_context,
    load_static_test_page,
    files,
):
    await load_static_test_page(page="files.html")

    with pytest.raises(NoSuchElementException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element={"sharedId": "invalid"},
            files=files,
        )


@pytest.mark.parametrize(
    "files",
    [
        ["path/to/noop.txt"],
        [],
    ],
)
async def test_params_element_disabled(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    files,
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input-disabled")

    with pytest.raises(UnableToSetFileInputException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=files,
        )


@pytest.mark.parametrize(
    "files",
    [
        ["path/to/noop.txt"],
        [],
    ],
)
async def test_params_element_non_file_input(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    files,
):
    await load_static_test_page(page="files.html")

    element = await get_element("#text-input")

    with pytest.raises(UnableToSetFileInputException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=files,
        )


async def test_params_element_not_multiple(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input")

    with pytest.raises(UnableToSetFileInputException):
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=["path/to/noop.txt", "path/to/noop-2.txt"],
        )
