# Testing: https://w3c.github.io/core-aam/#ariaControls

TEST_HTML = "<div role='combobox' id='test' aria-controls='list'> <div role='textbox'>content</div> </div> <div role='listbox' id='list'> <div role='option'>content</div> </div>"

def test_atspi(atspi, session, inline):
    session.url = inline(TEST_HTML)

    # Spec:
    # Relation: RELATION_CONTROLLER_FOR: points to accessible nodes matching IDREFs
    # Reverse Relation: RELATION_CONTROLLED_BY: points to element

    node = atspi.find_node("test", session.url)
    relations = atspi.get_relations_dictionary_helper(node)
    assert 'RELATION_CONTROLLER_FOR' in relations
    assert 'list' in relations['RELATION_CONTROLLER_FOR']
    reverse_node = atspi.find_node('list', session.url)
    reverse_relations = atspi.get_relations_dictionary_helper(reverse_node)
    assert 'RELATION_CONTROLLED_BY' in reverse_relations
    assert 'test' in reverse_relations['RELATION_CONTROLLED_BY']

# def test_axapi(axapi, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: AXLinkedUIElements: pointers to accessible nodes matching IDREFs

# def test_ia2(ia2, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Relation: IA2_RELATION_CONTROLLER_FOR: points to accessible nodes matching IDREFs
#     # Reverse Relation: IA2_RELATION_CONTROLLED_BY: points to element
#     # See also: Mapping Additional Relations

# def test_uia(uia, session, inline):
#     session.url = inline(TEST_HTML)
#
#     # Spec:
#     # Property: ControllerFor: pointers to accessible nodes matching IDREFs
