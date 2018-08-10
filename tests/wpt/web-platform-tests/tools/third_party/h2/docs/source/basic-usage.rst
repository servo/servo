Getting Started: Writing Your Own HTTP/2 Server
===============================================

This document explains how to get started writing fully-fledged HTTP/2
implementations using Hyper-h2 as the underlying protocol stack. It covers the
basic concepts you need to understand, and talks you through writing a very
simple HTTP/2 server.

This document assumes you're moderately familiar with writing Python, and have
*some* understanding of how computer networks work. If you don't, you'll find
it a lot easier if you get some understanding of those concepts first and then
return to this documentation.


.. _h2-connection-basic:

Connections
-----------

Hyper-h2's core object is the
:class:`H2Connection <h2.connection.H2Connection>` object. This object is an
abstract representation of the state of a single HTTP/2 connection, and holds
all the important protocol state. When using Hyper-h2, this object will be the
first thing you create and the object that does most of the heavy lifting.

The interface to this object is relatively simple. For sending data, you
call the object with methods indicating what actions you want to perform: for
example, you may want to send headers (you'd use the
:meth:`send_headers <h2.connection.H2Connection.send_headers>` method), or
send data (you'd use the
:meth:`send_data <h2.connection.H2Connection.send_data>` method). After you've
decided what actions you want to perform, you get some bytes out of the object
that represent the HTTP/2-encoded representation of your actions, and send them
out over the network however you see fit.

When you receive data from the network, you pass that data in to the
``H2Connection`` object, which returns a list of *events*.
These events, covered in more detail later in :ref:`h2-events-basic`, define
the set of actions the remote peer has performed on the connection, as
represented by the HTTP/2-encoded data you just passed to the object.

Thus, you end up with a simple loop (which you may recognise as a more-specific
form of an `event loop`_):

    1. First, you perform some actions.
    2. You send the data created by performing those actions to the network.
    3. You read data from the network.
    4. You decode those into events.
    5. The events cause you to trigger some actions: go back to step 1.

Of course, HTTP/2 is more complex than that, but in the very simplest case you
can write a fairly effective HTTP/2 tool using just that kind of loop. Later in
this document, we'll do just that.

Some important subtleties of ``H2Connection`` objects are covered in
:doc:`advanced-usage`: see :ref:`h2-connection-advanced` for more information.
However, one subtlety should be covered, and that is this: Hyper-h2's
``H2Connection`` object doesn't do I/O. Let's talk briefly about why.

I/O
~~~

Any useful HTTP/2 tool eventually needs to do I/O. This is because it's not
very useful to be able to speak to other computers using a protocol like HTTP/2
unless you actually *speak* to them sometimes.

However, doing I/O is not a trivial thing: there are lots of different ways to
do it, and once you choose a way to do it your code usually won't work well
with the approaches you *didn't* choose.

While there are lots of different ways to do I/O, when it comes down to it
all HTTP/2 implementations transform bytes received into events, and events
into bytes to send. So there's no reason to have lots of different versions of
this core protocol code: one for Twisted, one for gevent, one for threading,
and one for synchronous code.

This is why we said at the top that Hyper-h2 is a *HTTP/2 Protocol Stack*, not
a *fully-fledged implementation*. Hyper-h2 knows how to transform bytes into
events and back, but that's it. The I/O and smarts might be different, but
the core HTTP/2 logic is the same: that's what Hyper-h2 provides.

Not doing I/O makes Hyper-h2 general, and also relatively simple. It has an
easy-to-understand performance envelope, it's easy to test (and as a result
easy to get correct behaviour out of), and it behaves in a reproducible way.
These are all great traits to have in a library that is doing something quite
complex.

This document will talk you through how to build a relatively simple HTTP/2
implementation using Hyper-h2, to give you an understanding of where it fits in
your software.


.. _h2-events-basic:

Events
------

When writing a HTTP/2 implementation it's important to know what the remote
peer is doing: if you didn't care, writing networked programs would be a lot
easier!

Hyper-h2 encodes the actions of the remote peer in the form of *events*. When
you receive data from the remote peer and pass it into your ``H2Connection``
object (see :ref:`h2-connection-basic`), the ``H2Connection`` returns a list
of objects, each one representing a single event that has occurred. Each
event refers to a single action the remote peer has taken.

