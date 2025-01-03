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

testdriver.js supports the following test types:
* [testharness.js](testharness) tests
* [reftests](reftests) and [print-reftests](print-reftests) that use the
  `class=reftest-wait` attribute on the root element to control completion
* [crashtests](crashtest) that use the `class=test-wait` attribute to control
  completion

## Markup ##

The `testdriver.js` and `testdriver-vendor.js` must both be included
in any document that uses testdriver (and in the top-level test
document when using testdriver from a different context):

```html
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>
```

## WebDriver BiDi ##

The api in `test_driver.bidi` provides access to the
[WebDriver BiDi](https://w3c.github.io/webdriver-bidi) protocol.

### Markup ###

To use WebDriver BiDi, enable the `bidi` feature in `testdriver.js` by adding the
`feature=bidi` query string parameter. Details are in [RFC 214: Add testdriver features](https://github.com/web-platform-tests/rfcs/blob/master/rfcs/testdriver-features.md).
```html
<script src="/resources/testdriver.js?feature=bidi"></script>
```

```javascript
// META: script=/resources/testdriver.js?feature=bidi
```

[Example](https://github.com/web-platform-tests/wpt/blob/aae46926b1fdccd460e1c6eaaf01ca20b941fbce/infrastructure/webdriver/bidi/subscription.html#L6).

### Context ###

A WebDriver BiDi "browsing context" is equivalent to an
[HTML navigable](https://html.spec.whatwg.org/multipage/document-sequences.html#navigable).
In WebDriver BiDi, you can interact with any browsing context, regardless of whether
it's currently active. You can target a specific browsing context using either its
unique string ID or its `WindowProxy` object.

```eval_rst
:Context: (*String|WindowProxy*)  A browsing context. Can be specified by its ID
          (a string) or using a `WindowProxy` object.
```

### Events ###

To receive WebDriver BiDi [events](https://w3c.github.io/webdriver-bidi/#events), you
need to subscribe to them. Events are only emitted for browsing contexts with an
active subscription. You can also create a global subscription to receive events from
all the contexts.

If there are
[buffered events](https://w3c.github.io/webdriver-bidi/#log-event-buffer), they will
be emitted before the `subcsribe` command's promise is resolved.

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
.. js:autofunction:: test_driver.get_all_cookies
.. js:autofunction:: test_driver.get_named_cookie
```

### Permissions ###
```eval_rst
.. js:autofunction:: test_driver.set_permission
.. js:autofunction:: test_driver.bidi.permissions.set_permission
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

### Accessibility ###
```eval_rst
.. js:autofunction:: test_driver.get_computed_label
.. js:autofunction:: test_driver.get_computed_role

```

### Secure Payment Confirmation ###
```eval_rst
.. js:autofunction:: test_driver.set_spc_transaction_mode
```

### Federated Credential Management ###
```eval_rst
.. js:autofunction:: test_driver.cancel_fedcm_dialog
.. js:autofunction:: test_driver.click_fedcm_dialog_button
.. js:autofunction:: test_driver.select_fedcm_account
.. js:autofunction:: test_driver.get_fedcm_account_list
.. js:autofunction:: test_driver.get_fedcm_dialog_title
.. js:autofunction:: test_driver.get_fedcm_dialog_type
.. js:autofunction:: test_driver.set_fedcm_delay_enabled
.. js:autofunction:: test_driver.reset_fedcm_cooldown
```

### Sensors ###
```eval_rst
.. js:autofunction:: test_driver.create_virtual_sensor
.. js:autofunction:: test_driver.update_virtual_sensor
.. js:autofunction:: test_driver.remove_virtual_sensor
.. js:autofunction:: test_driver.get_virtual_sensor_information
```

### Device Posture ###
```eval_rst
.. js:autofunction:: test_driver.set_device_posture
.. js:autofunction:: test_driver.clear_device_posture
```

### Bounce Tracking Mitigations ###

```eval_rst
.. js:autofunction:: test_driver.run_bounce_tracking_mitigations
```

### Compute Pressure ###
```eval_rst
.. js:autofunction:: test_driver.create_virtual_pressure_source
.. js:autofunction:: test_driver.update_virtual_pressure_source
.. js:autofunction:: test_driver.remove_virtual_pressure_source
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
function that can be used to send arbitrary messages to the test
window. For example, in an auxillary browsing context:

```js
test_driver.set_test_context(window.opener)
await test_driver.click(document.getElementsByTagName("button")[0])
test_driver.message_test("click complete")
```

The requirement to have a handle to the test window does mean it's
currently not possible to write tests where such handles can't be
obtained e.g. in the case of `rel=noopener`.

### Actions ###

#### Markup ####

To use the [Actions](#Actions) API `testdriver-actions.js` must be
included in the document, in addition to `testdriver.js`:

```html
<script src="/resources/testdriver-actions.js"></script>
```

#### API ####

```eval_rst
.. js:autoclass:: Actions
   :members:
```


#### Using in other browsing contexts ####

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
derived from that element, and must match any explicitly set
context. Using elements in multiple contexts in a single action chain
is not supported.

### Log

This module corresponds to the WebDriver BiDi
[Log](https://w3c.github.io/webdriver-bidi/#module-log) module and provides access to
browser logs.

#### Entry added

This provides methods to subscribe to and listen for the
[`log.entryAdded`](https://w3c.github.io/webdriver-bidi/#event-log-entryAdded) event.

**Example:**

```javascript
await test_driver.bidi.log.entry_added.subscribe();
const log_entry_promise = test_driver.bidi.log.entry_added.once();
console.log("some message");
const event = await log_entry_promise;
```

```eval_rst
.. js:autofunction:: test_driver.bidi.log.entry_added.subscribe
.. js:autofunction:: test_driver.bidi.log.entry_added.on
.. js:autofunction:: test_driver.bidi.log.entry_added.once
```

### Bluetooth ###

The module provides access to [Web Bluetooth](https://webbluetoothcg.github.io/web-bluetooth).

```eval_rst
.. js:autofunction:: test_driver.bidi.bluetooth.simulate_adapter
```
