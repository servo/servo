Release Procedure
-----------------

Our current policy for releasing is to aim for a bug-fix release every few weeks and a minor release every 2-3 months. The idea
is to get fixes and new features out instead of trying to cram a ton of features into a release and by consequence
taking a lot of time to make a new one.

The git commands assume the following remotes are setup:

* ``origin``: your own fork of the repository.
* ``upstream``: the ``pytest-dev/pytest`` official repository.

Preparing: Automatic Method
~~~~~~~~~~~~~~~~~~~~~~~~~~~

We have developed an automated workflow for releases, that uses GitHub workflows and is triggered
by `manually running <https://docs.github.com/en/actions/managing-workflow-runs/manually-running-a-workflow>`__
the `prepare-release-pr workflow <https://github.com/pytest-dev/pytest/actions/workflows/prepare-release-pr.yml>`__
on GitHub Actions.

The automation will decide the new version number based on the following criteria:

- If the "major release" input is set to "yes", release a new major release
  (e.g. 7.0.0 -> 8.0.0)
- If there are any ``.feature.rst`` or ``.breaking.rst`` files in the
  ``changelog`` directory, release a new minor release (e.g. 7.0.0 -> 7.1.0)
- Otherwise, release a bugfix release (e.g. 7.0.0 -> 7.0.1)
- If the "prerelease" input is set, append the string to the version number
  (e.g. 7.0.0 -> 8.0.0rc1), if "major" is set, and "prerelease" is set to `rc1`)

Bug-fix and minor releases
^^^^^^^^^^^^^^^^^^^^^^^^^^

Bug-fix and minor releases are always done from a maintenance branch. First,
consider double-checking the ``changelog`` directory to see if there are any
breaking changes or new features.

For a new minor release, first create a new maintenance branch from ``main``::

     git fetch --all
     git branch 7.1.x upstream/main
     git push upstream 7.1.x

Then, trigger the workflow with the following inputs:

- branch: **7.1.x**
- major release: **no**
- prerelease: empty

Or via the commandline using `GitHub's cli <https://github.com/cli/cli>`__::

    gh workflow run prepare-release-pr.yml -f branch=7.1.x -f major=no -f prerelease=

Where ``7.1.x`` is the maintenance branch for the ``7.1`` series. The automated
workflow will publish a PR for a branch ``release-7.1.0``.

Similarly, for a bug-fix release, use the existing maintenance branch and
trigger the workflow with e.g. ``branch: 7.0.x`` to get a new ``release-7.0.1``
PR.

Major releases
^^^^^^^^^^^^^^

1. Create a new maintenance branch from ``main``::

        git fetch --all
        git branch 8.0.x upstream/main
        git push upstream 8.0.x

2. Trigger the workflow with the following inputs:

   - branch: **8.0.x**
   - major release: **yes**
   - prerelease: empty

Or via the commandline::

    gh workflow run prepare-release-pr.yml -f branch=8.0.x -f major=yes -f prerelease=

The automated workflow will publish a PR for a branch ``release-8.0.0``.

At this point on, this follows the same workflow as other maintenance branches: bug-fixes are merged
into ``main`` and ported back to the maintenance branch, even for release candidates.

Release candidates
^^^^^^^^^^^^^^^^^^

To release a release candidate, set the "prerelease" input to the version number
suffix to use. To release a ``8.0.0rc1``, proceed like under "major releases", but set:

- branch: 8.0.x
- major release: yes
- prerelease: **rc1**

Or via the commandline::

    gh workflow run prepare-release-pr.yml -f branch=8.0.x -f major=yes -f prerelease=rc1

The automated workflow will publish a PR for a branch ``release-8.0.0rc1``.

**A note about release candidates**

During release candidates we can merge small improvements into
the maintenance branch before releasing the final major version, however we must take care
to avoid introducing big changes at this stage.

Preparing: Manual Method
~~~~~~~~~~~~~~~~~~~~~~~~

**Important**: pytest releases must be prepared on **Linux** because the docs and examples expect
to be executed on that platform.

To release a version ``MAJOR.MINOR.PATCH``, follow these steps:

#. For major and minor releases, create a new branch ``MAJOR.MINOR.x`` from
   ``upstream/main`` and push it to ``upstream``.

#. Create a branch ``release-MAJOR.MINOR.PATCH`` from the ``MAJOR.MINOR.x`` branch.

   Ensure your are updated and in a clean working tree.

#. Using ``tox``, generate docs, changelog, announcements::

    $ tox -e release -- MAJOR.MINOR.PATCH

   This will generate a commit with all the changes ready for pushing.

#. Open a PR for the ``release-MAJOR.MINOR.PATCH`` branch targeting ``MAJOR.MINOR.x``.


Releasing
~~~~~~~~~

Both automatic and manual processes described above follow the same steps from this point onward.

#. After all tests pass and the PR has been approved, tag the release commit
   in the ``release-MAJOR.MINOR.PATCH`` branch and push it. This will publish to PyPI::

     git fetch --all
     git tag MAJOR.MINOR.PATCH upstream/release-MAJOR.MINOR.PATCH
     git push git@github.com:pytest-dev/pytest.git MAJOR.MINOR.PATCH

   Wait for the deploy to complete, then make sure it is `available on PyPI <https://pypi.org/project/pytest>`_.

#. Merge the PR.

#. Cherry-pick the CHANGELOG / announce files to the ``main`` branch::

       git fetch --all --prune
       git checkout upstream/main -b cherry-pick-release
       git cherry-pick -x -m1 upstream/MAJOR.MINOR.x

#. Open a PR for ``cherry-pick-release`` and merge it once CI passes. No need to wait for approvals if there were no conflicts on the previous step.

#. For major and minor releases, tag the release cherry-pick merge commit in main with
   a dev tag for the next feature release::

       git checkout main
       git pull
       git tag MAJOR.{MINOR+1}.0.dev0
       git push git@github.com:pytest-dev/pytest.git MAJOR.{MINOR+1}.0.dev0

#. Send an email announcement with the contents from::

     doc/en/announce/release-<VERSION>.rst

   To the following mailing lists:

   * pytest-dev@python.org (all releases)
   * python-announce-list@python.org (all releases)
   * testing-in-python@lists.idyll.org (only major/minor releases)

   And announce it on `Twitter <https://twitter.com/>`_ with the ``#pytest`` hashtag.
