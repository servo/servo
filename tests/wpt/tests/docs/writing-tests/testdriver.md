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
.. js:autofunction:: test_driver.get_all_cookies
.. js:autofunction:: test_driver.get_named_cookie
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
.. js:autofunction:: test_driver.confirm_idp_login
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
