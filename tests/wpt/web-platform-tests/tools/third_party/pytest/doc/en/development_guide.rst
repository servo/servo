=================
Development Guide
=================

Some general guidelines regarding development in pytest for maintainers and contributors. Nothing here
is set in stone and can't be changed, feel free to suggest improvements or changes in the workflow.


Code Style
----------

* `PEP-8 <https://www.python.org/dev/peps/pep-0008>`_
* `flake8 <https://pypi.org/project/flake8/>`_ for quality checks
* `invoke <http://www.pyinvoke.org/>`_ to automate development tasks


Branches
--------

We have two long term branches:

* ``master``: contains the code for the next bugfix release.
* ``features``: contains the code with new features for the next minor release.

The official repository usually does not contain topic branches, developers and contributors should create topic
branches in their own forks.

Exceptions can be made for cases where more than one contributor is working on the same
topic or where it makes sense to use some automatic capability of the main repository, such as automatic docs from
`readthedocs <readthedocs.org>`_ for a branch dealing with documentation refactoring.

Issues
------

Any question, feature, bug or proposal is welcome as an issue. Users are encouraged to use them whenever they need.

GitHub issues should use labels to categorize them. Labels should be created sporadically, to fill a niche; we should
avoid creating labels just for the sake of creating them.

Each label should include a description in the GitHub's interface stating its purpose.

Temporary labels
~~~~~~~~~~~~~~~~

To classify issues for a special event it is encouraged to create a temporary label. This helps those involved to find
the relevant issues to work on. Examples of that are sprints in Python events or global hacking events.

* ``temporary: EP2017 sprint``: candidate issues or PRs tackled during the EuroPython 2017

Issues created at those events should have other relevant labels added as well.

Those labels should be removed after they are no longer relevant.


.. include:: ../../HOWTORELEASE.rst