Some events are fairly high-level, referring to things that are more general
than HTTP/2: for example, the
:class:`RequestReceived <h2.events.RequestReceived>` event is a general HTTP
concept, not just a HTTP/2 one. Other events are extremely HTTP/2-specific:
for example, :class:`PushedStreamReceived <h2.events.PushedStreamReceived>`
refers to Server Push, a very HTTP/2-specific concept.

The reason these events exist is that Hyper-h2 is intended to be very general.
This means that, in many cases, Hyper-h2 does not know exactly what to do in
response to an event. Your code will need to handle these events, and make
decisions about what to do. That's the major role of any HTTP/2 implementation
built on top of Hyper-h2.

A full list of events is available in :ref:`h2-events-api`. For the purposes
of this example, we will handle only a small set of events.


Writing Your Server
-------------------

Armed with the knowledge you just obtained, we're going to write a very simple
HTTP/2 web server. The goal of this server is to write a server that can handle
a HTTP GET, and that returns the headers sent by the client, encoded in JSON.
Basically, something a lot like `httpbin.org/get`_. Nothing fancy, but this is
a good way to get a handle on how you should interact with Hyper-h2.

For the sake of simplicity, we're going to write this using the Python standard
library, in Python 3. In reality, you'll probably want to use an asynchronous
framework of some kind: see the `examples directory`_ in the repository for
some examples of how you'd do that.

Before we start, create a new file called ``h2server.py``: we'll use that as
our workspace. Additionally, you should install Hyper-h2: follow the
instructions in :doc:`installation`.

Step 1: Sockets
~~~~~~~~~~~~~~~

To begin with, we need to make sure we can listen for incoming data and send it
back. To do that, we need to use the `standard library's socket module`_. For
now we're going to skip doing TLS: if you want to reach your server from your
web browser, though, you'll need to add TLS and some other function. Consider
looking at our examples in our `examples directory`_ instead.

Let's begin. First, open up ``h2server.py``. We need to import the socket
module and start listening for connections.

This is not a socket tutorial, so we're not going to dive too deeply into how
this works. If you want more detail about sockets, there are lots of good
tutorials on the web that you should investigate.

When you want to listen for incoming connections, the you need to *bind* an
address first. So let's do that. Try setting up your file to look like this:

.. code-block:: python

    import socket

    sock = socket.socket()
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('0.0.0.0', 8080))
    sock.listen(5)

    while True:
        print(sock.accept())

In a shell window, execute this program (``python h2server.py``). Then, open
another shell and run ``curl http://localhost:8080/``. In the first shell, you
should see something like this:

.. code-block:: console

    $ python h2server.py
    (<socket.socket fd=4, family=AddressFamily.AF_INET, type=SocketKind.SOCK_STREAM, proto=0, laddr=('127.0.0.1', 8080), raddr=('127.0.0.1', 58800)>, ('127.0.0.1', 58800))

Run that ``curl`` command a few more times. You should see a few more similar
lines appear. Note that the ``curl`` command itself will exit with an error.
That's fine: it happens because we didn't send any data.

Now go ahead and stop the server running by hitting Ctrl+C in the first shell.
You should see a ``KeyboardInterrupt`` error take the process down.

What's the program above doing? Well, first it creates a
:func:`socket <python:socket.socket>` object. This socket is then *bound* to
a specific address: ``('0.0.0.0', 8080)``. This is a special address: it means
that this socket should be listening for any traffic to TCP port 8080. Don't
worry about the call to ``setsockopt``: it just makes sure you can run this
program repeatedly.

We then loop forever calling the :meth:`accept <python:socket.socket.accept>`
method on the socket. The accept method blocks until someone attempts to
connect to our TCP port: when they do, it returns a tuple: the first element is
a new socket object, the second element is a tuple of the address the new
connection is from. You can see this in the output from our ``h2server.py``
script.

At this point, we have a script that can accept inbound connections. This is a
good start! Let's start getting HTTP/2 involved.


Step 2: Add a H2Connection
~~~~~~~~~~~~~~~~~~~~~~~~~~

