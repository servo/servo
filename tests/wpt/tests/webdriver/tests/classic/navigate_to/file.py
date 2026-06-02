from pathlib import Path

from tests.support.asserts import assert_success


def navigate_to(session, url):
    return session.transport.send(
        "POST", "session/{session_id}/url".format(**vars(session)),
        {"url": url})


def test_file_protocol(session, target_platform, server_config):
    # tests that the browsing context remains the same
    # when navigated privileged documents
    path = Path(server_config["doc_root"]) / "common" / "blank.html"

    # not all borwsers support "loading" file URLs which aren't files,
    # so check this is one
    assert path.is_file()

    # and then create the file URL
    url = path.as_uri()
    assert url.startswith("file://")

    response = navigate_to(session, url)
    assert_success(response)

    assert session.url == url
