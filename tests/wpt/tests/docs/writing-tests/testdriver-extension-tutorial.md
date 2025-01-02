# Testdriver extension tutorial
Adding new commands to testdriver.js

## Assumptions
We assume the following in this writeup:
 - You know what web-platform-tests is and you have a working checkout and can run tests
 - You know what [WebDriver Classic](https://w3c.github.io/webdriver/) and
  [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) protocols are
 - Familiarity with JavaScript and Python

## Introduction!

Let's implement window resizing. We can do this via the [Set Window Rect](https://w3c.github.io/webdriver/#set-window-rect) command in WebDriver.

The process of extending `testdriver.js` is similar for [WebDriver Classic](https://w3c.github.io/webdriver/) and [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) commands. This tutorial highlights the differences inline.

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

Note that for extensions to testdriver.js that directly reflect a [WebDriver Classic](https://w3c.github.io/webdriver/) command, [WebDriver BiDi](https://w3c.github.io/webdriver-bidi) command, or event, the [RFC](https://github.com/web-platform-tests/rfcs) process isn't required.

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

#### WebDriver BiDi

For commands using WebDriver BiDi, add the methods to `test_driver.bidi`. Parameters are passed as a single object `params`. For example [`test_driver.bidi.permissions.set_permission`](https://github.com/web-platform-tests/wpt/blob/5ec8ba6d68f27d49a056cbf940e3bc9a8324c538/resources/testdriver.js#L183).

Before calling `test_driver_internal` method, assert the `bidi` testdriver feature is enabled.
```javascript
set_permission: function (params) {
    assertBidiIsEnabled();
    return window.test_driver_internal.bidi.permissions.set_permission(
        params);
}
```

### [tools/wptrunner/wptrunner/testdriver-extra.js](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/testdriver-extra.js)

This will be the default function called when invoking the test driver commands (sometimes it is overridden by testdriver-vendor.js, but that is outside the scope of this tutorial). In most cases this is just boilerplate:

```javascript
window.test_driver_internal.set_element_rect = function(x, y, width, height) {
    return create_action("set_element_rect", {x, y, width, height});
};
```

The `create_action` helper function does the heavy lifting of setting up a postMessage to the wptrunner internals as well as returning a promise that will resolve once the call is complete.

The action's `name` is important and will be used later when defining the corresponding Python action representation. Keep this name in mind for the next steps.

#### WebDriver BiDi

For actions related to WebDriver BiDi, the `name` should follow the format `bidi.{MODULE_NAME}.{COMMAND}`, for example, [`bidi.session.subscribe`](https://github.com/web-platform-tests/wpt/blob/107c5fc03139b3247920a0f40983bd1fe4d1fac2/tools/wptrunner/wptrunner/testdriver-extra.js#L202).

### Protocol part

Next, this is passed to the executor and protocol in wptrunner. Time to switch to Python!

To add this command, you'll need to create a corresponding protocol part class in [tools/wptrunner/wptrunner/executors/protocol.py](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/executors/protocol.py).

```python
class WindowRectProtocolPart(ProtocolPart):
    """Protocol part for resizing and changing location of window"""
    __metaclass__ = ABCMeta

    name = "window_rect"

    @abstractmethod
    def set_window_rect(self, x, y, width, height):
        """Change the window rect

        :param x: The x coordinate of the top left of the window.
        :param y: The y coordinate of the top left of the window.
        :param width: The width of the window.
        :param height: The height of the window."""
        pass
```

The protocol part's `name` is important. It will be used when we define the action that uses this protocol part. Make a note of this name for the next steps.

#### WebDriver BiDi

When working with WebDriver BiDi, organize protocol parts by WebDriver BiDi modules. Name these parts using the prefix `Bidi{ModuleName}ProtocolPart` (e.g., [`BidiScriptProtocolPart`](https://github.com/web-platform-tests/wpt/blob/107c5fc03139b3247920a0f40983bd1fe4d1fac2/tools/wptrunner/wptrunner/executors/protocol.py#L388)) and use the prefix `bidi_` for their corresponding methods (e.g., [`bidi_script`](https://github.com/web-platform-tests/wpt/blob/107c5fc03139b3247920a0f40983bd1fe4d1fac2/tools/wptrunner/wptrunner/executors/protocol.py#L392)).

### Action representation

Next create an action representation in [tools/wptrunner/wptrunner/executors/actions.py](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/executors/actions.py). This defines how the command's parameters are processed and how the command is executed using the protocol part we defined earlier.

```python
class SetWindowRectAction:
    name = "set_window_rect"

    def __init__(self, logger, protocol):
        self.logger = logger
        self.protocol = protocol

    def __call__(self, payload):
        x, y, width, height = payload["x"], payload["y"], payload["width"], payload["height"]
        self.logger.debug("Setting window rect to be: x=%s, y=%s, width=%s, height=%s"
                          .format(x, y, width, height))
        self.protocol.window_rect.set_window_rect(x, y, width, height)
```

The `name` property should match the `name` used in [tools/wptrunner/wptrunner/testdriver-extra.js](#tools-wptrunner-wptrunner-testdriver-extra-js). This name acts as the key that connects the testdriver function in JavaScript with its corresponding Python action.

You can access the `WindowRectProtocolPart` using its name `window_rect` we defined earlier:
```python
self.protocol.window_rect.set_window_rect(x, y, width, height)
```

Then add your newly created class to the [`actions = [...]`](https://github.com/web-platform-tests/wpt/blob/107c5fc03139b3247920a0f40983bd1fe4d1fac2/tools/wptrunner/wptrunner/executors/actions.py#L514) list at the end of the file.

Don't forget to write docs in ```testdriver.md```.

#### WebDriver BiDi

For WebDriver BiDi actions, add the new action representation class to [`tools/wptrunner/wptrunner/executors/asyncactions.py`](https://github.com/web-platform-tests/wpt/blob/107c5fc03139b3247920a0f40983bd1fe4d1fac2/tools/wptrunner/wptrunner/executors/asyncactions.py) and include it in the [`async_actions = [...]`](https://github.com/web-platform-tests/wpt/blob/107c5fc03139b3247920a0f40983bd1fe4d1fac2/tools/wptrunner/wptrunner/executors/asyncactions.py#L35C1-L35C14) list.

Note that the BiDi actions' `__call__` can be `async`.

### Browser specific implementations

Now we write the browser specific implementations.

#### Chrome

We will modify [executorwebdriver.py](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/executors/executorwebdriver.py) and use the WebDriver API.

There isn't too much work to do here, we just need to define a subclass of the protocol part we defined earlier.

##### Implement protocol part

```python
class WebDriverWindowRectProtocolPart(WindowRectProtocolPart):
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
                       WindowRectProtocolPart, # add this!
                       TestDriverProtocolPart)
```

Here we have the setup method which just redefines the webdriver object at this level. The important part is the `set_window_rect` function (and it's important it is named that since we called it that earlier). This will call the WebDriver API for [set window rect](https://w3c.github.io/webdriver/#set-window-rect).

###### WebDriver BiDi

You can access the [BidiSession](https://github.com/web-platform-tests/wpt/blob/b142861632efcdec53a86ef9ca26c8b79474493b/tools/webdriver/webdriver/bidi/client.py#L13) through the webdriver object using `self.webdriver.bidi_session`, similar to how it's done for the [`WebDriverBidiScriptProtocolPart.call_function`](https://github.com/web-platform-tests/wpt/blob/b142861632efcdec53a86ef9ca26c8b79474493b/tools/wptrunner/wptrunner/executors/executorwebdriver.py#L214C22-L214C49).

##### (WebDriver Classic only) Extend `WebDriverProtocol` implementation

Finally, we just need to tell the WebDriverProtocol to implement this part.

```python
class WebDriverProtocol(Protocol):
    implements = [WebDriverBaseProtocolPart,
                  WebDriverTestharnessProtocolPart,
                  WebDriverSelectorProtocolPart,
                  WebDriverClickProtocolPart,
                  WebDriverSendKeysProtocolPart,
                  {... other protocol parts}
                  WebDriverWindowRectProtocolPart, # add this!
                  WebDriverTestDriverProtocolPart]
```

##### (WebDriver BiDi only) Extend `WebDriverBidiProtocol` implementation

To make this new WebDriver BiDi command available, add the newly added protocol part implementation (e.g., `WebDriverBidiScriptProtocolPart`) to the [`implements`](https://github.com/web-platform-tests/wpt/blob/b142861632efcdec53a86ef9ca26c8b79474493b/tools/wptrunner/wptrunner/executors/executorwebdriver.py#L681) list of the [`WebDriverBidiProtocol`](https://github.com/web-platform-tests/wpt/blob/b142861632efcdec53a86ef9ca26c8b79474493b/tools/wptrunner/wptrunner/executors/executorwebdriver.py#L679C7-L679C28) class.

```python
class WebDriverBidiProtocol(WebDriverProtocol):
    enable_bidi = True
    implements = [{... other bidi protocol parts},
                  WebDriverBidiScriptProtocolPart, # add this
                  *(part for part in WebDriverProtocol.implements)
                  ]
```

#### Firefox
<!-- TODO: Document adding WebDriver BiDi protocol parts. -->
We use the [set window rect](https://firefox-source-docs.mozilla.org/python/marionette_driver.html#marionette_driver.marionette.Marionette.set_window_rect) Marionette command.

We will modify [executormarionette.py](https://github.com/web-platform-tests/wpt/blob/master/tools/wptrunner/wptrunner/executors/executormarionette.py) and use the Marionette Python API.

We have little actual work to do here! We just need to define a subclass of the protocol part we defined earlier.

```python
class MarionetteWindowRectProtocolPart(WindowRectProtocolPart):
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
                       WindowRectProtocolPart, # add this!
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
                  MarionetteWindowRectProtocolPart, # add this
                  MarionetteTestDriverProtocolPart]
```

#### Other Browsers

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

#### WebDriver BiDi

For an example of how to write an infra test for a WebDriver BiDi command, you can refer to the existing test for [`test_driver.bidi.log.entry_added`](https://github.com/web-platform-tests/wpt/blob/b142861632efcdec53a86ef9ca26c8b79474493b/infrastructure/webdriver/bidi/subscription.html).
<!-- TODO: replace example with directly mapped method like `test_driver.bidi.permissions.set_permission`, once it is implemented -->

### What about testdriver-vendor.js?

The file [testdriver-vendor.js](https://github.com/web-platform-tests/wpt/blob/master/resources/testdriver-vendor.js) is the equivalent to testdriver-extra.js above, except it is
run instead of testdriver-extra.js in browser-specific test environments. For example, in [Chromium web_tests](https://cs.chromium.org/chromium/src/third_party/blink/web_tests/).

### What if I need to return a value from my testdriver API?

You can return values from testdriver by just making your Action and Protocol classes use return statements. The data being returned will be serialized into JSON and passed back to the test on the resolving promise. The test can then deserialize the JSON to access the return values. Here is an example of a theoretical GetWindowRect API:

```python
class GetWindowRectAction(object):
    name = "get_window_rect"
    def __call__(self, payload):
        return self.protocol.window_rect.get_window_rect()
```

Extend pProtocol part:
```python
class WindowRectProtocolPart(ProtocolPart):

    ...

    @abstractmethod
    def get_window_rect(self):
        pass
```

And implement it:
```python
class WebDriverWindowRectProtocolPart(WindowRectProtocolPart):
    ...
    def get_window_rect(self):
        # The communication channel between testharness and backend is blocked until the end of the action.
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

### What if I need to run an async code in the action?

For actions that involve asynchronous operations, `async_actions` provide a non-blocking approach. Similar to the [`BidiSessionSubscribeAction`](https://github.com/web-platform-tests/wpt/blob/b142861632efcdec53a86ef9ca26c8b79474493b/tools/wptrunner/wptrunner/executors/asyncactions.py#L18) example:
<!-- TODO: replace example with directly mapped method like `test_driver.bidi.permissions.set_permission`, once it is implemented -->
```python
    async def __call__(self, payload):
        # The communication channel between testharness and backend is not blocked.
        ...
        return await self.protocol.bidi_events.subscribe(events, contexts)
```
