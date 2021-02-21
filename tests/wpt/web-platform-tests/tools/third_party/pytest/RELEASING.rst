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
by opening an issue.

Bug-fix releases
^^^^^^^^^^^^^^^^

A bug-fix release is always done from a maintenance branch, so for example to release bug-fix
``5.1.2``, open a new issue and add this comment to the body::

    @pytestbot please prepare release from 5.1.x

Where ``5.1.x`` is the maintenance branch for the ``5.1`` series.

The automated workflow will publish a PR for a branch ``release-5.1.2``
and notify it as a comment in the issue.

Minor releases
^^^^^^^^^^^^^^

1. Create a new maintenance branch from ``master``::

        git fetch --all
        git branch 5.2.x upstream/master
        git push upstream 5.2.x

2. Open a new issue and add this comment to the body::

    @pytestbot please prepare release from 5.2.x

The automated workflow will publish a PR for a branch ``release-5.2.0`` and
notify it as a comment in the issue.

Major and release candidates
^^^^^^^^^^^^^^^^^^^^^^^^^^^^

1. Create a new maintenance branch from ``master``::

        git fetch --all
        git branch 6.0.x upstream/master
        git push upstream 6.0.x

2. For a **major release**, open a new issue and add this comment in the body::

        @pytestbot please prepare major release from 6.0.x

   For a **release candidate**, the comment must be (TODO: `#7551 <https://github.com/pytest-dev/pytest/issues/7551>`__)::

        @pytestbot please prepare release candidate from 6.0.x

The automated workflow will publish a PR for a branch ``release-6.0.0`` and
notify it as a comment in the issue.

At this point on, this follows the same workflow as other maintenance branches: bug-fixes are merged
into ``master`` and ported back to the maintenance branch, even for release candidates.

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
   ``upstream/master`` and push it to ``upstream``.

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

#. Cherry-pick the CHANGELOG / announce files to the ``master`` branch::

       git fetch --all --prune
       git checkout origin/master -b cherry-pick-release
       git cherry-pick -x -m1 upstream/MAJOR.MINOR.x

#. Open a PR for ``cherry-pick-release`` and merge it once CI passes. No need to wait for approvals if there were no conflicts on the previous step.

#. Send an email announcement with the contents from::

     doc/en/announce/release-<VERSION>.rst

   To the following mailing lists:

   * pytest-dev@python.org (all releases)
   * python-announce-list@python.org (all releases)
   * testing-in-python@lists.idyll.org (only major/minor releases)

   And announce it on `Twitter <https://twitter.com/>`_ with the ``#pytest`` hashtag.
