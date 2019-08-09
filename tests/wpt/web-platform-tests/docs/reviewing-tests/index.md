# Reviewing Tests

In order to encourage a high level of quality in the W3C test
suites, test contributions must be reviewed by a peer.

```eval_rst
.. toctree::
   :maxdepth: 1

   checklist
   email
   git
   reverting
```

## Test Review Policy

The reviewer can be anyone (other than the original test author) that
has the required experience with both the spec under test and with
the [general test guidelines](../writing-tests/general-guidelines).

The review must happen in public, but there is no requirement for it
to happen in any specific location. In particular if a vendor is
submitting tests that have already been publicly reviewed in their own
review system, that review may be carried forward. For other tests, we
strongly recommend using either Reviewable or GitHub's built-in review
tools.

Regardless of what review tool is used, the review must be clearly
linked in the pull request.

In general, we tend on the side of merging things with nits (i.e.,
anything sub-optimal that isn't absolutely required to be right) and
then opening issues to leaving pull requests open indefinitely waiting
on the original submitter to fix them; when tests are being upstreamed
from vendors it is frequently the case that the author has moved on to
working on other things as tests frequently only get pushed upstream
once the code lands in their implementation.

To assist with test reviews, a [review checklist](checklist) is available.