Now that we can listen for socket information, we want to prepare our HTTP/2
connection object and start handing it data. For now, let's just see what
happens as we feed it data.

To make HTTP/2 connections, we need a tool that knows how to speak HTTP/2.
Most versions of curl in the wild don't, so let's install a Python tool. In
your Python environment, run ``pip install hyper``. This will install a Python
command-line HTTP/2 tool called ``hyper``. To confirm that it works, try
running this command and verifying that the output looks similar to the one
shown below:

.. code-block:: console

    $ hyper GET http://http2bin.org/get
    {'args': {},
     'headers': {'Connection': 'keep-alive',
                 'Host': 'http2bin.org',
                 'Via': '2 http2bin.org'},
     'origin': '10.0.0.2',
     'url': 'http://http2bin.org/get'}

Assuming it works, you're now ready to start sending HTTP/2 data.

Back in our ``h2server.py`` script, we're going to want to start handling data.
Let's add a function that takes a socket returned from ``accept``, and reads
data from it. Let's call that function ``handle``. That function should create
a :class:`H2Connection <h2.connection.H2Connection>` object and then loop on
the socket, reading data and passing it to the connection.

To read data from a socket we need to call ``recv``. The ``recv`` function
takes a number as its argument, which is the *maximum* amount of data to be
returned from a single call (note that ``recv`` will return as soon as any data
is available, even if that amount is vastly less than the number you passed to
it). For the purposes of writing this kind of software the specific value is
not enormously useful, but should not be overly large. For that reason, when
you're unsure, a number like 4096 or 65535 is a good bet. We'll use 65535 for
this example.

The function should look something like this:

.. code-block:: python

    import h2.connection

    def handle(sock):
        conn = h2.connection.H2Connection(client_side=False)

        while True:
            data = sock.recv(65535)
            print(conn.receive_data(data))

Let's update our main loop so that it passes data on to our new data handling
function. Your ``h2server.py`` should end up looking a like this:

.. code-block:: python

    import socket

    import h2.connection

    def handle(sock):
        conn = h2.connection.H2Connection(client_side=False)

        while True:
            data = sock.recv(65535)
            if not data:
                break

            print(conn.receive_data(data))


    sock = socket.socket()
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('0.0.0.0', 8080))
    sock.listen(5)

    while True:
        handle(sock.accept()[0])

Running that in one shell, in your other shell you can run
``hyper --h2 GET http://localhost:8080/``. That shell should hang, and you
should then see the following output from your ``h2server.py`` shell:

.. code-block:: console

    $ python h2server.py
    [<h2.events.RemoteSettingsChanged object at 0x10c4ee390>]

You'll then need to kill ``hyper`` and ``h2server.py`` with Ctrl+C. Feel free
to do this a few times, to see how things behave.

So, what did we see here? When the connection was opened, we used the
:meth:`recv <python:socket.socket.recv>` method to read some data from the
socket, in a loop. We then passed that data to the connection object, which
returned us a single event object:
:class:`RemoteSettingsChanged <h2.events.RemoteSettingsChanged>`.

But what we didn't see was anything else. So it seems like all ``hyper`` did
was change its settings, but nothing else. If you look at the other ``hyper``
window, you'll notice that it hangs for a while and then eventually fails with
a socket timeout. It was waiting for something: what?

Well, it turns out that at the start of a connection, both sides need to send
a bit of data, called "the HTTP/2 preamble". We don't need to get into too much
detail here, but basically both sides need to send a single block of HTTP/2
data that tells the other side what their settings are. ``hyper`` did that,
but we didn't.

Let's do that next.


Step 3: Sending the Preamble
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Hyper-h2 makes doing connection setup really easy. All you need to do is call
the
:meth:`initiate_connection <h2.connection.H2Connection.initiate_connection>`
method, and then send the corresponding data. Let's update our ``handle``
function to do just that:

.. code-block:: python

    def handle(sock):
        conn = h2.connection.H2Connection(client_side=False)
        conn.initiate_connection()
        sock.sendall(conn.data_to_send())

        while True:
            data = sock.recv(65535)
            print(conn.receive_data(data))


