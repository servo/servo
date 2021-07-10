Python 2.7 and 3.4 support
==========================

It is demanding on the maintainers of an open source project to support many Python versions, as
there's extra cost of keeping code compatible between all versions, while holding back on
features only made possible on newer Python versions.

In case of Python 2 and 3, the difference between the languages makes it even more prominent,
because many new Python 3 features cannot be used in a Python 2/3 compatible code base.

Python 2.7 EOL has been reached `in 2020 <https://legacy.python.org/dev/peps/pep-0373/#id4>`__, with
the last release made in April, 2020.

Python 3.4 EOL has been reached `in 2019 <https://www.python.org/dev/peps/pep-0429/#release-schedule>`__, with the last release made in March, 2019.

For those reasons, in Jun 2019 it was decided that **pytest 4.6** series will be the last to support Python 2.7 and 3.4.

What this means for general users
---------------------------------

Thanks to the `python_requires`_ setuptools option,
Python 2.7 and Python 3.4 users using a modern pip version
will install the last pytest 4.6.X version automatically even if 5.0 or later versions
are available on PyPI.

Users should ensure they are using the latest pip and setuptools versions for this to work.

Maintenance of 4.6.X versions
-----------------------------

Until January 2020, the pytest core team ported many bug-fixes from the main release into the
``4.6.x`` branch, with several 4.6.X releases being made along the year.

From now on, the core team will **no longer actively backport patches**, but the ``4.6.x``
branch will continue to exist so the community itself can contribute patches.

The core team will be happy to accept those patches, and make new 4.6.X releases **until mid-2020**
(but consider that date as a ballpark, after that date the team might still decide to make new releases
for critical bugs).

.. _`python_requires`: https://packaging.python.org/guides/distributing-packages-using-setuptools/#python-requires

Technical aspects
~~~~~~~~~~~~~~~~~

(This section is a transcript from `#5275 <https://github.com/pytest-dev/pytest/issues/5275>`__).

In this section we describe the technical aspects of the Python 2.7 and 3.4 support plan.

What goes into 4.6.X releases
+++++++++++++++++++++++++++++

New 4.6.X releases will contain bug fixes only.

When will 4.6.X releases happen
+++++++++++++++++++++++++++++++

New 4.6.X releases will happen after we have a few bugs in place to release, or if a few weeks have
passed (say a single bug has been fixed a month after the latest 4.6.X release).

No hard rules here, just ballpark.

Who will handle applying bug fixes
++++++++++++++++++++++++++++++++++

We core maintainers expect that people still using Python 2.7/3.4 and being affected by
bugs to step up and provide patches and/or port bug fixes from the active branches.

We will be happy to guide users interested in doing so, so please don't hesitate to ask.

**Backporting changes into 4.6**

Please follow these instructions:

#. ``git fetch --all --prune``

#. ``git checkout origin/4.6.x -b backport-XXXX`` # use the PR number here

#. Locate the merge commit on the PR, in the *merged* message, for example:

    nicoddemus merged commit 0f8b462 into pytest-dev:features

#. ``git cherry-pick -m1 REVISION`` # use the revision you found above (``0f8b462``).

#. Open a PR targeting ``4.6.x``:

   * Prefix the message with ``[4.6]`` so it is an obvious backport
   * Delete the PR body, it usually contains a duplicate commit message.

**Providing new PRs to 4.6**

Fresh pull requests to ``4.6.x`` will be accepted provided that
the equivalent code in the active branches does not contain that bug (for example, a bug is specific
to Python 2 only).

Bug fixes that also happen in the mainstream version should be first fixed
there, and then backported as per instructions above.
