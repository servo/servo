Submitting patches
==================

* Always make a new branch for your work.
* Patches should be small to facilitate easier review. `Studies have shown`_
  that review quality falls off as patch size grows. Sometimes this will result
  in many small PRs to land a single large feature.
* Larger changes should be discussed in a ticket before submission.
* New features and significant bug fixes should be documented in the
  :doc:`/changelog`.
* You must have legal permission to distribute any code you contribute and it
  must be available under both the BSD and Apache Software License Version 2.0
  licenses.

If you believe you've identified a security issue in packaging, please
follow the directions on the :doc:`security page </security>`.

Code
----

This project's source is auto-formatted with |black|. You can check if your
code meets our requirements by running our linters against it with ``nox -s
lint`` or ``pre-commit run --all-files``.

`Write comments as complete sentences.`_

Every code file must start with the boilerplate licensing notice:

.. code-block:: python

    # This file is dual licensed under the terms of the Apache License, Version
    # 2.0, and the BSD License. See the LICENSE file in the root of this repository
    # for complete details.

Tests
-----

All code changes must be accompanied by unit tests with 100% code coverage (as
measured by the combined metrics across our build matrix).


Documentation
-------------

All features should be documented with prose in the ``docs`` section.

When referring to a hypothetical individual (such as "a person receiving an
encrypted message") use gender neutral pronouns (they/them/their).

Docstrings are typically only used when writing abstract classes, but should
be written like this if required:

.. code-block:: python

    def some_function(some_arg):
        """
        Does some things.

        :param some_arg: Some argument.
        """

So, specifically:

* Always use three double quotes.
* Put the three double quotes on their own line.
* No blank line at the end.
* Use Sphinx parameter/attribute documentation `syntax`_.


.. |black| replace:: ``black``
.. _black: https://pypi.org/project/black/
.. _`Write comments as complete sentences.`: https://nedbatchelder.com/blog/201401/comments_should_be_sentences.html
.. _`syntax`: http://sphinx-doc.org/domains.html#info-field-lists
.. _`Studies have shown`: http://www.ibm.com/developerworks/rational/library/11-proven-practices-for-peer-review/
