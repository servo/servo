from tests.support import platform_name
from tests.support.asserts import assert_success


def navigate_to(session, url):
    return session.transport.send(
        "POST", "session/{session_id}/url".format(**vars(session)),
        {"url": url})


def test_file_protocol(session, server_config):
    # tests that the browsing context remains the same
    # when navigated privileged documents
    path = server_config["doc_root"]
    if platform_name == "windows":
        # Convert the path into the format eg. /c:/foo/bar
        path = "/{}".format(path.replace("\\", "/"))
    url = u"file://{}".format(path)

    response = navigate_to(session, url)
    assert_success(response)

    if session.url.endswith('/'):
        url += '/'
    assert session.url == url
