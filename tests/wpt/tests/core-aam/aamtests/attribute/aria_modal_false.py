# Testing: https://w3c.github.io/core-aam/#ariaModalFalse

TEST_HTML = "<div role='group' id='group'>content</div> <div role='dialog' id='test' aria-modal='false'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # State: STATE_MODAL: not exposed

    node = atspi.find_node("test", session.url)
    assert "STATE_MODAL" not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Grow the accessibility tree such that the background content is exposed. No specific property is set on the <a class="termref">accessible object</a> that corresponds to the <a class="termref">element</a> with <code>aria-modal="false"</code>.

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # State: IA2_STATE_MODAL: not exposed

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Window.IsModal: false
