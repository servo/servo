============================
Contribution getting started
============================

Contributions are highly welcomed and appreciated.  Every little help counts,
so do not hesitate!

.. contents:: Contribution links
   :depth: 2


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

:ref:`Talk <contact>` to developers to find out how you can fix specific bugs.

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

    The built documentation should be available in the ``doc/en/_build/``.

    Where 'en' refers to the documentation language.

.. _submitplugin:

Submitting Plugins to pytest-dev
--------------------------------

Pytest development of the core, some plugins and support code happens
in repositories living under the ``pytest-dev`` organisations:

- `pytest-dev on GitHub <https://github.com/pytest-dev>`_

- `pytest-dev on Bitbucket <https://bitbucket.org/pytest-dev>`_

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

- PyPI presence with a ``setup.py`` that contains a license, ``pytest-``
  prefixed name, version number, authors, short and long description.

- a ``tox.ini`` for running tests using `tox <https://tox.readthedocs.io>`_.

- a ``README.txt`` describing how to use the plugin and on which
  platforms it runs.

- a ``LICENSE.txt`` file or equivalent containing the licensing
  information, with matching info in ``setup.py``.

- an issue tracker for bug reports and enhancement requests.

- a `changelog <http://keepachangelog.com/>`_

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
#. Target ``master`` for bugfixes and doc changes.
#. Target ``features`` for new features or functionality changes.
#. Follow **PEP-8** for naming and `black <https://github.com/python/black>`_ for formatting.
#. Tests are run using ``tox``::

    tox -e linting,py27,py37

   The test environments above are usually enough to cover most cases locally.

#. Write a ``changelog`` entry: ``changelog/2574.bugfix``, use issue id number
   and one of ``bugfix``, ``removal``, ``feature``, ``vendor``, ``doc`` or
   ``trivial`` for the issue type.
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
    # now, to fix a bug create your own branch off "master":

        $ git checkout -b your-bugfix-branch-name master

    # or to instead add a feature create your own branch off "features":

        $ git checkout -b your-feature-branch-name features

   Given we have "major.minor.micro" version numbers, bugfixes will usually
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
   (will implicitly use http://www.virtualenv.org/en/latest/)::

    $ pip install tox

#. Run all the tests

   You need to have Python 2.7 and 3.7 available in your system.  Now
   running tests is as simple as issuing this command::

    $ tox -e linting,py27,py37

   This command will run tests via the "tox" tool against Python 2.7 and 3.7
   and also perform "lint" coding-style checks.

#. You can now edit your local working copy and run the tests again as necessary. Please follow PEP-8 for naming.

   You can pass different options to ``tox``. For example, to run tests on Python 2.7 and pass options to pytest
   (e.g. enter pdb on failure) to pytest you can do::

    $ tox -e py27 -- --pdb

   Or to only run tests in a particular test module on Python 3.7::

    $ tox -e py37 -- testing/test_config.py


   When committing, ``pre-commit`` will re-format the files if necessary.

#. Commit and push once your tests pass and you are happy with your change(s)::

    $ git commit -a -m "<commit message>"
    $ git push -u

#. Create a new changelog entry in ``changelog``. The file should be named ``<issueid>.<type>``,
   where *issueid* is the number of the issue related to the change and *type* is one of
   ``bugfix``, ``removal``, ``feature``, ``vendor``, ``doc`` or ``trivial``.

#. Add yourself to ``AUTHORS`` file if not there yet, in alphabetical order.

#. Finally, submit a pull request through the GitHub website using this data::

    head-fork: YOUR_GITHUB_USERNAME/pytest
    compare: your-branch-name

    base-fork: pytest-dev/pytest
    base: master          # if it's a bugfix
    base: features        # if it's a feature


Writing Tests
----------------------------

Writing tests for plugins or for pytest itself is often done using the `testdir fixture <https://docs.pytest.org/en/latest/reference.html#testdir>`_, as a "black-box" test.

For example, to ensure a simple test passes you can write:

.. code-block:: python

    def test_true_assertion(testdir):
        testdir.makepyfile(
            """
            def test_foo():
                assert True
        """
        )
        result = testdir.runpytest()
        result.assert_outcomes(failed=0, passed=1)


Alternatively, it is possible to make checks based on the actual output of the termal using
*glob-like* expressions:

.. code-block:: python

    def test_true_assertion(testdir):
        testdir.makepyfile(
            """
            def test_foo():
                assert False
        """
        )
        result = testdir.runpytest()
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
reminder).  This does not mean your workflow to contribute changes,
everyone goes through the same pull-request-and-review process and
no-one merges their own pull requests unless already approved.  It does however mean you can
participate in the development process more fully since you can merge
pull requests from other contributors yourself after having reviewed
them.
