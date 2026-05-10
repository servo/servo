import pytest

# Testing: https://w3c.github.io/core-aam/#ariaAutocompleteInlineListBoth

TEST_HTML = {
    "both": "<input role='combobox' id='test' aria-autocomplete='both'>",
    "inline": "<input role='combobox' id='test' aria-autocomplete='inline'>",
    "list": "<input role='combobox' id='test' aria-autocomplete='list'>",
}

@pytest.mark.parametrize("test_value,test_html", TEST_HTML.items(), ids=TEST_HTML.keys())
def test_atspi(atspi, session, inline, test_value, test_html):
    session.url = inline(test_html)

    # Spec:
    # Object Attribute: autocomplete:<value>
    # State: STATE_SUPPORTS_AUTOCOMPLETION

    node = atspi.find_node("test", session.url)
    assert f"autocomplete:{test_value}" in atspi.Accessible.get_attributes_as_array(node)
    assert "STATE_SUPPORTS_AUTOCOMPLETION" in atspi.get_state_list_helper(node)

# Intentionally no AX API test. AX API does not surface this attribute.

# @pytest.mark.parametrize("test_value,test_html", TEST_HTML.items(), ids=TEST_HTML.keys())
# def test_ia2(ia2, session, inline, test_value, test_html):
#     session.url = inline(test_html)
#
#     # Spec:
#     # Object Attribute: autocomplete:<value>
#     # State: IA2_STATE_SUPPORTS_AUTOCOMPLETION

# Intentionally no UIA test. UIA does not surface this attribute.
