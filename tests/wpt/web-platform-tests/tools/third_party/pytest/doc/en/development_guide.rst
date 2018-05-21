=================
Development Guide
=================

Some general guidelines regarding development in pytest for core maintainers and general contributors. Nothing here
is set in stone and can't be changed, feel free to suggest improvements or changes in the workflow.


Code Style
----------

* `PEP-8 <https://www.python.org/dev/peps/pep-0008>`_
* `flake8 <https://pypi.python.org/pypi/flake8>`_ for quality checks
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

Here is a list of labels and a brief description mentioning their intent.


**Type**

* ``type: backward compatibility``: issue that will cause problems with old pytest versions.
* ``type: bug``: problem that needs to be addressed.
* ``type: deprecation``: feature that will be deprecated in the future.
* ``type: docs``: documentation missing or needing clarification.
* ``type: enhancement``: new feature or API change, should be merged into ``features``.
* ``type: feature-branch``: new feature or API change, should be merged into ``features``.
* ``type: infrastructure``: improvement to development/releases/CI structure.
* ``type: performance``: performance or memory problem/improvement.
* ``type: proposal``: proposal for a new feature, often to gather opinions or design the API around the new feature.
* ``type: question``: question regarding usage, installation, internals or how to test something.
* ``type: refactoring``: internal improvements to the code.
* ``type: regression``: indicates a problem that was introduced in a release which was working previously.

**Status**

* ``status: critical``: grave problem or usability issue that affects lots of users.
* ``status: easy``: easy issue that is friendly to new contributors.
* ``status: help wanted``: core developers need help from experts on this topic.
* ``status: needs information``: reporter needs to provide more information; can be closed after 2 or more weeks of inactivity.

**Topic**

* ``topic: collection``
* ``topic: fixtures``
* ``topic: parametrize``
* ``topic: reporting``
* ``topic: selection``
* ``topic: tracebacks``

**Plugin (internal or external)**

* ``plugin: cache``
* ``plugin: capture``
* ``plugin: doctests``
* ``plugin: junitxml``
* ``plugin: monkeypatch``
* ``plugin: nose``
* ``plugin: pastebin``
* ``plugin: pytester``
* ``plugin: tmpdir``
* ``plugin: unittest``
* ``plugin: warnings``
* ``plugin: xdist``


**OS**

Issues specific to a single operating system. Do not use as a means to indicate where an issue originated from, only
for problems that happen **only** in that system.

* ``os: linux``
* ``os: mac``
* ``os: windows``

**Temporary**

Used to classify issues for limited time, to help find issues related in events for example.
They should be removed after they are no longer relevant.

* ``temporary: EP2017 sprint``:
* ``temporary: sprint-candidate``:


.. include:: ../../HOWTORELEASE.rst
