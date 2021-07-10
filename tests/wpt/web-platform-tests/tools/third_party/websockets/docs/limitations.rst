Limitations
-----------

The client doesn't attempt to guarantee that there is no more than one
connection to a given IP address in a CONNECTING state.

The client doesn't support connecting through a proxy.

There is no way to fragment outgoing messages. A message is always sent in a
single frame.
