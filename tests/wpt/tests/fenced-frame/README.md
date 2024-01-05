# Fenced Frames

This directory contains [Web Platform
Tests](third_party/blink/web_tests/external/wpt) for the [Fenced
Frames](https://github.com/shivanigithub/fenced-frame) feature.).

In general, these tests should follow Chromium's [web tests
guidelines](docs/testing/web_tests_tips.md) and [web-platform-tests
guidelines](/docs/testing/web_platform_tests.md). This document describes
how to use the specific fenced frame testing infrastructure.

## How to run tests
Fenced frames feature needs to be enabled to run tests. A convenient way to
do this is to define the following variable for fenced frames [virtual test
suites](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/testing/web_tests.md#virtual-test-suites)
directories.
```bash
export MPTEST=virtual/fenced-frame-mparch/external/wpt/fenced-frame
```

Then run tests under the virtual test suite. This will include necessary
fenced frame flags.
```bash
third_party/blink/tools/run_web_tests.py -t Default $MPTEST/test-name.https.html
```

## How to write tests

The `<fencedframe>` element has a strict requirement that it cannot directly
communicate with or reach its embedder document. The fenced frame does have
network access however, so we can use a server as a middleman to communicate
with the outer page. There are two main test patterns we use: remote execution
(recommended) and message passing (deprecated).

### Remote execution

Remote execution uses the helper `attachFencedFrameContext()` defined in
[resources/utils.js](resources/utils.js), which requires
[/common/dispatcher/dispatcher.js](/common/dispatcher/dispatcher.js) and
[/common/utils.js](/common/utils.js). This returns a fenced frame that is
wrapped with additional functionality from RemoteContext, which allows you to
perform a remote procedure call into the frame using the function
`execute(function, [arguments]=[])`.

This interface allows us to write an entire test in only one file, with minimal
boilerplate and an obvious control flow between all the frames on the page
(including nested fenced frames, which can be achieved with nested `execute`
calls).

Let's see an example of communication between the top-level frame and the fenced
frame.

```js
promise_test(async () => {
  const important_value = "Hello";

  // First, create an empty fenced frame.
  const frame = attachFencedFrameContext();

  // Next, make a function call into the frame, passing a particular string
  // "Hello" as an argument. Make sure to `await` the call.
  const response = await frame.execute((message_from_embedder) => {

    // This code runs inside the fenced frame.
    if (message_from_embedder == "Hello") {
      // Message that we received was expected.
      return "Hello to you too");
    } else {
      // Message that we received was *not* expected, let's report an error to
      // the outer page so it fails the test.
      return "Unexpected message";
    }

  }, [important_value]);

  // Assert that the returned value was what we expected.
  // Keep in mind that in a less contrived example, you can perform this assert
  // inside the fenced frame.
  assert_equals(response, "Hello to you too",
      "The fenced frame received the message, and said hello back to us".)
}, "Fenced frame and receive and send a greeting");
```

For test examples, see
[document-referrer.https.html](document-referrer.https.html),
[hid.https.html](hid.https.html),
or [web-usb.https.html](web-usb.https.html).

Some tips to keep in mind while writing tests using remote execution:
* The functions `attachFencedFrameContext()` and `attachIFrameContext()`
  optionally take a dictionary of configs as an argument. You can use it to
  pass:
  * The API you want to use to generate the fenced frame urn. Either `'fledge'`,
    `'sharedstorage'`, or default (case-insensitive). When you use this option,
    the return value becomes a promise so you **must** await it.For example:
    ```
    await attachFencedFrameContext({generator_api: 'fledge'});
    ```
  * HTML source code to inject into the frame's DOM tree. For example:
    ```
    attachFencedFrameContext({html: '<button id="Button">Click me!</button>'});
    ```
  * Response headers. For example:
    ```
    attachFencedFrameContext({headers: [["Content-Security-Policy", "frame-src 'self'"]]});
    ```
  * Attributes to set on the frame. For example:
    ```
    attachIFrameContext({attributes: [["csp", "frame-src 'self'"]]})
    ```
  * Origin of the url to allow cross-origin test. For example:
    ```
    attachIFrameContext({origin:get_host_info().HTTPS_REMOTE_ORIGIN})
    ```
  * Number of ad components to create the frame with. Note that this only works
    with `generator_api: 'fledge'`. Protected Audience supports up to 20 ad
    components per auction.
    ```
    attachFencedFrameContext({num_components: 1});
    attachIFrameContext({num_components: 20});
    ```
    After creating the frame with ad components, the ad component frame won't
    be created until you explicitly call a special creator from within the
    frame.
    ```
    attachComponentFencedFrameContext(0, {html: "<b>Hello, world!</b>"});
    attachComponentIFrameContext(19);
    ```
    This takes in an index, and, optionally, the `html` and `attributes` fields
    as described above.
* There is also a helper `attachIFrameContext()`, which does the same thing
  but for iframes instead of fencedframes.
* There is also a helper `replaceFrameContext(frame, {options})` which will
  replace an existing frame context using the same underlying element (i.e., you
  can use it to test when happens when you navigate an existing frame).
* Make sure to `await` the result of an `execute` call, even if it doesn't
  return anything.
* In order to save a global variable, you need to explicitly assign to
  `window.variable_name`. Assigning to `variable_name` without declaring it
  will not persist across `execute` calls. This is especially important for
  tests with nested frames, if you want to keep a handle to the nested frame
  across multiple calls.
* Remember to declare the function passed to `execute` as async if it itself
  needs to invoke any async functions, including to create nested frames.

### Message passing (deprecated)

Message passing is done by using the helpers
defined in
[resources/utils.js](third_party/blink/web_tests/wpt_internal/fenced_frame/resources/utils.js)
to send a message to the server, and poll the server for a response. All
messages have a unique key associated with them so that documents that want to
receive messages can poll the server for a given message that can be identified
by a unique key.

Let's see an example of sending a message to the server that a fenced frame will
receive and respond to.

**outer-page.js:**
```js
promise_test(async () => {
  const important_message_key = token();
  const fenced_frame_ack_key = token();
  const important_value = "Hello";

  // First, let's embed a new fenced frame in our test, and pass the key we
  // just created into it as a parameter.
  const frame_url = generateURL("resources/page-inner.html",
      [important_message_key, fenced_frame_ack_key]);
  attachFencedFrame(frame_url);

  // Then, let's send the message over to the fenced frame.
  writeValueToServer(important_message_key, important_value);

  // Now that the message has been sent to the fenced frame, let's wait for its
  // ACK, so that we don't exit the test before the fenced frame gets the
  // message.
  const response_from_fenced_frame = await
      nextValueFromServer(fenced_frame_ack_key);
  assert_equals(response_from_fenced_frame, "Hello to you too",
      "The fenced frame received the message, and said hello back to us");
}, "Fenced frame and receive and send a greeting");
```

**inner-fenced-frame.js:**

```js
async function init() { // Needed in order to use top-level await.
  const [important_message_key, fenced_frame_ack_key] = parseKeylist();
  const greeting_from_embedder = await nextValueFromServer(important_message_key);

  if (greeting_from_embedder == "Hello") {
    // Message that we received was expected.
    writeValueToServer(fenced_frame_ack_key, "Hello to you too");
  } else {
    // Message that we received was *not* expected, let's report an error to the
    // outer page so it fails the test.
    writeValueToServer(fenced_frame_ack_key, "Unexpected message");
  }
}

init();
```

When you write a new web platform test, it will likely involve passing a _new_
message like the messages above, to and from the fenced frame. Keep in mind
that you may have to use a _pair_ of keys, so that when one document writes a
message associated with one unique key, it can listen for an ACK from the
receiving document, so that it doesn't write over the message again before the
receiving document actually reads it. **No two tests should ever use the same
key to communicate information to and from a fenced frame**, as this will cause
server-side race conditions.

For a good test example, see
[window-parent.html](window-parent.html).

## Underlying implementations

This directory contains <fencedframe> tests that exercise the
`blink::features::kFencedFrames` feature.

## Wrap lines at 80 columns

This is the convention for most Chromium/WPT style tests. Note that
`git cl format [--js]` does not reformat js code in .html files.
