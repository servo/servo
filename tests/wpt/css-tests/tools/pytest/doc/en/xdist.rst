
.. _`xdist`:

xdist: pytest distributed testing plugin
===============================================================

The `pytest-xdist`_ plugin extends ``pytest`` with some unique
test execution modes:

* Looponfail: run your tests repeatedly in a subprocess.  After each
  run, ``pytest`` waits until a file in your project changes and then
  re-runs the previously failing tests.  This is repeated until all
  tests pass.  At this point a full run is again performed.

* multiprocess Load-balancing: if you have multiple CPUs or hosts you can use
  them for a combined test run.  This allows to speed up
  development or to use special resources of remote machines.

* Multi-Platform coverage: you can specify different Python interpreters
  or different platforms and run tests in parallel on all of them.

Before running tests remotely, ``pytest`` efficiently "rsyncs" your
program source code to the remote place.  All test results
are reported back and displayed to your local terminal.
You may specify different Python versions and interpreters.


Installation of xdist plugin
------------------------------

Install the plugin with::

    easy_install pytest-xdist

    # or
    
    pip install pytest-xdist

or use the package in develop/in-place mode with
a checkout of the `pytest-xdist repository`_ ::

    python setup.py develop


Usage examples
---------------------

.. _`xdistcpu`:

Speed up test runs by sending tests to multiple CPUs
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

To send tests to multiple CPUs, type::

    py.test -n NUM

Especially for longer running tests or tests requiring
a lot of I/O this can lead to considerable speed ups.


Running tests in a Python subprocess
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

To instantiate a Python-2.7 subprocess and send tests to it, you may type::

    py.test -d --tx popen//python=python2.7

This will start a subprocess which is run with the "python2.7"
Python interpreter, found in your system binary lookup path.

If you prefix the --tx option value like this::

    py.test -d --tx 3*popen//python=python2.7

then three subprocesses would be created and the tests
will be distributed to three subprocesses and run simultanously.

.. _looponfailing:


Running tests in looponfailing mode
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

For refactoring a project with a medium or large test suite
you can use the looponfailing mode. Simply add the ``--f`` option::

    py.test -f
   
and ``pytest`` will run your tests. Assuming you have failures it will then
wait for file changes and re-run the failing test set.  File changes are detected by looking at ``looponfailingroots`` root directories and all of their contents (recursively).  If the default for this value does not work for you you
can change it in your project by setting a configuration option::

    # content of a pytest.ini, setup.cfg or tox.ini file
    [pytest]
    looponfailroots = mypkg testdir

This would lead to only looking for file changes in the respective directories, specified relatively to the ini-file's directory.

Sending tests to remote SSH accounts
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

Suppose you have a package ``mypkg`` which contains some
tests that you can successfully run locally. And you also
have a ssh-reachable machine ``myhost``.  Then
you can ad-hoc distribute your tests by typing::

    py.test -d --tx ssh=myhostpopen --rsyncdir mypkg mypkg

This will synchronize your ``mypkg`` package directory
with a remote ssh account and then collect and run your
tests at the remote side.

You can specify multiple ``--rsyncdir`` directories
to be sent to the remote side.

.. XXX CHECK

    **NOTE:** For ``pytest`` to collect and send tests correctly
    you not only need to make sure all code and tests
    directories are rsynced, but that any test (sub) directory
    also has an ``__init__.py`` file because internally
    ``pytest`` references tests as a fully qualified python
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

    py.test -d --tx socket=192.168.1.102:8888 --rsyncdir mypkg mypkg


.. _`atonce`:

Running tests on many platforms at once
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

The basic command to run tests on multiple platforms is::

    py.test --dist=each --tx=spec1 --tx=spec2

If you specify a windows host, an OSX host and a Linux
environment this command will send each tests to all
platforms - and report back failures from all platforms
at once.   The specifications strings use the `xspec syntax`_.

.. _`xspec syntax`: http://codespeak.net/execnet/basics.html#xspec

.. _`socketserver.py`: http://bitbucket.org/hpk42/execnet/raw/2af991418160/execnet/script/socketserver.py

.. _`execnet`: http://codespeak.net/execnet

Specifying test exec environments in an ini file
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

pytest (since version 2.0) supports ini-style configuration.
For example, you could make running with three subprocesses your default::

    [pytest]
    addopts = -n3

You can also add default environments like this::

    [pytest]
    addopts = --tx ssh=myhost//python=python2.7 --tx ssh=myhost//python=python2.6

and then just type::

    py.test --dist=each

to run tests in each of the environments.

Specifying "rsync" dirs in an ini-file
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

In a ``tox.ini`` or ``setup.cfg`` file in your root project directory
you may specify directories to include or to exclude in synchronisation::

    [pytest]
    rsyncdirs = . mypkg helperpkg
    rsyncignore = .hg

These directory specifications are relative to the directory
where the configuration file was found.

.. _`pytest-xdist`: http://pypi.python.org/pypi/pytest-xdist
.. _`pytest-xdist repository`: http://bitbucket.org/pytest-dev/pytest-xdist
.. _`pytest`: http://pytest.org

