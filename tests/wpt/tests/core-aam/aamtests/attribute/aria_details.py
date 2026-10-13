# Testing: https://w3c.github.io/core-aam/#ariaDetails

TEST_HTML = "<div role='group' id='test' aria-details='details'>content</div> <div id='details'>hello world</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Relation: RELATION_DETAILS: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
    # Reverse Relation: RELATION_DETAILS_FOR: points to element

    node = atspi.find_node("test", session.url)
    relations = atspi.get_relations_dictionary_helper(node)
    assert 'RELATION_DETAILS' in relations
    assert 'details' in relations['RELATION_DETAILS']
    reverse_node = atspi.find_node('details', session.url)
    reverse_relations = atspi.get_relations_dictionary_helper(reverse_node)
    assert 'RELATION_DETAILS_FOR' in reverse_relations
    assert 'test' in reverse_relations['RELATION_DETAILS_FOR']

# Intentionally no AX API test. AX API does not surface this node or attribute.

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Relation: IA2_RELATION_DETAILS: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
#     # Reverse Relation: IA2_RELATION_DETAILS_FOR: points to element
#     # See also: Mapping Additional Relations

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: DescribedBy: points to accessible nodes matching IDREFs, if the referenced objects are in the accessibility tree
