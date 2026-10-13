# Testing: https://w3c.github.io/core-aam/#ariaFlowto

TEST_HTML = "<div role='group' id='test' aria-flowto='next'>content</div> <div role='group' id='next'>content</div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Relation: RELATION_FLOWS_TO: points to accessible nodes matching IDREFs
    # Reverse Relation: RELATION_FLOWS_FROM: points to element

    node = atspi.find_node("test", session.url)
    relations = atspi.get_relations_dictionary_helper(node)
    assert 'RELATION_FLOWS_TO' in relations
    assert 'next' in relations['RELATION_FLOWS_TO']
    reverse_node = atspi.find_node('next', session.url)
    reverse_relations = atspi.get_relations_dictionary_helper(reverse_node)
    assert 'RELATION_FLOWS_FROM' in reverse_relations
    assert 'test' in reverse_relations['RELATION_FLOWS_FROM']

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXLinkedUIElements: pointers to accessible nodes matching IDREFs

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Relation: IA2_RELATION_FLOW_TO: points to accessible nodes matching IDREFs
#     # Reverse Relation: IA2_RELATION_FLOW_FROM: points to element
#     # See also: Mapping Additional Relations

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: FlowsTo: pointers to accessible nodes matching IDREFs
