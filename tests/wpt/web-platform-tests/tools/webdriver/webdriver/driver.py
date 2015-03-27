"""Entry point for WebDriver."""

import alert
import command
import searchcontext
import webelement

import base64


class WebDriver(searchcontext.SearchContext):
    """Controls a web browser."""

    def __init__(self, host, required, desired, mode='strict'):
        args = { 'desiredCapabilities': desired }
        if required:
            args['requiredCapabilities'] = required

        self._executor = command.CommandExecutor(host, mode)

        resp = self._executor.execute(
            'POST', '/session', None, 'newSession', args)
        self.capabilities = resp['value']
        self._session_id = resp['sessionId']
        self.mode = mode

    def execute(self, method, path, name, parameters= None):
        """Execute a command against the current WebDriver session."""
        data = self._executor.execute(
            method,
            '/session/' + self._session_id + path,
            self._session_id,
            name,
            parameters,
            self._object_hook)
        if data:
            return data['value']

    def get(self, url):
        """Navigate to url."""
        self.execute('POST', '/url', 'get', { 'url': url })

    def get_current_url(self):
        """Get the current value of the location bar."""
        return self.execute('GET', '/url', 'getCurrentUrl')

    def go_back(self):
        """Hit the browser back button."""
        self.execute('POST', '/back', 'goBack')

    def go_forward(self):
        """Hit the browser forward button."""
        self.execute('POST', '/forward', 'goForward')

    def refresh(self):
        """Refresh the current page in the browser."""
        self.execute('POST', '/refresh', 'refresh')

    def quit(self):
        """Shutdown the current WebDriver session."""
        self.execute('DELETE', '', 'quit')

    def get_window_handle(self):
        """Get the handle for the browser window/tab currently accepting
        commands.
        """
        return self.execute('GET', '/window_handle', 'getWindowHandle')

    def get_window_handles(self):
        """Get handles for all open windows/tabs."""
        return self.execute('GET', '/window_handles', 'getWindowHandles')

    def close(self):
        """Close the current tab or window.

        If this is the last tab or window, then this is the same as
        calling quit.
        """
        self.execute('DELETE', '/window', 'close')

    def maximize_window(self):
        """Maximize the current window."""
        return self._window_command('POST', '/maximize', 'maximize')

    def get_window_size(self):
        """Get the dimensions of the current window."""
        result = self._window_command('GET', '/size', 'getWindowSize')
        return { 'height': result[height], 'width': result[width] }

    def set_window_size(self, height, width):
        """Set the size of the current window."""
        self._window_command(
            'POST',
            '/size',
            'setWindowSize',
            { 'height': height, 'width': width})

    def fullscreen_window(self):
        """Make the current window fullscreen."""
        pass # implement when end point is defined

    def switch_to_window(self, name):
        """Switch to the window with the given handle or name."""
        self.execute('POST', '/window', 'switchToWindow', { 'name': name })

    def switch_to_frame(self, id):
        """Switch to a frame.

        id can be either a WebElement or an integer.
        """
        self.execute('POST', '/frame', 'switchToFrame', { 'id': id})

    def switch_to_parent_frame(self):
        """Move to the browsing context containing the currently selected frame.

        If in the top-level browsing context, this is a no-op.
        """
        self.execute('POST', '/frame/parent', 'switchToParentFrame')

    def switch_to_alert(self):
        """Return an Alert object to interact with a modal dialog."""
        alert_ = alert.Alert(self)
        alert_.get_text()
        return alert_

    def execute_script(self, script, args=[]):
        """Execute a Javascript script in the current browsing context."""
        return self.execute(
            'POST',
            '/execute',
            'executeScript',
            { 'script': script, 'args': args })

    def execute_script_async(self, script, args=[]):
        """Execute a Javascript script in the current browsing context."""
        return self.execute(
            'POST',
            '/execute_async',
            'executeScriptAsync',
            { 'script': script, 'args': args })

    def take_screenshot(self, element=None):
        """Take a screenshot.

        If element is not provided, the screenshot should be of the
        current page, otherwise the screenshot should be of the given element.
        """
        if self.mode == 'strict':
            pass # implement when endpoint is defined
        elif self.mode == 'compatibility':
            if element:
                pass # element screenshots are unsupported in compatibility
            else:
                return base64.standard_b64decode(
                    self.execute('GET', '/screenshot', 'takeScreenshot'))

    def add_cookie(self, cookie):
        """Add a cookie to the browser."""
        self.execute('POST', '/cookie', 'addCookie', { 'cookie': cookie })

    def get_cookie(self, name = None):
        """Get the cookies accessible from the current page."""
        if self.mode == 'compatibility':
            cookies = self.execute('GET', '/cookie', 'getCookie')
            if name:
                cookies_ = []
                for cookie in cookies:
                    if cookie['name'] == name:
                        cookies_.append(cookie)
                return cookies_
            return cookies
        elif self.mode == 'strict':
             pass # implement when wire protocol for this has been defined

    def set_implicit_timeout(self, ms):
        self._set_timeout('implicit', ms)

    def set_page_load_timeout(self, ms):
        self._set_timeout('page load', ms)

    def set_script_timeout(self, ms):
        self._set_timeout('script', ms)

    def _set_timeout(self, type, ms):
        params = { 'type': type, 'ms': ms }
        self.execute('POST', '/timeouts', 'timeouts', params)

    def _window_command(self, method, path, name, parameters = None):
        if self.mode == 'compatibility':
            return self.execute(
                method, '/window/current' + path, name, parameters)
        elif self.mode == 'strict':
            pass # implement this when end-points are defined in doc

    def _object_hook(self, obj):
        if 'ELEMENT' in obj:
            return webelement.WebElement(self, obj['ELEMENT'])
        return obj

