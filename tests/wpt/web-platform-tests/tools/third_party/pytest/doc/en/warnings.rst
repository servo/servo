.. _`warnings`:

Warnings Capture
================

.. versionadded:: 3.1

Starting from version ``3.1``, pytest now automatically catches warnings during test execution
and displays them at the end of the session::

    # content of test_show_warnings.py
    import warnings

    def api_v1():
        warnings.warn(UserWarning("api v1, should use functions from v2"))
        return 1

    def test_one():
        assert api_v1() == 1

Running pytest now produces this output::

    $ pytest test_show_warnings.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR, inifile:
    collected 1 item

    test_show_warnings.py .                                              [100%]

    ============================= warnings summary =============================
    test_show_warnings.py::test_one
      $REGENDOC_TMPDIR/test_show_warnings.py:4: UserWarning: api v1, should use functions from v2
        warnings.warn(UserWarning("api v1, should use functions from v2"))

    -- Docs: http://doc.pytest.org/en/latest/warnings.html
    =================== 1 passed, 1 warnings in 0.12 seconds ===================

Pytest by default catches all warnings except for ``DeprecationWarning`` and ``PendingDeprecationWarning``.

The ``-W`` flag can be passed to control which warnings will be displayed or even turn
them into errors::

    $ pytest -q test_show_warnings.py -W error::UserWarning
    F                                                                    [100%]
    ================================= FAILURES =================================
    _________________________________ test_one _________________________________

        def test_one():
    >       assert api_v1() == 1

    test_show_warnings.py:8:
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

        def api_v1():
    >       warnings.warn(UserWarning("api v1, should use functions from v2"))
    E       UserWarning: api v1, should use functions from v2

    test_show_warnings.py:4: UserWarning
    1 failed in 0.12 seconds

The same option can be set in the ``pytest.ini`` file using the ``filterwarnings`` ini option.
For example, the configuration below will ignore all user warnings, but will transform
all other warnings into errors.

.. code-block:: ini

    [pytest]
    filterwarnings =
        error
        ignore::UserWarning


When a warning matches more than one option in the list, the action for the last matching option
is performed.

Both ``-W`` command-line option and ``filterwarnings`` ini option are based on Python's own
`-W option`_ and `warnings.simplefilter`_, so please refer to those sections in the Python
documentation for other examples and advanced usage.


.. _`filterwarnings`:

``@pytest.mark.filterwarnings``
-------------------------------

.. versionadded:: 3.2

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
decorator or to all tests in a module by setting the ``pytestmark`` variable:

.. code-block:: python

    # turns all warnings into errors for this module
    pytestmark = pytest.mark.filterwarnings("error")


.. note::

    Except for these features, pytest does not change the python warning filter; it only captures
    and displays the warnings which are issued with respect to the currently configured filter,
    including changes to the filter made by test functions or by the system under test.

.. note::

    ``DeprecationWarning`` and ``PendingDeprecationWarning`` are hidden by the standard library
    by default so you have to explicitly configure them to be displayed in your ``pytest.ini``:

    .. code-block:: ini

        [pytest]
        filterwarnings =
            once::DeprecationWarning
            once::PendingDeprecationWarning


*Credits go to Florian Schulze for the reference implementation in the* `pytest-warnings`_
*plugin.*

.. _`-W option`: https://docs.python.org/3/using/cmdline.html?highlight=#cmdoption-W
.. _warnings.simplefilter: https://docs.python.org/3/library/warnings.html#warnings.simplefilter
.. _`pytest-warnings`: https://github.com/fschulze/pytest-warnings


Disabling warning capture
-------------------------

This feature is enabled by default but can be disabled entirely in your ``pytest.ini`` file with:

    .. code-block:: ini

        [pytest]
        addopts = -p no:warnings

Or passing ``-p no:warnings`` in the command-line.

.. _`asserting warnings`:

.. _assertwarnings:

.. _`asserting warnings with the warns function`:

.. _warns:

Asserting warnings with the warns function
-----------------------------------------------

.. versionadded:: 2.8

You can check that code raises a particular warning using ``pytest.warns``,
which works in a similar manner to :ref:`raises <assertraises>`::

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
    Failed: DID NOT WARN. No warnings of type ...UserWarning... was emitted...

You can also call ``pytest.warns`` on a function or code string::

    pytest.warns(expected_warning, func, *args, **kwargs)
    pytest.warns(expected_warning, "func(*args, **kwargs)")

The function also returns a list of all raised warnings (as
``warnings.WarningMessage`` objects), which you can query for
additional information::

    with pytest.warns(RuntimeWarning) as record:
        warnings.warn("another warning", RuntimeWarning)

    # check that only one warning was raised
    assert len(record) == 1
    # check that the message matches
    assert record[0].message.args[0] == "another warning"

Alternatively, you can examine raised warnings in detail using the
:ref:`recwarn <recwarn>` fixture (see below).

.. note::
    ``DeprecationWarning`` and ``PendingDeprecationWarning`` are treated
    differently; see :ref:`ensuring_function_triggers`.

.. _`recording warnings`:

.. _recwarn:

Recording warnings
------------------------

You can record raised warnings either using ``pytest.warns`` or with
the ``recwarn`` fixture.

To record with ``pytest.warns`` without asserting anything about the warnings,
pass ``None`` as the expected warning type::

    with pytest.warns(None) as record:
        warnings.warn("user", UserWarning)
        warnings.warn("runtime", RuntimeWarning)

    assert len(record) == 2
    assert str(record[0].message) == "user"
    assert str(record[1].message) == "runtime"

The ``recwarn`` fixture will record warnings for the whole function::

    import warnings

    def test_hello(recwarn):
        warnings.warn("hello", UserWarning)
        assert len(recwarn) == 1
        w = recwarn.pop(UserWarning)
        assert issubclass(w.category, UserWarning)
        assert str(w.message) == "hello"
        assert w.filename
        assert w.lineno

Both ``recwarn`` and ``pytest.warns`` return the same interface for recorded
warnings: a WarningsRecorder instance. To view the recorded warnings, you can
iterate over this instance, call ``len`` on it to get the number of recorded
warnings, or index into it to get a particular recorded warning.

.. currentmodule:: _pytest.warnings

Full API: :class:`WarningsRecorder`.

.. _`ensuring a function triggers a deprecation warning`:

.. _ensuring_function_triggers:

Ensuring a function triggers a deprecation warning
-------------------------------------------------------

You can also call a global helper for checking
that a certain function call triggers a ``DeprecationWarning`` or
``PendingDeprecationWarning``::

    import pytest

    def test_global():
        pytest.deprecated_call(myfunction, 17)

By default, ``DeprecationWarning`` and ``PendingDeprecationWarning`` will not be
caught when using ``pytest.warns`` or ``recwarn`` because default Python warnings filters hide
them. If you wish to record them in your own code, use the
command ``warnings.simplefilter('always')``::

    import warnings
    import pytest

    def test_deprecation(recwarn):
        warnings.simplefilter('always')
        warnings.warn("deprecated", DeprecationWarning)
        assert len(recwarn) == 1
        assert recwarn.pop(DeprecationWarning)

You can also use it as a contextmanager::

    def test_global():
        with pytest.deprecated_call():
            myobject.deprecated_method()
