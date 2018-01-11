import pytest
from tests.support.asserts import assert_error, assert_success
from tests.support.inline import inline


def clear(session, element):
    return session.transport.send("POST", "session/{session_id}/element/{element_id}/clear"
                                  .format(session_id=session.session_id,
                                          element_id=element.id))


# 14.2 Element Clear

def test_no_browsing_context(session, create_window):
    # 14.2 step 1
    session.url = inline("<p>This is not an editable paragraph.")
    element = session.find.css("p", all=False)

    session.window_handle = create_window()
    session.close()

    response = clear(session, element)
    assert_error(response, "no such window")


def test_element_not_found(session):
    # 14.2 Step 2
    response = session.transport.send("POST", "session/{session_id}/element/{element_id}/clear"
                                      .format(session_id=session.session_id,
                                              element_id="box1"))

    assert_error(response, "no such element")


def test_element_not_editable(session):
    # 14.2 Step 3
    session.url = inline("<p>This is not an editable paragraph.")

    element = session.find.css("p", all=False)
    response = clear(session, element)
    assert_error(response, "invalid element state")


def test_button_element_not_resettable(session):
    # 14.2 Step 3
    session.url = inline("<input type=button value=Federer>")

    element = session.find.css("input", all=False)
    response = clear(session, element)
    assert_error(response, "invalid element state")


def test_disabled_element_not_resettable(session):
    # 14.2 Step 3
    session.url = inline("<input type=text value=Federer disabled>")

    element = session.find.css("input", all=False)
    response = clear(session, element)
    assert_error(response, "invalid element state")


def test_scroll_into_element_view(session):
    # 14.2 Step 4
    session.url = inline("<input type=text value=Federer><div style= \"height: 200vh; width: 5000vh\">")

    # Scroll to the bottom right of the page
    session.execute_script("window.scrollTo(document.body.scrollWidth, document.body.scrollHeight);")
    element = session.find.css("input", all=False)
    # Clear and scroll back to the top of the page
    response = clear(session, element)
    assert_success(response)

    # Check if element cleared is scrolled into view
    rect = session.execute_script("return document.getElementsByTagName(\"input\")[0].getBoundingClientRect()")

    pageDict = {}

    pageDict["innerHeight"] = session.execute_script("return window.innerHeight")
    pageDict["innerWidth"] = session.execute_script("return window.innerWidth")
    pageDict["pageXOffset"] = session.execute_script("return window.pageXOffset")
    pageDict["pageYOffset"] = session.execute_script("return window.pageYOffset")

    assert rect["top"] < (pageDict["innerHeight"] + pageDict["pageYOffset"]) and \
           rect["left"] < (pageDict["innerWidth"] + pageDict["pageXOffset"]) and \
           (rect["top"] + element.rect["height"]) > pageDict["pageYOffset"] and \
           (rect["left"] + element.rect["width"]) > pageDict["pageXOffset"]


# TODO
# Any suggestions on implementation?
# def test_session_implicit_wait_timeout(session):
    # 14.2 Step 5

# TODO
# Any suggestions on implementation?
# def test_element_not_interactable(session):
#     # 14.2 Step 6
#     assert_error(response, "element not interactable")


def test_element_readonly(session):
    # 14.2 Step 7
    session.url = inline("<input type=text readonly value=Federer>")

    element = session.find.css("input", all=False)
    response = clear(session, element)
    assert_error(response, "invalid element state")


def test_element_disabled(session):
    # 14.2 Step 7
    session.url = inline("<input type=text disabled value=Federer>")

    element = session.find.css("input", all=False)
    response = clear(session, element)
    assert_error(response, "invalid element state")


def test_element_pointer_events_disabled(session):
    # 14.2 Step 7
    session.url = inline("<input type=text value=Federer style=\"pointer-events: none\">")

    element = session.find.css("input", all=False)
    response = clear(session, element)
    assert_error(response, "invalid element state")


@pytest.mark.parametrize("element", [["text", "<input id=text type=text value=\"Federer\"><input id=empty type=text value=\"\">"],
                                    ["search", "<input id=search type=search value=\"Federer\"><input id=empty type=search value=\"\">"],
                                    ["url", "<input id=url type=url value=\"www.hello.com\"><input id=empty type=url value=\"\">"],
                                    ["tele", "<input id=tele type=telephone value=\"2061234567\"><input id=empty type=telephone value=\"\">"],
                                    ["email", "<input id=email type=email value=\"hello@world.com\"><input id=empty type=email value=\"\">"],
                                    ["password", "<input id=password type=password value=\"pass123\"><input id=empty type=password value=\"\">"],
                                    ["date", "<input id=date type=date value=\"2017-12-25\"><input id=empty type=date value=\"\">"],
                                    ["time", "<input id=time type=time value=\"11:11\"><input id=empty type=time value=\"\">"],
                                    ["number", "<input id=number type=number value=\"19\"><input id=empty type=number value=\"\">"],
                                    ["range", "<input id=range type=range min=\"0\" max=\"10\"><input id=empty type=range value=\"\">"],
                                    ["color", "<input id=color type=color value=\"#ff0000\"><input id=empty type=color value=\"\">"],
                                    ["file", "<input id=file type=file value=\"C:\\helloworld.txt\"><input id=empty type=file value=\"\">"],
                                    ["textarea", "<textarea id=textarea>Hello World</textarea><textarea id=empty></textarea>"],
                                    ["sel", "<select id=sel><option></option><option>a</option><option>b</option></select><select id=empty><option></option></select>"],
                                    ["out", "<output id=out value=100></output><output id=empty></output>"],
                                    ["para", "<p id=para contenteditable=true>This is an editable paragraph.</p><p id=empty contenteditable=true></p>"]])

def test_clear_content_editable_resettable_element(session, element):
    # 14.2 Step 8
    url = element[1] + """<input id=focusCheck type=checkbox>
                    <input id=blurCheck type=checkbox>
                    <script>
                    var id = "%s";
                    document.getElementById(id).addEventListener("focus", checkFocus);
                    document.getElementById(id).addEventListener("blur", checkBlur);
                    document.getElementById("empty").addEventListener("focus", checkFocus);
                    document.getElementById("empty").addEventListener("blur", checkBlur);

                    function checkFocus() {
                        document.getElementById("focusCheck").checked = true;
                    }
                    function checkBlur() {
                        document.getElementById("blurCheck").checked = true;
                    }
                    </script>""" % element[0]
    session.url = inline(url)
    # Step 1
    empty_element = session.find.css("#empty", all=False)
    clear_element_test_helper(session, empty_element, False)
    session.execute_script("document.getElementById(\"focusCheck\").checked = false;")
    session.execute_script("document.getElementById(\"blurCheck\").checked = false;")
    # Step 2 - 4
    test_element = session.find.css("#" + element[0], all=False)
    clear_element_test_helper(session, test_element, True)


def clear_element_test_helper(session, element, value):
    response = clear(session, element)
    assert_success(response)
    response = session.execute_script("return document.getElementById(\"focusCheck\").checked;")
    assert response is value
    response = session.execute_script("return document.getElementById(\"blurCheck\").checked;")
    assert response is value
    if element.name == "p":
        response = session.execute_script("return document.getElementById(\"para\").innerHTML;")
        assert response == ""
    else:
        assert element.property("value") == ""
