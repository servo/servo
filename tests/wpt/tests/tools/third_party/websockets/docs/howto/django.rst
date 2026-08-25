Integrate with Django
=====================

If you're looking at adding real-time capabilities to a Django project with
WebSocket, you have two main options.

1. Using Django Channels_, a project adding WebSocket to Django, among other
   features. This approach is fully supported by Django. However, it requires
   switching to a new deployment architecture.

2. Deploying a separate WebSocket server next to your Django project. This
   technique is well suited when you need to add a small set of real-time
   features — maybe a notification service — to an HTTP application.

.. _Channels: https://channels.readthedocs.io/

This guide shows how to implement the second technique with websockets. It
assumes familiarity with Django.

Authenticate connections
------------------------

Since the websockets server runs outside of Django, we need to integrate it
with ``django.contrib.auth``.

We will generate authentication tokens in the Django project. Then we will
send them to the websockets server, where they will authenticate the user.

Generating a token for the current user and making it available in the browser
is up to you. You could render the token in a template or fetch it with an API
call.

Refer to the topic guide on :doc:`authentication <../topics/authentication>`
for details on this design.

Generate tokens
...............

We want secure, short-lived tokens containing the user ID. We'll rely on
`django-sesame`_, a small library designed exactly for this purpose.

.. _django-sesame: https://github.com/aaugustin/django-sesame

Add django-sesame to the dependencies of your Django project, install it, and
configure it in the settings of the project:

.. code-block:: python

    AUTHENTICATION_BACKENDS = [
        "django.contrib.auth.backends.ModelBackend",
        "sesame.backends.ModelBackend",
    ]

(If your project already uses another authentication backend than the default
``"django.contrib.auth.backends.ModelBackend"``, adjust accordingly.)

You don't need ``"sesame.middleware.AuthenticationMiddleware"``. It is for
authenticating users in the Django server, while we're authenticating them in
the websockets server.

We'd like our tokens to be valid for 30 seconds. We expect web pages to load
and to establish the WebSocket connection within this delay. Configure
django-sesame accordingly in the settings of your Django project:

.. code-block:: python

    SESAME_MAX_AGE = 30

If you expect your web site to load faster for all clients, a shorter lifespan
is possible. However, in the context of this document, it would make manual
testing more difficult.

You could also enable single-use tokens. However, this would update the last
login date of the user every time a WebSocket connection is established. This
doesn't seem like a good idea, both in terms of behavior and in terms of
performance.

Now you can generate tokens in a ``django-admin shell`` as follows:

.. code-block:: pycon

    >>> from django.contrib.auth import get_user_model
    >>> User = get_user_model()
    >>> user = User.objects.get(username="<your username>")
    >>> from sesame.utils import get_token
    >>> get_token(user)
    '<your token>'

Keep this console open: since tokens expire after 30 seconds, you'll have to
generate a new token every time you want to test connecting to the server.

Validate tokens
...............

Let's move on to the websockets server.

Add websockets to the dependencies of your Django project and install it.
Indeed, we're going to reuse the environment of the Django project, so we can
call its APIs in the websockets server.

Now here's how to implement authentication.

.. literalinclude:: ../../example/django/authentication.py

Let's unpack this code.

We're calling ``django.setup()`` before doing anything with Django because
we're using Django in a `standalone script`_. This assumes that the
``DJANGO_SETTINGS_MODULE`` environment variable is set to the Python path to
your settings module.

.. _standalone script: https://docs.djangoproject.com/en/stable/topics/settings/#calling-django-setup-is-required-for-standalone-django-usage

The connection handler reads the first message received from the client, which
is expected to contain a django-sesame token. Then it authenticates the user
with ``get_user()``, the API for `authentication outside a view`_. If
authentication fails, it closes the connection and exits.

.. _authentication outside a view: https://django-sesame.readthedocs.io/en/stable/howto.html#outside-a-view

When we call an API that makes a database query such as ``get_user()``, we
wrap the call in :func:`~asyncio.to_thread`. Indeed, the Django ORM doesn't
support asynchronous I/O. It would block the event loop if it didn't run in a
separate thread. :func:`~asyncio.to_thread` is available since Python 3.9. In
earlier versions, use :meth:`~asyncio.loop.run_in_executor` instead.

Finally, we start a server with :func:`~websockets.server.serve`.

We're ready to test!

Save this code to a file called ``authentication.py``, make sure the
``DJANGO_SETTINGS_MODULE`` environment variable is set properly, and start the
websockets server:

.. code-block:: console

    $ python authentication.py

Generate a new token — remember, they're only valid for 30 seconds — and use
it to connect to your server. Paste your token and press Enter when you get a
prompt:

