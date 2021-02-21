:orphan:

.. warnings about this file not being included in any toctree will be suppressed by :orphan:


April 2015 is "adopt pytest month"
=============================================

Are you an enthusiastic pytest user, the local testing guru in your workplace? Or are you considering using pytest for your open source project, but not sure how to get started? Then you may be interested in "adopt pytest month"!

We will pair experienced pytest users with open source projects, for a month's effort of getting new development teams started with pytest.

In 2015 we are trying this for the first time. In February and March 2015 we will gather volunteers on both sides, in April we will do the work, and in May we will evaluate how it went. This effort is being coordinated by Brianna Laugher. If you have any questions or comments, you can raise them on the `@pytestdotorg twitter account <https://twitter.com/pytestdotorg>`_ the `issue tracker`_ or the `pytest-dev mailing list`_.


.. _`issue tracker`: https://github.com/pytest-dev/pytest/issues/676
.. _`pytest-dev mailing list`: https://mail.python.org/mailman/listinfo/pytest-dev


The ideal pytest helper
-----------------------------------------

 - will be able to commit 2-4 hours a week to working with their particular project (this might involve joining their mailing list, installing the software and exploring any existing tests, offering advice, writing some example tests)
 - feels confident in using pytest (e.g. has explored command line options, knows how to write parametrized tests, has an idea about conftest contents)
 - does not need to be an expert in every aspect!

Pytest helpers, sign up here! (preferably in February, hard deadline 22 March)



The ideal partner project
-----------------------------------------

 - is open source, and predominantly written in Python
 - has an automated/documented install process for developers
 - has more than one core developer
 - has at least one official release (e.g. is available on pypi)
 - has the support of the core development team, in trying out pytest adoption
 - has no tests... or 100% test coverage... or somewhere in between!

Partner projects, sign up here! (by 22 March)



What does it mean to "adopt pytest"?
-----------------------------------------

There can be many different definitions of "success". Pytest can run many nose_ and unittest_ tests by default, so using pytest as your testrunner may be possible from day 1. Job done, right?

Progressive success might look like:

 - tests can be run (by pytest) without errors (there may be failures)
 - tests can be run (by pytest) without failures
 - test runner is integrated into CI server
 - existing tests are rewritten to take advantage of pytest features - this can happen in several iterations, for example:
    - changing to native assert_ statements (pycmd_ has a script to help with that, ``pyconvert_unittest.py``)
    - changing `setUp/tearDown methods`_ to fixtures_
    - adding markers_
    - other changes to reduce boilerplate
 - assess needs for future tests to be written, e.g. new fixtures, distributed_ testing tweaks

"Success" should also include that the development team feels comfortable with their knowledge of how to use pytest. In fact this is probably more important than anything else. So spending a lot of time on communication, giving examples, etc will probably be important - both in running the tests, and in writing them.

It may be after the month is up, the partner project decides that pytest is not right for it. That's okay - hopefully the pytest team will also learn something about its weaknesses or deficiencies.

.. _nose: nose.html
.. _unittest: unittest.html
.. _assert: assert.html
.. _pycmd: https://bitbucket.org/hpk42/pycmd/overview
.. _`setUp/tearDown methods`: xunit_setup.html
.. _fixtures: fixture.html
.. _markers: mark.html
.. _distributed: xdist.html


Other ways to help
-----------------------------------------

Promote! Do your favourite open source Python projects use pytest? If not, why not tell them about this page?
