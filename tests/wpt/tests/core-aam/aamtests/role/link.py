# Testing: https://w3c.github.io/core-aam/#role-map-link

import pytest

TEST_HTML = {
    "role-link": "<div id=test role=link>click me</div>",
    "aria-disabled": "<div id=test role=link aria-disabled=true>click me</div>",
    "aria-expanded": "<div id=test role=link aria-expanded=false>click me</div>",
}

@pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
def test_atspi(atspi, session, inline, test_html):
    session.url = inline(test_html)

    node = atspi.find_node("test", session.url)

    assert atspi.Accessible.get_role(node) == atspi.Role.LINK
    assert atspi.Accessible.get_hypertext_iface(node) is not None

@pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
def test_axapi(axapi, session, inline, test_html):
    session.url = inline(test_html)
    node = axapi.find_node("test", session.url)

    role = axapi.AXUIElementCopyAttributeValue(node, "AXRole", None)[1]
    assert role == "AXLink"

    subrole = axapi.AXUIElementCopyAttributeValue(node, "AXSubrole", None)[1]
    assert subrole == None

@pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
def test_ia2(ia2, session, inline, test_html):
    session.url = inline(test_html)
    node = ia2.find_node("test", session.url)

    assert ia2.get_role(node) == "ROLE_SYSTEM_LINK"

    state = ia2.get_state_list(node)
    assert 'OPAQUE' in state

    msaa_state = ia2.get_msaa_state_list(node)

    expected_msaa = ["LINKED"]

    if "aria-disabled=true" in test_html:
        expected_msaa.append("UNAVAILABLE")
    elif "aria-expanded=false" in test_html:
        expected_msaa.append("COLLAPSED")
    else:
        pass

    for expected in expected_msaa:
        assert expected in msaa_state

    if "aria-disabled=true" in test_html:
        assert "FOCUSABLE" not in msaa_state

    assert ia2.get_hyperlink_interface(node) is not None

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_uia(uia, session, inline, test_html):
#     session.url = inline(test_html)
#
#     # Spec:
#     # Control Type: HyperLink
#     # Control Pattern: Value
