Release Procedure
-----------------

#. From a clean work tree, execute::

    tox -e release -- VERSION

   This will create the branch ready to be pushed.

#. Open a PR targeting ``main``.

#. All tests must pass and the PR must be approved by at least another maintainer.

#. Publish to PyPI by pushing a tag::

     git tag X.Y.Z release-X.Y.Z
     git push git@github.com:pytest-dev/pluggy.git X.Y.Z

   The tag will trigger a new build, which will deploy to PyPI.

#. Make sure it is `available on PyPI <https://pypi.org/project/pluggy>`_.

#. Merge the PR into ``main``, either manually or using GitHub's web interface.
