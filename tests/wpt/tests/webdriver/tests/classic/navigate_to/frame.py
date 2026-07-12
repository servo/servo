from tests.support.classic.asserts import assert_success

from . import navigate_to


def test_subframe_replacestate(session, inline):
    script_url = "/webdriver/tests/bidi/browsing_context/support/empty.js"
    # An iframe calling history.replaceState on itself triggers a same-document
    # location change. This must not cause the navigate command to complete
    # prematurely. The slow parser-blocking script delays the page load so that
    # premature completion is detectable via document.readyState.
    iframe_url = inline('<script>history.replaceState(null, "", "#hash");</script>')
    url = inline(
        f'<iframe src="{iframe_url}"></iframe>'
        f'<script src="{script_url}?pipe=trickle(d1)"></script>'
    )

    response = navigate_to(session, url)
    assert_success(response)

    assert session.url == url
    assert session.execute_script("return document.readyState") == "complete"


def test_subframe_error_page(session, inline):
    # An iframe served with X-Frame-Options: DENY will fail to load and show
    # an error page. This must not cause the navigate command to fail.
    iframe_url = inline(
        "<div>foo</div>",
        parameters={"pipe": "header(X-Frame-Options, DENY)"},
    )
    url = inline(f'<iframe src="{iframe_url}"></iframe>')

    response = navigate_to(session, url)
    assert_success(response)

    assert session.url == url
    assert session.execute_script("return document.readyState") == "complete"
