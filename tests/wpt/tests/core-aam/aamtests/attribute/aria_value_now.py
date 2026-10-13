# Testing: https://w3c.github.io/core-aam/#ariaValueNow

TEST_HTML = "<div role='scrollbar' id='test' aria-valuenow='5'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Method: atk_value_get_current_value(): <value>

    node = atspi.find_node("test", session.url)
    assert atspi.Value.get_current_value(node) == 5

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXValue: <value>
#     # See also: Handling Author Errors for States and Properties

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Method: IAccessibleValue::currentValue(): <value>
#     # Method: IAccessible::get_accValue(): <value> if aria-valuetext is not defined
#     # See also: Handling Author Errors for States and Properties

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: RangeValue.Value: <value>
#     # See also: Handling Author Errors for States and Properties
