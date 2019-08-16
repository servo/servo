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
review system, that review may be carried forward. For other submissions, we
recommend using GitHub's built-in review tools.

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

[GitHub.com allows reviewers to formally signal their approval of a pull
request through a dedicated user
interface.](https://help.github.com/en/articles/about-pull-request-reviews)
Every pull request submitted to WPT must be approved by at least one project
collaborator before it can be merged.

## Notifications

META.yml files are used only to indicate who should be notified of pull
requests.  If you are interested in receiving notifications of proposed
changes to tests in a given directory, feel free to add yourself to the
META.yml file.

## Finding contributions to review

Here are a few search filters to find things to review:

* [Open PRs (excluding vendor exports)](https://github.com/web-platform-tests/wpt/pulls?utf8=%E2%9C%93&q=is%3Apr+is%3Aopen+-label%3A%22mozilla%3Agecko-sync%22+-label%3A%22chromium-export%22+-label%3A%22webkit-export%22+-label%3A%22servo-export%22+-label%3Avendor-imports)
* [Reviewed but still open PRs (excluding vendor exports)](https://github.com/web-platform-tests/wpt/pulls?utf8=%E2%9C%93&q=is%3Apr+is%3Aopen+-label%3Amozilla%3Agecko-sync+-label%3Achromium-export+-label%3Awebkit-export+-label%3Aservo-export+-label%3Avendor-imports+review%3Aapproved+-label%3A%22do+not+merge+yet%22+-label%3A%22status%3Aneeds-spec-decision%22) (Merge? Something left to fix? Ping other reviewer?)
* [Open PRs without reviewers](https://github.com/web-platform-tests/wpt/pulls?q=is%3Apr+is%3Aopen+label%3Astatus%3Aneeds-reviewers)
* [Open PRs with label `infra` (excluding vendor exports)](https://github.com/web-platform-tests/wpt/pulls?utf8=%E2%9C%93&q=is%3Apr+is%3Aopen+label%3Ainfra+-label%3A%22mozilla%3Agecko-sync%22+-label%3A%22chromium-export%22+-label%3A%22webkit-export%22+-label%3A%22servo-export%22+-label%3Avendor-imports)
* [Open PRs with label `docs` (excluding vendor exports)](https://github.com/web-platform-tests/wpt/pulls?utf8=%E2%9C%93&q=is%3Apr+is%3Aopen+label%3Adocs+-label%3A%22mozilla%3Agecko-sync%22+-label%3A%22chromium-export%22+-label%3A%22webkit-export%22+-label%3A%22servo-export%22+-label%3Avendor-imports)
