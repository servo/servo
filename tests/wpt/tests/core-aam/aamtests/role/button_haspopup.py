import pytest

# Testing: https://w3c.github.io/core-aam/#role-map-button-haspopup

TEST_HTML = {
    "true": "<div id=test role=button aria-haspopup=true>click me</div>",
    "menu": "<div id=test role=button aria-haspopup=menu>click me</div>",
    "listbox": "<div id=test role=button aria-haspopup=listbox>click me</div>",
    "tree": "<div id=test role=button aria-haspopup=tree>click me</div>",
    "grid": "<div id=test role=button aria-haspopup=grid>click me</div>",
    "dialog": "<div id=test role=button aria-haspopup=dialog>click me</div>"
}

@pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
def test_atspi(atspi, session, inline, test_html):
    session.url = inline(test_html)

    # Spec:
    # Role: ROLE_PUSH_BUTTON

    node = atspi.find_node("test", session.url)
    assert atspi.Accessible.get_role(node) == atspi.Role.PUSH_BUTTON

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_axapi(axapi, session, inline, test_html):
#     session.url = inline(test_html)
#
#     # Spec:
#     # AXRole: AXPopUpButton
#     # AXSubrole: <nil>

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_ia2(ia2, session, inline, test_html):
#     session.url = inline(test_html)
#
#     # Spec:
#     # Role: ROLE_SYSTEM_BUTTONMENU

# @pytest.mark.parametrize("test_html", TEST_HTML.values(), ids=TEST_HTML.keys())
# def test_uia(uia, session, inline, test_html):
#     session.url = inline(test_html)
#
#     # Spec:
#     # Control Type: Button
