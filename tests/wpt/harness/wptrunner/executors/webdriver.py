import errno
import httplib
import json
import socket
import time
import urlparse
from collections import defaultdict

element_key = "element-6066-11e4-a52e-4f735466cecf"


class WebDriverException(Exception):
    http_status = None
    status_code = None

    def __init__(self, message):
        self.message = message


class ElementNotSelectableException(WebDriverException):
    http_status = 400
    status_code = "element not selectable"


class ElementNotVisibleException(WebDriverException):
    http_status = 400
    status_code = "element not visible"


class InvalidArgumentException(WebDriverException):
    http_status = 400
    status_code = "invalid argument"


class InvalidCookieDomainException(WebDriverException):
    http_status = 400
    status_code = "invalid cookie domain"


class InvalidElementCoordinatesException(WebDriverException):
    http_status = 400
    status_code = "invalid element coordinates"


class InvalidElementStateException(WebDriverException):
    http_status = 400
    status_code = "invalid cookie domain"


class InvalidSelectorException(WebDriverException):
    http_status = 400
    status_code = "invalid selector"


class InvalidSessionIdException(WebDriverException):
    http_status = 404
    status_code = "invalid session id"


class JavascriptErrorException(WebDriverException):
    http_status = 500
    status_code = "javascript error"


class MoveTargetOutOfBoundsException(WebDriverException):
    http_status = 500
    status_code = "move target out of bounds"


class NoSuchAlertException(WebDriverException):
    http_status = 400
    status_code = "no such alert"


class NoSuchElementException(WebDriverException):
    http_status = 404
    status_code = "no such element"


class NoSuchFrameException(WebDriverException):
    http_status = 400
    status_code = "no such frame"


class NoSuchWindowException(WebDriverException):
    http_status = 400
    status_code = "no such window"


class ScriptTimeoutException(WebDriverException):
    http_status = 408
    status_code = "script timeout"


class SessionNotCreatedException(WebDriverException):
    http_status = 500
    status_code = "session not created"


class StaleElementReferenceException(WebDriverException):
    http_status = 400
    status_code = "stale element reference"


class TimeoutException(WebDriverException):
    http_status = 408
    status_code = "timeout"


class UnableToSetCookieException(WebDriverException):
    http_status = 500
    status_code = "unable to set cookie"


class UnexpectedAlertOpenException(WebDriverException):
    http_status = 500
    status_code = "unexpected alert open"


class UnknownErrorException(WebDriverException):
    http_status = 500
    status_code = "unknown error"


class UnknownCommandException(WebDriverException):
    http_status = (404, 405)
    status_code = "unknown command"


class UnsupportedOperationException(WebDriverException):
    http_status = 500
    status_code = "unsupported operation"


def group_exceptions():
    exceptions = defaultdict(dict)
    for item in _objs:
        if type(item) == type and issubclass(item, WebDriverException):
            if not isinstance(item.http_status, tuple):
                statuses = (item.http_status,)
            else:
                statuses = item.http_status

            for status in statuses:
                exceptions[status][item.status_code] = item
    return exceptions


_objs = locals().values()
_exceptions = group_exceptions()
del _objs
del group_exceptions


def wait_for_port(host, port, timeout=60):
    """ Wait for the specified Marionette host/port to be available."""
    starttime = time.time()
    poll_interval = 0.1
    while time.time() - starttime < timeout:
        sock = None
        try:
            sock = socket.socket()
            sock.connect((host, port))
            return True
        except socket.error as e:
            if e[0] != errno.ECONNREFUSED:
                raise
        finally:
            if sock:
                sock.close()
        time.sleep(poll_interval)
    return False


class Transport(object):
    def __init__(self, host, port, url_prefix="", port_timeout=60):
        self.host = host
        self.port = port
        self.port_timeout = port_timeout
        if url_prefix == "":
            self.path_prefix = "/"
        else:
            self.path_prefix = "/%s/" % url_prefix.strip("/")
        self._connection = None

    def connect(self):
        wait_for_port(self.host, self.port, self.port_timeout)
        self._connection = httplib.HTTPConnection(self.host, self.port)

    def close_connection(self):
        if self._connection:
            self._connection.close()
        self._connection = None

    def url(self, suffix):
        return urlparse.urljoin(self.url_prefix, suffix)

    def send(self, method, url, body=None, headers=None, key=None):
        if not self._connection:
            self.connect()

        if body is None and method == "POST":
            body = {}

        if isinstance(body, dict):
            body = json.dumps(body)

        if isinstance(body, unicode):
            body = body.encode("utf-8")

        if headers is None:
            headers = {}

        url = self.path_prefix + url

        self._connection.request(method, url, body, headers)

        try:
            resp = self._connection.getresponse()
        except Exception:
            # This should probably be more specific
            raise IOError
        body = resp.read()

        try:
            data = json.loads(body)
        except:
            raise
            raise WebDriverException("Could not parse response body as JSON: %s" % body)

        if resp.status != 200:
            cls = _exceptions.get(resp.status, {}).get(data.get("status", None), WebDriverException)
            raise cls(data.get("message", ""))

        if key is not None:
            data = data[key]

        if not data:
            data = None

        return data


