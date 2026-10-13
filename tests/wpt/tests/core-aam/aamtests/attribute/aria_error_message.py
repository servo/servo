# Testing: https://w3c.github.io/core-aam/#ariaErrorMessage

TEST_HTML = "<div role='checkbox' id='test' aria-errormessage='error' aria-invalid='true'>content</div> <div id='error'>hello world</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Relation: RELATION_ERROR_MESSAGE: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
    # Reverse Relation: RELATION_ERROR_FOR: points to element

    node = atspi.find_node("test", session.url)
    relations = atspi.get_relations_dictionary_helper(node)
    assert 'RELATION_ERROR_MESSAGE' in relations
    assert 'error' in relations['RELATION_ERROR_MESSAGE']
    reverse_node = atspi.find_node('error', session.url)
    reverse_relations = atspi.get_relations_dictionary_helper(reverse_node)
    assert 'RELATION_ERROR_FOR' in reverse_relations
    assert 'test' in reverse_relations['RELATION_ERROR_FOR']

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXErrorMessageElements: pointers to accessible nodes matching IDREFs

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Relation: IA2_RELATION_ERROR: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
#     # Reverse Relation: IA2_RELATION_ERROR_FOR: points to element
#     # See also: Mapping Additional Relations

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: ControllerFor: pointer to the target accessible object
