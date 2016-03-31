How to release pytest
--------------------------------------------

Note: this assumes you have already registered on pypi.

0. create the branch release-VERSION
   use features as base for minor/major releases
   and master as base for bugfix releases

1. Bump version numbers in _pytest/__init__.py (setup.py reads it)

2. Check and finalize CHANGELOG

3. Write doc/en/announce/release-VERSION.txt and include
   it in doc/en/announce/index.txt::

        git log 2.8.2..HEAD --format='%aN' | sort -u # lists the names of authors involved

4. Use devpi for uploading a release tarball to a staging area::

     devpi use https://devpi.net/USER/dev
     devpi upload --formats sdist,bdist_wheel

5. Run from multiple machines::

     devpi use https://devpi.net/USER/dev
     devpi test pytest==VERSION

6. Check that tests pass for relevant combinations with::

       devpi list pytest

   or look at failures with "devpi list -f pytest".

7. Regenerate the docs examples using tox, and check for regressions::

      tox -e regen
      git diff


8. Build the docs, you need a virtualenv with py and sphinx
   installed::

      cd doc/en      
      make html

   Commit any changes before tagging the release.

9. Tag the release::

      git tag VERSION
      git push

10. Upload the docs using doc/en/Makefile::

      cd doc/en
      make install  # or "installall" if you have LaTeX installed for PDF

    This requires ssh-login permission on pytest.org because it uses
    rsync.
    Note that the ``install`` target of ``doc/en/Makefile`` defines where the
    rsync goes to, typically to the "latest" section of pytest.org.

    If you are making a minor release (e.g. 5.4), you also need to manually
    create a symlink for "latest"::

       ssh pytest-dev@pytest.org
       ln -s 5.4 latest

    Browse to pytest.org to verify.

11. Publish to pypi::

      devpi push pytest-VERSION pypi:NAME

    where NAME is the name of pypi.python.org as configured in your ``~/.pypirc``
    file `for devpi <http://doc.devpi.net/latest/quickstart-releaseprocess.html?highlight=pypirc#devpi-push-releasing-to-an-external-index>`_.


12. Send release announcement to mailing lists:

    - pytest-dev
    - testing-in-python
    - python-announce-list@python.org


13. **after the release** Bump the version number in ``_pytest/__init__.py``,
    to the next Minor release version (i.e. if you released ``pytest-2.8.0``,
    set it to ``pytest-2.9.0.dev1``).

14. merge the actual release into the master branch and do a pull request against it
15. merge from master to features
