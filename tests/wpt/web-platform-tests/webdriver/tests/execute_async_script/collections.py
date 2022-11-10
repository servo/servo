import os

from webdriver.client import ShadowRoot

from tests.support.asserts import assert_same_element, assert_success
from . import execute_async_script


def test_arguments(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        function func() {
            return arguments;
        }
        resolve(func("foo", "bar"));
        """)
    assert_success(response, [u"foo", u"bar"])


def test_array(session):
    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve([1, 2]);
        """)
    assert_success(response, [1, 2])


def test_file_list(session, tmpdir, inline):
    files = [tmpdir.join("foo.txt"), tmpdir.join("bar.txt")]

    session.url = inline("<input type=file multiple>")
    upload = session.find.css("input", all=False)
    for file in files:
        file.write("morn morn")
        upload.send_keys(str(file))

    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(document.querySelector('input').files);
        """)
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

    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(document.all);
        """)
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

    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(document.getElementsByTagName('p'));
        """)
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

    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(document.forms[0].elements);
        """)
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

    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(document.querySelector('select').options);
        """)
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

    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(document.querySelectorAll('p'));
        """)
    value = assert_success(response)
    assert isinstance(value, list)
    assert len(value) == 2
    for expected, actual in zip(ps, value):
        assert_same_element(session, expected, actual)


def test_shadow_root(session, inline):
    session.url = inline("""
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
    custom_element = session.find.css("custom-checkbox-element", all=False)
    expected = custom_element.shadow_root
    response = execute_async_script(session, """
        let resolve = arguments[0];
        resolve(document.querySelector('custom-checkbox-element').shadowRoot);
        """)
    value = assert_success(response)
    assert isinstance(value, ShadowRoot)
    assert value.id == expected.id
