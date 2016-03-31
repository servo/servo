
Asserting Warnings
=====================================================

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

The test will fail if the warning in question is not raised.

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
warnings, or index into it to get a particular recorded warning. It also
provides these methods:

.. autoclass:: _pytest.recwarn.WarningsRecorder()
    :members:

Each recorded warning has the attributes ``message``, ``category``,
``filename``, ``lineno``, ``file``, and ``line``. The ``category`` is the
class of the warning. The ``message`` is the warning itself; calling
``str(message)`` will return the actual message of the warning.

.. note::
    ``DeprecationWarning`` and ``PendingDeprecationWarning`` are treated
    differently; see :ref:`ensuring_function_triggers`.

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
