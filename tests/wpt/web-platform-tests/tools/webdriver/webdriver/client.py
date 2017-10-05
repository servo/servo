import urlparse

import error
import transport


element_key = "element-6066-11e4-a52e-4f735466cecf"


def command(func):
    def inner(self, *args, **kwargs):
        if hasattr(self, "session"):
            session = self.session
        else:
            session = self

        if session.session_id is None:
            session.start()
        assert session.session_id is not None

        return func(self, *args, **kwargs)

    inner.__name__ = func.__name__
    inner.__doc__ = func.__doc__

    return inner


class Timeouts(object):

    def __init__(self, session):
        self.session = session

    def _get(self, key=None):
        timeouts = self.session.send_session_command("GET", "timeouts")
        if key is not None:
            return timeouts[key]
        return timeouts

    def _set(self, key, secs):
        body = {key: secs * 1000}
        timeouts = self.session.send_session_command("POST", "timeouts", body)
        return timeouts[key]

    @property
    def script(self):
        return self._get("script")

    @script.setter
    def script(self, secs):
        return self._set("script", secs)

    @property
    def page_load(self):
        return self._get("pageLoad")

    @page_load.setter
    def page_load(self, secs):
        return self._set("pageLoad", secs)

    @property
    def implicit(self):
        return self._get("implicit")

    @implicit.setter
    def implicit(self, secs):
        return self._set("implicit", secs)

    def __str__(self):
        name = "%s.%s" % (self.__module__, self.__class__.__name__)
        return "<%s script=%d, load=%d, implicit=%d>" % \
            (name, self.script, self.page_load, self.implicit)


class ActionSequence(object):
    """API for creating and performing action sequences.

    Each action method adds one or more actions to a queue. When perform()
    is called, the queued actions fire in order.

    May be chained together as in::

         ActionSequence(session, "key", id) \
            .key_down("a") \
            .key_up("a") \
            .perform()
    """
    def __init__(self, session, action_type, input_id, pointer_params=None):
        """Represents a sequence of actions of one type for one input source.

        :param session: WebDriver session.
        :param action_type: Action type; may be "none", "key", or "pointer".
        :param input_id: ID of input source.
        :param pointer_params: Optional dictionary of pointer parameters.
        """
        self.session = session
        self._id = input_id
        self._type = action_type
        self._actions = []
        self._pointer_params = pointer_params

    @property
    def dict(self):
        d = {
            "type": self._type,
            "id": self._id,
            "actions": self._actions,
        }
        if self._pointer_params is not None:
            d["parameters"] = self._pointer_params
        return d

    @command
    def perform(self):
        """Perform all queued actions."""
        self.session.actions.perform([self.dict])

    def _key_action(self, subtype, value):
        self._actions.append({"type": subtype, "value": value})

    def _pointer_action(self, subtype, button):
        self._actions.append({"type": subtype, "button": button})

    def pause(self, duration):
        self._actions.append({"type": "pause", "duration": duration})
        return self

    def pointer_move(self, x, y, duration=None, origin=None):
        """Queue a pointerMove action.

        :param x: Destination x-axis coordinate of pointer in CSS pixels.
        :param y: Destination y-axis coordinate of pointer in CSS pixels.
        :param duration: Number of milliseconds over which to distribute the
                         move. If None, remote end defaults to 0.
        :param origin: Origin of coordinates, either "viewport", "pointer" or
                       an Element. If None, remote end defaults to "viewport".
        """
        action = {
            "type": "pointerMove",
            "x": x,
            "y": y
        }
        if duration is not None:
            action["duration"] = duration
        if origin is not None:
            action["origin"] = origin if isinstance(origin, basestring) else origin.json()
        self._actions.append(action)
        return self

    def pointer_up(self, button=0):
        """Queue a pointerUp action for `button`.

        :param button: Pointer button to perform action with.
                       Default: 0, which represents main device button.
        """
        self._pointer_action("pointerUp", button)
        return self

    def pointer_down(self, button=0):
        """Queue a pointerDown action for `button`.

        :param button: Pointer button to perform action with.
                       Default: 0, which represents main device button.
        """
        self._pointer_action("pointerDown", button)
        return self

    def click(self, element=None, button=0):
        """Queue a click with the specified button.

        If an element is given, move the pointer to that element first,
        otherwise click current pointer coordinates.

        :param element: Optional element to click.
        :param button: Integer representing pointer button to perform action
                       with. Default: 0, which represents main device button.
        """
        if element:
            self.pointer_move(0, 0, origin=element)
        return self.pointer_down(button).pointer_up(button)

    def key_up(self, value):
        """Queue a keyUp action for `value`.

        :param value: Character to perform key action with.
        """
        self._key_action("keyUp", value)
        return self

    def key_down(self, value):
        """Queue a keyDown action for `value`.

        :param value: Character to perform key action with.
        """
        self._key_action("keyDown", value)
        return self

    def send_keys(self, keys):
        """Queue a keyDown and keyUp action for each character in `keys`.

        :param keys: String of keys to perform key actions with.
        """
        for c in keys:
            self.key_down(c)
            self.key_up(c)
        return self


