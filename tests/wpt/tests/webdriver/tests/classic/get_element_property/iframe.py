import pytest

from tests.support.classic.asserts import assert_success


def get_element_property(session, element_id, prop):
    return session.transport.send(
        "GET", "session/{session_id}/element/{element_id}/property/{prop}".format(
            session_id=session.session_id,
            element_id=element_id,
            prop=prop))


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_content_attribute_property_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<input id='x' value='foobar'>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#x", all=False)
    response = get_element_property(session, element.id, "value")
    assert_success(response, "foobar")


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
def test_idl_attribute_property_in_frame(session, inline, iframe, domain):
    session.url = inline(iframe("<input id='x' value='foo'>", domain=domain))

    frame = session.find.css("iframe", all=False)
    session.switch_to_frame(frame)

    element = session.find.css("#x", all=False)
    session.execute_script("""arguments[0].value = "bar";""", args=(element,))

    response = get_element_property(session, element.id, "value")
    assert_success(response, "bar")
