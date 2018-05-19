# Adding new commands to testdriver.js

## Assumptions
We assume the following in this writeup:
 - You know what web-platform-tests is and you have a working checkout and can run tests
 - You know what WebDriver or Selenium is
 - Familiarity with JavaScript and Python

## Introduction!

Let's implement window resizing. We can do this via the [Set Window Rect](https://w3c.github.io/webdriver/webdriver-spec.html#dfn-set-window-rect) command in WebDriver.

First, we need to think of what the API will look like a little. We will be using Selenium and Marionette for this, so we can look and see that they take in x, y coordinates, width and height integers.

The first part of this will be browser agnostic, but later we will need to implement a specific layer for each browser (here we will do Firefox and Chrome).

## Code!

### [resources/testdriver.js](resources/testdriver.js)

This is the main entry point the tests get. Here we need to add a function to the `test_driver` object that will call the `test_driver_internal` object.

```javascript
window.test_driver = {

    // other commands...

    /**
    * Triggers browser window to be resized and relocated
    *
    * This matches the behaviour of the {@link
    * https://w3c.github.io/webdriver/webdriver-spec.html#dfn-set-window-rect|WebDriver
    * Set Window Rect command}.
    *
    * @param {Integer} x - The x coordinate of the top left of the window
    * @param {Integer} y - The x coordinate of the top left of the window
    * @param {Integer} width - The width of the window
    * @param {Integer} height - The width of the window
    * @returns {Promise} fulfilled after window rect is set occurs, or rejected in
    *                    the cases the WebDriver command errors
    */
    set_window_rect: function(x, y, width, height) {
        return window.test_driver_internal.set_element_rect(x, y, width, height);
    }
```

In the same file, lets add to the internal object. ( do we need to do this?) (make sure to do this if the internal call has different arguments than the external call, especially if it calls multiple internal calls)

```javascript
window.test_driver_internal = {

    // other commands...

    /**
     * Triggers browser window to be resized and relocated
     *
     * This matches the behaviour of the {@link
     * https://w3c.github.io/webdriver/webdriver-spec.html#dfn-set-window-rect|WebDriver
     * Set Window Rect command}.
     *
     * @param {Integer} x - The x coordinate of the top left of the window
     * @param {Integer} y - The x coordinate of the top left of the window
     * @param {Integer} width - The width of the window
     * @param {Integer} height - The width of the window
     * @returns {Promise} fulfilled after window rect is set occurs, or rejected in
     *                    the cases the WebDriver command errors
     */
    set_window_rect: function(x, y, width, height) {
        return Promise.reject(new Error("unimplemented"))
    }
```
We will leave this unimplemented and override it in another file. Lets do that now!

### [wptrunner/wptrunner/testdriver-extra.js](tools/wptrunner/wptrunner/testdriver-extra.js)

This will be the default function called when invoking the test driver commands (sometimes it is overridden by testdriver-vendor.js, but this is outside the scope of this writeup).

```javascript
window.test_driver_internal.set_element_rect = function(x, y, width, height) {
    const pending_promise = new Promise(function(resolve, reject) {
        pending_resolve = resolve;
        pending_reject = reject;
    });
    window.opener.postMessage(
        {"type": "action", "action": "set_window_rect", "x": x, "y": y, "width": width, "height": height}, "*");
    return pending_promise;
};
```
The main thing here is the `postMessage` argument. The first argument is an object with properties
 - `type`: this always has to be the string `"action"`
 - `action`: the name of the testdriver command this defines (in this case, `set_window_rect`)
 - any other things you want to pass to the next point of execution (in this case, the x, y coordinates and the width and height)

<!-- The pending promise needs to be there as it is resolved when the window recieves a completion message from the executor. -->
The pending promise is out of scope of this function and is resolved when the window recieves a completion message from the executor.
This happens here in the same file:

```javascript
    let pending_resolve = null;
    let pending_reject = null;
    window.addEventListener("message", function(event) {
        const data = event.data;

        if (typeof data !== "object" && data !== null) {
            return;
        }

        if (data.type !== "testdriver-complete") {
            return;
        }

        if (data.status === "success") {
            pending_resolve();
        } else {
            pending_reject();
        }
    });
```

One limitation this introduces is that only one testdriver call can be made at one time since the `pending_resolve` and `pending_reject` variables are in an outer scope.

Next, this is passed to the executor and protocol in wptrunner. Time to switch to Python!

[tools/wptrunner/wptrunner/executors/protocol.py](tools/wptrunner/wptrunner/executors/protocol.py)

```python
class SetWindowRectProtocolPart(ProtocolPart):
    """Protocol part for resizing and changing location of window"""
    __metaclass__ = ABCMeta

    name = "set_window_rect"

    @abstractmethod
    def set_window_rect(self, x, y, width, height):
        """Change the window rect

        :param x: The x coordinate of the top left of the window.
        :param y: The y coordinate of the top left of the window.
        :param width: The width of the window.
        :param height: The height of the window."""
        pass
```

Next we change the base executor.

[tools/wptrunner/wptrunner/executors/base.py](tools/wptrunner/wptrunner/executors/base.py)

```python
class CallbackHandler(object):
    """Handle callbacks from testdriver-using tests.

    The default implementation here makes sense for things that are roughly like
    WebDriver. Things that are more different to WebDriver may need to create a
    fully custom implementation."""

    def __init__(self, logger, protocol, test_window):
        self.protocol = protocol
        self.test_window = test_window
        self.logger = logger
        self.callbacks = {
            "action": self.process_action,
            "complete": self.process_complete
        }

        self.actions = {
            "click": ClickAction(self.logger, self.protocol),
            "send_keys": SendKeysAction(self.logger, self.protocol),
            {other actions},
            "set_window_rect": SetWindowRectAction(self.logger, self.protocol) # add this!
        }
```

