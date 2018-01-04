
loop on failing tests, distribute test runs to CPUs and hosts.
==============================================================


.. contents::
  :local:

The `pytest-xdist`_ plugin extends ``pytest`` with some unique
test execution modes:

* Looponfail: run your tests repeatedly in a subprocess.  After each run
  ``pytest`` waits until a file in your project changes and then re-runs the
  previously failing tests.  This is repeated until all tests pass after which
  again a full run is performed.

* Load-balancing: if you have multiple CPUs or hosts you can use
  those for a combined test run.  This allows to speed up
  development or to use special resources of remote machines.

* Multi-Platform coverage: you can specify different Python interpreters
  or different platforms and run tests in parallel on all of them.

Before running tests remotely, ``pytest`` efficiently synchronizes your
program source code to the remote place.  All test results
are reported back and displayed to your local test session.
You may specify different Python versions and interpreters.

.. _`pytest-xdist`: http://pypi.python.org/pypi/pytest-xdist

Usage examples
---------------------

Speed up test runs by sending tests to multiple CPUs
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

To send tests to multiple CPUs, type::

    pytest -n NUM

Especially for longer running tests or tests requiring
a lot of IO this can lead to considerable speed ups.


Running tests in a Python subprocess
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

To instantiate a python2.4 sub process and send tests to it, you may type::

    pytest -d --tx popen//python=python2.4

This will start a subprocess which is run with the "python2.4"
Python interpreter, found in your system binary lookup path.

If you prefix the --tx option value like this::

    --tx 3*popen//python=python2.4

then three subprocesses would be created and tests
will be load-balanced across these three processes.


Sending tests to remote SSH accounts
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

Suppose you have a package ``mypkg`` which contains some
tests that you can successfully run locally. And you
have a ssh-reachable machine ``myhost``.  Then
you can ad-hoc distribute your tests by typing::

    pytest -d --tx ssh=myhostpopen --rsyncdir mypkg mypkg

This will synchronize your ``mypkg`` package directory
to a remote ssh account and then locally collect tests
and send them to remote places for execution.

You can specify multiple ``--rsyncdir`` directories
to be sent to the remote side.

**NOTE:** For ``pytest`` to collect and send tests correctly
you not only need to make sure all code and tests
directories are rsynced, but that any test (sub) directory
also has an ``__init__.py`` file because internally
``pytest`` references tests using their fully qualified python
module path.  **You will otherwise get strange errors**
during setup of the remote side.

Sending tests to remote Socket Servers
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

Download the single-module `socketserver.py`_ Python program
and run it like this::

    python socketserver.py

It will tell you that it starts listening on the default
port.  You can now on your home machine specify this
new socket host with something like this::

    pytest -d --tx socket=192.168.1.102:8888 --rsyncdir mypkg mypkg


.. _`atonce`:

Running tests on many platforms at once
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

The basic command to run tests on multiple platforms is::

    pytest --dist=each --tx=spec1 --tx=spec2

If you specify a windows host, an OSX host and a Linux
environment this command will send each tests to all
platforms - and report back failures from all platforms
at once.   The specifications strings use the `xspec syntax`_.

.. _`xspec syntax`: http://codespeak.net/execnet/trunk/basics.html#xspec

.. _`socketserver.py`: http://codespeak.net/svn/py/dist/py/execnet/script/socketserver.py

.. _`execnet`: http://codespeak.net/execnet

Specifying test exec environments in a conftest.py
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

Instead of specifying command line options, you can
put options values in a ``conftest.py`` file like this::

    option_tx = ['ssh=myhost//python=python2.7', 'popen//python=python2.7']
    option_dist = True

Any commandline ``--tx`` specifications  will add to the list of
available execution environments.

Specifying "rsync" dirs in a conftest.py
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

In your ``mypkg/conftest.py`` you may specify directories to synchronise
or to exclude::

    rsyncdirs = ['.', '../plugins']
    rsyncignore = ['_cache']

These directory specifications are relative to the directory
where the ``conftest.py`` is found.

command line options
--------------------


``-f, --looponfail``
    run tests in subprocess, wait for modified files and re-run failing test set until all pass.
``-n numprocesses``
    shortcut for '--dist=load --tx=NUM*popen'
``--boxed``
    box each test run in a separate process (unix)
``--dist=distmode``
    set mode for distributing tests to exec environments.

    each: send each test to each available environment.

    load: send each test to one available environment so it is run only once.

    (default) no: run tests inprocess, don't distribute.
``--tx=xspec``
    add a test execution environment. some examples: --tx popen//python=python2.7 --tx socket=192.168.1.102:8888 --tx ssh=user@codespeak.net//chdir=testcache
``-d``
    load-balance tests.  shortcut for '--dist=load'
``--rsyncdir=dir1``
    add directory for rsyncing to remote tx nodes.

.. include:: links.txt
