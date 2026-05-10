import pytest

# Testing: https://w3c.github.io/core-aam/#role-map-button

TEST_HTML = {
    "no-attributes": "<div id=test role=button>click me</div>",
    "aria-pressed-undefined": "<div id=test role=button aria-pressed>click me</div>",
    "aria-haspopup-undefined": "<div id=test role=button aria-haspopup>click me</div>",
    "aria-haspopup-false": "<div id=test role=button aria-haspopup=false>click me</div>",
}

@pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
def test_atspi(atspi, session, inline, test_html):
    session.url = inline(test_html)

    # Spec:
    # Role: ROLE_PUSH_BUTTON

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.PUSH_BUTTON

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_axapi(axapi, session, inline):
#     session.url = inline(test_html)
#
#     # Spec:
#     # AXRole: AXButton
#     # AXSubrole: <nil>

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_ia2(ia2, session, inline):
#     session.url = inline(test_html)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_PUSHBUTTON

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_uia(uia, session, inline):
#     session.url = inline(test_html)
#
#     # Spec:
#     # Control Type: Button
