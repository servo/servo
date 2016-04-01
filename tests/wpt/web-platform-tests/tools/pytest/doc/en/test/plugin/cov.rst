
produce code coverage reports using the 'coverage' package, including support for distributed testing.
======================================================================================================


.. contents::
  :local:

This plugin produces coverage reports.  It supports centralised testing and distributed testing in
both load and each modes.  It also supports coverage of subprocesses.

All features offered by the coverage package should be available, either through pytest-cov or
through coverage's config file.


Installation
------------

The `pytest-cov`_ package may be installed with pip or easy_install::

    pip install pytest-cov
    easy_install pytest-cov

.. _`pytest-cov`: http://pypi.python.org/pypi/pytest-cov/


Uninstallation
--------------

Uninstalling packages is supported by pip::

    pip uninstall pytest-cov

However easy_install does not provide an uninstall facility.

.. IMPORTANT::

    Ensure that you manually delete the init_covmain.pth file in your
    site-packages directory.

    This file starts coverage collection of subprocesses if appropriate during
    site initialization at python startup.


Usage
-----

Centralised Testing
~~~~~~~~~~~~~~~~~~~

Centralised testing will report on the combined coverage of the main process and all of it's
subprocesses.

Running centralised testing::

    py.test --cov myproj tests/

Shows a terminal report::

    -------------------- coverage: platform linux2, python 2.6.4-final-0 ---------------------
    Name                 Stmts   Miss  Cover
    ----------------------------------------
    myproj/__init__          2      0   100%
    myproj/myproj          257     13    94%
    myproj/feature4286      94      7    92%
    ----------------------------------------
    TOTAL                  353     20    94%


Distributed Testing: Load
~~~~~~~~~~~~~~~~~~~~~~~~~

Distributed testing with dist mode set to load will report on the combined coverage of all slaves.
The slaves may be spread out over any number of hosts and each slave may be located anywhere on the
file system.  Each slave will have it's subprocesses measured.

Running distributed testing with dist mode set to load::

    py.test --cov myproj -n 2 tests/

Shows a terminal report::

    -------------------- coverage: platform linux2, python 2.6.4-final-0 ---------------------
    Name                 Stmts   Miss  Cover
    ----------------------------------------
    myproj/__init__          2      0   100%
    myproj/myproj          257     13    94%
    myproj/feature4286      94      7    92%
    ----------------------------------------
    TOTAL                  353     20    94%


Again but spread over different hosts and different directories::

    py.test --cov myproj --dist load
            --tx ssh=memedough@host1//chdir=testenv1
            --tx ssh=memedough@host2//chdir=/tmp/testenv2//python=/tmp/env1/bin/python
            --rsyncdir myproj --rsyncdir tests --rsync examples
            tests/

Shows a terminal report::

    -------------------- coverage: platform linux2, python 2.6.4-final-0 ---------------------
    Name                 Stmts   Miss  Cover
    ----------------------------------------
    myproj/__init__          2      0   100%
    myproj/myproj          257     13    94%
    myproj/feature4286      94      7    92%
    ----------------------------------------
    TOTAL                  353     20    94%


Distributed Testing: Each
~~~~~~~~~~~~~~~~~~~~~~~~~

Distributed testing with dist mode set to each will report on the combined coverage of all slaves.
Since each slave is running all tests this allows generating a combined coverage report for multiple
environments.

Running distributed testing with dist mode set to each::

    py.test --cov myproj --dist each
            --tx popen//chdir=/tmp/testenv3//python=/usr/local/python27/bin/python
            --tx ssh=memedough@host2//chdir=/tmp/testenv4//python=/tmp/env2/bin/python
            --rsyncdir myproj --rsyncdir tests --rsync examples
            tests/

Shows a terminal report::

    ---------------------------------------- coverage ----------------------------------------
                              platform linux2, python 2.6.5-final-0
                              platform linux2, python 2.7.0-final-0
    Name                 Stmts   Miss  Cover
    ----------------------------------------
    myproj/__init__          2      0   100%
    myproj/myproj          257     13    94%
    myproj/feature4286      94      7    92%
    ----------------------------------------
    TOTAL                  353     20    94%


Reporting
---------

It is possible to generate any combination of the reports for a single test run.

The available reports are terminal (with or without missing line numbers shown), HTML, XML and
annotated source code.

The terminal report without line numbers (default)::

    py.test --cov-report term --cov myproj tests/

    -------------------- coverage: platform linux2, python 2.6.4-final-0 ---------------------
    Name                 Stmts   Miss  Cover
    ----------------------------------------
    myproj/__init__          2      0   100%
    myproj/myproj          257     13    94%
    myproj/feature4286      94      7    92%
    ----------------------------------------
    TOTAL                  353     20    94%


The terminal report with line numbers::

    py.test --cov-report term-missing --cov myproj tests/

    -------------------- coverage: platform linux2, python 2.6.4-final-0 ---------------------
    Name                 Stmts   Miss  Cover   Missing
    --------------------------------------------------
    myproj/__init__          2      0   100%
    myproj/myproj          257     13    94%   24-26, 99, 149, 233-236, 297-298, 369-370
    myproj/feature4286      94      7    92%   183-188, 197
    --------------------------------------------------
    TOTAL                  353     20    94%


The remaining three reports output to files without showing anything on the terminal (useful for
when the output is going to a continuous integration server)::

    py.test --cov-report html --cov-report xml --cov-report annotate --cov myproj tests/


Coverage Data File
------------------

The data file is erased at the beginning of testing to ensure clean data for each test run.

The data file is left at the end of testing so that it is possible to use normal coverage tools to
examine it.


Limitations
-----------

For distributed testing the slaves must have the pytest-cov package installed.  This is needed since
the plugin must be registered through setuptools / distribute for pytest to start the plugin on the
slave.

For subprocess measurement environment variables must make it from the main process to the
subprocess.  The python used by the subprocess must have pytest-cov installed.  The subprocess must
do normal site initialization so that the environment variables can be detected and coverage
started.


Acknowledgments
----------------

Holger Krekel for pytest with its distributed testing support.

Ned Batchelder for coverage and its ability to combine the coverage results of parallel runs.

Whilst this plugin has been built fresh from the ground up to support distributed testing it has
been influenced by the work done on pytest-coverage (Ross Lawley, James Mills, Holger Krekel) and
nose-cover (Jason Pellerin) which are other coverage plugins for pytest and nose respectively.

No doubt others have contributed to these tools as well.

command line options
--------------------


``--cov=path``
    measure coverage for filesystem path (multi-allowed)
``--cov-report=type``
    type of report to generate: term, term-missing, annotate, html, xml (multi-allowed)
``--cov-config=path``
    config file for coverage, default: .coveragerc

.. include:: links.txt
