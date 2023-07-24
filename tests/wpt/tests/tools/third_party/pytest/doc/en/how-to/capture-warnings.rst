.. _`warnings`:

How to capture warnings
=======================



Starting from version ``3.1``, pytest now automatically catches warnings during test execution
and displays them at the end of the session:

.. code-block:: python

    # content of test_show_warnings.py
    import warnings


    def api_v1():
        warnings.warn(UserWarning("api v1, should use functions from v2"))
        return 1


    def test_one():
        assert api_v1() == 1

Running pytest now produces this output:

.. code-block:: pytest

    $ pytest test_show_warnings.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-7.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 1 item

    test_show_warnings.py .                                              [100%]

    ============================= warnings summary =============================
    test_show_warnings.py::test_one
      /home/sweet/project/test_show_warnings.py:5: UserWarning: api v1, should use functions from v2
        warnings.warn(UserWarning("api v1, should use functions from v2"))

    -- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
    ======================= 1 passed, 1 warning in 0.12s =======================

Controlling warnings
--------------------

Similar to Python's `warning filter`_ and :option:`-W option <python:-W>` flag, pytest provides
its own ``-W`` flag to control which warnings are ignored, displayed, or turned into
errors. See the `warning filter`_ documentation for more
advanced use-cases.

.. _`warning filter`: https://docs.python.org/3/library/warnings.html#warning-filter

This code sample shows how to treat any ``UserWarning`` category class of warning
as an error:

.. code-block:: pytest

    $ pytest -q test_show_warnings.py -W error::UserWarning
    F                                                                    [100%]
    ================================= FAILURES =================================
    _________________________________ test_one _________________________________

        def test_one():
    >       assert api_v1() == 1

    test_show_warnings.py:10:
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

        def api_v1():
    >       warnings.warn(UserWarning("api v1, should use functions from v2"))
    E       UserWarning: api v1, should use functions from v2

    test_show_warnings.py:5: UserWarning
    ========================= short test summary info ==========================
    FAILED test_show_warnings.py::test_one - UserWarning: api v1, should use ...
    1 failed in 0.12s

The same option can be set in the ``pytest.ini`` or ``pyproject.toml`` file using the
``filterwarnings`` ini option. For example, the configuration below will ignore all
user warnings and specific deprecation warnings matching a regex, but will transform
all other warnings into errors.

.. code-block:: ini

    # pytest.ini
    [pytest]
    filterwarnings =
        error
        ignore::UserWarning
        ignore:function ham\(\) is deprecated:DeprecationWarning

.. code-block:: toml

    # pyproject.toml
    [tool.pytest.ini_options]
    filterwarnings = [
        "error",
        "ignore::UserWarning",
        # note the use of single quote below to denote "raw" strings in TOML
        'ignore:function ham\(\) is deprecated:DeprecationWarning',
    ]


When a warning matches more than one option in the list, the action for the last matching option
is performed.


.. _`filterwarnings`:

``@pytest.mark.filterwarnings``
-------------------------------



You can use the ``@pytest.mark.filterwarnings`` to add warning filters to specific test items,
allowing you to have finer control of which warnings should be captured at test, class or
even module level:

.. code-block:: python

    import warnings


    def api_v1():
        warnings.warn(UserWarning("api v1, should use functions from v2"))
        return 1


    @pytest.mark.filterwarnings("ignore:api v1")
    def test_one():
        assert api_v1() == 1


Filters applied using a mark take precedence over filters passed on the command line or configured
by the ``filterwarnings`` ini option.

You may apply a filter to all tests of a class by using the ``filterwarnings`` mark as a class
decorator or to all tests in a module by setting the :globalvar:`pytestmark` variable:

.. code-block:: python

    # turns all warnings into errors for this module
    pytestmark = pytest.mark.filterwarnings("error")



*Credits go to Florian Schulze for the reference implementation in the* `pytest-warnings`_
*plugin.*

