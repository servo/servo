import pytest

from tests.support.asserts import assert_error, assert_files_uploaded, assert_success
from tests.support.inline import inline

from . import map_files_to_multiline_text


def element_send_keys(session, element, text):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/value".format(
            session_id=session.session_id,
            element_id=element.id),
        {"text": text})


def test_empty_text(session):
    session.url = inline("<input type=file>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, "")
    assert_error(response, "invalid argument")


def test_multiple_files(session, create_files):
    files = create_files(["foo", "bar"])

    session.url = inline("<input type=file multiple>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element,
                                 map_files_to_multiline_text(files))
    assert_success(response)

    assert_files_uploaded(session, element, files)


def test_multiple_files_last_path_not_found(session, create_files):
    files = create_files(["foo", "bar"])
    files.append("foo bar")

    session.url = inline("<input type=file multiple>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element,
                                 map_files_to_multiline_text(files))
    assert_error(response, "invalid argument")

    assert_files_uploaded(session, element, [])


def test_multiple_files_without_multiple_attribute(session, create_files):
    files = create_files(["foo", "bar"])

    session.url = inline("<input type=file>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element,
                                 map_files_to_multiline_text(files))
    assert_error(response, "invalid argument")

    assert_files_uploaded(session, element, [])


def test_multiple_files_send_twice(session, create_files):
    first_files = create_files(["foo", "bar"])
    second_files = create_files(["john", "doe"])

    session.url = inline("<input type=file multiple>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element,
                                 map_files_to_multiline_text(first_files))
    assert_success(response)

    response = element_send_keys(session, element,
                                 map_files_to_multiline_text(second_files))
    assert_success(response)

    assert_files_uploaded(session, element, first_files + second_files)


def test_multiple_files_reset_with_element_clear(session, create_files):
    first_files = create_files(["foo", "bar"])
    second_files = create_files(["john", "doe"])

    session.url = inline("<input type=file multiple>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element,
                                 map_files_to_multiline_text(first_files))
    assert_success(response)

    # Reset already uploaded files
    element.clear()
    assert_files_uploaded(session, element, [])

    response = element_send_keys(session, element,
                                 map_files_to_multiline_text(second_files))
    assert_success(response)

    assert_files_uploaded(session, element, second_files)


def test_single_file(session, create_files):
    files = create_files(["foo"])

    session.url = inline("<input type=file>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_success(response)

    assert_files_uploaded(session, element, files)


def test_single_file_replaces_without_multiple_attribute(session, create_files):
    files = create_files(["foo", "bar"])

    session.url = inline("<input type=file>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_success(response)

    response = element_send_keys(session, element, str(files[1]))
    assert_success(response)

    assert_files_uploaded(session, element, [files[1]])


def test_single_file_appends_with_multiple_attribute(session, create_files):
    files = create_files(["foo", "bar"])

    session.url = inline("<input type=file multiple>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_success(response)

    response = element_send_keys(session, element, str(files[1]))
    assert_success(response)

    assert_files_uploaded(session, element, files)


def test_transparent(session, create_files):
    files = create_files(["foo"])
    session.url = inline("""<input type=file style="opacity: 0">""")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_success(response)
    assert_files_uploaded(session, element, files)


def test_obscured(session, create_files):
    files = create_files(["foo"])
    session.url = inline("""
        <style>
          div {
            position: absolute;
            width: 100vh;
            height: 100vh;
            background: blue;
            top: 0;
            left: 0;
          }
        </style>

        <input type=file>
        <div></div>
        """)
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_success(response)
    assert_files_uploaded(session, element, files)


def test_outside_viewport(session, create_files):
    files = create_files(["foo"])
    session.url = inline("""<input type=file style="margin-left: -100vh">""")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_success(response)
    assert_files_uploaded(session, element, files)


def test_hidden(session, create_files):
    files = create_files(["foo"])
    session.url = inline("<input type=file hidden>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_success(response)
    assert_files_uploaded(session, element, files)


def test_display_none(session, create_files):
    files = create_files(["foo"])
    session.url = inline("""<input type=file style="display: none">""")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_success(response)
    assert_files_uploaded(session, element, files)


@pytest.mark.capabilities({"strictFileInteractability": True})
def test_strict_hidden(session, create_files):
    files = create_files(["foo"])
    session.url = inline("<input type=file hidden>")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_error(response, "element not interactable")


@pytest.mark.capabilities({"strictFileInteractability": True})
def test_strict_display_none(session, create_files):
    files = create_files(["foo"])
    session.url = inline("""<input type=file style="display: none">""")
    element = session.find.css("input", all=False)

    response = element_send_keys(session, element, str(files[0]))
    assert_error(response, "element not interactable")
