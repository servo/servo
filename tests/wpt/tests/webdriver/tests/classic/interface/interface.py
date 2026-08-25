def test_navigator_webdriver_active(session):
    assert session.execute_script("return navigator.webdriver") is True