.. _`pytest-warnings`: https://github.com/fschulze/pytest-warnings

Disabling warnings summary
--------------------------

Although not recommended, you can use the ``--disable-warnings`` command-line option to suppress the
warning summary entirely from the test run output.

Disabling warning capture entirely
----------------------------------

This plugin is enabled by default but can be disabled entirely in your ``pytest.ini`` file with:

    .. code-block:: ini

        [pytest]
        addopts = -p no:warnings

Or passing ``-p no:warnings`` in the command-line. This might be useful if your test suites handles warnings
using an external system.


.. _`deprecation-warnings`:

DeprecationWarning and PendingDeprecationWarning
------------------------------------------------


By default pytest will display ``DeprecationWarning`` and ``PendingDeprecationWarning`` warnings from
user code and third-party libraries, as recommended by :pep:`565`.
This helps users keep their code modern and avoid breakages when deprecated warnings are effectively removed.

Sometimes it is useful to hide some specific deprecation warnings that happen in code that you have no control over
(such as third-party libraries), in which case you might use the warning filters options (ini or marks) to ignore
those warnings.

For example:

.. code-block:: ini

    [pytest]
    filterwarnings =
        ignore:.*U.*mode is deprecated:DeprecationWarning


This will ignore all warnings of type ``DeprecationWarning`` where the start of the message matches
the regular expression ``".*U.*mode is deprecated"``.

.. note::

    If warnings are configured at the interpreter level, using
    the :envvar:`python:PYTHONWARNINGS` environment variable or the
    ``-W`` command-line option, pytest will not configure any filters by default.

    Also pytest doesn't follow :pep:`506` suggestion of resetting all warning filters because
    it might break test suites that configure warning filters themselves
    by calling :func:`warnings.simplefilter` (see :issue:`2430` for an example of that).


.. _`ensuring a function triggers a deprecation warning`:

.. _ensuring_function_triggers:

Ensuring code triggers a deprecation warning
--------------------------------------------

You can also use :func:`pytest.deprecated_call` for checking
that a certain function call triggers a ``DeprecationWarning`` or
``PendingDeprecationWarning``:

.. code-block:: python

    import pytest


    def test_myfunction_deprecated():
        with pytest.deprecated_call():
            myfunction(17)

This test will fail if ``myfunction`` does not issue a deprecation warning
when called with a ``17`` argument.




.. _`asserting warnings`:

.. _assertwarnings:

.. _`asserting warnings with the warns function`:

.. _warns:

Asserting warnings with the warns function
------------------------------------------



You can check that code raises a particular warning using :func:`pytest.warns`,
which works in a similar manner to :ref:`raises <assertraises>`:

.. code-block:: python

    import warnings
    import pytest


    def test_warning():
        with pytest.warns(UserWarning):
            warnings.warn("my warning", UserWarning)

The test will fail if the warning in question is not raised. The keyword
argument ``match`` to assert that the exception matches a text or regex::

    >>> with warns(UserWarning, match='must be 0 or None'):
    ...     warnings.warn("value must be 0 or None", UserWarning)

    >>> with warns(UserWarning, match=r'must be \d+$'):
    ...     warnings.warn("value must be 42", UserWarning)

    >>> with warns(UserWarning, match=r'must be \d+$'):
    ...     warnings.warn("this is not here", UserWarning)
    Traceback (most recent call last):
      ...
    Failed: DID NOT WARN. No warnings of type ...UserWarning... were emitted...

You can also call :func:`pytest.warns` on a function or code string:

.. code-block:: python

    pytest.warns(expected_warning, func, *args, **kwargs)
    pytest.warns(expected_warning, "func(*args, **kwargs)")

The function also returns a list of all raised warnings (as
``warnings.WarningMessage`` objects), which you can query for
additional information:

.. code-block:: python

    with pytest.warns(RuntimeWarning) as record:
        warnings.warn("another warning", RuntimeWarning)

    # check that only one warning was raised
    assert len(record) == 1
    # check that the message matches
    assert record[0].message.args[0] == "another warning"