class Actions(object):
    def __init__(self, session):
        self.session = session

    @command
    def perform(self, actions=None):
        """Performs actions by tick from each action sequence in `actions`.

        :param actions: List of input source action sequences. A single action
                        sequence may be created with the help of
                        ``ActionSequence.dict``.
        """
        body = {"actions": [] if actions is None else actions}
        return self.session.send_session_command("POST", "actions", body)

    @command
    def release(self):
        return self.session.send_session_command("DELETE", "actions")

    def sequence(self, *args, **kwargs):
        """Return an empty ActionSequence of the designated type.

        See ActionSequence for parameter list.
        """
        return ActionSequence(self.session, *args, **kwargs)


class Window(object):
    def __init__(self, session):
        self.session = session

    @property
    @command
    def size(self):
        resp = self.session.send_session_command("GET", "window/rect")
        return (resp["width"], resp["height"])

    @size.setter
    @command
    def size(self, data):
        width, height = data
        body = {"width": width, "height": height}
        self.session.send_session_command("POST", "window/rect", body)

    @property
    @command
    def position(self):
        resp = self.session.send_session_command("GET", "window/rect")
        return (resp["x"], resp["y"])

    @position.setter
    @command
    def position(self, data):
        data = x, y
        body = {"x": x, "y": y}
        self.session.send_session_command("POST", "window/rect", body)

    @property
    @command
    def maximize(self):
        return self.session.send_session_command("POST", "window/maximize")


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

        data = self.session.send_session_command("POST", route, body)

        if all:
            rv = [self.session._element(item) for item in data]
        else:
            rv = self.session._element(data)

        return rv


class Cookies(object):
    def __init__(self, session):
        self.session = session

    def __getitem__(self, name):
        self.session.send_session_command("GET", "cookie/%s" % name, {})

    def __setitem__(self, name, value):
        cookie = {"name": name,
                  "value": None}

        if isinstance(name, (str, unicode)):
            cookie["value"] = value
        elif hasattr(value, "value"):
            cookie["value"] = value.value
        self.session.send_session_command("POST", "cookie/%s" % name, {})


class UserPrompt(object):
    def __init__(self, session):
        self.session = session

    @command
    def dismiss(self):
        self.session.send_session_command("POST", "alert/dismiss")

    @command
    def accept(self):
        self.session.send_session_command("POST", "alert/accept")

    @property
    @command
    def text(self):
        return self.session.send_session_command("GET", "alert/text")

    @text.setter
    @command
    def text(self, value):
        body = {"value": list(value)}
        self.session.send_session_command("POST", "alert/text", body=body)


