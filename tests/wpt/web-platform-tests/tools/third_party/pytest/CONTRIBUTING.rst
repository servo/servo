============================
Contribution getting started
============================

Contributions are highly welcomed and appreciated.  Every little bit of help counts,
so do not hesitate!

.. contents::
   :depth: 2
   :backlinks: none


.. _submitfeedback:

Feature requests and feedback
-----------------------------

Do you like pytest?  Share some love on Twitter or in your blog posts!

We'd also like to hear about your propositions and suggestions.  Feel free to
`submit them as issues <https://github.com/pytest-dev/pytest/issues>`_ and:

* Explain in detail how they should work.
* Keep the scope as narrow as possible.  This will make it easier to implement.


.. _reportbugs:

Report bugs
-----------

Report bugs for pytest in the `issue tracker <https://github.com/pytest-dev/pytest/issues>`_.

If you are reporting a bug, please include:

* Your operating system name and version.
* Any details about your local setup that might be helpful in troubleshooting,
  specifically the Python interpreter version, installed libraries, and pytest
  version.
* Detailed steps to reproduce the bug.

If you can write a demonstration test that currently fails but should pass
(xfail), that is a very useful commit to make as well, even if you cannot
fix the bug itself.


.. _fixbugs:

Fix bugs
--------

Look through the `GitHub issues for bugs <https://github.com/pytest-dev/pytest/labels/type:%20bug>`_.

:ref:`Talk <contact>` to developers to find out how you can fix specific bugs. To indicate that you are going
to work on a particular issue, add a comment to that effect on the specific issue.

Don't forget to check the issue trackers of your favourite plugins, too!

.. _writeplugins:

Implement features
------------------

Look through the `GitHub issues for enhancements <https://github.com/pytest-dev/pytest/labels/type:%20enhancement>`_.

:ref:`Talk <contact>` to developers to find out how you can implement specific
features.

Write documentation
-------------------

Pytest could always use more documentation.  What exactly is needed?

* More complementary documentation.  Have you perhaps found something unclear?
* Documentation translations.  We currently have only English.
* Docstrings.  There can never be too many of them.
* Blog posts, articles and such -- they're all very appreciated.

You can also edit documentation files directly in the GitHub web interface,
without using a local copy.  This can be convenient for small fixes.

.. note::
    Build the documentation locally with the following command:

    .. code:: bash

        $ tox -e docs

    The built documentation should be available in ``doc/en/_build/html``,
    where 'en' refers to the documentation language.

Pytest has an API reference which in large part is
`generated automatically <https://www.sphinx-doc.org/en/master/usage/extensions/autodoc.html>`_
from the docstrings of the documented items. Pytest uses the
`Sphinx docstring format <https://sphinx-rtd-tutorial.readthedocs.io/en/latest/docstrings.html>`_.
For example:

.. code-block:: python

    def my_function(arg: ArgType) -> Foo:
        """Do important stuff.

        More detailed info here, in separate paragraphs from the subject line.
        Use proper sentences -- start sentences with capital letters and end
        with periods.

        Can include annotated documentation:

        :param short_arg: An argument which determines stuff.
        :param long_arg:
            A long explanation which spans multiple lines, overflows
            like this.
        :returns: The result.
        :raises ValueError:
            Detailed information when this can happen.

        .. versionadded:: 6.0

        Including types into the annotations above is not necessary when
        type-hinting is being used (as in this example).
        """


.. _submitplugin:

Submitting Plugins to pytest-dev
--------------------------------

Pytest development of the core, some plugins and support code happens
in repositories living under the ``pytest-dev`` organisations:

- `pytest-dev on GitHub <https://github.com/pytest-dev>`_

All pytest-dev Contributors team members have write access to all contained
repositories.  Pytest core and plugins are generally developed
using `pull requests`_ to respective repositories.

The objectives of the ``pytest-dev`` organisation are:

* Having a central location for popular pytest plugins
* Sharing some of the maintenance responsibility (in case a maintainer no
  longer wishes to maintain a plugin)

You can submit your plugin by subscribing to the `pytest-dev mail list
<https://mail.python.org/mailman/listinfo/pytest-dev>`_ and writing a
mail pointing to your existing pytest plugin repository which must have
the following:

- PyPI presence with packaging metadata that contains a ``pytest-``
  prefixed name, version number, authors, short and long description.

