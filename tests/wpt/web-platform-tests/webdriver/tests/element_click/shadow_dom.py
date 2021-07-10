import pytest
from tests.support.asserts import assert_error, assert_success


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id,
            element_id=element.id))


@pytest.fixture
def get_checkbox_dom(inline):
    return inline("""
        <style>
            custom-checkbox-element {
                display:block; width:20px; height:20px;
            }
        </style>
        <custom-checkbox-element></custom-checkbox-element>
        <script>
            customElements.define('custom-checkbox-element',
                class extends HTMLElement {
                    constructor() {
                            super();
                            this.attachShadow({mode: 'open'}).innerHTML = `
                                <div><input type="checkbox"/></div>
                            `;
                        }
                });
        </script>""")


@pytest.mark.parametrize("click_on", ["custom_element", "checkbox_element"])
def test_shadow_element_click(session, get_checkbox_dom, click_on):
    session.url = get_checkbox_dom
    custom_element = session.find.css("custom-checkbox-element", all=False)
    checkbox_element = session.execute_script("return arguments[0].shadowRoot.querySelector('input')",
                                              args=(custom_element,))
    is_pre_checked = session.execute_script("return arguments[0].checked",
                                            args=(checkbox_element,))
    assert is_pre_checked == False
    response = element_click(session, locals()[click_on])
    assert_success(response)
    is_post_checked = session.execute_script("return arguments[0].checked",
                                             args=(checkbox_element,))
    assert is_post_checked == True


@pytest.fixture
def get_nested_shadow_checkbox_dom(inline):
    return inline("""
        <style>
            custom-nesting-element {
                display:block; width:20px; height:20px;
            }
        </style>
        <custom-nesting-element></custom-nesting-element>
        <script>
         customElements.define('custom-nesting-element',
                class extends HTMLElement {
                    constructor() {
                            super();
                            this.attachShadow({mode: 'open'}).innerHTML = `
                                <style>
                                    custom-checkbox-element {
                                        display:block; width:20px; height:20px;
                                    }
                                </style>
                                <div><custom-checkbox-element></custom-checkbox-element></div>
                            `;
                        }
                });
            customElements.define('custom-checkbox-element',
                class extends HTMLElement {
                    constructor() {
                            super();
                            this.attachShadow({mode: 'open'}).innerHTML = `
                                <div><input type="checkbox"/></div>
                            `;
                        }
                });
        </script>""")


@pytest.mark.parametrize("click_on", ["outer_element", "inner_element", "checkbox_element"])
def test_nested_shadow_element_click(session, get_nested_shadow_checkbox_dom, click_on):
    session.url = get_nested_shadow_checkbox_dom
    outer_element = session.find.css("custom-nesting-element", all=False)
    inner_element = session.execute_script("return arguments[0].shadowRoot.querySelector('custom-checkbox-element')",
                                           args=(outer_element,))
    checkbox_element = session.execute_script("return arguments[0].shadowRoot.querySelector('input')",
                                              args=(inner_element,))
    is_pre_checked = session.execute_script("return arguments[0].checked",
                                            args=(checkbox_element,))
    assert is_pre_checked == False
    click_response = element_click(session, locals()[click_on])
    assert_success(click_response)
    is_post_checked = session.execute_script("return arguments[0].checked",
                                             args=(checkbox_element,))
    assert is_post_checked == True