The big change here is the call to ``initiate_connection``, but there's another
new method in there:
:meth:`data_to_send <h2.connection.H2Connection.data_to_send>`.

When you make function calls on your ``H2Connection`` object, these will often
want to cause HTTP/2 data to be written out to the network. But Hyper-h2
doesn't do any I/O, so it can't do that itself. Instead, it writes it to an
internal buffer. You can retrieve data from this buffer using the
``data_to_send`` method. There are some subtleties about that method, but we
don't need to worry about them right now: all we need to do is make sure we're
sending whatever data is outstanding.

Your ``h2server.py`` script should now look like this:

.. code-block:: python

    import socket

    import h2.connection

    def handle(sock):
        conn = h2.connection.H2Connection(client_side=False)
        conn.initiate_connection()
        sock.sendall(conn.data_to_send())

        while True:
            data = sock.recv(65535)
            if not data:
                break

            print(conn.receive_data(data))


    sock = socket.socket()
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('0.0.0.0', 8080))
    sock.listen(5)

    while True:
        handle(sock.accept()[0])


With this change made, rerun your ``h2server.py`` script and hit it with the
same ``hyper`` command: ``hyper --h2 GET http://localhost:8080/``. The
``hyper`` command still hangs, but this time we get a bit more output from our
``h2server.py`` script:

.. code-block:: console

    $ python h2server.py
    [<h2.events.RemoteSettingsChanged object at 0x10292d390>]
    [<h2.events.SettingsAcknowledged object at 0x102b3a160>]
    [<h2.events.RequestReceived object at 0x102b3a3c8>, <h2.events.StreamEnded object at 0x102b3a400>]

So, what's happening?

The first thing to note is that we're going around our loop more than once now.
First, we receive some data that triggers a
:class:`RemoteSettingsChanged <h2.events.RemoteSettingsChanged>` event.
Then, we get some more data that triggers a
:class:`SettingsAcknowledged <h2.events.SettingsAcknowledged>` event.
Finally, even more data that triggers *two* events:
:class:`RequestReceived <h2.events.RequestReceived>` and
:class:`StreamEnded <h2.events.StreamEnded>`.

So, what's happening is that ``hyper`` is telling us about its settings,
acknowledging ours, and then sending us a request. Then it ends a *stream*,
which is a HTTP/2 communications channel that holds a request and response
pair.

A stream isn't done until it's either *reset* or both sides *close* it:
in this sense it's bi-directional. So what the ``StreamEnded`` event tells us
is that ``hyper`` is closing its half of the stream: it won't send us any more
data on that stream. That means the request is done.

So why is ``hyper`` hanging? Well, we haven't sent a response yet: let's do
that.


Step 4: Handling Events
~~~~~~~~~~~~~~~~~~~~~~~

What we want to do is send a response when we receive a request. Happily, we
get an event when we receive a request, so we can use that to be our signal.

Let's define a new function that sends a response. For now, this response can
just be a little bit of data that prints "it works!".

The function should take the ``H2Connection`` object, and the event that
signaled the request. Let's define it.

.. code-block:: python

    def send_response(conn, event):
        stream_id = event.stream_id
        conn.send_headers(
            stream_id=stream_id,
            headers=[
                (':status', '200'),
                ('server', 'basic-h2-server/1.0')
            ],
        )
        conn.send_data(
            stream_id=stream_id,
            data=b'it works!',
            end_stream=True
        )

So while this is only a short function, there's quite a lot going on here we
need to unpack. Firstly, what's a stream ID? Earlier we discussed streams
briefly, to say that they're a bi-directional communications channel that holds
a request and response pair. Part of what makes HTTP/2 great is that there can
be lots of streams going on at once, sending and receiving different requests
and responses. To identify each stream, we use a *stream ID*. These are unique
across the lifetime of a connection, and they go in ascending order.

Most ``H2Connection`` functions take a stream ID: they require you to actively
tell the connection which one to use. In this case, as a simple server, we will
never need to choose a stream ID ourselves: the client will always choose one
for us. That means we'll always be able to get the one we need off the events
that fire.

Next, we send some *headers*. In HTTP/2, a response is made up of some set of
headers, and optionally some data. The headers have to come first: if you're a
client then you'll be sending *request* headers, but in our case these headers
are our *response* headers.