- a  `tox configuration <https://tox.readthedocs.io/en/latest/config.html#configuration-discovery>`_
  for running tests using `tox <https://tox.readthedocs.io>`_.

- a ``README`` describing how to use the plugin and on which
  platforms it runs.

- a ``LICENSE`` file containing the licensing information, with
  matching info in its packaging metadata.

- an issue tracker for bug reports and enhancement requests.

- a `changelog <https://keepachangelog.com/>`_.

If no contributor strongly objects and two agree, the repository can then be
transferred to the ``pytest-dev`` organisation.

Here's a rundown of how a repository transfer usually proceeds
(using a repository named ``joedoe/pytest-xyz`` as example):

* ``joedoe`` transfers repository ownership to ``pytest-dev`` administrator ``calvin``.
* ``calvin`` creates ``pytest-xyz-admin`` and ``pytest-xyz-developers`` teams, inviting ``joedoe`` to both as **maintainer**.
* ``calvin`` transfers repository to ``pytest-dev`` and configures team access:

  - ``pytest-xyz-admin`` **admin** access;
  - ``pytest-xyz-developers`` **write** access;

The ``pytest-dev/Contributors`` team has write access to all projects, and
every project administrator is in it. We recommend that each plugin has at least three
people who have the right to release to PyPI.

Repository owners can rest assured that no ``pytest-dev`` administrator will ever make
releases of your repository or take ownership in any way, except in rare cases
where someone becomes unresponsive after months of contact attempts.
As stated, the objective is to share maintenance and avoid "plugin-abandon".


.. _`pull requests`:
.. _pull-requests:

Preparing Pull Requests
-----------------------

Short version
~~~~~~~~~~~~~

#. Fork the repository.
#. Enable and install `pre-commit <https://pre-commit.com>`_ to ensure style-guides and code checks are followed.
#. Follow **PEP-8** for naming and `black <https://github.com/psf/black>`_ for formatting.
#. Tests are run using ``tox``::

    tox -e linting,py37

   The test environments above are usually enough to cover most cases locally.

#. Write a ``changelog`` entry: ``changelog/2574.bugfix.rst``, use issue id number
   and one of ``feature``, ``improvement``, ``bugfix``, ``doc``, ``deprecation``,
   ``breaking``, ``vendor`` or ``trivial`` for the issue type.


#. Unless your change is a trivial or a documentation fix (e.g., a typo or reword of a small section) please
   add yourself to the ``AUTHORS`` file, in alphabetical order.


Long version
~~~~~~~~~~~~

What is a "pull request"?  It informs the project's core developers about the
changes you want to review and merge.  Pull requests are stored on
`GitHub servers <https://github.com/pytest-dev/pytest/pulls>`_.
Once you send a pull request, we can discuss its potential modifications and
even add more commits to it later on. There's an excellent tutorial on how Pull
Requests work in the
`GitHub Help Center <https://help.github.com/articles/using-pull-requests/>`_.

Here is a simple overview, with pytest-specific bits:

#. Fork the
   `pytest GitHub repository <https://github.com/pytest-dev/pytest>`__.  It's
   fine to use ``pytest`` as your fork repository name because it will live
   under your user.

#. Clone your fork locally using `git <https://git-scm.com/>`_ and create a branch::

    $ git clone git@github.com:YOUR_GITHUB_USERNAME/pytest.git
    $ cd pytest
    # now, create your own branch off "main":

        $ git checkout -b your-bugfix-branch-name main

   Given we have "major.minor.micro" version numbers, bug fixes will usually
   be released in micro releases whereas features will be released in
   minor releases and incompatible changes in major releases.

   If you need some help with Git, follow this quick start
   guide: https://git.wiki.kernel.org/index.php/QuickStart

#. Install `pre-commit <https://pre-commit.com>`_ and its hook on the pytest repo::

     $ pip install --user pre-commit
     $ pre-commit install

   Afterwards ``pre-commit`` will run whenever you commit.

   https://pre-commit.com/ is a framework for managing and maintaining multi-language pre-commit hooks
   to ensure code-style and code formatting is consistent.

