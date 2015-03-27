The moving parts
================

html5lib consists of a number of components, which are responsible for
handling its features.


Tree builders
-------------

The parser reads HTML by tokenizing the content and building a tree that
the user can later access. There are three main types of trees that
html5lib can build:

* ``etree`` - this is the default; builds a tree based on ``xml.etree``,
  which can be found in the standard library. Whenever possible, the
  accelerated ``ElementTree`` implementation (i.e.
  ``xml.etree.cElementTree`` on Python 2.x) is used.

* ``dom`` - builds a tree based on ``xml.dom.minidom``.

* ``lxml.etree`` - uses lxml's implementation of the ``ElementTree``
  API.  The performance gains are relatively small compared to using the
  accelerated ``ElementTree`` module.

You can specify the builder by name when using the shorthand API:

.. code-block:: python

  import html5lib
  with open("mydocument.html", "rb") as f:
      lxml_etree_document = html5lib.parse(f, treebuilder="lxml")

When instantiating a parser object, you have to pass a tree builder
class in the ``tree`` keyword attribute:

.. code-block:: python

  import html5lib
  parser = html5lib.HTMLParser(tree=SomeTreeBuilder)
  document = parser.parse("<p>Hello World!")

To get a builder class by name, use the ``getTreeBuilder`` function:

.. code-block:: python

  import html5lib
  parser = html5lib.HTMLParser(tree=html5lib.getTreeBuilder("dom"))
  minidom_document = parser.parse("<p>Hello World!")

The implementation of builders can be found in `html5lib/treebuilders/
<https://github.com/html5lib/html5lib-python/tree/master/html5lib/treebuilders>`_.


Tree walkers
------------

Once a tree is ready, you can work on it either manually, or using
a tree walker, which provides a streaming view of the tree. html5lib
provides walkers for all three supported types of trees (``etree``,
``dom`` and ``lxml``).

The implementation of walkers can be found in `html5lib/treewalkers/
<https://github.com/html5lib/html5lib-python/tree/master/html5lib/treewalkers>`_.

Walkers make consuming HTML easier. html5lib uses them to provide you
with has a couple of handy tools.


HTMLSerializer
~~~~~~~~~~~~~~

The serializer lets you write HTML back as a stream of bytes.

.. code-block:: pycon

  >>> import html5lib
  >>> element = html5lib.parse('<p xml:lang="pl">Witam wszystkich')
  >>> walker = html5lib.getTreeWalker("etree")
  >>> stream = walker(element)
  >>> s = html5lib.serializer.HTMLSerializer()
  >>> output = s.serialize(stream)
  >>> for item in output:
  ...   print("%r" % item)
  '<p'
  ' '
  'xml:lang'
  '='
  'pl'
  '>'
  'Witam wszystkich'

You can customize the serializer behaviour in a variety of ways, consult
the :class:`~html5lib.serializer.htmlserializer.HTMLSerializer`
documentation.


Filters
~~~~~~~

You can alter the stream content with filters provided by html5lib:

* :class:`alphabeticalattributes.Filter
  <html5lib.filters.alphabeticalattributes.Filter>` sorts attributes on
  tags to be in alphabetical order

* :class:`inject_meta_charset.Filter
  <html5lib.filters.inject_meta_charset.Filter>` sets a user-specified
  encoding in the correct ``<meta>`` tag in the ``<head>`` section of
  the document

* :class:`lint.Filter <html5lib.filters.lint.Filter>` raises
  ``LintError`` exceptions on invalid tag and attribute names, invalid
  PCDATA, etc.

* :class:`optionaltags.Filter <html5lib.filters.optionaltags.Filter>`
  removes tags from the stream which are not necessary to produce valid
  HTML

* :class:`sanitizer.Filter <html5lib.filters.sanitizer.Filter>` removes
  unsafe markup and CSS. Elements that are known to be safe are passed
  through and the rest is converted to visible text. The default
  configuration of the sanitizer follows the `WHATWG Sanitization Rules
  <http://wiki.whatwg.org/wiki/Sanitization_rules>`_.

* :class:`whitespace.Filter <html5lib.filters.whitespace.Filter>`
  collapses all whitespace characters to single spaces unless they're in
  ``<pre/>`` or ``textarea`` tags.

To use a filter, simply wrap it around a stream:

.. code-block:: python

  >>> import html5lib
  >>> from html5lib.filters import sanitizer
  >>> dom = html5lib.parse("<p><script>alert('Boo!')", treebuilder="dom")
  >>> walker = html5lib.getTreeWalker("dom")
  >>> stream = walker(dom)
  >>> sane_stream = sanitizer.Filter(stream) clean_stream = sanitizer.Filter(stream)


Tree adapters
-------------

Used to translate one type of tree to another. More documentation
pending, sorry.


Encoding discovery
------------------

Parsed trees are always Unicode. However a large variety of input
encodings are supported. The encoding of the document is determined in
the following way:

* The encoding may be explicitly specified by passing the name of the
  encoding as the encoding parameter to the
  :meth:`~html5lib.html5parser.HTMLParser.parse` method on
  ``HTMLParser`` objects.

* If no encoding is specified, the parser will attempt to detect the
  encoding from a ``<meta>``  element in the first 512 bytes of the
  document (this is only a partial implementation of the current HTML
  5 specification).

* If no encoding can be found and the chardet library is available, an
  attempt will be made to sniff the encoding from the byte pattern.

* If all else fails, the default encoding will be used. This is usually
  `Windows-1252 <http://en.wikipedia.org/wiki/Windows-1252>`_, which is
  a common fallback used by Web browsers.


Tokenizers
----------

The part of the parser responsible for translating a raw input stream
into meaningful tokens is the tokenizer. Currently html5lib provides
two.

To set up a tokenizer, simply pass it when instantiating
a :class:`~html5lib.html5parser.HTMLParser`:

.. code-block:: python

  import html5lib
  from html5lib import sanitizer

  p = html5lib.HTMLParser(tokenizer=sanitizer.HTMLSanitizer)
  p.parse("<p>Surprise!<script>alert('Boo!');</script>")

HTMLTokenizer
~~~~~~~~~~~~~

This is the default tokenizer, the heart of html5lib. The implementation
can be found in `html5lib/tokenizer.py
<https://github.com/html5lib/html5lib-python/blob/master/html5lib/tokenizer.py>`_.

HTMLSanitizer
~~~~~~~~~~~~~

This is a tokenizer that removes unsafe markup and CSS styles from the
input. Elements that are known to be safe are passed through and the
rest is converted to visible text. The default configuration of the
sanitizer follows the `WHATWG Sanitization Rules
<http://wiki.whatwg.org/wiki/Sanitization_rules>`_.

The implementation can be found in `html5lib/sanitizer.py
<https://github.com/html5lib/html5lib-python/blob/master/html5lib/sanitizer.py>`_.
