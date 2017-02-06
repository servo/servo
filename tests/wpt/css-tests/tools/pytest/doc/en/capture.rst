
.. _`captures`:

Capturing of the stdout/stderr output
=========================================================

Default stdout/stderr/stdin capturing behaviour
---------------------------------------------------------

During test execution any output sent to ``stdout`` and ``stderr`` is
captured.  If a test or a setup method fails its according captured
output will usually be shown along with the failure traceback.

In addition, ``stdin`` is set to a "null" object which will
fail on attempts to read from it because it is rarely desired
to wait for interactive input when running automated tests.

By default capturing is done by intercepting writes to low level
file descriptors.  This allows to capture output from simple
print statements as well as output from a subprocess started by
a test.

Setting capturing methods or disabling capturing
-------------------------------------------------

There are two ways in which ``pytest`` can perform capturing:

* file descriptor (FD) level capturing (default): All writes going to the
  operating system file descriptors 1 and 2 will be captured.

* ``sys`` level capturing: Only writes to Python files ``sys.stdout``
  and ``sys.stderr`` will be captured.  No capturing of writes to
  filedescriptors is performed.

.. _`disable capturing`:

You can influence output capturing mechanisms from the command line::

    py.test -s            # disable all capturing
    py.test --capture=sys # replace sys.stdout/stderr with in-mem files
    py.test --capture=fd  # also point filedescriptors 1 and 2 to temp file

.. _printdebugging:

Using print statements for debugging
---------------------------------------------------

One primary benefit of the default capturing of stdout/stderr output
is that you can use print statements for debugging::

    # content of test_module.py

    def setup_function(function):
        print ("setting up %s" % function)

    def test_func1():
        assert True

    def test_func2():
        assert False

and running this module will show you precisely the output
of the failing function and hide the other one::

    $ py.test
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR, inifile: 
    collected 2 items
    
    test_module.py .F
    
    ======= FAILURES ========
    _______ test_func2 ________
    
        def test_func2():
    >       assert False
    E       assert False
    
    test_module.py:9: AssertionError
    -------------------------- Captured stdout setup ---------------------------
    setting up <function test_func2 at 0xdeadbeef>
    ======= 1 failed, 1 passed in 0.12 seconds ========

Accessing captured output from a test function
---------------------------------------------------

The ``capsys`` and ``capfd`` fixtures allow to access stdout/stderr
output created during test execution.  Here is an example test function
that performs some output related checks:

.. code-block:: python

    def test_myoutput(capsys): # or use "capfd" for fd-level
        print ("hello")
        sys.stderr.write("world\n")
        out, err = capsys.readouterr()
        assert out == "hello\n"
        assert err == "world\n"
        print "next"
        out, err = capsys.readouterr()
        assert out == "next\n"

The ``readouterr()`` call snapshots the output so far -
and capturing will be continued.  After the test
function finishes the original streams will
be restored.  Using ``capsys`` this way frees your
test from having to care about setting/resetting
output streams and also interacts well with pytest's
own per-test capturing.

If you want to capture on filedescriptor level you can use
the ``capfd`` function argument which offers the exact
same interface but allows to also capture output from
libraries or subprocesses that directly write to operating
system level output streams (FD1 and FD2).

.. include:: links.inc
