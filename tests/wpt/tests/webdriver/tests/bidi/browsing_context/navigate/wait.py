import pytest
import asyncio

pytestmark = pytest.mark.asyncio


async def wait_for_navigation(bidi_session, context, url, wait, expect_timeout):
    # Ultimately, "interactive" and "complete" should support a timeout argument.
    # See https://github.com/w3c/webdriver-bidi/issues/188.
    if expect_timeout:
        with pytest.raises(asyncio.TimeoutError):
            await asyncio.wait_for(
                asyncio.shield(bidi_session.browsing_context.navigate(
                    context=context, url=url, wait=wait
                )),
                timeout=1,
            )
    else:
        await bidi_session.browsing_context.navigate(
            context=context, url=url, wait=wait
        )


@pytest.mark.parametrize("value", ["none", "interactive", "complete"])
async def test_expected_url(bidi_session, inline, new_tab, value):
    url = inline("<div>foo</div>")
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait=value
    )
    assert result["url"] == url
    if value != "none":
        contexts = await bidi_session.browsing_context.get_tree(
            root=new_tab["context"], max_depth=0
        )
        assert contexts[0]["url"] == url


@pytest.mark.parametrize(
    "wait, expect_timeout",
    [
        ("none", False),
        ("interactive", False),
        ("complete", True),
    ],
)
async def test_slow_image_blocks_load(bidi_session, inline, new_tab, wait, expect_timeout):
    image_url = "/webdriver/tests/bidi/browsing_context/support/empty.svg"
    url = inline(f"<img src='{image_url}?pipe=trickle(d10)'>")

    await wait_for_navigation(bidi_session, new_tab["context"], url, wait, expect_timeout)

    # We cannot assert the URL for "none" by definition, and for "complete", since
    # we expect a timeout. For the timeout case, the wait_for_navigation helper will
    # resume  after 1 second, there is no guarantee that the URL has been updated.
    if wait == "interactive":
        contexts = await bidi_session.browsing_context.get_tree(
            root=new_tab["context"], max_depth=0
        )
        assert contexts[0]["url"] == url


@pytest.mark.parametrize(
    "wait, expect_timeout",
    [
        ("none", False),
        ("interactive", True),
        ("complete", True),
    ],
)
async def test_slow_page(bidi_session, new_tab, url, wait, expect_timeout):
    page_url = url(
        "/webdriver/tests/bidi/browsing_context/support/empty.html?pipe=trickle(d10)"
    )

    await wait_for_navigation(bidi_session, new_tab["context"], page_url, wait, expect_timeout)

    # Note that we cannot assert the top context url here, because the navigation
    # is blocked on the initial url for this test case.


@pytest.mark.parametrize(
    "wait, expect_timeout",
    [
        ("none", False),
        ("interactive", True),
        ("complete", True),
    ],
)
async def test_slow_script_blocks_domContentLoaded(bidi_session, inline, new_tab, wait, expect_timeout):
    script_url = "/webdriver/tests/bidi/browsing_context/support/empty.js"
    url = inline(f"<script src='{script_url}?pipe=trickle(d10)'></script>")

    await wait_for_navigation(bidi_session, new_tab["context"], url, wait, expect_timeout)

    # In theory we could also assert the top context URL has been updated here
    # but since we expect both "interactive" and "complete" to timeout, the
    # wait_for_navigation helper will resume arbitrarily after 1 second, and
    # there is no guarantee that the URL has been updated.
