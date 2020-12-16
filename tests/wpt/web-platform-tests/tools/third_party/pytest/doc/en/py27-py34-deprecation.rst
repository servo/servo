Python 2.7 and 3.4 support plan
===============================

Python 2.7 EOL is fast approaching, with
upstream support `ending in 2020 <https://legacy.python.org/dev/peps/pep-0373/#id4>`__.
Python 3.4's last release is scheduled for
`March 2019 <https://www.python.org/dev/peps/pep-0429/#release-schedule>`__. pytest is one of
the participating projects of the https://python3statement.org.

The **pytest 4.6** series will be the last to support Python 2.7 and 3.4, and is scheduled
to be released by **mid-2019**. **pytest 5.0** and onwards will support only Python 3.5+.

Thanks to the `python_requires`_ ``setuptools`` option,
Python 2.7 and Python 3.4 users using a modern ``pip`` version
will install the last pytest ``4.6`` version automatically even if ``5.0`` or later
are available on PyPI.

While pytest ``5.0`` will be the new mainstream and development version, until **January 2020**
the pytest core team plans to make bug-fix releases of the pytest ``4.6`` series by
back-porting patches to the ``4.6.x`` branch that affect Python 2 users.

**After 2020**, the core team will no longer actively backport patches, but the ``4.6.x``
branch will continue to exist so the community itself can contribute patches. The core team will
be happy to accept those patches and make new ``4.6`` releases **until mid-2020**.

.. _`python_requires`: https://packaging.python.org/guides/distributing-packages-using-setuptools/#python-requires>
