# Testing: https://w3c.github.io/core-aam/#ariaReadonlyUnspecifiedOnGridcell

TEST_HTML = "<div role='grid' aria-readonly='true'> <div role='row'> <div role='gridcell' id='test'>content</div> </div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # The <code>gridcell</code> MUST inherit any author-provided value for <code>aria-readonly</code> from the containing <code>grid</code> or <code>treegrid</code>. Expose the inherited value on the <code>gridcell</code> as described for <a href="#ariaReadonlyTrue"><code>aria-readonly="true"</code></a> and <a href="#ariaReadonlyFalse"><code>aria-readonly="false"</code></a>.

    node = atspi.find_node("test", session.url)
    assert 'STATE_READ_ONLY' in atspi.get_state_list_helper(node)
    assert 'STATE_EDITABLE' not in atspi.get_state_list_helper(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # The <code>gridcell</code> MUST inherit any author-provided value for <code>aria-readonly</code> from the containing <code>grid</code> or <code>treegrid</code>. Expose the inherited value on the <code>gridcell</code> as described for <a href="#ariaReadonlyTrue"><code>aria-readonly="true"</code></a> and <a href="#ariaReadonlyFalse"><code>aria-readonly="false"</code></a>.

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # The <code>gridcell</code> MUST inherit any author-provided value for <code>aria-readonly</code> from the containing <code>grid</code> or <code>treegrid</code>. Expose the inherited value on the <code>gridcell</code> as described for <a href="#ariaReadonlyTrue"><code>aria-readonly="true"</code></a> and <a href="#ariaReadonlyFalse"><code>aria-readonly="false"</code></a>.

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # The <code>gridcell</code> MUST inherit any author-provided value for <code>aria-readonly</code> from the containing <code>grid</code> or <code>treegrid</code>. Expose the inherited value on the <code>gridcell</code> as described for <a href="#ariaReadonlyTrue"><code>aria-readonly="true"</code></a> and <a href="#ariaReadonlyFalse"><code>aria-readonly="false"</code></a>.
