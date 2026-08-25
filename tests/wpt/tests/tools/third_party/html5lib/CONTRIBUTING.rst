Contributing
============

Pull requests are more than welcome — both to the library and to the
documentation. Some useful information:

- We aim to follow PEP 8 in the library, but ignoring the
  79-character-per-line limit, instead following a soft limit of 99,
  but allowing lines over this where it is the readable thing to do.

- We aim to follow PEP 257 for all docstrings, and make them properly
  parseable by Sphinx while generating API documentation.

- We keep ``pyflakes`` reporting no errors or warnings at all times.

- We keep the master branch passing all tests at all times on all
  supported versions.

`Travis CI <https://travis-ci.org/html5lib/html5lib-python/>`_ is run
against all pull requests and should enforce all of the above.

We use `Opera Critic <https://critic.hoppipolla.co.uk/>`_ as an external
code-review tool, which uses your GitHub login to authenticate.  You'll
get email notifications for issues raised in the review.


Patch submission guidelines
---------------------------

- **Create a new Git branch specific to your change.** Do not put
  multiple fixes/features in the same pull request. If you find an
  unrelated bug, create a distinct branch and submit a separate pull
  request for the bugfix. This makes life much easier for maintainers
  and will speed up merging your patches.

- **Write a test** whenever possible. Following existing tests is often
  easiest, and a good way to tell whether the feature you're modifying
  is easily testable.

- **Make sure documentation is updated.** Keep docstrings current, and
  if necessary, update the Sphinx documentation in ``doc/``.

- **Add a changelog entry** at the top of ``CHANGES.rst`` following
  existing entries' styles.

- **Run tests with tox** if possible, to make sure your changes are
  compatible with all supported Python versions.

- **Squash commits** before submitting the pull request so that a single
  commit contains the entire change, and only that change (see the first
  bullet).

- **Don't rebase after creating the pull request.** Merge with upstream,
  if necessary, and use ``git commit --fixup`` for fixing issues raised
  in a Critic review or by a failing Travis build. The reviewer will
  squash and rebase your pull request while accepting it. Even though
  GitHub won't recognize the pull request as accepted, the squashed
  commits will properly specify you as the author.

- **Attribute yourself** in ``AUTHORS.rst``.
