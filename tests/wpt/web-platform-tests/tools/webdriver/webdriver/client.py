from . import error
from . import protocol
from . import transport

from six import string_types
from six.moves.urllib import parse as urlparse


def command(func):
    def inner(self, *args, **kwargs):
        if hasattr(self, "session"):
            session = self.session
        else:
            session = self

        if session.session_id is None:
            session.start()

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
        self.session.send_session_command("POST", "timeouts", body)
        return None

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
            action["origin"] = origin
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
        actions = self.session.send_session_command("POST", "actions", body)
        """WebDriver window should be set to the top level window when wptrunner
        processes the next event.
        """
        self.session.switch_frame(None)
        return actions

    @command
    def release(self):
        return self.session.send_session_command("DELETE", "actions")

    def sequence(self, *args, **kwargs):
        """Return an empty ActionSequence of the designated type.

        See ActionSequence for parameter list.
        """
        return ActionSequence(self.session, *args, **kwargs)


class Window(object):
    identifier = "window-fcc6-11e5-b4f8-330a88ab9d7f"

    def __init__(self, session):
        self.session = session

    @property
    @command
    def rect(self):
        return self.session.send_session_command("GET", "window/rect")

    @property
    @command
    def size(self):
        """Gets the window size as a tuple of `(width, height)`."""
        rect = self.rect
        return (rect["width"], rect["height"])

    @size.setter
    @command
    def size(self, new_size):
        """Set window size by passing a tuple of `(width, height)`."""
        width, height = new_size
        body = {"width": width, "height": height}
        self.session.send_session_command("POST", "window/rect", body)

    @property
    @command
    def position(self):
        """Gets the window position as a tuple of `(x, y)`."""
        rect = self.rect
        return (rect["x"], rect["y"])

    @position.setter
    @command
    def position(self, new_position):
        """Set window position by passing a tuple of `(x, y)`."""
        x, y = new_position
        body = {"x": x, "y": y}
        self.session.send_session_command("POST", "window/rect", body)

    @command
    def maximize(self):
        return self.session.send_session_command("POST", "window/maximize")

    @command
    def minimize(self):
        return self.session.send_session_command("POST", "window/minimize")

    @command
    def fullscreen(self):
        return self.session.send_session_command("POST", "window/fullscreen")

    @classmethod
    def from_json(cls, json, session):
        uuid = json[Window.identifier]
        return cls(uuid, session)


class Frame(object):
    identifier = "frame-075b-4da1-b6ba-e579c2d3230a"

    def __init__(self, session):
        self.session = session

    @classmethod
    def from_json(cls, json, session):
        uuid = json[Frame.identifier]
        return cls(uuid, session)


class Find(object):
    def __init__(self, session):
        self.session = session

    @command
    def css(self, element_selector, all=True, frame="window"):
        if (frame != "window"):
            self.session.switch_frame(frame)
        elements = self._find_element("css selector", element_selector, all)
        return elements

    def _find_element(self, strategy, selector, all):
        route = "elements" if all else "element"
        body = {"using": strategy,
                "value": selector}
        return self.session.send_session_command("POST", route, body)


class Cookies(object):
    def __init__(self, session):
        self.session = session

    def __getitem__(self, name):
        self.session.send_session_command("GET", "cookie/%s" % name, {})

    def __setitem__(self, name, value):
        cookie = {"name": name,
                  "value": None}

        if isinstance(name, string_types):
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
        body = {"text": value}
        self.session.send_session_command("POST", "alert/text", body=body)


