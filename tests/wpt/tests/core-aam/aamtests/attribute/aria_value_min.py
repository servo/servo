# Testing: https://w3c.github.io/core-aam/#ariaValueMin

TEST_HTML = "<div role='scrollbar' id='test' aria-valuemin='2'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Method: atk_value_get_minimum_value(): <value>

    node = atspi.find_node("test", session.url)
    assert atspi.Value.get_minimum_value(node) == 2

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXMinValue: <value>
#     # See also: Handling Author Errors for States and Properties

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Method: IAccessibleValue::minimumValue(): <value>
#     # See also: Handling Author Errors for States and Properties

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: RangeValue.Minimum: <value>
#     # See also: Handling Author Errors for States and Properties