class Session(object):
    def __init__(self, host, port, url_prefix="/", capabilities=None,
                 timeout=None, extension=None):
        self.transport = transport.HTTPWireProtocol(
            host, port, url_prefix, timeout=timeout)
        self.capabilities = capabilities
        self.session_id = None
        self.timeouts = None
        self.window = None
        self.find = None
        self._element_cache = {}
        self.extension = None
        self.extension_cls = extension

        self.timeouts = Timeouts(self)
        self.window = Window(self)
        self.find = Find(self)
        self.alert = UserPrompt(self)
        self.actions = Actions(self)

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, *args, **kwargs):
        self.end()

    def __del__(self):
        self.end()

    def start(self):
        if self.session_id is not None:
            return

        body = {}

        if self.capabilities is not None:
            body["capabilities"] = self.capabilities

        value = self.send_command("POST", "session", body=body)
        self.session_id = value["sessionId"]

        if self.extension_cls:
            self.extension = self.extension_cls(self)

        return value

    def end(self):
        if self.session_id is None:
            return

        url = "session/%s" % self.session_id
        self.send_command("DELETE", url)

        self.session_id = None
        self.timeouts = None
        self.window = None
        self.find = None
        self.extension = None

    def send_command(self, method, url, body=None):
        """
        Send a command to the remote end and validate its success.

        :param method: HTTP method to use in request.
        :param uri: "Command part" of the HTTP request URL,
            e.g. `window/rect`.
        :param body: Optional body of the HTTP request.

        :return: `None` if the HTTP response body was empty, otherwise
            the result of parsing the body as JSON.

        :raises error.WebDriverException: If the remote end returns
            an error.
        """
        response = self.transport.send(method, url, body)
        value = response.body["value"]

        if response.status != 200:
            cls = error.get(value.get("error"))
            raise cls(value.get("message"))

        return value

    def send_session_command(self, method, uri, body=None):
        """
        Send a command to an established session and validate its success.

        :param method: HTTP method to use in request.
        :param url: "Command part" of the HTTP request URL,
            e.g. `window/rect`.
        :param body: Optional body of the HTTP request.  Must be JSON
            serialisable.

        :return: `None` if the HTTP response body was empty, otherwise
            the result of parsing the body as JSON.

        :raises error.SessionNotCreatedException: If there is no active
            session.
        :raises error.WebDriverException: If the remote end returns
            an error.
        """
        if self.session_id is None:
            raise error.SessionNotCreatedException()

        url = urlparse.urljoin("session/%s/" % self.session_id, uri)
        return self.send_command(method, url, body)

    @property
    @command
    def url(self):
        return self.send_session_command("GET", "url")

    @url.setter
    @command
    def url(self, url):
        if urlparse.urlsplit(url).netloc is None:
            return self.url(url)
        body = {"url": url}
        return self.send_session_command("POST", "url", body)

    @command
    def back(self):
        return self.send_session_command("POST", "back")

    @command
    def forward(self):
        return self.send_session_command("POST", "forward")

    @command
    def refresh(self):
        return self.send_session_command("POST", "refresh")

    @property
    @command
    def title(self):
        return self.send_session_command("GET", "title")

    @property
    @command
    def window_handle(self):
        return self.send_session_command("GET", "window")

    @window_handle.setter
    @command
    def window_handle(self, handle):
        body = {"handle": handle}
        return self.send_session_command("POST", "window", body=body)

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

        return self.send_session_command("POST", url, body)

    @command
    def close(self):
        return self.send_session_command("DELETE", "window")

    @property
    @command
    def handles(self):
        return self.send_session_command("GET", "window/handles")

    @property
    @command
    def active_element(self):
        data = self.send_session_command("GET", "element/active")
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
        return self.send_session_command("GET", url, {})

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
        self.send_session_command("POST", "cookie", {"cookie": body})

    def delete_cookie(self, name=None):
        if name is None:
            url = "cookie"
        else:
            url = "cookie/%s" % name
        self.send_session_command("DELETE", url, {})

    #[...]

    @command
    def execute_script(self, script, args=None):
        if args is None:
            args = []

        body = {
            "script": script,
            "args": args
        }
        return self.send_session_command("POST", "execute/sync", body)

    @command
    def execute_async_script(self, script, args=None):
        if args is None:
            args = []

        body = {
            "script": script,
            "args": args
        }
        return self.send_session_command("POST", "execute/async", body)

    #[...]

    @command
    def screenshot(self):
        return self.send_session_command("GET", "screenshot")


class Element(object):
    def __init__(self, session, id):
        self.session = session
        self.id = id
        assert id not in self.session._element_cache
        self.session._element_cache[self.id] = self

    def send_element_command(self, method, uri, body=None):
        url = "element/%s/%s" % (self.id, uri)
        return self.session.send_session_command(method, url, body)

    def json(self):
        return {element_key: self.id}

    @command
    def find_element(self, strategy, selector):
        body = {"using": strategy,
                "value": selector}

        elem = self.send_element_command("POST", "element", body)
        return self.session.element(elem)

    @command
    def click(self):
        self.send_element_command("POST", "click", {})

    @command
    def tap(self):
        self.send_element_command("POST", "tap", {})

    @command
    def clear(self):
        self.send_element_command("POST", self.url("clear"), {})

    @command
    def send_keys(self, text):
        return self.send_element_command("POST", "value", {"text": text})

    @property
    @command
    def text(self):
        return self.send_element_command("GET", "text")

    @property
    @command
    def name(self):
        return self.send_element_command("GET", "name")

    @command
    def style(self, property_name):
        return self.send_element_command("GET", "css/%s" % property_name)

    @property
    @command
    def rect(self):
        return self.send_element_command("GET", "rect")

    @command
    def property(self, name):
        return self.send_element_command("GET", "property/%s" % name)

    @command
    def attribute(self, name):
        return self.send_element_command("GET", "attribute/%s" % name)
