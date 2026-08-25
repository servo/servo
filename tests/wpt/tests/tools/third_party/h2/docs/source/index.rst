.. hyper-h2 documentation master file, created by
   sphinx-quickstart on Thu Sep 17 10:06:02 2015.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

h2: A pure-Python HTTP/2 protocol stack
=======================================

h2 is a HTTP/2 protocol stack, written entirely in Python. The goal of
h2 is to be a common HTTP/2 stack for the Python ecosystem,
usable in all programs regardless of concurrency model or environment.

To achieve this, h2 is entirely self-contained: it does no I/O of any
kind, leaving that up to a wrapper library to control. This ensures that it can
seamlessly work in all kinds of environments, from single-threaded code to
Twisted.

Its goal is to be 100% compatible with RFC 7540, implementing a complete HTTP/2
protocol stack build on a set of finite state machines. Its secondary goals are
to be fast, clear, and efficient.

For usage examples, see :doc:`basic-usage` or consult the examples in the
repository.

Contents
--------

.. toctree::
   :maxdepth: 2

   installation
   basic-usage
   negotiating-http2
   examples
   advanced-usage
   low-level
   api
   testimonials
   release-process
   release-notes
