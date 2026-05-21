import pytest

# Testing: https://w3c.github.io/core-aam/#role-map-button-pressed

TEST_HTML = {
    "true": "<div id=test role=button aria-pressed=true>press me</div>",
    "false": "<div id=test role=button aria-pressed=false>press me</div>"
}

@pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
def test_atspi(atspi, session, inline, test_html):
    session.url = inline(test_html)

    # Spec:
    # Role: ROLE_TOGGLE_BUTTON

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.TOGGLE_BUTTON

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_axapi(axapi, session, inline, test_html):
#     session.url = inline(test_html)
#
#     # Spec:
#     # AXRole: AXCheckBox
#     # AXSubrole: AXToggle

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_ia2(ia2, session, inline, test_html):
#     session.url = inline(test_html)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_PUSHBUTTON
#     # Role: IA2_ROLE_TOGGLE_BUTTON

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_uia(uia, session, inline, test_html):
#     session.url = inline(test_html)
#
#     # Spec:
#     # Control Type: Button
