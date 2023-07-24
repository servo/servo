import os

from tests.support.asserts import assert_same_element, assert_success
from . import execute_script


def test_arguments(session):
    response = execute_script(session, """
        function func() {
            return arguments;
        }
        return func("foo", "bar");
        """)
    assert_success(response, [u"foo", u"bar"])


def test_array(session):
    response = execute_script(session, "return [1, 2]")
    assert_success(response, [1, 2])


def test_array_in_array(session):
    response = execute_script(session, "const arr = [1]; return [arr, arr]")
    assert_success(response, [[1], [1]])


def test_dom_token_list(session, inline):
    session.url = inline("""<div class="no cheese">foo</div>""")
    element = session.find.css("div", all=False)

    response = execute_script(session, "return arguments[0].classList", args=[element])
    value = assert_success(response)

    assert value == ["no", "cheese"]


def test_file_list(session, tmpdir, inline):
    files = [tmpdir.join("foo.txt"), tmpdir.join("bar.txt")]

    session.url = inline("<input type=file multiple>")
    upload = session.find.css("input", all=False)
    for file in files:
        file.write("morn morn")
        upload.send_keys(str(file))

    response = execute_script(session, "return document.querySelector('input').files")
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == len(files)
    for expected, actual in zip(files, value):
        assert isinstance(actual, dict)
        assert "name" in actual
        assert isinstance(actual["name"], str)
        assert os.path.basename(str(expected)) == actual["name"]


def test_html_all_collection(session, inline):
    session.url = inline("""
        <p>foo
        <p>bar
        """)
    html = session.find.css("html", all=False)
    head = session.find.css("head", all=False)
    meta = session.find.css("meta", all=False)
    body = session.find.css("body", all=False)
    ps = session.find.css("p")

    response = execute_script(session, "return document.all")
    value = assert_success(response)
    assert isinstance(value, list)
    # <html>, <head>, <meta>, <body>, <p>, <p>
    assert len(value) == 6

    assert_same_element(session, html, value[0])
    assert_same_element(session, head, value[1])
    assert_same_element(session, meta, value[2])
    assert_same_element(session, body, value[3])
    assert_same_element(session, ps[0], value[4])
    assert_same_element(session, ps[1], value[5])


def test_html_collection(session, inline):
    session.url = inline("""
        <p>foo
        <p>bar
        """)
    ps = session.find.css("p")

    response = execute_script(session, "return document.getElementsByTagName('p')")
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 2
    for expected, actual in zip(ps, value):
        assert_same_element(session, expected, actual)


def test_html_form_controls_collection(session, inline):
    session.url = inline("""
        <form>
            <input>
            <input>
        </form>
        """)
    inputs = session.find.css("input")

    response = execute_script(session, "return document.forms[0].elements")
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 2
    for expected, actual in zip(inputs, value):
        assert_same_element(session, expected, actual)


def test_html_options_collection(session, inline):
    session.url = inline("""
        <select>
            <option>
            <option>
        </select>
        """)
    options = session.find.css("option")

    response = execute_script(session, "return document.querySelector('select').options")
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 2
    for expected, actual in zip(options, value):
        assert_same_element(session, expected, actual)


def test_node_list(session, inline):
    session.url = inline("""
        <p>foo
        <p>bar
        """)
    ps = session.find.css("p")

    response = execute_script(session, "return document.querySelectorAll('p')")
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 2
    for expected, actual in zip(ps, value):
        assert_same_element(session, expected, actual)
