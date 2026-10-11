# Testing: https://w3c.github.io/aria/#aria-errormessage
# User agents MUST NOT expose aria-errormessage for an object with an aria-invalid value of false.
# See: https://w3c.github.io/aria/html-aam/#att-pattern
# If the value doesn't match the pattern: aria-invalid="true"; Otherwise, aria-invalid="false"

import pytest

TEST_HTML = {
    "explicit": "<input id='target' aria-invalid='false' aria-errormessage='error' /><span id='error'>Error</span>",
    "implicit": "<input id='target' pattern='[a-z]' value='a' aria-errormessage='error' /><span id='error'>Error</span>",
}

@pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
def test_atspi(atspi, session, inline, test_html):
    session.url = inline(test_html)

    node = atspi.find_node("target", session.url)
    relations = atspi.get_relations_dictionary_helper(node)
    assert 'RELATION_ERROR_MESSAGE' not in relations

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
