.. _backwards-compatibility:

Backwards Compatibility Policy
==============================

Keeping backwards compatibility has a very high priority in the pytest project. Although we have deprecated functionality over the years, most of it is still supported. All deprecations in pytest were done because simpler or more efficient ways of accomplishing the same tasks have emerged, making the old way of doing things unnecessary.

With the pytest 3.0 release we introduced a clear communication scheme for when we will actually remove the old busted joint and politely ask you to use the new hotness instead, while giving you enough time to adjust your tests or raise concerns if there are valid reasons to keep deprecated functionality around.

To communicate changes we issue deprecation warnings using a custom warning hierarchy (see :ref:`internal-warnings`). These warnings may be suppressed using the standard means: ``-W`` command-line flag or ``filterwarnings`` ini options (see :ref:`warnings`), but we suggest to use these sparingly and temporarily, and heed the warnings when possible.

We will only start the removal of deprecated functionality in major releases (e.g. if we deprecate something in 3.0 we will start to remove it in 4.0), and keep it around for at least two minor releases (e.g. if we deprecate something in 3.9 and 4.0 is the next release, we start to remove it in 5.0, not in 4.0).

When the deprecation expires (e.g. 4.0 is released), we won't remove the deprecated functionality immediately, but will use the standard warning filters to turn them into **errors** by default. This approach makes it explicit that removal is imminent, and still gives you time to turn the deprecated feature into a warning instead of an error so it can be dealt with in your own time. In the next minor release (e.g. 4.1), the feature will be effectively removed.


Deprecation Roadmap
-------------------

Features currently deprecated and removed in previous releases can be found in :ref:`deprecations`.

We track future deprecation and removal of features using milestones and the `deprecation <https://github.com/pytest-dev/pytest/issues?q=label%3A%22type%3A+deprecation%22>`_ and `removal <https://github.com/pytest-dev/pytest/labels/type%3A%20removal>`_ labels on GitHub.
