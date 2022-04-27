# testdriver.js Automation

```eval_rst

.. contents:: Table of Contents
   :depth: 3
   :local:
   :backlinks: none
```

testdriver.js provides a means to automate tests that cannot be
written purely using web platform APIs. Outside of automation
contexts, it allows human operators to provide expected input
manually (for operations which may be described in simple terms).

It is currently supported only for [testharness.js](testharness)
tests.

## Markup ##

The `testdriver.js` and `testdriver-vendor.js` must both be included
in any document that uses testdriver (and in the top-level test
document when using testdriver from a different context):

```html
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
```

## API ##

testdriver.js exposes its API through the `test_driver` variable in
the global scope.

### User Interaction ###

```eval_rst
.. js:autofunction:: test_driver.click
.. js:autofunction:: test_driver.send_keys
.. js:autofunction:: test_driver.action_sequence
.. js:autofunction:: test_driver.bless
```

### Window State ###
```eval_rst
.. js:autofunction:: test_driver.minimize_window
.. js:autofunction:: test_driver.set_window_rect
```

### Cookies ###
```eval_rst
.. js:autofunction:: test_driver.delete_all_cookies
```

### Permissions ###
```eval_rst
.. js:autofunction:: test_driver.set_permission
```

### Authentication ###

```eval_rst
.. js:autofunction:: test_driver.add_virtual_authenticator
.. js:autofunction:: test_driver.remove_virtual_authenticator
.. js:autofunction:: test_driver.add_credential
.. js:autofunction:: test_driver.get_credentials
.. js:autofunction:: test_driver.remove_credential
.. js:autofunction:: test_driver.remove_all_credentials
.. js:autofunction:: test_driver.set_user_verified
```

### Page Lifecycle ###
```eval_rst
.. js:autofunction:: test_driver.freeze
```

### Reporting Observer ###
```eval_rst
.. js:autofunction:: test_driver.generate_test_report
```

### Storage ###
```eval_rst
.. js:autofunction:: test_driver.set_storage_access

```

### Seure Payment Confirmation ###
```eval_rst
.. js:autofunction:: test_driver.set_spc_transaction_mode
```

### Using test_driver in other browsing contexts ###

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

```eval_rst
.. js:autofunction:: test_driver.set_test_context
.. js:autofunction:: test_driver.message_test
```

For cross-origin cases, passing in the `context` doesn't work because
of limitations in the WebDriver protocol used to implement testdriver
in a cross-browser fashion. Instead one may include the testdriver
scripts directly in the relevant document, and use the
[`test_driver.set_test_context`](#test_driver.set_test_context) API to
specify the browsing context containing testharness.js. Commands are
then sent via `postMessage` to the test context. For convenience there
is also a [`test_driver.message_test`](#test_driver.message_test)
function that can be used to send arbitary messages to the test
window. For example, in an auxillary browsing context:

```js
testdriver.set_test_context(window.opener)
await testdriver.click(document.getElementsByTagName("button")[0])
testdriver.message_test("click complete")
```

The requirement to have a handle to the test window does mean it's
currently not possible to write tests where such handles can't be
obtained e.g. in the case of `rel=noopener`.

## Actions ##

### Markup ###

To use the [Actions](#Actions) API `testdriver-actions.js` must be
included in the document, in addition to `testdriver.js`:

```html
<script src="/resources/testdriver-actions.js"></script>
```

### API ###

```eval_rst
.. js:autoclass:: Actions
   :members:
```


### Using in other browsing contexts ###

For the actions API, the context can be set using the `setContext`
method on the builder:

```js
let actions = new test_driver.Actions()
    .setContext(frames[0])
    .keyDown("p")
    .keyUp("p");
await actions.send();
```

Note that if an action uses an element reference, the context will be
derived from that element, and must match any explictly set
context. Using elements in multiple contexts in a single action chain
is not supported.

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

_Note: these special-key codepoints are not necessarily what you would expect. For example, <kbd>Esc</kbd> is the invalid Unicode character `\uE00C`, not the `\u001B` Escape character from ASCII._

[activation]: https://html.spec.whatwg.org/multipage/interaction.html#activation

### set_permission

Usage: `test_driver.set_permission(descriptor, state, one_realm=false, context=null)`
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
