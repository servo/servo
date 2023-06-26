from tests.support import platform_name
from tests.support.asserts import assert_success


def get_current_url(session):
    return session.transport.send(
        "GET", "session/{session_id}/url".format(**vars(session)))


def test_get_current_url_file_protocol(session, server_config):
    # tests that the browsing context remains the same
    # when navigated privileged documents
    path = server_config["doc_root"]
    if platform_name == "windows":
        # Convert the path into the format eg. /c:/foo/bar
        path = "/{}".format(path.replace("\\", "/"))
    url = u"file://{}".format(path)
    session.url = url

    response = get_current_url(session)
    if response.status == 200 and response.body['value'].endswith('/'):
        url += '/'
    assert_success(response, url)
