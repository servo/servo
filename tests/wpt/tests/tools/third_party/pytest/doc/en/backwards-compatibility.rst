.. _backwards-compatibility:

Backwards Compatibility Policy
==============================

.. versionadded: 6.0

pytest is actively evolving and is a project that has been decades in the making,
we keep learning about new and better structures to express different details about testing.

While we implement those modifications we try to ensure an easy transition and don't want to impose unnecessary churn on our users and community/plugin authors.

As of now, pytest considers multiple types of backward compatibility transitions:

a) trivial: APIs which trivially translate to the new mechanism,
   and do not cause problematic changes.

   We try to support those indefinitely while encouraging users to switch to newer/better mechanisms through documentation.

b) transitional: the old and new API don't conflict
   and we can help users transition by using warnings, while supporting both for a prolonged time.

   We will only start the removal of deprecated functionality in major releases (e.g. if we deprecate something in 3.0 we will start to remove it in 4.0), and keep it around for at least two minor releases (e.g. if we deprecate something in 3.9 and 4.0 is the next release, we start to remove it in 5.0, not in 4.0).

   A deprecated feature scheduled to be removed in major version X will use the warning class `PytestRemovedInXWarning` (a subclass of :class:`~pytest.PytestDeprecationwarning`).

   When the deprecation expires (e.g. 4.0 is released), we won't remove the deprecated functionality immediately, but will use the standard warning filters to turn `PytestRemovedInXWarning` (e.g. `PytestRemovedIn4Warning`) into **errors** by default. This approach makes it explicit that removal is imminent, and still gives you time to turn the deprecated feature into a warning instead of an error so it can be dealt with in your own time. In the next minor release (e.g. 4.1), the feature will be effectively removed.


c) true breakage: should only be considered when normal transition is unreasonably unsustainable and would offset important development/features by years.
   In addition, they should be limited to APIs where the number of actual users is very small (for example only impacting some plugins), and can be coordinated with the community in advance.

   Examples for such upcoming changes:

   * removal of ``pytest_runtest_protocol/nextitem`` - :issue:`895`
   * rearranging of the node tree to include ``FunctionDefinition``
   * rearranging of ``SetupState`` :issue:`895`

   True breakages must be announced first in an issue containing:

   * Detailed description of the change
   * Rationale
   * Expected impact on users and plugin authors (example in :issue:`895`)

   After there's no hard *-1* on the issue it should be followed up by an initial proof-of-concept Pull Request.

   This POC serves as both a coordination point to assess impact and potential inspiration to come up with a transitional solution after all.

   After a reasonable amount of time the PR can be merged to base a new major release.

   For the PR to mature from POC to acceptance, it must contain:
   * Setup of deprecation errors/warnings that help users fix and port their code. If it is possible to introduce a deprecation period under the current series, before the true breakage, it should be introduced in a separate PR and be part of the current release stream.
   * Detailed description of the rationale and examples on how to port code in ``doc/en/deprecations.rst``.


History
=========


Focus primary on smooth transition - stance (pre 6.0)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Keeping backwards compatibility has a very high priority in the pytest project. Although we have deprecated functionality over the years, most of it is still supported. All deprecations in pytest were done because simpler or more efficient ways of accomplishing the same tasks have emerged, making the old way of doing things unnecessary.

With the pytest 3.0 release we introduced a clear communication scheme for when we will actually remove the old busted joint and politely ask you to use the new hotness instead, while giving you enough time to adjust your tests or raise concerns if there are valid reasons to keep deprecated functionality around.

To communicate changes we issue deprecation warnings using a custom warning hierarchy (see :ref:`internal-warnings`). These warnings may be suppressed using the standard means: ``-W`` command-line flag or ``filterwarnings`` ini options (see :ref:`warnings`), but we suggest to use these sparingly and temporarily, and heed the warnings when possible.

We will only start the removal of deprecated functionality in major releases (e.g. if we deprecate something in 3.0 we will start to remove it in 4.0), and keep it around for at least two minor releases (e.g. if we deprecate something in 3.9 and 4.0 is the next release, we start to remove it in 5.0, not in 4.0).

When the deprecation expires (e.g. 4.0 is released), we won't remove the deprecated functionality immediately, but will use the standard warning filters to turn them into **errors** by default. This approach makes it explicit that removal is imminent, and still gives you time to turn the deprecated feature into a warning instead of an error so it can be dealt with in your own time. In the next minor release (e.g. 4.1), the feature will be effectively removed.


Deprecation Roadmap
-------------------

Features currently deprecated and removed in previous releases can be found in :ref:`deprecations`.

We track future deprecation and removal of features using milestones and the `deprecation <https://github.com/pytest-dev/pytest/issues?q=label%3A%22type%3A+deprecation%22>`_ and `removal <https://github.com/pytest-dev/pytest/labels/type%3A%20removal>`_ labels on GitHub.