class Session(object):
    def __init__(self,
                 host,
                 port,
                 url_prefix="/",
                 capabilities=None,
                 timeout=None,
                 extension=None):
        self.transport = transport.HTTPWireProtocol(
            host, port, url_prefix, timeout=timeout)
        self.requested_capabilities = capabilities
        self.capabilities = None
        self.session_id = None
        self.timeouts = None
        self.window = None
        self.find = None
        self.extension = None
        self.extension_cls = extension

        self.timeouts = Timeouts(self)
        self.window = Window(self)
        self.find = Find(self)
        self.alert = UserPrompt(self)
        self.actions = Actions(self)

    def __repr__(self):
        return "<%s %s>" % (self.__class__.__name__, self.session_id or "(disconnected)")

    def __eq__(self, other):
        return (self.session_id is not None and isinstance(other, Session) and
                self.session_id == other.session_id)

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, *args, **kwargs):
        self.end()

    def __del__(self):
        self.end()

    def start(self):
        """Start a new WebDriver session.

        :return: Dictionary with `capabilities` and `sessionId`.

        :raises error.WebDriverException: If the remote end returns
            an error.
        """
        if self.session_id is not None:
            return

        body = {"capabilities": {}}

        if self.requested_capabilities is not None:
            body["capabilities"] = self.requested_capabilities

        value = self.send_command("POST", "session", body=body)
        self.session_id = value["sessionId"]
        self.capabilities = value["capabilities"]

        if self.extension_cls:
            self.extension = self.extension_cls(self)

        return value

    def end(self):
        """Try to close the active session."""
        if self.session_id is None:
            return

        try:
            self.send_command("DELETE", "session/%s" % self.session_id)
        except error.InvalidSessionIdException:
            pass
        finally:
            self.session_id = None

    def send_command(self, method, url, body=None):
        """
        Send a command to the remote end and validate its success.

        :param method: HTTP method to use in request.
        :param uri: "Command part" of the HTTP request URL,
            e.g. `window/rect`.
        :param body: Optional body of the HTTP request.

        :return: `None` if the HTTP response body was empty, otherwise
            the `value` field returned after parsing the response
            body as JSON.

        :raises error.WebDriverException: If the remote end returns
            an error.
        :raises ValueError: If the response body does not contain a
            `value` key.
        """
        response = self.transport.send(
            method, url, body,
            encoder=protocol.Encoder, decoder=protocol.Decoder,
            session=self)

        if response.status != 200:
            err = error.from_response(response)

            if isinstance(err, error.InvalidSessionIdException):
                # The driver could have already been deleted the session.
                self.session_id = None

            raise err

        if "value" in response.body:
            value = response.body["value"]
            """
            Edge does not yet return the w3c session ID.
            We want the tests to run in Edge anyway to help with REC.
            In order to run the tests in Edge, we need to hack around
            bug:
            https://developer.microsoft.com/en-us/microsoft-edge/platform/issues/14641972
            """
            if url == "session" and method == "POST" and "sessionId" in response.body and "sessionId" not in value:
                value["sessionId"] = response.body["sessionId"]
        else:
            raise ValueError("Expected 'value' key in response body:\n"
                "%s" % response)

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

        :raises error.WebDriverException: If the remote end returns
            an error.
        """
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
    def source(self):
        return self.send_session_command("GET", "source")

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
            body = {"id": frame}

        return self.send_session_command("POST", url, body)

    @command
    def close(self):
        handles = self.send_session_command("DELETE", "window")
        if handles is not None and len(handles) == 0:
            # With no more open top-level browsing contexts, the session is closed.
            self.session_id = None

        return handles

    @property
    @command
    def handles(self):
        return self.send_session_command("GET", "window/handles")

    @property
    @command
    def active_element(self):
        return self.send_session_command("GET", "element/active")

    @command
    def cookies(self, name=None):
        if name is None:
            url = "cookie"
        else:
            url = "cookie/%s" % name
        return self.send_session_command("GET", url, {})

    @command
    def set_cookie(self, name, value, path=None, domain=None,
            secure=None, expiry=None, http_only=None):
        body = {
            "name": name,
            "value": value,
        }

        if domain is not None:
            body["domain"] = domain
        if expiry is not None:
            body["expiry"] = expiry
        if http_only is not None:
            body["httpOnly"] = http_only
        if path is not None:
            body["path"] = path
        if secure is not None:
            body["secure"] = secure
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
    """
    Representation of a web element.

    A web element is an abstraction used to identify an element when
    it is transported via the protocol, between remote- and local ends.
    """
    identifier = "element-6066-11e4-a52e-4f735466cecf"

    def __init__(self, id, session):
        """
        Construct a new web element representation.

        :param id: Web element UUID which must be unique across
            all browsing contexts.
        :param session: Current ``webdriver.Session``.
        """
        self.id = id
        self.session = session

    def __repr__(self):
        return "<%s %s>" % (self.__class__.__name__, self.id)

    def __eq__(self, other):
        return (isinstance(other, Element) and self.id == other.id and
                self.session == other.session)

    @classmethod
    def from_json(cls, json, session):
        uuid = json[Element.identifier]
        return cls(uuid, session)

    def send_element_command(self, method, uri, body=None):
        url = "element/%s/%s" % (self.id, uri)
        return self.session.send_session_command(method, url, body)

    @command
    def find_element(self, strategy, selector):
        body = {"using": strategy,
                "value": selector}
        return self.send_element_command("POST", "element", body)

    @command
    def click(self):
        self.send_element_command("POST", "click", {})

    @command
    def tap(self):
        self.send_element_command("POST", "tap", {})

    @command
    def clear(self):
        self.send_element_command("POST", "clear", {})

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

    @property
    @command
    def selected(self):
        return self.send_element_command("GET", "selected")

    @command
    def screenshot(self):
        return self.send_element_command("GET", "screenshot")

    @command
    def attribute(self, name):
        return self.send_element_command("GET", "attribute/%s" % name)

    # This MUST come last because otherwise @property decorators above
    # will be overridden by this.
    @command
    def property(self, name):
        return self.send_element_command("GET", "property/%s" % name)
