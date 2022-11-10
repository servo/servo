Release Procedure
-----------------

#. Create a branch ``release-X.Y.Z`` from the latest ``master``.

#. Manually update the ``CHANGELOG.rst`` and commit.

#. Open a PR for this branch targeting ``master``.

#. After all tests pass and the PR has been approved by at least another maintainer, publish to PyPI by creating and pushing a tag::

     git tag X.Y.Z
     git push git@github.com:pytest-dev/py X.Y.Z

   Wait for the deploy to complete, then make sure it is `available on PyPI <https://pypi.org/project/py>`_.

#. Merge your PR to ``master``.
