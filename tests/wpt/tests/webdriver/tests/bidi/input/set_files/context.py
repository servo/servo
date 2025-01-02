import pytest

from .. import get_events

pytestmark = pytest.mark.asyncio


async def test_set_files_сontext(
    bidi_session, top_context, new_tab, load_static_test_page, get_element, create_files
):
    await load_static_test_page(page="files.html")
    await load_static_test_page(page="files.html", context=new_tab)

    element = await get_element("#input", context=new_tab)

    await bidi_session.input.set_files(
        context=new_tab["context"],
        element=element,
        files=create_files(["noop.txt"]),
    )

    events_in_new_tab = await get_events(bidi_session, new_tab["context"])
    assert events_in_new_tab == [
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

    # Make sure that events are not sent in the other context.
    events_in_top_context = await get_events(bidi_session, top_context["context"])
    assert events_in_top_context == []


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_set_files_in_iframe(
    bidi_session, new_tab, get_element, inline, url, create_files, domain
):
    iframe_url = url("/webdriver/tests/support/html/files.html")
    page_url = inline(f"<iframe src='{iframe_url}'></iframe>", domain=domain)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=page_url,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]

    element = await get_element("#input", context=iframe_context)

    await bidi_session.input.set_files(
        context=iframe_context["context"],
        element=element,
        files=create_files(["noop.txt"]),
    )

    events_in_new_tab = await get_events(bidi_session, iframe_context["context"])
    assert events_in_new_tab == [
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
