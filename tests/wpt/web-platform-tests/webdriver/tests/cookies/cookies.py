def test_get_named_cookie(session, url):
    session.url = url("/common/blank.html")
    session.execute_script("document.cookie = 'foo=bar'")

    result = session.transport.send("GET", "session/%s/cookie/foo" % session.session_id)
    assert result.status == 200
    assert isinstance(result.body["value"], dict)

    # table for cookie conversion
    # https://w3c.github.io/webdriver/webdriver-spec.html#dfn-table-for-cookie-conversion
    cookie = result.body["value"]
    assert "name" in cookie
    assert isinstance(cookie["name"], basestring)
    assert "value" in cookie
    assert isinstance(cookie["value"], basestring)
    assert "path" in cookie
    assert isinstance(cookie["path"], basestring)
    assert "domain" in cookie
    assert isinstance(cookie["domain"], basestring)
    assert "secure" in cookie
    assert isinstance(cookie["secure"], bool)
    assert "httpOnly" in cookie
    assert isinstance(cookie["httpOnly"], bool)
    assert "expiry" in cookie
    assert isinstance(cookie["expiry"], (int, long))

    assert cookie["name"] == "foo"
    assert cookie["value"] == "bar"