#. Install tox

   Tox is used to run all the tests and will automatically setup virtualenvs
   to run the tests in.
   (will implicitly use https://virtualenv.pypa.io/en/latest/)::

    $ pip install tox

#. Run all the tests

   You need to have Python 3.7 available in your system.  Now
   running tests is as simple as issuing this command::

    $ tox -e linting,py37

   This command will run tests via the "tox" tool against Python 3.7
   and also perform "lint" coding-style checks.

#. You can now edit your local working copy and run the tests again as necessary. Please follow PEP-8 for naming.

   You can pass different options to ``tox``. For example, to run tests on Python 3.7 and pass options to pytest
   (e.g. enter pdb on failure) to pytest you can do::

    $ tox -e py37 -- --pdb

   Or to only run tests in a particular test module on Python 3.7::

    $ tox -e py37 -- testing/test_config.py


   When committing, ``pre-commit`` will re-format the files if necessary.

#. If instead of using ``tox`` you prefer to run the tests directly, then we suggest to create a virtual environment and use
   an editable install with the ``testing`` extra::

       $ python3 -m venv .venv
       $ source .venv/bin/activate  # Linux
       $ .venv/Scripts/activate.bat  # Windows
       $ pip install -e ".[testing]"

   Afterwards, you can edit the files and run pytest normally::

       $ pytest testing/test_config.py

#. Create a new changelog entry in ``changelog``. The file should be named ``<issueid>.<type>.rst``,
   where *issueid* is the number of the issue related to the change and *type* is one of
   ``feature``, ``improvement``, ``bugfix``, ``doc``, ``deprecation``, ``breaking``, ``vendor``
   or ``trivial``. You may skip creating the changelog entry if the change doesn't affect the
   documented behaviour of pytest.

#. Add yourself to ``AUTHORS`` file if not there yet, in alphabetical order.

#. Commit and push once your tests pass and you are happy with your change(s)::

    $ git commit -a -m "<commit message>"
    $ git push -u

#. Finally, submit a pull request through the GitHub website using this data::

    head-fork: YOUR_GITHUB_USERNAME/pytest
    compare: your-branch-name

    base-fork: pytest-dev/pytest
    base: main


Writing Tests
~~~~~~~~~~~~~

Writing tests for plugins or for pytest itself is often done using the `pytester fixture <https://docs.pytest.org/en/stable/reference/reference.html#pytester>`_, as a "black-box" test.

For example, to ensure a simple test passes you can write:

.. code-block:: python

    def test_true_assertion(pytester):
        pytester.makepyfile(
            """
            def test_foo():
                assert True
        """
        )
        result = pytester.runpytest()
        result.assert_outcomes(failed=0, passed=1)


Alternatively, it is possible to make checks based on the actual output of the termal using
*glob-like* expressions:

.. code-block:: python

    def test_true_assertion(pytester):
        pytester.makepyfile(
            """
            def test_foo():
                assert False
        """
        )
        result = pytester.runpytest()
        result.stdout.fnmatch_lines(["*assert False*", "*1 failed*"])

When choosing a file where to write a new test, take a look at the existing files and see if there's
one file which looks like a good fit. For example, a regression test about a bug in the ``--lf`` option
should go into ``test_cacheprovider.py``, given that this option is implemented in ``cacheprovider.py``.
If in doubt, go ahead and open a PR with your best guess and we can discuss this over the code.

Joining the Development Team
----------------------------

Anyone who has successfully seen through a pull request which did not
require any extra work from the development team to merge will
themselves gain commit access if they so wish (if we forget to ask please send a friendly
reminder).  This does not mean there is any change in your contribution workflow:
everyone goes through the same pull-request-and-review process and
no-one merges their own pull requests unless already approved.  It does however mean you can
participate in the development process more fully since you can merge
pull requests from other contributors yourself after having reviewed
them.


Backporting bug fixes for the next patch release
------------------------------------------------

Pytest makes feature release every few weeks or months. In between, patch releases
are made to the previous feature release, containing bug fixes only. The bug fixes
usually fix regressions, but may be any change that should reach users before the
next feature release.

Suppose for example that the latest release was 1.2.3, and you want to include
a bug fix in 1.2.4 (check https://github.com/pytest-dev/pytest/releases for the
actual latest release). The procedure for this is:

#. First, make sure the bug is fixed the ``main`` branch, with a regular pull
   request, as described above. An exception to this is if the bug fix is not
   applicable to ``main`` anymore.

#. ``git checkout origin/1.2.x -b backport-XXXX`` # use the main PR number here

#. Locate the merge commit on the PR, in the *merged* message, for example:

    nicoddemus merged commit 0f8b462 into pytest-dev:main

#. ``git cherry-pick -x -m1 REVISION`` # use the revision you found above (``0f8b462``).

#. Open a PR targeting ``1.2.x``:

   * Prefix the message with ``[1.2.x]``.
   * Delete the PR body, it usually contains a duplicate commit message.


Who does the backporting
~~~~~~~~~~~~~~~~~~~~~~~~

As mentioned above, bugs should first be fixed on ``main`` (except in rare occasions
that a bug only happens in a previous release). So, who should do the backport procedure described
above?

1. If the bug was fixed by a core developer, it is the main responsibility of that core developer
   to do the backport.
2. However, often the merge is done by another maintainer, in which case it is nice of them to
   do the backport procedure if they have the time.
3. For bugs submitted by non-maintainers, it is expected that a core developer will to do
   the backport, normally the one that merged the PR on ``main``.
4. If a non-maintainers notices a bug which is fixed on ``main`` but has not been backported
   (due to maintainers forgetting to apply the *needs backport* label, or just plain missing it),
   they are also welcome to open a PR with the backport. The procedure is simple and really
   helps with the maintenance of the project.

All the above are not rules, but merely some guidelines/suggestions on what we should expect
about backports.

Handling stale issues/PRs
-------------------------

Stale issues/PRs are those where pytest contributors have asked for questions/changes
and the authors didn't get around to answer/implement them yet after a somewhat long time, or
the discussion simply died because people seemed to lose interest.

There are many reasons why people don't answer questions or implement requested changes:
they might get busy, lose interest, or just forget about it,
but the fact is that this is very common in open source software.

The pytest team really appreciates every issue and pull request, but being a high-volume project
with many issues and pull requests being submitted daily, we try to reduce the number of stale
issues and PRs by regularly closing them. When an issue/pull request is closed in this manner,
it is by no means a dismissal of the topic being tackled by the issue/pull request, but it
is just a way for us to clear up the queue and make the maintainers' work more manageable. Submitters
can always reopen the issue/pull request in their own time later if it makes sense.

When to close
~~~~~~~~~~~~~

Here are a few general rules the maintainers use deciding when to close issues/PRs because
of lack of inactivity:

* Issues labeled ``question`` or ``needs information``: closed after 14 days inactive.
* Issues labeled ``proposal``: closed after six months inactive.
* Pull requests: after one month, consider pinging the author, update linked issue, or consider closing. For pull requests which are nearly finished, the team should consider finishing it up and merging it.

The above are **not hard rules**, but merely **guidelines**, and can be (and often are!) reviewed on a case-by-case basis.

Closing pull requests
~~~~~~~~~~~~~~~~~~~~~

When closing a Pull Request, it needs to be acknowledging the time, effort, and interest demonstrated by the person which submitted it. As mentioned previously, it is not the intent of the team to dismiss a stalled pull request entirely but to merely to clear up our queue, so a message like the one below is warranted when closing a pull request that went stale:

    Hi <contributor>,

    First of all, we would like to thank you for your time and effort on working on this, the pytest team deeply appreciates it.

    We noticed it has been awhile since you have updated this PR, however. pytest is a high activity project, with many issues/PRs being opened daily, so it is hard for us maintainers to track which PRs are ready for merging, for review, or need more attention.

    So for those reasons we, think it is best to close the PR for now, but with the only intention to clean up our queue, it is by no means a rejection of your changes. We still encourage you to re-open this PR (it is just a click of a button away) when you are ready to get back to it.

    Again we appreciate your time for working on this, and hope you might get back to this at a later time!

    <bye>

Closing Issues
--------------

When a pull request is submitted to fix an issue, add text like ``closes #XYZW`` to the PR description and/or commits (where ``XYZW`` is the issue number). See the `GitHub docs <https://help.github.com/en/github/managing-your-work-on-github/linking-a-pull-request-to-an-issue#linking-a-pull-request-to-an-issue-using-a-keyword>`_ for more information.

When an issue is due to user error (e.g. misunderstanding of a functionality), please politely explain to the user why the issue raised is really a non-issue and ask them to close the issue if they have no further questions. If the original requestor is unresponsive, the issue will be handled as described in the section `Handling stale issues/PRs`_ above.
