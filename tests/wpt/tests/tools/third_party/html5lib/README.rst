html5lib
========

.. image:: https://travis-ci.org/html5lib/html5lib-python.svg?branch=master
    :target: https://travis-ci.org/html5lib/html5lib-python


html5lib is a pure-python library for parsing HTML. It is designed to
conform to the WHATWG HTML specification, as is implemented by all major
web browsers.


Usage
-----

Simple usage follows this pattern:

.. code-block:: python

  import html5lib
  with open("mydocument.html", "rb") as f:
      document = html5lib.parse(f)

or:

.. code-block:: python

  import html5lib
  document = html5lib.parse("<p>Hello World!")

By default, the ``document`` will be an ``xml.etree`` element instance.
Whenever possible, html5lib chooses the accelerated ``ElementTree``
implementation (i.e. ``xml.etree.cElementTree`` on Python 2.x).

Two other tree types are supported: ``xml.dom.minidom`` and
``lxml.etree``. To use an alternative format, specify the name of
a treebuilder:

.. code-block:: python

  import html5lib
  with open("mydocument.html", "rb") as f:
      lxml_etree_document = html5lib.parse(f, treebuilder="lxml")

When using with ``urllib2`` (Python 2), the charset from HTTP should be
pass into html5lib as follows:

.. code-block:: python

  from contextlib import closing
  from urllib2 import urlopen
  import html5lib

  with closing(urlopen("http://example.com/")) as f:
      document = html5lib.parse(f, transport_encoding=f.info().getparam("charset"))

When using with ``urllib.request`` (Python 3), the charset from HTTP
should be pass into html5lib as follows:

.. code-block:: python

  from urllib.request import urlopen
  import html5lib

  with urlopen("http://example.com/") as f:
      document = html5lib.parse(f, transport_encoding=f.info().get_content_charset())

To have more control over the parser, create a parser object explicitly.
For instance, to make the parser raise exceptions on parse errors, use:

.. code-block:: python

  import html5lib
  with open("mydocument.html", "rb") as f:
      parser = html5lib.HTMLParser(strict=True)
      document = parser.parse(f)

When you're instantiating parser objects explicitly, pass a treebuilder
class as the ``tree`` keyword argument to use an alternative document
format:

.. code-block:: python

  import html5lib
  parser = html5lib.HTMLParser(tree=html5lib.getTreeBuilder("dom"))
  minidom_document = parser.parse("<p>Hello World!")

More documentation is available at https://html5lib.readthedocs.io/.


Installation
------------

html5lib works on CPython 2.7+, CPython 3.5+ and PyPy. To install:

.. code-block:: bash

    $ pip install html5lib

The goal is to support a (non-strict) superset of the versions that `pip
supports
<https://pip.pypa.io/en/stable/installing/#python-and-os-compatibility>`_.

Optional Dependencies
---------------------

The following third-party libraries may be used for additional
functionality:

- ``lxml`` is supported as a tree format (for both building and
  walking) under CPython (but *not* PyPy where it is known to cause
  segfaults);

- ``genshi`` has a treewalker (but not builder); and

- ``chardet`` can be used as a fallback when character encoding cannot
  be determined.


Bugs
----

Please report any bugs on the `issue tracker
<https://github.com/html5lib/html5lib-python/issues>`_.


Tests
-----

Unit tests require the ``pytest`` and ``mock`` libraries and can be
run using the ``py.test`` command in the root directory.

Test data are contained in a separate `html5lib-tests
<https://github.com/html5lib/html5lib-tests>`_ repository and included
as a submodule, thus for git checkouts they must be initialized::

  $ git submodule init
  $ git submodule update

If you have all compatible Python implementations available on your
system, you can run tests on all of them using the ``tox`` utility,
which can be found on PyPI.


Questions?
----------

There's a mailing list available for support on Google Groups,
`html5lib-discuss <http://groups.google.com/group/html5lib-discuss>`_,
though you may get a quicker response asking on IRC in `#whatwg on
irc.freenode.net <http://wiki.whatwg.org/wiki/IRC>`_.
