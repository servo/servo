Router
======

The router is used to match incoming requests to request handler
functions. Typically users don't interact with the router directly,
but instead send a list of routes to register when starting the
server. However it is also possible to add routes after starting the
server by calling the `register` method on the server's `router`
property.

Routes are represented by a three item tuple::

   (methods, path_match, handler)

`methods` is either a string or a list of strings indicating the HTTP
methods to match. In cases where all methods should match there is a
special sentinel value `any_method` provided as a property of the
`router` module that can be used.

`path_match` is an expression that will be evaluated against the
request path to decide if the handler should match. These expressions
follow a custom syntax intended to make matching URLs straightforward
and, in particular, to be easier to use than raw regexp for URL
matching. There are three possible components of a match expression:

* Literals. These match any character. The special characters \*, \{
  and \} must be escaped by prefixing them with a \\.

* Match groups. These match any character other than / and save the
  result as a named group. They are delimited by curly braces; for
  example::

    {abc}

  would create a match group with the name `abc`.

* Stars. These are denoted with a `*` and match any character
  including /. There can be at most one star
  per pattern and it must follow any match groups.

Path expressions always match the entire request path and a leading /
in the expression is implied even if it is not explicitly
provided. This means that `/foo` and `foo` are equivalent.

For example, the following pattern matches all requests for resources with the
extension `.py`::

  *.py

The following expression matches anything directly under `/resources`
with a `.html` extension, and places the "filename" in the `name`
group::

  /resources/{name}.html

The groups, including anything that matches a `*` are available in the
request object through the `route_match` property. This is a
dictionary mapping the group names, and any match for `*` to the
matching part of the route. For example, given a route::

  /api/{sub_api}/*

and the request path `/api/test/html/test.html`, `route_match` would
be::

  {"sub_api": "html", "*": "html/test.html"}

`handler` is a function taking a request and a response object that is
responsible for constructing the response to the HTTP request. See
:doc:`handlers` for more details on handler functions.

.. _router.Interface:

:mod:`Interface <wptserve.router>`
----------------------------------

.. automodule:: wptserve.router
   :members:
