# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import urlparse

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
        assert session.session_id != None

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


class Cookies(object):
    def __init__(self, session):
        self.session = session

    def __getitem__(self, name):
        self.session.send_command("GET", "cookie/%s" % name, {}, key="value")

    def __setitem__(self, name, value):
        cookie = {"name": name,
                  "value": None}

        if isinstance(name, (str, unicode)):
            cookie["value"] = value
        elif hasattr(value, "value"):
            cookie["value"] = value.value
        self.session.send_command("POST", "cookie/%s" % name, {}, key="value")


class Session(object):
    def __init__(self, host, port, url_prefix="", desired_capabilities=None,
                 required_capabilities=None, timeout=60, extension=None):
        self.transport = transport.HTTPWireProtocol(
            host, port, url_prefix, timeout=timeout)
        self.desired_capabilities = desired_capabilities
        self.required_capabilities = required_capabilities
        self.session_id = None
        self.timeouts = None
        self.window = None
        self.find = None
        self._element_cache = {}
        self.extension = None
        self.extension_cls = extension

    def start(self):
        if self.session_id is not None:
            return

        body = {}

        caps = {}
        if self.desired_capabilities is not None:
            caps["desiredCapabilities"] = self.desired_capabilities
        if self.required_capabilities is not None:
            caps["requiredCapabilities"] = self.required_capabilities
        body["capabilities"] = caps

        resp = self.transport.send("POST", "session", body=body)
        self.session_id = resp["sessionId"]

        self.timeouts = Timeouts(self)
        self.window = Window(self)
        self.find = Find(self)
        if self.extension_cls:
            self.extension = self.extension_cls(self)

        return resp["value"]

    def end(self):
        if self.session_id is None:
            return

        url = "session/%s" % self.session_id
        self.transport.send("DELETE", url)

        self.session_id = None
        self.timeouts = None
        self.window = None
        self.find = None
        self.extension = None
        self.transport.disconnect()

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
    def window_handle(self):
        return self.send_command("GET", "window_handle", key="value")

    @window_handle.setter
    @command
    def window_handle(self, handle):
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
        self.send_command("POST", "cookie", {"cookie": body})

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
    def style(self, property_name):
        return self.session.send_command("GET", self.url("css/%s" % property_name))

    @property
    @command
    def rect(self):
        return self.session.send_command("GET", self.url("rect"))

    @command
    def attribute(self, name):
        return self.session.send_command("GET", self.url("attribute/%s" % name))