Mostly these aren't very exciting, but you'll notice once special header in
there: ``:status``. This is a HTTP/2-specific header, and it's used to hold the
HTTP status code that used to go at the top of a HTTP response. Here, we're
saying the response is ``200 OK``, which is successful.

To send headers in Hyper-h2, you use the
:meth:`send_headers <h2.connection.H2Connection.send_headers>` function.

Next, we want to send the body data. To do that, we use the
:meth:`send_data <h2.connection.H2Connection.send_data>` function. This also
takes a stream ID. Note that the data is binary: Hyper-h2 does not work with
unicode strings, so you *must* pass bytestrings to the ``H2Connection``. The
one exception is headers: Hyper-h2 will automatically encode those into UTF-8.

The last thing to note is that on our call to ``send_data``, we set
``end_stream`` to ``True``. This tells Hyper-h2 (and the remote peer) that
we're done with sending data: the response is over. Because we know that
``hyper`` will have ended its side of the stream, when we end ours the stream
will be totally done with.

We're nearly ready to go with this: we just need to plumb this function in.
Let's amend our ``handle`` function again:

.. code-block:: python

    import h2.events

    def handle(sock):
        conn = h2.connection.H2Connection(client_side=False)
        conn.initiate_connection()
        sock.sendall(conn.data_to_send())

        while True:
            data = sock.recv(65535)
            if not data:
                break

            events = conn.receive_data(data)
            for event in events:
                if isinstance(event, h2.events.RequestReceived):
                    send_response(conn, event)

            data_to_send = conn.data_to_send()
            if data_to_send:
                sock.sendall(data_to_send)

The changes here are all at the end. Now, when we receive some events, we
look through them for the ``RequestReceived`` event. If we find it, we make
sure we send a response.

Then, at the bottom of the loop we check whether we have any data to send, and
if we do, we send it. Then, we repeat again.

With these changes, your ``h2server.py`` file should look like this:

.. code-block:: python

    import socket

    import h2.connection
    import h2.events

    def send_response(conn, event):
        stream_id = event.stream_id
        conn.send_headers(
            stream_id=stream_id,
            headers=[
                (':status', '200'),
                ('server', 'basic-h2-server/1.0')
            ],
        )
        conn.send_data(
            stream_id=stream_id,
            data=b'it works!',
            end_stream=True
        )

    def handle(sock):
        conn = h2.connection.H2Connection(client_side=False)
        conn.initiate_connection()
        sock.sendall(conn.data_to_send())

        while True:
            data = sock.recv(65535)
            if not data:
                break

            events = conn.receive_data(data)
            for event in events:
                if isinstance(event, h2.events.RequestReceived):
                    send_response(conn, event)

            data_to_send = conn.data_to_send()
            if data_to_send:
                sock.sendall(data_to_send)


    sock = socket.socket()
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('0.0.0.0', 8080))
    sock.listen(5)

    while True:
        handle(sock.accept()[0])

Alright. Let's run this, and then run our ``hyper`` command again.

This time, nothing is printed from our server, and the ``hyper`` side prints
``it works!``. Success! Try running it a few more times, and we can see that
not only does it work the first time, it works the other times too!

We can speak HTTP/2! Let's add the final step: returning the JSON-encoded
request headers.

Step 5: Returning Headers
~~~~~~~~~~~~~~~~~~~~~~~~~

If we want to return the request headers in JSON, the first thing we have to do
is find them. Handily, if you check the documentation for
:class:`RequestReceived <h2.events.RequestReceived>` you'll find that this
event carries, in addition to the stream ID, the request headers.

This means we can make a really simple change to our ``send_response``
function to take those headers and encode them as a JSON object. Let's do that:

.. code-block:: python

    import json

    def send_response(conn, event):
        stream_id = event.stream_id
        response_data = json.dumps(dict(event.headers)).encode('utf-8')

        conn.send_headers(
            stream_id=stream_id,
            headers=[
                (':status', '200'),
                ('server', 'basic-h2-server/1.0'),
                ('content-length', str(len(response_data))),
                ('content-type', 'application/json'),
            ],
        )
        conn.send_data(
            stream_id=stream_id,
            data=response_data,
            end_stream=True
        )