def command(func):
    def inner(self, *args, **kwargs):
        if hasattr(self, "session"):
            session_id = self.session.session_id
        else:
            session_id = self.session_id

        if session_id is None:
            raise SessionNotCreatedException("Session not created")
        return func(self, *args, **kwargs)

    inner.__name__ = func.__name__
    inner.__doc__ = func.__doc__

    return inner


class Timeouts(object):
    def __init__(self, session):
        self.session = session
        self._script = 30
        self._load = 0
        self._implicit_wait = 0

    def _set_timeouts(self, name, value):
        body = {"type": name,
                "ms": value * 1000}
        return self.session.send_command("POST", "timeouts", body)

    @property
    def script(self):
        return self._script

    @script.setter
    def script(self, value):
        self._set_timeouts("script", value)
        self._script = value

    @property
    def load(self):
        return self._load

    @load.setter
    def set_load(self, value):
        self._set_timeouts("page load", value)
        self._script = value

    @property
    def implicit_wait(self):
        return self._implicit_wait

    @implicit_wait.setter
    def implicit_wait(self, value):
        self._set_timeouts("implicit wait", value)
        self._implicit_wait = value


class Window(object):
    def __init__(self, session):
        self.session = session

    @property
    @command
    def size(self):
        return self.session.send_command("GET", "window/size")

    @size.setter
    @command
    def size(self, (height, width)):
        body = {"width": width,
                "height": height}

        return self.session.send_command("POST", "window/size", body)

    @property
    @command
    def maximize(self):
        return self.session.send_command("POST", "window/maximize")


class Find(object):
    def __init__(self, session):
        self.session = session

    @command
    def css(self, selector, all=True):
        return self._find_element("css selector", selector, all)

    def _find_element(self, strategy, selector, all):
        route = "elements" if all else "element"

        body = {"using": strategy,
                "value": selector}

        data = self.session.send_command("POST", route, body, key="value")

        if all:
            rv = [self.session._element(item) for item in data]
        else:
            rv = self.session._element(data)

        return rv


