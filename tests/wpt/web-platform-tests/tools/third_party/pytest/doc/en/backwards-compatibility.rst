.. _backwards-compatibility:

Backwards Compatibility Policy
==============================

Keeping backwards compatibility has a very high priority in the pytest project. Although we have deprecated functionality over the years, most of it is still supported. All deprecations in pytest were done because simpler or more efficient ways of accomplishing the same tasks have emerged, making the old way of doing things unnecessary.

With the pytest 3.0 release we introduced a clear communication scheme for when we will actually remove the old busted joint and politely ask you to use the new hotness instead, while giving you enough time to adjust your tests or raise concerns if there are valid reasons to keep deprecated functionality around.

To communicate changes we are already issuing deprecation warnings, but they are not displayed by default. In pytest 3.0 we changed the default setting so that pytest deprecation warnings are displayed if not explicitly silenced (with ``--disable-pytest-warnings``).

We will only remove deprecated functionality in major releases (e.g. if we deprecate something in 3.0 we will remove it in 4.0), and keep it around for at least two minor releases (e.g. if we deprecate something in 3.9 and 4.0 is the next release, we will not remove it in 4.0 but in 5.0).


Deprecation Roadmap
-------------------

This page lists deprecated features and when we plan to remove them. It is important to list the feature, the version where it got deprecated and the version we plan to remove it.

Following our deprecation policy, we should aim to keep features for *at least* two minor versions after it was considered deprecated.


Future Releases
~~~~~~~~~~~~~~~

3.4
^^^

**Old style classes**

Issue: `#2147 <https://github.com/pytest-dev/pytest/issues/2147>`_.

Deprecated in ``3.2``.

4.0
^^^

**Yield tests**

Deprecated in ``3.0``.

**pytest-namespace hook**

deprecated in ``3.2``.

**Marks in parameter sets**

Deprecated in ``3.2``.

**--result-log**

Deprecated in ``3.0``.

See `#830 <https://github.com/pytest-dev/pytest/issues/830>`_ for more information. Suggested alternative: `pytest-tap <https://pypi.python.org/pypi/pytest-tap>`_.

**metafunc.addcall**

Issue: `#2876 <https://github.com/pytest-dev/pytest/issues/2876>`_.

Deprecated in ``3.3``.

**pytest_plugins in non-toplevel conftests**

There is a deep conceptual confusion as ``conftest.py`` files themselves are activated/deactivated based on path, but the plugins they depend on aren't.

Issue: `#2639 <https://github.com/pytest-dev/pytest/issues/2639>`_.

Not yet officially deprecated.

**passing a single string to pytest.main()**

Pass a list of strings to ``pytest.main()`` instead.

Deprecated in ``3.1``.

**[pytest] section in setup.cfg**

Use ``[tool:pytest]`` instead for compatibility with other tools.

Deprecated in ``3.0``.

Past Releases
~~~~~~~~~~~~~

3.0
^^^

* The following deprecated commandline options were removed:

  * ``--genscript``: no longer supported;
  * ``--no-assert``: use ``--assert=plain`` instead;
  * ``--nomagic``: use ``--assert=plain`` instead;
  * ``--report``: use ``-r`` instead;

* Removed all ``py.test-X*`` entry points. The versioned, suffixed entry points
  were never documented and a leftover from a pre-virtualenv era. These entry
  points also created broken entry points in wheels, so removing them also
  removes a source of confusion for users.



3.3
^^^

* Dropped support for EOL Python 2.6 and 3.3.