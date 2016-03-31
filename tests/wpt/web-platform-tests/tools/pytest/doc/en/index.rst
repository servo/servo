
.. _features:

pytest: helps you write better programs
=============================================

**a mature full-featured Python testing tool**

 - runs on Posix/Windows, Python 2.6-3.5, PyPy and (possibly still) Jython-2.5.1
 - free and open source software, distributed under the terms of the :ref:`MIT license <license>`
 - **well tested** with more than a thousand tests against itself
 - **strict backward compatibility policy** for safe pytest upgrades
 - :ref:`comprehensive online <toc>` and `PDF documentation <pytest.pdf>`_
 - many :ref:`third party plugins <extplugins>` and :ref:`builtin helpers <pytest helpers>`,
 - used in :ref:`many small and large projects and organisations <projects>`
 - comes with many :ref:`tested examples <examples>`

**provides easy no-boilerplate testing**

 - makes it :ref:`easy to get started <getstarted>`,
   has many :ref:`usage options <usage>`
 - :ref:`assert with the assert statement`
 - helpful :ref:`traceback and failing assertion reporting <tbreportdemo>`
 - :ref:`print debugging <printdebugging>` and :ref:`the
   capturing of standard output during test execution <captures>`

**scales from simple unit to complex functional testing**

 - :ref:`modular parametrizeable fixtures <fixture>` (new in 2.3,
   continuously improved)
 - :ref:`parametrized test functions <parametrized test functions>`
 - :ref:`mark`
 - :ref:`skipping` (improved in 2.4)
 - :ref:`distribute tests to multiple CPUs <xdistcpu>` through :ref:`xdist plugin <xdist>`
 - :ref:`continuously re-run failing tests <looponfailing>`
 - :doc:`cache`
 - flexible :ref:`Python test discovery`

**integrates with other testing methods and tools**:

 - multi-paradigm: pytest can run ``nose``, ``unittest`` and
   ``doctest`` style test suites, including running testcases made for
   Django and trial
 - supports :ref:`good integration practices <goodpractices>`
 - supports extended :ref:`xUnit style setup <xunitsetup>`
 - supports domain-specific :ref:`non-python tests`
 - supports generating `test coverage reports
   <https://pypi.python.org/pypi/pytest-cov>`_
 - supports :pep:`8` compliant coding styles in tests

**extensive plugin and customization system**:

 - all collection, reporting, running aspects are delegated to hook functions
 - customizations can be per-directory, per-project or per PyPI released plugin
 - it is easy to add command line options or customize existing behaviour
 - :ref:`easy to write your own plugins <writing-plugins>`


.. _`easy`: http://bruynooghe.blogspot.com/2009/12/skipping-slow-test-by-default-in-pytest.html


