"""WebDriver alert handling."""

class Alert(object):
    """Class that provides access to the WebDriver alert handling functions."""

    def __init__(self, driver):
        self._driver = driver

    def _execute(self, method, path, name, body=None):
        return self._driver.execute(method, path, name, body)

    def dismiss(self):
        """Dismiss the alert."""
        self._execute('POST', '/dismiss_alert', 'dismiss')

    def accept(self):
        """Accept the alert."""
        self._execute('POST', '/accept_alert', 'accept')

    def get_text(self):
        """Get the text displayed in the alert."""
        return self._execute('GET', '/alert_text', 'getText')

    def send_keys(self, keys):
        """Type into the text input of the alert if available."""
        self._execute('POST', '/alert_text', 'sendKeys', { 'text': keys })
