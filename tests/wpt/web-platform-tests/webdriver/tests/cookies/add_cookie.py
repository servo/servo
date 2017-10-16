from tests.support.fixtures import clear_all_cookies
from tests.support.fixtures import server_config

def test_add_domain_cookie(session, url):
    session.url = url("/common/blank.html")
    clear_all_cookies(session)
    create_cookie_request = {
        "cookie": {
            "name": "hello",
            "value": "world",
            "domain": "web-platform.test",
            "path": "/",
            "httpOnly": False,
            "secure": False
        }
    }
    result = session.transport.send("POST", "session/%s/cookie" % session.session_id, create_cookie_request)
    assert result.status == 200
    assert "value" in result.body
    assert isinstance(result.body["value"], dict)

    result = session.transport.send("GET", "session/%s/cookie" % session.session_id)
    assert result.status == 200
    assert "value" in result.body
    assert isinstance(result.body["value"], list)
    assert len(result.body["value"]) == 1
    assert isinstance(result.body["value"][0], dict)

    cookie = result.body["value"][0]
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], basestring)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["domain"] == ".web-platform.test"

def test_add_cookie_for_ip(session, url, server_config):
    session.url = "http://127.0.0.1:%s/404" % (server_config["ports"]["http"][0])
    clear_all_cookies(session)
    create_cookie_request = {
        "cookie": {
            "name": "hello",
            "value": "world",
            "domain": "127.0.0.1",
            "path": "/",
            "httpOnly": False,
            "secure": False
        }
    }
    result = session.transport.send("POST", "session/%s/cookie" % session.session_id, create_cookie_request)
    assert result.status == 200
    assert "value" in result.body
    assert isinstance(result.body["value"], dict)

    result = session.transport.send("GET", "session/%s/cookie" % session.session_id)
    assert result.status == 200
    assert "value" in result.body
    assert isinstance(result.body["value"], list)
    assert len(result.body["value"]) == 1
    assert isinstance(result.body["value"][0], dict)

    cookie = result.body["value"][0]
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], basestring)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["domain"] == "127.0.0.1"
