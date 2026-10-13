# Testing: https://w3c.github.io/core-aam/#ariaSelectedUndefined

TEST_HTML = "<div role='grid'> <div role='row'> <div role='gridcell' id='test' aria-selected=''>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Not mapped

    node = atspi.find_node("test", session.url)
    assert 'STATE_SELECTABLE' not in atspi.get_state_list_helper(node)
    assert 'STATE_SELECTED' not in atspi.get_state_list_helper(node)

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
