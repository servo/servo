from tests.support.fixtures import clear_all_cookies
from datetime import datetime, timedelta

def test_add_domain_cookie(session, url, server_config):
    session.url = url("/common/blank.html")
    clear_all_cookies(session)
    create_cookie_request = {
        "cookie": {
            "name": "hello",
            "value": "world",
            "domain": server_config["domains"][""],
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
    assert cookie["domain"] == ".%s" % server_config["domains"][""]

def test_add_cookie_for_ip(session, url, server_config, configuration):
    session.url = "http://127.0.0.1:%s/404" % (server_config["ports"]["http"][0])
    clear_all_cookies(session)
    create_cookie_request = {
        "cookie": {
            "name": "hello",
            "value": "world",
            "domain": configuration["host"],
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

def test_add_non_session_cookie(session, url):
    session.url = url("/common/blank.html")
    clear_all_cookies(session)
    a_year_from_now = int((datetime.utcnow() + timedelta(days=365)).strftime("%s"))
    create_cookie_request = {
        "cookie": {
            "name": "hello",
            "value": "world",
            "expiry": a_year_from_now
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
    assert "expiry" in cookie
    assert isinstance(cookie["expiry"], int)

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"
    assert cookie["expiry"] == a_year_from_now

def test_add_session_cookie(session, url):
    session.url = url("/common/blank.html")
    clear_all_cookies(session)
    create_cookie_request = {
        "cookie": {
            "name": "hello",
            "value": "world"
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
    assert "expiry" in cookie
    assert cookie.get("expiry") is None

    assert cookie["name"] == "hello"
    assert cookie["value"] == "world"

def test_add_session_cookie_with_leading_dot_character_in_domain(session, url, server_config):
    session.url = url("/common/blank.html")
    clear_all_cookies(session)
    create_cookie_request = {
        "cookie": {
            "name": "hello",
            "value": "world",
            "domain": ".%s" % server_config["domains"][""]
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
    assert cookie["domain"] == ".%s" % server_config["domains"][""]
