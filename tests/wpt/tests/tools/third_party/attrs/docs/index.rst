.. module:: attr
.. module:: attrs

======================================
``attrs``: Classes Without Boilerplate
======================================

Release v\ |release| (`What's new? <changelog>`).

.. include:: ../README.rst
   :start-after: teaser-begin
   :end-before: teaser-end


Getting Started
===============

``attrs`` is a Python-only package `hosted on PyPI <https://pypi.org/project/attrs/>`_.
The recommended installation method is `pip <https://pip.pypa.io/en/stable/>`_-installing into a `virtualenv <https://hynek.me/articles/virtualenv-lives/>`_:

.. code-block:: console

   $ python -m pip install attrs

The next three steps should bring you up and running in no time:

- `overview` will show you a simple example of ``attrs`` in action and introduce you to its philosophy.
  Afterwards, you can start writing your own classes and understand what drives ``attrs``'s design.
- `examples` will give you a comprehensive tour of ``attrs``'s features.
  After reading, you will know about our advanced features and how to use them.
- If you're confused by all the ``attr.s``, ``attr.ib``, ``attrs``, ``attrib``, ``define``, ``frozen``, and ``field``, head over to `names` for a very short explanation, and optionally a quick history lesson.
- Finally `why` gives you a rundown of potential alternatives and why we think ``attrs`` is superior.
  Yes, we've heard about ``namedtuple``\ s and Data Classes!
- If at any point you get confused by some terminology, please check out our `glossary`.


If you need any help while getting started, feel free to use the ``python-attrs`` tag on `Stack Overflow <https://stackoverflow.com/questions/tagged/python-attrs>`_ and someone will surely help you out!


Day-to-Day Usage
================

- `types` help you to write *correct* and *self-documenting* code.
  ``attrs`` has first class support for them, yet keeps them optional if you’re not convinced!
- Instance initialization is one of ``attrs`` key feature areas.
  Our goal is to relieve you from writing as much code as possible.
  `init` gives you an overview what ``attrs`` has to offer and explains some related philosophies we believe in.
- Comparing and ordering objects is a common task.
  `comparison` shows you how ``attrs`` helps you with that and how you can customize it.
- If you want to put objects into sets or use them as keys in dictionaries, they have to be hashable.
  The simplest way to do that is to use frozen classes, but the topic is more complex than it seems and `hashing` will give you a primer on what to look out for.
- Once you're comfortable with the concepts, our `api` contains all information you need to use ``attrs`` to its fullest.
- ``attrs`` is built for extension from the ground up.
  `extending` will show you the affordances it offers and how to make it a building block of your own projects.


.. include:: ../README.rst
   :start-after: -getting-help-
   :end-before: -project-information-


----


Full Table of Contents
======================

.. toctree::
   :maxdepth: 2

   overview
   why
   examples
   types
   init
   comparison
   hashing
   api
   extending
   how-does-it-work
   names
   glossary


.. include:: ../README.rst
   :start-after: -project-information-

.. toctree::
   :maxdepth: 1

   license
   python-2
   changelog


Indices and tables
==================

* `genindex`
* `search`
