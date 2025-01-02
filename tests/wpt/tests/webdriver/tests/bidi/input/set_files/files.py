import pytest
from webdriver.bidi.error import UnsupportedOperationException

from .. import get_events

pytestmark = pytest.mark.asyncio


async def test_set_files(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input")

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["path/to/noop.txt"]),
    )

    events = await get_events(bidi_session, top_context["context"])
    assert events == [
        {
            "files": [
                "noop.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "change",
        },
    ]


async def test_set_files_empty(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input")

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=[],
    )

    events = await get_events(bidi_session, top_context["context"])
    assert events == [
        {
            "files": [],
            "type": "cancel",
        },
    ]


async def test_set_files_multiple(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input-multiple")

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["path/to/noop.txt", "path/to/noop-2.txt"]),
    )

    events = await get_events(bidi_session, top_context["context"])
    assert events == [
        {
            "files": [
                "noop.txt",
                "noop-2.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
                "noop-2.txt",
            ],
            "type": "change",
        },
    ]


async def test_set_files_something_then_empty(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input")

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["path/to/noop.txt"]),
    )

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=[],
    )

    events = await get_events(bidi_session, top_context["context"])
    assert events == [
        {
            "files": [
                "noop.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "change",
        },
        {
            "files": [],
            "type": "input",
        },
        {
            "files": [],
            "type": "change",
        },
    ]


async def test_set_files_twice(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input")

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["path/to/noop.txt"]),
    )

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["path/to/noop-2.txt"]),
    )

    events = await get_events(bidi_session, top_context["context"])
    assert events == [
        {
            "files": [
                "noop.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "change",
        },
        {
            "files": [
                "noop-2.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop-2.txt",
            ],
            "type": "change",
        },
    ]


async def test_set_files_twice_intersected(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input-multiple")

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["noop.txt"]),
    )

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["noop.txt", "noop-2.txt"]),
    )

    events = await get_events(bidi_session, top_context["context"])
    assert events == [
        {
            "files": [
                "noop.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "change",
        },
        {
            "files": [
                "noop.txt",
                "noop-2.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
                "noop-2.txt",
            ],
            "type": "change",
        },
    ]


async def test_set_files_twice_same(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input")

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["path/to/noop.txt"]),
    )

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["path/to/noop.txt"]),
    )

    events = await get_events(bidi_session, top_context["context"])
    assert events == [
        {
            "files": [
                "noop.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "change",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "cancel",
        },
    ]


async def test_set_files_twice_same_in_different_folders(
    bidi_session,
    top_context,
    load_static_test_page,
    get_element,
    create_files
):
    await load_static_test_page(page="files.html")

    element = await get_element("#input")

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["path/to/noop.txt"]),
    )

    await bidi_session.input.set_files(
        context=top_context["context"],
        element=element,
        files=create_files(["different/to/noop.txt"]),
    )

    events = await get_events(bidi_session, top_context["context"])
    assert events == [
        {
            "files": [
                "noop.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "change",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "input",
        },
        {
            "files": [
                "noop.txt",
            ],
            "type": "change",
        },
    ]


async def test_non_existent_file(
    bidi_session, top_context, load_static_test_page, get_element
):
    file = "non_existent_file.txt"

    await load_static_test_page(page="files.html")
    element = await get_element("#input")

    # Firefox is unable to set non-existent files.
    if bidi_session.capabilities.get("browserName") == "firefox":
        with pytest.raises(UnsupportedOperationException):
            await bidi_session.input.set_files(
                context=top_context["context"],
                element=element,
                files=[file],
            )
    else:
        await bidi_session.input.set_files(
            context=top_context["context"],
            element=element,
            files=[file],
        )

        events = await get_events(bidi_session, top_context["context"])
        assert events == [
            {
                "files": [file],
                "type": "input",
            },
            {
                "files": [file],
                "type": "change",
            },
        ]
