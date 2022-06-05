
.. _`captures`:

How to capture stdout/stderr output
=========================================================

Default stdout/stderr/stdin capturing behaviour
---------------------------------------------------------

During test execution any output sent to ``stdout`` and ``stderr`` is
captured.  If a test or a setup method fails its according captured
output will usually be shown along with the failure traceback. (this
behavior can be configured by the ``--show-capture`` command-line option).

In addition, ``stdin`` is set to a "null" object which will
fail on attempts to read from it because it is rarely desired
to wait for interactive input when running automated tests.

By default capturing is done by intercepting writes to low level
file descriptors.  This allows to capture output from simple
print statements as well as output from a subprocess started by
a test.

.. _capture-method:

Setting capturing methods or disabling capturing
-------------------------------------------------

There are three ways in which ``pytest`` can perform capturing:

* ``fd`` (file descriptor) level capturing (default): All writes going to the
  operating system file descriptors 1 and 2 will be captured.

* ``sys`` level capturing: Only writes to Python files ``sys.stdout``
  and ``sys.stderr`` will be captured.  No capturing of writes to
  filedescriptors is performed.

* ``tee-sys`` capturing: Python writes to ``sys.stdout`` and ``sys.stderr``
  will be captured, however the writes will also be passed-through to
  the actual ``sys.stdout`` and ``sys.stderr``. This allows output to be
  'live printed' and captured for plugin use, such as junitxml (new in pytest 5.4).

.. _`disable capturing`:

You can influence output capturing mechanisms from the command line:

.. code-block:: bash

    pytest -s                  # disable all capturing
    pytest --capture=sys       # replace sys.stdout/stderr with in-mem files
    pytest --capture=fd        # also point filedescriptors 1 and 2 to temp file
    pytest --capture=tee-sys   # combines 'sys' and '-s', capturing sys.stdout/stderr
                               # and passing it along to the actual sys.stdout/stderr

.. _printdebugging:

Using print statements for debugging
---------------------------------------------------

One primary benefit of the default capturing of stdout/stderr output
is that you can use print statements for debugging:

.. code-block:: python

    # content of test_module.py


    def setup_function(function):
        print("setting up", function)


    def test_func1():
        assert True


    def test_func2():
        assert False

and running this module will show you precisely the output
of the failing function and hide the other one:

.. code-block:: pytest

    $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-7.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 2 items

    test_module.py .F                                                    [100%]

    ================================= FAILURES =================================
    ________________________________ test_func2 ________________________________

        def test_func2():
    >       assert False
    E       assert False

    test_module.py:12: AssertionError
    -------------------------- Captured stdout setup ---------------------------
    setting up <function test_func2 at 0xdeadbeef0001>
    ========================= short test summary info ==========================
    FAILED test_module.py::test_func2 - assert False
    ======================= 1 failed, 1 passed in 0.12s ========================

Accessing captured output from a test function
---------------------------------------------------

The ``capsys``, ``capsysbinary``, ``capfd``, and ``capfdbinary`` fixtures
allow access to stdout/stderr output created during test execution.  Here is
an example test function that performs some output related checks:

.. code-block:: python

    def test_myoutput(capsys):  # or use "capfd" for fd-level
        print("hello")
        sys.stderr.write("world\n")
        captured = capsys.readouterr()
        assert captured.out == "hello\n"
        assert captured.err == "world\n"
        print("next")
        captured = capsys.readouterr()
        assert captured.out == "next\n"

The ``readouterr()`` call snapshots the output so far -
and capturing will be continued.  After the test
function finishes the original streams will
be restored.  Using ``capsys`` this way frees your
test from having to care about setting/resetting
output streams and also interacts well with pytest's
own per-test capturing.

If you want to capture on filedescriptor level you can use
the ``capfd`` fixture which offers the exact
same interface but allows to also capture output from
libraries or subprocesses that directly write to operating
system level output streams (FD1 and FD2).



The return value from ``readouterr`` changed to a ``namedtuple`` with two attributes, ``out`` and ``err``.



If the code under test writes non-textual data, you can capture this using
the ``capsysbinary`` fixture which instead returns ``bytes`` from
the ``readouterr`` method.




If the code under test writes non-textual data, you can capture this using
the ``capfdbinary`` fixture which instead returns ``bytes`` from
the ``readouterr`` method.  The ``capfdbinary`` fixture operates on the
filedescriptor level.




To temporarily disable capture within a test, both ``capsys``
and ``capfd`` have a ``disabled()`` method that can be used
as a context manager, disabling capture inside the ``with`` block:

.. code-block:: python

    def test_disabling_capturing(capsys):
        print("this output is captured")
        with capsys.disabled():
            print("output not captured, going directly to sys.stdout")
        print("this output is also captured")
