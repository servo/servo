---
layout: page
title: testdriver.js Automation
order: 8.5
---

testdriver.js provides a means to automate tests that cannot be
written purely using web platform APIs. Outside of automation
contexts, it allows human operators to provide expected input
manually (for operations which may be described in simple terms).

It is currently supported only for [testharness.js][testharness]
tests.

## API

testdriver.js exposes its API through the `test_driver` variable in
the global scope.

NB: presently, testdriver.js only works in the top-level test browsing
context (and not therefore in any frame or window opened from it).

### action_sequence
Usage: `test_driver.action_sequence(actions)`
 * `actions`: an array of `Action` objects

This function causes a sequence of actions to be sent to the browser. It is based on the [WebDriver API](https://w3c.github.io/webdriver/#actions).
The action can be a keyboard action, a pointer action or a pause. It returns a `Promise` that
resolves after the actions have been sent or rejects if an error was thrown.

Test authors are encouraged to use the builder API to generate the sequence of actions. The builder
API can be accessed via the `new test_driver.Actions()` object.

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

Calling into `send()` is going to dispatch the action sequence (via `test_driver.action_sequence`) and also returns a `Promise` which should be handled however is appropriate in the test. The other functions in the `Actions()` object are going to modify the state of the object by adding a new action in the sequence and return the same object. So the functions can be easily chained as shown in the example above. Here is a list of helper functions in the `Actions` class:

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

### bless

Usage: `test_driver.bless(intent, action)`
 * `intent`: a string describing the motivation for this invocation
 * `action`: an optional function

This function simulates [activation][activation], allowing tests to
perform privileged operations that require user interaction. For
example, sandboxed iframes with the
`allow-top-navigation-by-user-activation` may only navigate their
parent's browsing context under these circumstances. The `intent`
string is presented to human operators when the test is not run in
automation.

This method returns a promise which is resolved with the result of
invoking the `action` function. If no such function is provided, the
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
 * `element`: a DOM Element object

This function causes a click to occur on the target element (an
`Element` object), potentially scrolling the document to make it
possible to click it. It returns a `Promise` that resolves after the
click has occured or rejects if the element cannot be clicked (for
example, it is obscured by an element on top of it).

Note that if the element to be clicked does not have a unique ID, the
document must not have any DOM mutations made between the function
being called and the promise settling.

### send_keys

Usage: `test_driver.send_keys(element, keys)`
 * `element`: a DOM Element object
 * `keys`: string to send to the element

This function causes the string `keys` to be send to the target
element (an `Element` object), potentially scrolling the document to
make it possible to send keys. It returns a `Promise` that resolves
after the keys have been send or rejects if the keys cannot be sent
to the element.

Note that if the element that's keys need to be send to does not have
a unique ID, the document must not have any DOM mutations made
between the function being called and the promise settling.

To send special keys, one must send the respective key's codepoint. Since this uses the WebDriver protocol, you can find a [list for code points to special keys in the spec](https://w3c.github.io/webdriver/webdriver-spec.html#keyboard-actions).
For example, to send the tab key you would send "\uE004".

[activation]: https://html.spec.whatwg.org/multipage/interaction.html#activation
[testharness]: {{ site.baseurl }}{% link _writing-tests/testharness.md %}
