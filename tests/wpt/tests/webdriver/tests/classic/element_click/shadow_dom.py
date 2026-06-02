import pytest
from tests.support.asserts import assert_success


def element_click(session, element):
    return session.transport.send(
        "POST", "session/{session_id}/element/{element_id}/click".format(
            session_id=session.session_id, element_id=element.id))


@pytest.mark.parametrize("click_on", ["host_element", "checkbox_element"])
def test_shadow_element_click(session, get_test_page, click_on):
    session.url = get_test_page()

    host_element = session.find.css("custom-element", all=False)
    checkbox_element = session.execute_script("""
        return arguments[0].shadowRoot.querySelector("input")
        """,
                                              args=(host_element, ))

    is_pre_checked = session.execute_script("""
        return arguments[0].checked
        """,
                                            args=(checkbox_element, ))
    assert is_pre_checked is False

    response = element_click(session, locals()[click_on])
    assert_success(response)

    is_post_checked = session.execute_script("""
        return arguments[0].checked
        """,
                                             args=(checkbox_element, ))
    assert is_post_checked is True


@pytest.mark.parametrize("click_on",
                         ["outer_element", "inner_element", "checkbox"])
def test_nested_shadow_element_click(session, get_test_page, click_on):
    session.url = get_test_page(nested_shadow_dom=True)

    outer_element = session.find.css("custom-element", all=False)
    inner_element = session.execute_script("""
        return arguments[0].shadowRoot.querySelector("inner-custom-element")
        """,
                                           args=(outer_element, ))
    checkbox = session.execute_script("""
        return arguments[0].shadowRoot.querySelector("input")
        """,
                                      args=(inner_element, ))

    is_pre_checked = session.execute_script("return arguments[0].checked",
                                            args=(checkbox, ))
    assert is_pre_checked is False

    click_response = element_click(session, locals()[click_on])
    assert_success(click_response)
    is_post_checked = session.execute_script("return arguments[0].checked",
                                             args=(checkbox, ))
    assert is_post_checked is True
