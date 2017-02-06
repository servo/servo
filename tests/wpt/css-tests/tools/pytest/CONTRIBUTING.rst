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
  specifically Python interpreter version,
  installed libraries and pytest version.
* Detailed steps to reproduce the bug.

If you can write a demonstration test that currently fails but should pass (xfail),
that is a very useful commit to make as well, even if you can't find how
to fix the bug yet.


.. _fixbugs:

Fix bugs
--------

Look through the GitHub issues for bugs.  Here is sample filter you can use:
https://github.com/pytest-dev/pytest/labels/bug

:ref:`Talk <contact>` to developers to find out how you can fix specific bugs.

Don't forget to check the issue trackers of your favourite plugins, too!

.. _writeplugins:

Implement features
------------------

Look through the GitHub issues for enhancements.  Here is sample filter you
can use:
https://github.com/pytest-dev/pytest/labels/enhancement

:ref:`Talk <contact>` to developers to find out how you can implement specific
features.

Write documentation
-------------------

pytest could always use more documentation.  What exactly is needed?

* More complementary documentation.  Have you perhaps found something unclear?
* Documentation translations.  We currently have only English.
* Docstrings.  There can never be too many of them.
* Blog posts, articles and such -- they're all very appreciated.

You can also edit documentation files directly in the Github web interface
without needing to make a fork and local copy. This can be convenient for
small fixes.


.. _submitplugin:

Submitting Plugins to pytest-dev
--------------------------------

Pytest development of the core, some plugins and support code happens
in repositories living under the ``pytest-dev`` organisations:

- `pytest-dev on GitHub <https://github.com/pytest-dev>`_

- `pytest-dev on Bitbucket <https://bitbucket.org/pytest-dev>`_

All pytest-dev Contributors team members have write access to all contained
repositories.  pytest core and plugins are generally developed
using `pull requests`_ to respective repositories.

The objectives of the ``pytest-dev`` organisation are:

* Having a central location for popular pytest plugins
* Sharing some of the maintenance responsibility (in case a maintainer no longer whishes to maintain a plugin)

You can submit your plugin by subscribing to the `pytest-dev mail list
<https://mail.python.org/mailman/listinfo/pytest-dev>`_ and writing a
mail pointing to your existing pytest plugin repository which must have
the following:

- PyPI presence with a ``setup.py`` that contains a license, ``pytest-``
  prefixed name, version number, authors, short and long description.

- a ``tox.ini`` for running tests using `tox <http://tox.testrun.org>`_.

- a ``README.txt`` describing how to use the plugin and on which
  platforms it runs.

- a ``LICENSE.txt`` file or equivalent containing the licensing
  information, with matching info in ``setup.py``.

- an issue tracker for bug reports and enhancement requests.

If no contributor strongly objects and two agree, the repository can then be
transferred to the ``pytest-dev`` organisation.

Here's a rundown of how a repository transfer usually proceeds
(using a repository named ``joedoe/pytest-xyz`` as example):

* One of the ``pytest-dev`` administrators creates:

  - ``pytest-xyz-admin`` team, with full administration rights to
    ``pytest-dev/pytest-xyz``.
  - ``pytest-xyz-developers`` team, with write access to
    ``pytest-dev/pytest-xyz``.

* ``joedoe`` is invited to the ``pytest-xyz-admin`` team;

* After accepting the invitation, ``joedoe`` transfers the repository from its
  original location to ``pytest-dev/pytest-xyz`` (A nice feature is that GitHub handles URL redirection from
  the old to the new location automatically).

* ``joedoe`` is free to add any other collaborators to the
  ``pytest-xyz-admin`` or ``pytest-xyz-developers`` team as desired.

The ``pytest-dev/Contributors`` team has write access to all projects, and
every project administrator is in it. We recommend that each plugin has at least three
people who have the right to release to PyPI.

Repository owners can be assured that no ``pytest-dev`` administrator will ever make
releases of your repository or take ownership in any way, except in rare cases
where someone becomes unresponsive after months of contact attempts.
As stated, the objective is to share maintenance and avoid "plugin-abandon".


.. _`pull requests`:
.. _pull-requests:

Preparing Pull Requests on GitHub
---------------------------------

There's an excellent tutorial on how Pull Requests work in the
`GitHub Help Center <https://help.github.com/articles/using-pull-requests/>`_


.. note::
  What is a "pull request"?  It informs project's core developers about the
  changes you want to review and merge.  Pull requests are stored on
  `GitHub servers <https://github.com/pytest-dev/pytest/pulls>`_.
  Once you send pull request, we can discuss it's potential modifications and
  even add more commits to it later on.

There's an excellent tutorial on how Pull Requests work in the
`GitHub Help Center <https://help.github.com/articles/using-pull-requests/>`_,
but here is a simple overview:

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

#. Install tox

   Tox is used to run all the tests and will automatically setup virtualenvs
   to run the tests in.
   (will implicitly use http://www.virtualenv.org/en/latest/)::

    $ pip install tox

#. Run all the tests

   You need to have Python 2.7 and 3.5 available in your system.  Now
   running tests is as simple as issuing this command::

    $ python runtox.py -e linting,py27,py35

   This command will run tests via the "tox" tool against Python 2.7 and 3.5
   and also perform "lint" coding-style checks.  ``runtox.py`` is
   a thin wrapper around ``tox`` which installs from a development package
   index where newer (not yet released to pypi) versions of dependencies
   (especially ``py``) might be present.

#. You can now edit your local working copy.

   You can now make the changes you want and run the tests again as necessary.

   To run tests on py27 and pass options to pytest (e.g. enter pdb on failure)
   to pytest you can do::

    $ python runtox.py -e py27 -- --pdb

   or to only run tests in a particular test module on py35::

    $ python runtox.py -e py35 -- testing/test_config.py

#. Commit and push once your tests pass and you are happy with your change(s)::

    $ git commit -a -m "<commit message>"
    $ git push -u

   Make sure you add a CHANGELOG message, and add yourself to AUTHORS. If you
   are unsure about either of these steps, submit your pull request and we'll
   help you fix it up.

#. Finally, submit a pull request through the GitHub website using this data::

    head-fork: YOUR_GITHUB_USERNAME/pytest
    compare: your-branch-name

    base-fork: pytest-dev/pytest
    base: master          # if it's a bugfix
    base: feature         # if it's a feature