This is a really simple change, but it's all we need to do: a few extra headers
and the JSON dump, but that's it.

Section 6: Bringing It All Together
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

This should be all we need!

Let's take all the work we just did and throw that into our ``h2server.py``
file, which should now look like this:

.. code-block:: python

    import json
    import socket

    import h2.connection
    import h2.events

    def send_response(conn, event):
        stream_id = event.stream_id
        response_data = json.dumps(dict(event.headers)).encode('utf-8')

        conn.send_headers(
            stream_id=stream_id,
            headers=[
                (':status', '200'),
                ('server', 'basic-h2-server/1.0'),
                ('content-length', str(len(response_data))),
                ('content-type', 'application/json'),
            ],
        )
        conn.send_data(
            stream_id=stream_id,
            data=response_data,
            end_stream=True
        )

    def handle(sock):
        conn = h2.connection.H2Connection(client_side=False)
        conn.initiate_connection()
        sock.sendall(conn.data_to_send())

        while True:
            data = sock.recv(65535)
            if not data:
                break

            events = conn.receive_data(data)
            for event in events:
                if isinstance(event, h2.events.RequestReceived):
                    send_response(conn, event)

            data_to_send = conn.data_to_send()
            if data_to_send:
                sock.sendall(data_to_send)


    sock = socket.socket()
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('0.0.0.0', 8080))
    sock.listen(5)

    while True:
        handle(sock.accept()[0])

Now, execute ``h2server.py`` and then point ``hyper`` at it again. You should
see something like the following output from ``hyper``:

.. code-block:: console

    $ hyper --h2 GET http://localhost:8080/
    {":scheme": "http", ":authority": "localhost", ":method": "GET", ":path": "/"}

Here you can see the HTTP/2 request 'special headers' that ``hyper`` sends.
These are similar to the ``:status`` header we have to send on our response:
they encode important parts of the HTTP request in a clearly-defined way. If
you were writing a client stack using Hyper-h2, you'd need to make sure you
were sending those headers.

Congratulations!
~~~~~~~~~~~~~~~~

Congratulations! You've written your first HTTP/2 server! If you want to extend
it, there are a few directions you could investigate:

- We didn't handle a few events that we saw were being raised: you could add
  some methods to handle those appropriately.
- Right now our server is single threaded, so it can only handle one client at
  a time. Consider rewriting this server to use threads, or writing this
  server again using your favourite asynchronous programming framework.

  If you plan to use threads, you should know that a ``H2Connection`` object is
  deliberately not thread-safe. As a possible design pattern, consider creating
  threads and passing the sockets returned by ``accept`` to those threads, and
  then letting those threads create their own ``H2Connection`` objects.
- Take a look at some of our long-form code examples in :doc:`examples`.
- Alternatively, try playing around with our examples in our repository's
  `examples directory`_. These examples are a bit more fully-featured, and can
  be reached from your web browser. Try adjusting what they do, or adding new
  features to them!
- You may want to make this server reachable from your web browser. To do that,
  you'll need to add proper TLS support to your server. This can be tricky, and
  in many cases requires `PyOpenSSL`_ in addition to the other libraries you
  have installed. Check the `Eventlet example`_ to see what PyOpenSSL code is
  required to TLS-ify your server.



.. _event loop: https://en.wikipedia.org/wiki/Event_loop
.. _httpbin.org/get: https://httpbin.org/get
.. _examples directory: https://github.com/python-hyper/hyper-h2/tree/master/examples
.. _standard library's socket module: https://docs.python.org/3.5/library/socket.html
.. _Application Layer Protocol Negotiation: https://en.wikipedia.org/wiki/Application-Layer_Protocol_Negotiation
.. _get your certificate here: https://raw.githubusercontent.com/python-hyper/hyper-h2/master/examples/twisted/server.crt
.. _get your private key here: https://raw.githubusercontent.com/python-hyper/hyper-h2/master/examples/twisted/server.key
.. _PyOpenSSL: http://pyopenssl.readthedocs.org/
.. _Eventlet example: https://github.com/python-hyper/hyper-h2/blob/master/examples/eventlet/eventlet-server.py
