import pytest
import time

from support.asserts import assert_error, assert_success, assert_dialog_handled
from support.fixtures import create_dialog
from support.inline import inline

def read_global(session, name):
    return session.execute_script("return %s;" % name)

# 1. If the current top-level browsing context is no longer open, return error
#    with error code no such window.
def test_title_from_closed_context(session, create_window):
    new_window = create_window()
    session.window_handle = new_window
    session.close()

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_error(result, "no such window")

# [...]
# 2. Handle any user prompts and return its value if it is an error.
# [...]
# In order to handle any user prompts a remote end must take the following
# steps:
# 2. Run the substeps of the first matching user prompt handler:
#
#    [...]
#    - dismiss state
#      1. Dismiss the current user prompt.
#    [...]
#
# 3. Return success.
def test_title_handle_prompt_dismiss(new_session):
    _, session = new_session({"alwaysMatch": {"unhandledPromptBehavior": "dismiss"}})
    session.url = inline("<title>WD doc title</title>")

    expected_title = read_global(session, "document.title")
    create_dialog(session)("alert", text="dismiss #1", result_var="dismiss1")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, expected_title)
    assert_dialog_handled(session, "dismiss #1")
    assert read_global(session, "dismiss1") == None

    expected_title = read_global(session, "document.title")
    create_dialog(session)("confirm", text="dismiss #2", result_var="dismiss2")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, expected_title)
    assert_dialog_handled(session, "dismiss #2")
    assert read_global(session, "dismiss2") == None

    expected_title = read_global(session, "document.title")
    create_dialog(session)("prompt", text="dismiss #3", result_var="dismiss3")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, expected_title)
    assert_dialog_handled(session, "dismiss #3")
    assert read_global(session, "dismiss3") == None

# [...]
# 2. Handle any user prompts and return its value if it is an error.
# [...]
# In order to handle any user prompts a remote end must take the following
# steps:
# 2. Run the substeps of the first matching user prompt handler:
#
#    [...]
#    - accept state
#      1. Accept the current user prompt.
#    [...]
#
# 3. Return success.
def test_title_handle_prompt_accept(new_session):
    _, session = new_session({"alwaysMatch": {"unhandledPromptBehavior": "accept"}})
    session.url = inline("<title>WD doc title</title>")
    create_dialog(session)("alert", text="accept #1", result_var="accept1")

    expected_title = read_global(session, "document.title")
    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, expected_title)
    assert_dialog_handled(session, "accept #1")
    assert read_global(session, "accept1") == None

    expected_title = read_global(session, "document.title")
    create_dialog(session)("confirm", text="accept #2", result_var="accept2")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, expected_title)
    assert_dialog_handled(session, "accept #2")
    assert read_global(session, "accept2"), True

    expected_title = read_global(session, "document.title")
    create_dialog(session)("prompt", text="accept #3", result_var="accept3")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, expected_title)
    assert_dialog_handled(session, "accept #3")
    assert read_global(session, "accept3") == ""

# [...]
# 2. Handle any user prompts and return its value if it is an error.
# [...]
# In order to handle any user prompts a remote end must take the following
# steps:
# 2. Run the substeps of the first matching user prompt handler:
#
#    [...]
#    - missing value default state
#    - not in the table of simple dialogs
#      1. Dismiss the current user prompt.
#      2. Return error with error code unexpected alert open.
def test_title_handle_prompt_missing_value(session, create_dialog):
    session.url = inline("<title>WD doc title</title>")
    create_dialog("alert", text="dismiss #1", result_var="dismiss1")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #1")
    assert read_global(session, "accept1") == None

    create_dialog("confirm", text="dismiss #2", result_var="dismiss2")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #2")
    assert read_global(session, "dismiss2") == False

    create_dialog("prompt", text="dismiss #3", result_var="dismiss3")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_error(result, "unexpected alert open")
    assert_dialog_handled(session, "dismiss #3")
    assert read_global(session, "dismiss3") == None

# The behavior of the `window.print` function is platform-dependent and may not
# trigger the creation of a dialog at all. Therefore, this test should only be
# run in contexts that support the dialog (a condition that may not be
# determined automatically).
#def test_title_with_non_simple_dialog(session):
#    document = "<title>With non-simple dialog</title><h2>Hello</h2>"
#    spawn = """
#        var done = arguments[0];
#        setTimeout(function() {
#            done();
#        }, 0);
#        setTimeout(function() {
#            window['print']();
#        }, 0);
#    """
#    session.url = inline(document)
#    session.execute_async_script(spawn)
#
#    result = session.transport.send("GET",
#                                    "session/%s/title" % session.session_id)
#    assert_error(result, "unexpected alert open")

# [...]
# 3. Let title be the initial value of the title IDL attribute of the current
#    top-level browsing context's active document.
# 4. Return success with data title.
# [...]
# The title attribute must, on getting, run the following algorithm:
# [...]
# 2. Otherwise, let value be the child text content of the title element [...]
# [...]
# 4. Return value.
def test_title_from_top_context(session):
    session.url = inline("<title>Foobar</title><h2>Hello</h2>")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)
    assert_success(result, read_global(session, "document.title"))

# [...]
# 3. Let title be the initial value of the title IDL attribute of the current
#    top-level browsing context's active document.
# 4. Return success with data title.
# [...]
# The title attribute must, on getting, run the following algorithm:
# [...]
# 2. Otherwise, let value be the child text content of the title element [...]
#
#    The title element of a document is the first title element in the document
#    (in tree order), if there is one, or null otherwise.
#
# [...]
# 4. Return value.
def test_title_with_duplicate_element(session):
    session.url = inline("<title>First</title><title>Second</title>")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, read_global(session, "document.title"))

# [...]
# 3. Let title be the initial value of the title IDL attribute of the current
#    top-level browsing context's active document.
# 4. Return success with data title.
# [...]
# The title attribute must, on getting, run the following algorithm:
# [...]
# 2. Otherwise, let value be the child text content of the title element, or
#    the empty string if the title element is null.
# [...]
# 4. Return value.
def test_title_without_element(session):
    session.url = inline("<h2>Hello</h2>")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, read_global(session, "document.title"))

# [...]
# 3. Let title be the initial value of the title IDL attribute of the current
#    top-level browsing context's active document.
# 4. Return success with data title.
def test_title_after_modification(session):
    session.url = inline("<title>Initial</title><h2>Hello</h2>")
    session.execute_script("document.title = 'updated'")

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, read_global(session, "document.title"))

# [...]
# 3. Let title be the initial value of the title IDL attribute of the current
#    top-level browsing context's active document.
# 4. Return success with data title.
# [...]
# The title attribute must, on getting, run the following algorithm:
# [...]
# 2. Otherwise, let value be the child text content of the title element [...]
# 3. Strip and collapse ASCII whitespace in value.
# 4. Return value.
def test_title_strip_and_collapse(session):
    document = "<title>   a b\tc\nd\t \n e\t\n </title><h2>Hello</h2>"
    session.url = inline(document)

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, read_global(session, "document.title"))

# [...]
# 3. Let title be the initial value of the title IDL attribute of the current
#    top-level browsing context's active document.
# 4. Return success with data title.
def test_title_from_frame(session, create_frame):
    session.url = inline("<title>Parent</title>parent")

    session.switch_frame(create_frame())
    session.switch_frame(create_frame())

    result = session.transport.send("GET",
                                    "session/%s/title" % session.session_id)

    assert_success(result, read_global(session, "document.title"))
