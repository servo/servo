:orphan:

..
    .. sidebar:: Next Open Trainings

       - `Professional Testing with Python <https://www.python-academy.com/courses/specialtopics/python_course_testing.html>`_, via `Python Academy <https://www.python-academy.com/>`_, February 1st to 3rd, 2022, Leipzig (Germany) and remote.

       Also see `previous talks and blogposts <talks.html>`_.

.. _features:

pytest: helps you write better programs
=======================================

.. module:: pytest

The ``pytest`` framework makes it easy to write small, readable tests, and can
scale to support complex functional testing for applications and libraries.


**Pythons**: ``pytest`` requires: Python 3.6, 3.7, 3.8, 3.9, or PyPy3.

**PyPI package name**: :pypi:`pytest`

**Documentation as PDF**: `download latest <https://media.readthedocs.org/pdf/pytest/latest/pytest.pdf>`_


A quick example
---------------

.. code-block:: python

    # content of test_sample.py
    def inc(x):
        return x + 1


    def test_answer():
        assert inc(3) == 5


To execute it:

.. code-block:: pytest

    $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-7.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 1 item

    test_sample.py F                                                     [100%]

    ================================= FAILURES =================================
    _______________________________ test_answer ________________________________

        def test_answer():
    >       assert inc(3) == 5
    E       assert 4 == 5
    E        +  where 4 = inc(3)

    test_sample.py:6: AssertionError
    ========================= short test summary info ==========================
    FAILED test_sample.py::test_answer - assert 4 == 5
    ============================ 1 failed in 0.12s =============================

Due to ``pytest``'s detailed assertion introspection, only plain ``assert`` statements are used.
See :ref:`Get started <getstarted>` for a basic introduction to using pytest.


Features
--------

- Detailed info on failing :ref:`assert statements <assert>` (no need to remember ``self.assert*`` names)

- :ref:`Auto-discovery <test discovery>` of test modules and functions

- :ref:`Modular fixtures <fixture>` for managing small or parametrized long-lived test resources

- Can run :ref:`unittest <unittest>` (including trial) and :ref:`nose <noseintegration>` test suites out of the box

- Python 3.6+ and PyPy 3

- Rich plugin architecture, with over 800+ :ref:`external plugins <plugin-list>` and thriving community


Documentation
-------------

* :ref:`Get started <get-started>` - install pytest and grasp its basics just twenty minutes
* :ref:`How-to guides <how-to>` - step-by-step guides, covering a vast range of use-cases and needs
* :ref:`Reference guides <reference>` - includes the complete pytest API reference, lists of plugins and more
* :ref:`Explanation <explanation>` - background, discussion of key topics, answers to higher-level questions


Bugs/Requests
-------------

Please use the `GitHub issue tracker <https://github.com/pytest-dev/pytest/issues>`_ to submit bugs or request features.


Changelog
---------

Consult the :ref:`Changelog <changelog>` page for fixes and enhancements of each version.

Support pytest
--------------

`Open Collective`_ is an online funding platform for open and transparent communities.
It provides tools to raise money and share your finances in full transparency.

It is the platform of choice for individuals and companies that want to make one-time or
monthly donations directly to the project.

See more details in the `pytest collective`_.

.. _Open Collective: https://opencollective.com
.. _pytest collective: https://opencollective.com/pytest


pytest for enterprise
---------------------

Available as part of the Tidelift Subscription.

The maintainers of pytest and thousands of other packages are working with Tidelift to deliver commercial support and
maintenance for the open source dependencies you use to build your applications.
Save time, reduce risk, and improve code health, while paying the maintainers of the exact dependencies you use.

`Learn more. <https://tidelift.com/subscription/pkg/pypi-pytest?utm_source=pypi-pytest&utm_medium=referral&utm_campaign=enterprise&utm_term=repo>`_

Security
~~~~~~~~

pytest has never been associated with a security vulnerability, but in any case, to report a
security vulnerability please use the `Tidelift security contact <https://tidelift.com/security>`_.
Tidelift will coordinate the fix and disclosure.


License
-------

Copyright Holger Krekel and others, 2004.

Distributed under the terms of the `MIT`_ license, pytest is free and open source software.

.. _`MIT`: https://github.com/pytest-dev/pytest/blob/main/LICENSE
