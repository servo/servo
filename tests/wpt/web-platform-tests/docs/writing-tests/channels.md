# Message Channels

```eval_rst

.. contents:: Table of Contents
   :depth: 3
   :local:
   :backlinks: none
```

Message channels provide a mechanism to communicate across globals,
including in cases where there is no client-side mechanism to
establish a communication channel (i.e. when the globals are in
different browsing context groups).

## Markup ##

```html
<script src="/resources/channels.sub.js"></script>
```

Channels can be used in any global and are not specifically linked to
`testharness.js`.

### High Level API ###

The high level API provides a way to message another global, and to
execute functions in that global and return the result.

Globals wanting to recieve messages using the high level API have to
be loaded with a `uuid` query parameter in their URL, with a value
that's a UUID. This will be used to identify the channel dedicated to
messages sent to that context.

The context must call either `global_channel` or
`start_global_channel` when it's ready to receive messages. This
returns a `RecvChannel` that can be used to add message handlers.

```eval_rst

.. js:autofunction:: global_channel
   :short-name:
.. js:autofunction:: start_global_channel
   :short-name:
.. js:autoclass:: RemoteGlobalCommandRecvChannel
   :members:
```

Contexts wanting to communicate with the remote context do so using a
`RemoteGlobal` object.

```eval_rst

.. js:autoclass:: RemoteGlobal
   :members:
```

#### Remote Objects ####

By default objects (e.g. script arguments) sent to the remote global
are cloned. In order to support referencing objects owned by the
originating global, there is a `RemoteObject` type which can pass a
reference to an object across a channel.

```eval_rst

.. js:autoclass:: RemoteObject
   :members:
```

#### Example ####

test.html

```html
<!doctype html>
<title>call example</title>
<script src="/resources/testharness.js">
<script src="/resources/testharnessreport.js">
<script src="/resources/channel.js">

<script>
promise_test(async t => {
  let remote = new RemoteGlobal();
  window.open(`child.html?uuid=${remote.uuid}`, "_blank", "noopener");
  let result = await remote.call(id => {
    return document.getElementById(id).textContent;
  }, "test");
  assert_equals("result", "PASS");
});
</script>
```

child.html

```html
<script src="/resources/channel.js">

<p id="nottest">FAIL</p>
<p id="test">PASS</p>
<script>
start_global_channel();
</script>
```

### Low Level API ###

The high level API is implemented in terms of a channel
abstraction. Each channel is identified by a UUID, and corresponds to
a message queue hosted by the server. Channels are multiple producer,
single consumer, so there's only only entity responsible for
processing messages sent to the channel. This is designed to
discourage race conditions where multiple consumers try to process the
same message.

On the client side, the read side of a channel is represented by a
`RecvChannel` object, and the send side by `SendChannel`. An initial
channel pair is created with the `channel()` function.

```eval_rst

.. js:autofunction:: channel
   :members:
.. js:autoclass:: Channel
   :members:
.. js:autoclass:: SendChannel
   :members:
.. js:autoclass:: RecvChannel
   :members:
```

### Navigation and bfcache

For specific use cases around bfcache, it's important to be able to
ensure that no network connections (including websockets) remain open
at the time of navigation, otherwise the page will be excluded from
bfcache. This is handled as follows:

* A `disconnectReader` method on `SendChannel`. This causes a
  server-initiated disconnect of the corresponding `RecvChannel`
  websocket. The idea is to allow a page to send a command that will
  initiate a navigation, then without knowing when the navigation is
  done, send further commands that will be processed when the
  `RecvChannel` reconnects. If the commands are sent before the
  navigation, but not processed, they can be buffered by the remote
  and then lost during navigation.

* A `close_all_channel_sockets()` function. This just closes all the open
  websockets associated with channels in the global in which it's
  called. Any channel then has to be reconnected to be used
  again. Calling `closeAllChannelSockets()` right before navigating
  will leave you in a state with no open websocket connections (unless
  something happens to reopen one before the navigation starts).

```eval_rst

.. js:autofunction:: close_all_channel_sockets
   :members:
```
