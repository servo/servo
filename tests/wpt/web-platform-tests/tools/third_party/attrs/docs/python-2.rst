Python 2 Statement
==================

While ``attrs`` has always been a Python 3-first package, we the maintainers are aware that Python 2 has not magically disappeared in 2020.
We are also aware that ``attrs`` is an important building block in many people's systems and livelihoods.

As such, we do **not** have any immediate plans to drop Python 2 support in ``attrs``.
We intend to support is as long as it will be technically feasible for us.

Feasibility in this case means:

1. Possibility to run the tests on our development computers,
2. and **free** CI options.

This can mean that we will have to run our tests on PyPy, whose maintainters have unequivocally declared that they do not intend to stop the development and maintenance of their Python 2-compatible line at all.
And this can mean that at some point, a sponsor will have to step up and pay for bespoke CI setups.

**However**: there is no promise of new features coming to ``attrs`` running under Python 2.
It is up to our discretion alone, to decide whether the introduced complexity or awkwardness are worth it, or whether we choose to make a feature available on modern platforms only.


Summary
-------

We will do our best to support existing users, but nobody is entitled to the latest and greatest features on a platform that is officially end of life.
