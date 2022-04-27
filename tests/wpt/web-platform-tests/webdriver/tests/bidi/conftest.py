import pytest


@pytest.fixture
async def new_tab(bidi_session, current_session):
    # Open and focus a new tab to run the test in a foreground tab.
    context_id = current_session.new_window(type_hint="tab")
    initial_window = current_session.window_handle
    current_session.window_handle = context_id

    # Retrieve the browsing context info for the new tab
    contexts = await bidi_session.browsing_context.get_tree(root=context_id, max_depth=0)
    yield contexts[0]

    # Restore the focus and current window for the WebDriver session before
    # closing the tab.
    current_session.window_handle = initial_window
    await bidi_session.browsing_context.close(context=contexts[0]["context"])


@pytest.fixture
def test_page(inline):
    return inline("<div>foo</div>")


@pytest.fixture
def test_page2(inline):
    return inline("<div>bar</div>")


@pytest.fixture
def test_page_cross_origin(inline):
    return inline("<div>bar</div>", domain="alt")


@pytest.fixture
def test_page_multiple_frames(inline, test_page, test_page2):
    return inline(f"<iframe src='{test_page}'></iframe><iframe src='{test_page2}'></iframe>")


@pytest.fixture
def test_page_nested_frames(inline, test_page_same_origin_frame):
    return inline(f"<iframe src='{test_page_same_origin_frame}'></iframe>")


@pytest.fixture
def test_page_cross_origin_frame(inline, test_page_cross_origin):
    return inline(f"<iframe src='{test_page_cross_origin}'></iframe>")


@pytest.fixture
def test_page_same_origin_frame(inline, test_page):
    return inline(f"<iframe src='{test_page}'></iframe>")


@pytest.fixture
async def top_context(bidi_session):
    contexts = await bidi_session.browsing_context.get_tree()
    return contexts[0]