Alternatively, you can examine raised warnings in detail using the
:ref:`recwarn <recwarn>` fixture (see below).


The :ref:`recwarn <recwarn>` fixture automatically ensures to reset the warnings
filter at the end of the test, so no global state is leaked.

.. _`recording warnings`:

.. _recwarn:

Recording warnings
------------------

You can record raised warnings either using :func:`pytest.warns` or with
the ``recwarn`` fixture.

To record with :func:`pytest.warns` without asserting anything about the warnings,
pass no arguments as the expected warning type and it will default to a generic Warning:

.. code-block:: python

    with pytest.warns() as record:
        warnings.warn("user", UserWarning)
        warnings.warn("runtime", RuntimeWarning)

    assert len(record) == 2
    assert str(record[0].message) == "user"
    assert str(record[1].message) == "runtime"

The ``recwarn`` fixture will record warnings for the whole function:

.. code-block:: python

    import warnings


    def test_hello(recwarn):
        warnings.warn("hello", UserWarning)
        assert len(recwarn) == 1
        w = recwarn.pop(UserWarning)
        assert issubclass(w.category, UserWarning)
        assert str(w.message) == "hello"
        assert w.filename
        assert w.lineno

Both ``recwarn`` and :func:`pytest.warns` return the same interface for recorded
warnings: a WarningsRecorder instance. To view the recorded warnings, you can
iterate over this instance, call ``len`` on it to get the number of recorded
warnings, or index into it to get a particular recorded warning.

.. currentmodule:: _pytest.warnings

Full API: :class:`~_pytest.recwarn.WarningsRecorder`.

.. _`warns use cases`:

Additional use cases of warnings in tests
-----------------------------------------

Here are some use cases involving warnings that often come up in tests, and suggestions on how to deal with them:

- To ensure that **any** warning is emitted, use:

.. code-block:: python

    with pytest.warns():
        ...

-  To ensure that **no** warnings are emitted, use:

.. code-block:: python

    with warnings.catch_warnings():
        warnings.simplefilter("error")
        ...

- To suppress warnings, use:

.. code-block:: python

    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        ...


.. _custom_failure_messages:

Custom failure messages
-----------------------

Recording warnings provides an opportunity to produce custom test
failure messages for when no warnings are issued or other conditions
are met.

.. code-block:: python

    def test():
        with pytest.warns(Warning) as record:
            f()
            if not record:
                pytest.fail("Expected a warning!")

If no warnings are issued when calling ``f``, then ``not record`` will
evaluate to ``True``.  You can then call :func:`pytest.fail` with a
custom error message.

.. _internal-warnings:

Internal pytest warnings
------------------------

pytest may generate its own warnings in some situations, such as improper usage or deprecated features.

For example, pytest will emit a warning if it encounters a class that matches :confval:`python_classes` but also
defines an ``__init__`` constructor, as this prevents the class from being instantiated:

.. code-block:: python

    # content of test_pytest_warnings.py
    class Test:
        def __init__(self):
            pass

        def test_foo(self):
            assert 1 == 1

.. code-block:: pytest

    $ pytest test_pytest_warnings.py -q

    ============================= warnings summary =============================
    test_pytest_warnings.py:1
      /home/sweet/project/test_pytest_warnings.py:1: PytestCollectionWarning: cannot collect test class 'Test' because it has a __init__ constructor (from: test_pytest_warnings.py)
        class Test:

    -- Docs: https://docs.pytest.org/en/stable/how-to/capture-warnings.html
    1 warning in 0.12s

These warnings might be filtered using the same builtin mechanisms used to filter other types of warnings.

Please read our :ref:`backwards-compatibility` to learn how we proceed about deprecating and eventually removing
features.

The full list of warnings is listed in :ref:`the reference documentation <warnings ref>`.
