# testdriver.js Automation

testdriver.js provides a means to automate tests that cannot be
written purely using web platform APIs. Outside of automation
contexts, it allows human operators to provide expected input
manually (for operations which may be described in simple terms).

It is currently supported only for [testharness.js](testharness)
tests.

## API

testdriver.js exposes its API through the `test_driver` variable in
the global scope.

### Actions
Usage:
```
let actions = new test_driver.Actions()
   .action1()
   .action2();
actions.send()
```

Test authors are encouraged to use the builder API to generate the
sequence of actions. The builder API can be accessed via the `new
test_driver.Actions()` object, and actions are defined in
[testdriver-actions.js](https://github.com/web-platform-tests/wpt/blob/master/resources/testdriver-actions.js)

The `actions.send()` function causes the sequence of actions to be
sent to the browser. It is based on the [WebDriver
API](https://w3c.github.io/webdriver/#actions).  The action can be a
keyboard action, a pointer action or a pause. It returns a promise
that resolves after the actions have been sent, or rejects if an error
was thrown.


Example:

```js
let text_box = document.getElementById("text");

let actions = new test_driver.Actions()
    .pointerMove(0, 0, {origin: text_box})
    .pointerDown()
    .pointerUp()
    .addTick()
    .keyDown("p")
    .keyUp("p");

actions.send();
```

Calling into `send()` is going to dispatch the action sequence (via
`test_driver.action_sequence`) and also returns a promise which should
be handled however is appropriate in the test. The other functions in
the `Actions()` object are going to modify the state of the object by
adding a new action in the sequence and returning the same object. So
the functions can be easily chained, as shown in the example
above. Here is a list of helper functions in the `Actions` class:

```
pointerDown: Create a pointerDown event for the current default pointer source
pointerUp: Create a pointerUp event for the current default pointer source
pointerMove: Create a move event for the current default pointer source
keyDown: Create a keyDown event for the current default key source
keyUp: Create a keyUp event for the current default key source
pause: Add a pause to the current tick
addTick: Insert a new actions tick
setPointer: Set the current default pointer source (By detault the pointerType is mouse)
addPointer: Add a new pointer input source with the given name
setKeyboard: Set the current default key source
addKeyboard: Add a new key input source with the given name
```

This works with elements in other frames/windows as long as they are
same-origin with the test, and the test does not depend on the
window.name property remaining unset on the target window.

### bless

Usage: `test_driver.bless(intent, action)`
 * _intent_: a string describing the motivation for this invocation
 * _action_: an optional function

This function simulates [activation][activation], allowing tests to
perform privileged operations that require user interaction. For
example, sandboxed iframes with
`allow-top-navigation-by-user-activation` may only navigate their
parent's browsing context under these circumstances. The _intent_
string is presented to human operators when the test is not run in
automation.

This method returns a promise which is resolved with the result of
invoking the _action_ function. If no such function is provided, the
promise is resolved with the value `undefined`.

Example:

```js
var mediaElement = document.createElement('video');

test_driver.bless('initiate media playback', function () {
  mediaElement.play();
});
```

### click

Usage: `test_driver.click(element)`
 * _element_: a DOM Element object

This function causes a click to occur on the target element (an
`Element` object), potentially scrolling the document to make it
possible to click it. It returns a promise that resolves after the
click has occurred or rejects if the element cannot be clicked (for
example, it is obscured by an element on top of it).

This works with elements in other frames/windows as long as they are
same-origin with the test, and the test does not depend on the
window.name property remaining unset on the target window.

Note that if the element to be clicked does not have a unique ID, the
document must not have any DOM mutations made between the function
being called and the promise settling.

## delete_all_cookies

Usage: `test_driver.delete_all_cookies(context=null)`
 * _context_: an optional WindowProxy for the browsing context in which to
              perform the call.

This function deletes all cookies for the current browsing context.

### send_keys

Usage: `test_driver.send_keys(element, keys)`
 * _element_: a DOM Element object
 * _keys_: string to send to the element

This function causes the string _keys_ to be sent to the target
element (an `Element` object), potentially scrolling the document to
make it possible to send keys. It returns a promise that resolves
after the keys have been sent, or rejects if the keys cannot be sent
to the element.

This works with elements in other frames/windows as long as they are
same-origin with the test, and the test does not depend on the
window.name property remaining unset on the target window.

Note that if the element that the keys need to be sent to does not have
a unique ID, the document must not have any DOM mutations made
between the function being called and the promise settling.

To send special keys, one must send the respective key's codepoint. Since this uses the WebDriver protocol, you can find a [list for code points to special keys in the spec](https://w3c.github.io/webdriver/#keyboard-actions).
For example, to send the tab key you would send "\uE004".

[activation]: https://html.spec.whatwg.org/multipage/interaction.html#activation

### set_permission

Usage: `test_driver.set_permission(descriptor, state, one_realm, context=null)`
 * _descriptor_: a
   [PermissionDescriptor](https://w3c.github.io/permissions/#dictdef-permissiondescriptor)
   or derived object
 * _state_: a
   [PermissionState](https://w3c.github.io/permissions/#enumdef-permissionstate)
   value
 * _one_realm_: a boolean that indicates whether the permission settings
   apply to only one realm
 * context: a WindowProxy for the browsing context in which to perform the call

This function causes permission requests and queries for the status of a
certain permission type (e.g. "push", or "background-fetch") to always
return _state_. It returns a promise that resolves after the permission has
been set to be overridden with _state_.

Example:

``` js
await test_driver.set_permission({ name: "background-fetch" }, "denied");
await test_driver.set_permission({ name: "push", userVisibleOnly: true }, "granted", true);
```

## Using testdriver in Other Browsing Contexts

Testdriver can be used in browsing contexts (i.e. windows or frames)
from which it's possible to get a reference to the top-level test
context. There are two basic approaches depending on whether the
context in which testdriver is used is same-origin with the test
context, or different origin.

For same-origin contexts, the context can be passed directly into the
testdriver API calls. For functions that take an element argument this
is done implicitly using the owner document of the element. For
functions that don't take an element, this is done via an explicit
context argument, which takes a WindowProxy object.

Example:
```
let win = window.open("example.html")
win.onload = () => {
  await test_driver.set_permission({ name: "background-fetch" }, "denied", win);
}
```

For the actions API, the context can be set using the `setContext`
method on the builder:

```
let actions = new test_driver.Actions()
    .setContext(frames[0])
    .keyDown("p")
    .keyUp("p");
actions.send();
```

Note that if an action uses an element reference, the context will be
derived from that element, and must match any explictly set
context. Using elements in multiple contexts in a single action chain
is not supported.


For cross-origin cases, passing in the context id doesn't work because
of limitations in the WebDriver protocol used to implement testdriver
in a cross-browser fashion. Instead one may include the testdriver
scripts directly in the relevant document, and use the
`set_test_context` API to specify the browsing context containing
testharness.js. Commands are then sent via postMessage to the test
context. For convenience there is also a `message_test` function that
can be used to send arbitary messages to the test window. For example,
in an auxillary browsing context:


```
testdriver.set_test_context(window.opener)
await testdriver.click(document.getElementsByTagName("button")[0])
testdriver.message_test("click complete")
```

The requirement to have a handle to the test window does mean it's
currently not possible to write tests where such handles can't be
obtained e.g. in the case of `rel=noopener`.
