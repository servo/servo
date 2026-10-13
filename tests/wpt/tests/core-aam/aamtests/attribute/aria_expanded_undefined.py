# Testing: https://w3c.github.io/core-aam/#ariaExpandedUndefined

TEST_HTML = "<div role='button' id='test' aria-expanded=''>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Not mapped

    node = atspi.find_node("test", session.url)
    assert 'STATE_EXPANDABLE' not in atspi.get_state_list_helper(node)
    assert 'STATE_EXPANDED' not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Not mapped
