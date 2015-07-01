.. _mach_logging:

=======
Logging
=======

Mach configures a built-in logging facility so commands can easily log
data.

What sets the logging facility apart from most loggers you've seen is
that it encourages structured logging. Instead of conventional logging
where simple strings are logged, the internal logging mechanism logs all
events with the following pieces of information:

* A string *action*
* A dict of log message fields
* A formatting string

Essentially, instead of assembling a human-readable string at
logging-time, you create an object holding all the pieces of data that
will constitute your logged event. For each unique type of logged event,
you assign an *action* name.

Depending on how logging is configured, your logged event could get
written a couple of different ways.

JSON Logging
============

Where machines are the intended target of the logging data, a JSON
logger is configured. The JSON logger assembles an array consisting of
the following elements:

* Decimal wall clock time in seconds since UNIX epoch
* String *action* of message
* Object with structured message data

The JSON-serialized array is written to a configured file handle.
Consumers of this logging stream can just perform a readline() then feed
that into a JSON deserializer to reconstruct the original logged
message. They can key off the *action* element to determine how to
process individual events. There is no need to invent a parser.
Convenient, isn't it?

Logging for Humans
==================

Where humans are the intended consumer of a log message, the structured
log message are converted to more human-friendly form. This is done by
utilizing the *formatting* string provided at log time. The logger
simply calls the *format* method of the formatting string, passing the
dict containing the message's fields.

When *mach* is used in a terminal that supports it, the logging facility
also supports terminal features such as colorization. This is done
automatically in the logging layer - there is no need to control this at
logging time.

In addition, messages intended for humans typically prepends every line
with the time passed since the application started.

Logging HOWTO
=============

Structured logging piggybacks on top of Python's built-in logging
infrastructure provided by the *logging* package. We accomplish this by
taking advantage of *logging.Logger.log()*'s *extra* argument. To this
argument, we pass a dict with the fields *action* and *params*. These
are the string *action* and dict of message fields, respectively. The
formatting string is passed as the *msg* argument, like normal.

If you were logging to a logger directly, you would do something like:

.. code-block:: python

   logger.log(logging.INFO, 'My name is {name}',
       extra={'action': 'my_name', 'params': {'name': 'Gregory'}})

The JSON logging would produce something like::

   [1339985554.306338, "my_name", {"name": "Gregory"}]

Human logging would produce something like::

   0.52 My name is Gregory

Since there is a lot of complexity using logger.log directly, it is
recommended to go through a wrapping layer that hides part of the
complexity for you. The easiest way to do this is by utilizing the
LoggingMixin:

.. code-block:: python

   import logging
   from mach.mixin.logging import LoggingMixin

   class MyClass(LoggingMixin):
       def foo(self):
            self.log(logging.INFO, 'foo_start', {'bar': True},
                'Foo performed. Bar: {bar}')
