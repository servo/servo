# Testing: https://w3c.github.io/core-aam/#ariaLabel

TEST_HTML = "<div role='group' id='test' aria-label='hello world'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Property: Name: <value>

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_name(node) == "hello world"

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXTitle: <value>
#     # See also: Name Computation

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: accName: <value>
#     # See also: Name Computation

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Name: <value>
#     # See also: Name Computation