```python
class SetWindowRectAction(object):
    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        x, y, width, height = payload["x"], payload["y"], payload["width"], payload["height"]
        self.logger.debug("Setting window rect to be: x=%s, y=%s, width=%s, height=%s"
                          .format(x, y, width, height))
        self.protocol.set_window_rect.set_window_rect(x, y, width, height)
```

Don't forget to write docs in ```testdriver.md```.
Now we write the browser specific implementations.

### Chrome

We will use [executorselenium](tools/wptrunner/wptrunner/executors/executorselenium.py) and use the Selenium API (in the future there are plans to use the WebDriver API directly).

There isn't too much work to do here, we just need to define a subclass of the protocol part we defined earlier.

```python
class SeleniumSetWindowRectProtocolPart(SetWindowRectProtocolPart):
    def setup(self):
        self.webdriver = self.parent.webdriver

    def set_window_rect(self, x, y, width, height):
        return self.webdriver.set_window_rect(x, y, width, height)
```

Make sure to import the protocol part too!

```python
from .protocol import (BaseProtocolPart,
                       TestharnessProtocolPart,
                       Protocol,
                       SelectorProtocolPart,
                       ClickProtocolPart,
                       SendKeysProtocolPart,
                       {... other protocol parts}
                       SetWindowRectProtocolPart, # add this!
                       TestDriverProtocolPart)
```

Here we have the setup method which just redefines the webdriver object at this level. The important part is the `set_window_rect` function (and it's important it is named that since we called it that earlier). This will be call the Selenium API for [set window rect](http://selenium-python.readthedocs.io/api.html#selenium.webdriver.remote.webdriver.WebDriver.set_window_rect) (`self.webdriver` is a Selenium WebDriver instance here).

Finally, we just need to tell the SeleniumProtocol to implement this part.

```python
class SeleniumProtocol(Protocol):
    implements = [SeleniumBaseProtocolPart,
                  SeleniumTestharnessProtocolPart,
                  SeleniumSelectorProtocolPart,
                  SeleniumClickProtocolPart,
                  SeleniumSendKeysProtocolPart,
                  {... other protocol parts}
                  SeleniumSetWindowRectProtocolPart,
                  SeleniumTestDriverProtocolPart]
```


### Firefox
We use the [set window rect](http://marionette-client.readthedocs.io/en/master/reference.html#marionette_driver.marionette.Marionette.set_window_rect) Marionette command.

We will use [executormarionette](tools/wptrunner/wptrunner/executors/executormarionette.py) and use the Marionette Python API.

We have little actual work to do here! We just need to define a subclass of the protocol part we defined earlier.

```python
class MarionetteSetWindowRectProtocolPart(SetWindowRectProtocolPart):
    def setup(self):
        self.marionette = self.parent.marionette

    def set_window_rect(self, x, y, width, height):
        return self.marionette.set_window_rect(x, y, width, height)
```

Make sure to import the protocol part too!

```python
from .protocol import (BaseProtocolPart,
                       TestharnessProtocolPart,
                       Protocol,
                       SelectorProtocolPart,
                       ClickProtocolPart,
                       SendKeysProtocolPart,
                       {... other protocol parts}
                       SetWindowRectProtocolPart, # add this!
                       TestDriverProtocolPart)
```

Here we have the setup method which just redefines the webdriver object at this level. The important part is the `set_window_rect` function (and it's important it is named that since we called it that earlier). This will be call the Marionette API for [set window rect](http://marionette-client.readthedocs.io/en/master/reference.html#marionette_driver.marionette.Marionette.set_window_rect) (`self.marionette` is a marionette instance here).

Finally, we just need to tell the SeleniumProtocol to implement this part.

```python
class MarionetteProtocol(Protocol):
    implements = [MarionetteBaseProtocolPart,
                  MarionetteTestharnessProtocolPart,
                  MarionettePrefsProtocolPart,
                  MarionetteStorageProtocolPart,
                  MarionetteSelectorProtocolPart,
                  MarionetteClickProtocolPart,
                  MarionetteSendKeysProtocolPart,
                  {... other protocol parts}
                  MarionetteSetWindowRectProtocolPart # add this
                  MarionetteTestDriverProtocolPart]
```

### Other Browsers

Other browsers may also use executorselenium (such as safari), or a completely new executor (such as servo). For these, you must change the executor in the same way as we did with chrome and firefox.

### Write an infra test

Make sure to add a test to `infrastructure/testdriver` :)

Here is some template code!

```html
<!DOCTYPE html>
<meta charset="utf-8">
<title>TestDriver set window rect method</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>

<script>
promise_test(async t => {
  await test_driver.set_window_rect(100, 100, 100, 100);
  // do something
}
</script>
```
### What about testdriver-vendor.js?

The file [testdriver-vendor.js](resources/testdriver-vendor.js) is the equivalent to testdriver-extra.js above, except is
run instead of testdriver-extra.js in browser specific test environments. For example, in [Chromium LayoutTests](https://cs.chromium.org/chromium/src/third_party/WebKit/LayoutTests/?q=LayoutTests&sq=package:chromium&dr).

### What if I need to return a value from my testdriver API?

We currently don't have this capability, but it is coming soon and will be documented. The bug is [here](https://github.com/w3c/web-platform-tests/issues/10716)

