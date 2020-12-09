# Testdriver extension tutorial
Adding new commands to testdriver.js

## Assumptions
We assume the following in this writeup:
 - You know what web-platform-tests is and you have a working checkout and can run tests
 - You know what WebDriver is
 - Familiarity with JavaScript and Python

## Introduction!

Let's implement window resizing. We can do this via the [Set Window Rect](https://w3c.github.io/webdriver/#set-window-rect) command in WebDriver.

First, we need to think of what the API will look like a little. We will be using WebDriver and Marionette for this, so we can look and see that they take in x, y coordinates, width and height integers.

The first part of this will be browser agnostic, but later we will need to implement a specific layer for each browser (here we will do Firefox and Chrome).

## RFC Process

Before we invest any significant work into extending the testdriver.js API, we should check in with other stakeholders of the Web Platform Tests community on the proposed changes, by writing an [RFC](https://github.com/web-platform-tests/rfcs) ("request for comments"). This is especially useful for changes that may affect test authors or downstream users of web-platform-tests.

The [process is given in more detail in the RFC repo](https://github.com/web-platform-tests/rfcs#the-rfc-process), but to start let's send in a PR to the RFCs repo by adding a file named `rfcs/testdriver_set_window_rect.md`:

```md
# RFC N: Add window resizing to testdriver.js
(*Note: N should be replaced by the PR number*)

## Summary

Add testdriver.js support for the [Set Window Rect command](https://w3c.github.io/webdriver/#set-window-rect).

## Details
(*add details here*)

## Risks
(*add risks here*)
```

Members of the community will then have the opportunity to comment on our proposed changes, and perhaps suggest improvements to our ideas. If all goes well it will be approved and merged in.

With that said, developing a prototype implementation may help others evaluate the proposal during the RFC process, so let's move on to writing some code.

## Code!

### [resources/testdriver.js](https://github.com/web-platform-tests/wpt/blob/master/resources/testdriver.js)

This is the main entry point the tests get. Here we need to add a function to the `test_driver` object that will call the `test_driver_internal` object.

```javascript
window.test_driver = {

    // other commands...

    /**
    * Triggers browser window to be resized and relocated
    *
    * This matches the behaviour of the {@link
    * https://w3c.github.io/webdriver/#set-window-rect|WebDriver
    * Set Window Rect command}.
    *
    * @param {Integer} x - The x coordinate of the top left of the window
    * @param {Integer} y - The y coordinate of the top left of the window
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

    set_window_rect: function(x, y, width, height) {
        return Promise.reject(new Error("unimplemented"))
    }
```
We will leave this unimplemented and override it in another file. Lets do that now!

### [tools/wptrunner/wptrunner/testdriver-extra.js](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/testdriver-extra.js)

This will be the default function called when invoking the test driver commands (sometimes it is overridden by testdriver-vendor.js, but that is outside the scope of this tutorial). In most cases this is just boilerplate:

```javascript
window.test_driver_internal.set_element_rect = function(x, y, width, height) {
    return create_action("set_element_rect", {x, y, width, height});
};
```

The `create_action` helper function does the heavy lifting of setting up a postMessage to the wptrunner internals as well as returning a promise that will resolve once the call is complete.

Next, this is passed to the executor and protocol in wptrunner. Time to switch to Python!

### [tools/wptrunner/wptrunner/executors/protocol.py](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/executors/protocol.py)

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

Next we create a representation of our new action.

### [tools/wptrunner/wptrunner/executors/actions.py](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/executors/actions.py)

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

Then add your new class to the `actions = [...]` list at the end of the file.

Don't forget to write docs in ```testdriver.md```.
Now we write the browser specific implementations.

### Chrome

We will modify [executorwebdriver.py](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/executors/executorwebdriver.py) and use the WebDriver API.

There isn't too much work to do here, we just need to define a subclass of the protocol part we defined earlier.

```python
class WebDriverSetWindowRectProtocolPart(SetWindowRectProtocolPart):
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

Here we have the setup method which just redefines the webdriver object at this level. The important part is the `set_window_rect` function (and it's important it is named that since we called it that earlier). This will call the WebDriver API for [set window rect](https://w3c.github.io/webdriver/#set-window-rect).

Finally, we just need to tell the WebDriverProtocol to implement this part.

```python
class WebDriverProtocol(Protocol):
    implements = [WebDriverBaseProtocolPart,
                  WebDriverTestharnessProtocolPart,
                  WebDriverSelectorProtocolPart,
                  WebDriverClickProtocolPart,
                  WebDriverSendKeysProtocolPart,
                  {... other protocol parts}
                  WebDriverSetWindowRectProtocolPart, # add this!
                  WebDriverTestDriverProtocolPart]
```


### Firefox
We use the [set window rect](https://firefox-source-docs.mozilla.org/python/marionette_driver.html#marionette_driver.marionette.Marionette.set_window_rect) Marionette command.

We will modify [executormarionette.py](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/executors/executormarionette.py) and use the Marionette Python API.

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

Here we have the setup method which just redefines the webdriver object at this level. The important part is the `set_window_rect` function (and it's important it is named that since we called it that earlier). This will call the Marionette API for [set window rect](https://firefox-source-docs.mozilla.org/python/marionette_driver.html#marionette_driver.marionette.Marionette.set_window_rect) (`self.marionette` is a marionette instance here).

Finally, we just need to tell the MarionetteProtocol to implement this part.

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
                  MarionetteSetWindowRectProtocolPart, # add this
                  MarionetteTestDriverProtocolPart]
```

### Other Browsers

Other browsers (such as safari) may use executorselenium, or a completely new executor (such as servo). For these, you must change the executor in the same way as we did with chrome and firefox.

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

The file [testdriver-vendor.js](https://github.com/web-platform-tests/wpt/blob/master/resources/testdriver-vendor.js) is the equivalent to testdriver-extra.js above, except it is
run instead of testdriver-extra.js in browser-specific test environments. For example, in [Chromium web_tests](https://cs.chromium.org/chromium/src/third_party/blink/web_tests/).

### What if I need to return a value from my testdriver API?

You can return values from testdriver by just making your Action and Protocol classes use return statements. The data being returned will be serialized into JSON and passed
back to the test on the resolving promise. The test can then deserialize the JSON to access the return values. Here is an example of a theoretical GetWindowRect API:

```python
class GetWindowRectAction(object):
    def __call__(self, payload):
        return self.protocol.get_window_rect.get_window_rect()
```

The WebDriver command will return a [WindowRect object](https://w3c.github.io/webdriver/#dfn-window-rect), which is a dictionary with keys `x`, `y`, `width`, and `height`.
```python
class WebDriverGetWindowRectProtocolPart(GetWindowRectProtocolPart):
    def get_window_rect(self):
        return self.webdriver.get_window_rect()
```

Then a test can access the return value as follows:
```html
<script>
async_test(t => {
  test_driver.get_window_rect()
  .then((result) => {
    assert_equals(result.x, 0)
    assert_equals(result.y, 10)
    assert_equals(result.width, 800)
    assert_equals(result.height, 600)
    t.done();
  })
});
</script>
```
