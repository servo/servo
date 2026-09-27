import pytest

# Testing: https://w3c.github.io/mathml-aam/#ma-element
TEST_CASES = {
    "mathml-link": (
        '<math><a href="https://baidu.com" id="test"><mtext>Link</mtext></a></math>'
    ),
    "mathml-link-without-href": (
        '<math><a id="test"><mtext>Link</mtext></a></math>'
    ),
    # Browser heuristic behavior: treats 'a' as a Link if it has mouse event listeners
    # Spec issue: https://github.com/w3c/mathml-aam/issues/39
    "mathml-link-with-onclick": (
        '<math><a id="test" onclick="void(0)"><mtext>Link</mtext></a></math>'
    ),
}

@pytest.mark.parametrize("test_id, test_html", TEST_CASES.items(), ids=TEST_CASES.keys())
class TestMathMLALink:

    def test_atspi(self, atspi, session, inline, test_id, test_html):
        session.url = inline(test_html)
        node = atspi.find_node("test", session.url)

        obj_attrs = atspi.Accessible.get_attributes(node)
        assert obj_attrs.get("tag") == "a"

        if test_id == "mathml-link":
            assert atspi.Accessible.get_role(node) == atspi.Role.LINK
            hyperlink = atspi.Accessible.get_hyperlink(node)
            assert hyperlink is not None, "hyperlink interface should be present for link role"
            assert hyperlink.get_uri(0) == "https://baidu.com/"

        elif test_id == "mathml-link-with-onclick":
            role = atspi.Accessible.get_role(node)
            assert role == atspi.Role.LINK

        elif test_id == "mathml-link-without-href":
            assert atspi.Accessible.get_role(node) == atspi.Role.SECTION

        else:
            raise ValueError(f"Unreachable code: missing ATSPI assertions for {test_id}")

    def test_axapi(self, axapi, session, inline, test_id, test_html):
        session.url = inline(test_html)
        node = axapi.find_node("test", session.url)

        role = axapi.AXUIElementCopyAttributeValue(node, "AXRole", None)[1]

        subrole = axapi.AXUIElementCopyAttributeValue(node, "AXSubrole", None)[1]

        if test_id == "mathml-link":
            assert role == "AXLink"
            assert subrole == "AXMathRow"

        elif test_id == "mathml-link-with-onclick":
            assert role == "AXLink"
            assert subrole == "AXMathRow"

        elif test_id == "mathml-link-without-href":
            assert role == "AXGroup"
        else:
            raise ValueError(f"Unreachable code: missing AXAPI assertions for {test_id}")
