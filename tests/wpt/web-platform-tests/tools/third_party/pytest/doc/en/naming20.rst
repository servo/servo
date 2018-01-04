
.. _naming20:

New pytest names in 2.0 (flat is better than nested)
----------------------------------------------------

If you used older version of the ``py`` distribution (which
included the py.test command line tool and Python name space)
you accessed helpers and possibly collection classes through
the ``py.test`` Python namespaces.  The new ``pytest``
Python module flaty provides the same objects, following
these renaming rules::

    py.test.XYZ          -> pytest.XYZ
    py.test.collect.XYZ  -> pytest.XYZ
    py.test.cmdline.main -> pytest.main

The old ``py.test.*`` ways to access functionality remain
valid but you are encouraged to do global renaming according
to the above rules in your test code.