class Session(object):
    def __init__(self, host, port, url_prefix="", desired_capabilities=None, port_timeout=60,
                 extension=None):
        self.transport = Transport(host, port, url_prefix, port_timeout)
        self.desired_capabilities = desired_capabilities
        self.session_id = None
        self.timeouts = None
        self.window = None
        self.find = None
        self._element_cache = {}
        self.extension = None
        self.extension_cls = extension

    def start(self):
        desired_capabilities = self.desired_capabilities if self.desired_capabilities else {}
        body = {"capabilities": {"desiredCapabilites": desired_capabilities}}

        rv = self.transport.send("POST", "session", body=body)
        self.session_id = rv["sessionId"]

        self.timeouts = Timeouts(self)
        self.window = Window(self)
        self.find = Find(self)
        if self.extension_cls:
            self.extension = self.extension_cls(self)

        return rv["value"]

    @command
    def end(self):
        url = "session/%s" % self.session_id
        self.transport.send("DELETE", url)
        self.session_id = None
        self.timeouts = None
        self.window = None
        self.find = None
        self.extension = None
        self.transport.close_connection()

    def __enter__(self):
        resp = self.start()
        if resp.error:
            raise Exception(resp)
        return self

    def __exit__(self, *args, **kwargs):
        resp = self.end()
        if resp.error:
            raise Exception(resp)

    def send_command(self, method, url, body=None, key=None):
        url = urlparse.urljoin("session/%s/" % self.session_id, url)
        return self.transport.send(method, url, body, key=key)

    @property
    @command
    def url(self):
        return self.send_command("GET", "url", key="value")

    @url.setter
    @command
    def url(self, url):
        if urlparse.urlsplit(url).netloc is None:
            return self.url(url)
        body = {"url": url}
        return self.send_command("POST", "url", body)

    @command
    def back(self):
        return self.send_command("POST", "back")

    @command
    def forward(self):
        return self.send_command("POST", "forward")

    @command
    def refresh(self):
        return self.send_command("POST", "refresh")

    @property
    @command
    def title(self):
        return self.send_command("GET", "title", key="value")

    @property
    @command
    def handle(self):
        return self.send_command("GET", "window_handle", key="value")

    @handle.setter
    @command
    def handle(self, handle):
        body = {"handle": handle}
        return self.send_command("POST", "window", body=body)

    def switch_frame(self, frame):
        if frame == "parent":
            url = "frame/parent"
            body = None
        else:
            url = "frame"
            if isinstance(frame, Element):
                body = {"id": frame.json()}
            else:
                body = {"id": frame}

        return self.send_command("POST", url, body)

    @command
    def close(self):
        return self.send_command("DELETE", "window_handle")

    @property
    @command
    def handles(self):
        return self.send_command("GET", "window_handles", key="value")

    @property
    @command
    def active_element(self):
        data = self.send_command("GET", "element/active", key="value")
        if data is not None:
            return self._element(data)

    def _element(self, data):
        elem_id = data[element_key]
        assert elem_id
        if elem_id in self._element_cache:
            return self._element_cache[elem_id]
        return Element(self, elem_id)

    @command
    def cookies(self, name=None):
        if name is None:
            url = "cookie"
        else:
            url = "cookie/%s" % name
        return self.send_command("GET", url, {}, key="value")

    @command
    def set_cookie(self, name, value, path=None, domain=None, secure=None, expiry=None):
        body = {"name": name,
                "value": value}
        if path is not None:
            body["path"] = path
        if domain is not None:
            body["domain"] = domain
        if secure is not None:
            body["secure"] = secure
        if expiry is not None:
            body["expiry"] = expiry
        self.send_command("POST", "cookie", body)

    def delete_cookie(self, name=None):
        if name is None:
            url = "cookie"
        else:
            url = "cookie/%s" % name
        self.send_command("DELETE", url, {}, key="value")

    #[...]

    @command
    def execute_script(self, script, args=None):
        if args is None:
            args = []

        body = {
            "script": script,
            "args": args
        }
        return self.send_command("POST", "execute", body, key="value")

    @command
    def execute_async_script(self, script, args=None):
        if args is None:
            args = []

        body = {
            "script": script,
            "args": args
        }
        return self.send_command("POST", "execute_async", body, key="value")

    #[...]

    @command
    def screenshot(self):
        return self.send_command("GET", "screenshot", key="value")


class Element(object):
    def __init__(self, session, id):
        self.session = session
        self.id = id
        assert id not in self.session._element_cache
        self.session._element_cache[self.id] = self

    def json(self):
        return {element_key: self.id}

    @property
    def session_id(self):
        return self.session.session_id

    def url(self, suffix):
        return "element/%s/%s" % (self.id, suffix)

    @command
    def find_element(self, strategy, selector):
        body = {"using": strategy,
                "value": selector}

        elem = self.session.send_command("POST", self.url("element"), body, key="value")
        return self.session.element(elem)

    @command
    def click(self):
        self.session.send_command("POST", self.url("click"), {})

    @command
    def tap(self):
        self.session.send_command("POST", self.url("tap"), {})

    @command
    def clear(self):
        self.session.send_command("POST", self.url("clear"), {})

    @command
    def send_keys(self, keys):
        if isinstance(keys, (str, unicode)):
            keys = [char for char in keys]

        body = {"value": keys}

        return self.session.send_command("POST", self.url("value"), body)

    @property
    @command
    def text(self):
        return self.session.send_command("GET", self.url("text"))

    @property
    @command
    def name(self):
        return self.session.send_command("GET", self.url("name"))

    @command
    def css(self, property_name):
        return self.session.send_command("GET", self.url("css/%s" % property_name))

    @property
    @command
    def rect(self):
        return self.session.send_command("GET", self.url("rect"))

class ServoExtensions(object):
    def __init__(self, session):
        self.session = session

    @command
    def get_prefs(self, *prefs):
        body = {"prefs": list(prefs)}
        return self.session.send_command("POST", "servo/prefs/get", body)

    @command
    def set_prefs(self, prefs):
        body = {"prefs": prefs}
        return self.session.send_command("POST", "servo/prefs/set", body)

    @command
    def reset_prefs(self, *prefs):
        body = {"prefs": list(prefs)}
        return self.session.send_command("POST", "servo/prefs/reset", body)
