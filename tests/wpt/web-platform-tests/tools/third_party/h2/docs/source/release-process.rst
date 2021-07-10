Release Process
===============

Because of Hyper-h2's place at the bottom of the dependency tree, it is
extremely important that the project maintains a diligent release schedule.
This document outlines our process for managing releases.

Versioning
----------

Hyper-h2 follows `semantic versioning`_ of its public API when it comes to
numbering releases. The public API of Hyper-h2 is strictly limited to the
entities listed in the :doc:`api` documentation: anything not mentioned in that
document is not considered part of the public API and is not covered by the
versioning guarantees given by semantic versioning.

Maintenance
-----------

Hyper-h2 has the notion of a "release series", given by a major and minor
version number: for example, there is the 2.1 release series. When each minor
release is made and a release series is born, a branch is made off the release
tag: for example, for the 2.1 release series, the 2.1.X branch.

All changes merged into the master branch will be evaluated for whether they
can be considered 'bugfixes' only (that is, they do not affect the public API).
If they can, they will also be cherry-picked back to all active maintenance
branches that require the bugfix. If the bugfix is not necessary, because the
branch in question is unaffected by that bug, the bugfix will not be
backported.

Supported Release Series'
-------------------------

The developers of Hyper-h2 commit to supporting the following release series:

- The most recent, as identified by the first two numbers in the highest
  version currently released.
- The immediately prior release series.

The only exception to this policy is that no release series earlier than the
2.1 series will be supported. In this context, "supported" means that they will
continue to receive bugfix releases.

For releases other than the ones identified above, no support is guaranteed.
The developers may *choose* to support such a release series, but they do not
promise to.

The exception here is for security vulnerabilities. If a security vulnerability
is identified in an out-of-support release series, the developers will do their
best to patch it and issue an emergency release. For more information, see
`our security documentation`_.


.. _semantic versioning: http://semver.org/
.. _our security documentation: http://python-hyper.org/en/latest/security.html
