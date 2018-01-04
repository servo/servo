======================================
``attrs``: Classes Without Boilerplate
======================================

Release v\ |release| (:doc:`What's new? <changelog>`).

.. include:: ../README.rst
   :start-after: teaser-begin
   :end-before: -spiel-end-


Getting Started
===============

``attrs`` is a Python-only package `hosted on PyPI <https://pypi.org/project/attrs/>`_.
The recommended installation method is `pip <https://pip.pypa.io/en/stable/>`_-installing into a `virtualenv <https://hynek.me/articles/virtualenv-lives/>`_:

.. code-block:: console

   $ pip install attrs

The next three steps should bring you up and running in no time:

- :doc:`overview` will show you a simple example of ``attrs`` in action and introduce you to its philosophy.
  Afterwards, you can start writing your own classes, understand what drives ``attrs``'s design, and know what ``@attr.s`` and ``attr.ib()`` stand for.
- :doc:`examples` will give you a comprehensive tour of ``attrs``'s features.
  After reading, you will know about our advanced features and how to use them.
- Finally :doc:`why` gives you a rundown of potential alternatives and why we think ``attrs`` is superior.
  Yes, we've heard about ``namedtuple``\ s!


If you need any help while getting started, feel free to use the ``python-attrs`` tag on `StackOverflow <https://stackoverflow.com/questions/tagged/python-attrs>`_ and someone will surely help you out!


Day-to-Day Usage
================

- Once you're comfortable with the concepts, our :doc:`api` contains all information you need to use ``attrs`` to its fullest.
- ``attrs`` is built for extension from the ground up.
  :doc:`extending` will show you the affordances it offers and how to make it a building block of your own projects.


.. include:: ../README.rst
   :start-after: -testimonials-
   :end-before: -end-

.. include:: ../README.rst
   :start-after: -project-information-

.. toctree::
   :maxdepth: 1

   license
   backward-compatibility
   contributing
   changelog


----


Full Table of Contents
======================

.. toctree::
   :maxdepth: 2

   overview
   why
   examples
   api
   extending
   how-does-it-work


Indices and tables
==================

* :ref:`genindex`
* :ref:`search`
