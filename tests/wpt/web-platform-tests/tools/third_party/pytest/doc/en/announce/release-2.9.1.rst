pytest-2.9.1
============

pytest is a mature Python testing tool with more than a 1100 tests
against itself, passing on many different interpreters and platforms.

See below for the changes and see docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to all who contributed to this release, among them:

    Bruno Oliveira                                                                                                                                                                                                                            
    Daniel Hahler                                                                                                                                                                                                                             
    Dmitry Malinovsky                                                                                                                                                                                                                         
    Florian Bruhin                                                                                                                                                                                                                            
    Floris Bruynooghe                                                                                                                                                                                                                         
    Matt Bachmann                                                                                                                                                                                                                             
    Ronny Pfannschmidt                                                                                                                                                                                                                        
    TomV                                                                                                                                                                                                                                      
    Vladimir Bolshakov                                                                                                                                                                                                                        
    Zearin                                                                                                                                                                                                                                     
    palaviv   


Happy testing,
The py.test Development Team


2.9.1 (compared to 2.9.0)
-------------------------

**Bug Fixes**

* Improve error message when a plugin fails to load.
  Thanks `@nicoddemus`_ for the PR.

* Fix (`#1178 <https://github.com/pytest-dev/pytest/issues/1178>`_):
  ``pytest.fail`` with non-ascii characters raises an internal pytest error.
  Thanks `@nicoddemus`_ for the PR.

* Fix (`#469`_): junit parses report.nodeid incorrectly, when params IDs
  contain ``::``. Thanks `@tomviner`_ for the PR (`#1431`_).

* Fix (`#578 <https://github.com/pytest-dev/pytest/issues/578>`_): SyntaxErrors
  containing non-ascii lines at the point of failure generated an internal
  py.test error.
  Thanks `@asottile`_ for the report and `@nicoddemus`_ for the PR.

* Fix (`#1437`_): When passing in a bytestring regex pattern to parameterize
  attempt to decode it as utf-8 ignoring errors.

* Fix (`#649`_): parametrized test nodes cannot be specified to run on the command line.


.. _#1437: https://github.com/pytest-dev/pytest/issues/1437
.. _#469: https://github.com/pytest-dev/pytest/issues/469
.. _#1431: https://github.com/pytest-dev/pytest/pull/1431
.. _#649: https://github.com/pytest-dev/pytest/issues/649

.. _@asottile: https://github.com/asottile
.. _@nicoddemus: https://github.com/nicoddemus
.. _@tomviner: https://github.com/tomviner
