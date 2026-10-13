# Testing: https://w3c.github.io/core-aam/#ariaValueText

TEST_HTML = "<div role='slider' id='test' aria-valuetext='hello world'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Object Attribute: valuetext:<value>

    node = atspi.find_node("test", session.url)
    assert "valuetext:hello world" in atspi.Accessible.get_attributes_as_array(node)

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXValueDescription: <value>
#     # See also: Handling Author Errors for States and Properties

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Method: IAccessible::get_accValue(): <value>
#     # Object Attribute: valuetext:<value>
#     # See also: Handling Author Errors for States and Properties

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: Value.Value: <value>
#     # See also: Handling Author Errors for States and Properties