.. code-block:: console

    $ python -m websockets ws://localhost:8888/
    Connected to ws://localhost:8888/
    > <your token>
    < Hello <your username>!
    Connection closed: 1000 (OK).

It works!

If you enter an expired or invalid token, authentication fails and the server
closes the connection:

.. code-block:: console

    $ python -m websockets ws://localhost:8888/
    Connected to ws://localhost:8888.
    > not a token
    Connection closed: 1011 (internal error) authentication failed.

You can also test from a browser by generating a new token and running the
following code in the JavaScript console of the browser:

.. code-block:: javascript

    websocket = new WebSocket("ws://localhost:8888/");
    websocket.onopen = (event) => websocket.send("<your token>");
    websocket.onmessage = (event) => console.log(event.data);

If you don't want to import your entire Django project into the websockets
server, you can build a separate Django project with ``django.contrib.auth``,
``django-sesame``, a suitable ``User`` model, and a subset of the settings of
the main project.

Stream events
-------------

We can connect and authenticate but our server doesn't do anything useful yet!

Let's send a message every time a user makes an action in the admin. This
message will be broadcast to all users who can access the model on which the
action was made. This may be used for showing notifications to other users.

Many use cases for WebSocket with Django follow a similar pattern.

Set up event bus
................

We need a event bus to enable communications between Django and websockets.
Both sides connect permanently to the bus. Then Django writes events and
websockets reads them. For the sake of simplicity, we'll rely on `Redis
Pub/Sub`_.

.. _Redis Pub/Sub: https://redis.io/topics/pubsub

The easiest way to add Redis to a Django project is by configuring a cache
backend with `django-redis`_. This library manages connections to Redis
efficiently, persisting them between requests, and provides an API to access
the Redis connection directly.

.. _django-redis: https://github.com/jazzband/django-redis

Install Redis, add django-redis to the dependencies of your Django project,
install it, and configure it in the settings of the project:

.. code-block:: python

    CACHES = {
        "default": {
            "BACKEND": "django_redis.cache.RedisCache",
            "LOCATION": "redis://127.0.0.1:6379/1",
        },
    }

If you already have a default cache, add a new one with a different name and
change ``get_redis_connection("default")`` in the code below to the same name.

Publish events
..............

Now let's write events to the bus.

Add the following code to a module that is imported when your Django project
starts. Typically, you would put it in a ``signals.py`` module, which you
would import in the ``AppConfig.ready()`` method of one of your apps:

.. literalinclude:: ../../example/django/signals.py

This code runs every time the admin saves a ``LogEntry`` object to keep track
of a change. It extracts interesting data, serializes it to JSON, and writes
an event to Redis.

Let's check that it works:

.. code-block:: console

    $ redis-cli
    127.0.0.1:6379> SELECT 1
    OK
    127.0.0.1:6379[1]> SUBSCRIBE events
    Reading messages... (press Ctrl-C to quit)
    1) "subscribe"
    2) "events"
    3) (integer) 1

Leave this command running, start the Django development server and make
changes in the admin: add, modify, or delete objects. You should see
corresponding events published to the ``"events"`` stream.

Broadcast events
................

Now let's turn to reading events and broadcasting them to connected clients.
We need to add several features:

* Keep track of connected clients so we can broadcast messages.
* Tell which content types the user has permission to view or to change.
* Connect to the message bus and read events.
* Broadcast these events to users who have corresponding permissions.

Here's a complete implementation.

.. literalinclude:: ../../example/django/notifications.py

Since the ``get_content_types()`` function makes a database query, it is
wrapped inside :func:`asyncio.to_thread()`. It runs once when each WebSocket
connection is open; then its result is cached for the lifetime of the
connection. Indeed, running it for each message would trigger database queries
for all connected users at the same time, which would hurt the database.

The connection handler merely registers the connection in a global variable,
associated to the list of content types for which events should be sent to
that connection, and waits until the client disconnects.

The ``process_events()`` function reads events from Redis and broadcasts them
to all connections that should receive them. We don't care much if a sending a
notification fails — this happens when a connection drops between the moment
we iterate on connections and the moment the corresponding message is sent —
so we start a task with for each message and forget about it. Also, this means
we're immediately ready to process the next event, even if it takes time to
send a message to a slow client.

Since Redis can publish a message to multiple subscribers, multiple instances
of this server can safely run in parallel.

Does it scale?
--------------

In theory, given enough servers, this design can scale to a hundred million
clients, since Redis can handle ten thousand servers and each server can
handle ten thousand clients. In practice, you would need a more scalable
message bus before reaching that scale, due to the volume of messages.
