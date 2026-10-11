# Testing: https://w3c.github.io/aria/#aria-errormessage
# User agents MUST NOT expose aria-errormessage for an object with an aria-invalid value of false.
# See: https://w3c.github.io/aria/html-aam/#att-pattern
# If the value doesn't match the pattern: aria-invalid="true"; Otherwise, aria-invalid="false"

import pytest

TEST_HTML = {
    "explicit": (
        "<input id='target' aria-invalid='true' aria-errormessage='error' /><span id='error'>Error</span>",
        "target.ariaInvalid = 'false'",
    ),
    "implicit": (
        "<input id='target' pattern='[a-z]' value='A' aria-errormessage='error' /><span id='error'>Error</span>",
        "target.value = 'a'",
    ),
}

@pytest.mark.parametrize("test_html,script", TEST_HTML.values(), ids=TEST_HTML.keys())
def test_atspi(atspi, session, inline, test_html, script):
    session.url = inline(test_html)

    node = atspi.find_node("target", session.url)
    relations = atspi.get_relations_dictionary_helper(node)
    assert 'RELATION_ERROR_MESSAGE' in relations
    assert 'error' in relations['RELATION_ERROR_MESSAGE']

    session.execute_script(script)

    def relation_removed():
        relations = atspi.get_relations_dictionary_helper(node)
        return 'RELATION_ERROR_MESSAGE' not in relations

    atspi._poll_for(
        relation_removed,
        f"Timed out waiting for RELATION_ERROR_MESSAGE to be removed after '{script}'"
    )

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
