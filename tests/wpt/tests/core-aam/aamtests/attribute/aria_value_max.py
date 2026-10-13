# Testing: https://w3c.github.io/core-aam/#ariaValueMax

TEST_HTML = "<div role='scrollbar' id='test' aria-valuemax='10'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Method: atk_value_get_maximum_value(): <value>

    node = atspi.find_node("test", session.url)
    assert atspi.Value.get_maximum_value(node) == 10

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXMaxValue: <value>
#     # See also: Handling Author Errors for States and Properties

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Method: IAccessibleValue::maximumValue(): <value>
#     # See also: Handling Author Errors for States and Properties

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: RangeValue.Maximum: <value>
#     # See also: Handling Author Errors for States and Properties
