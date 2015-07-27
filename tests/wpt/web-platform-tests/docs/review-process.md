## Test Review Policy

In order to encourage a high level of quality in the W3C test
suites, test contributions must be reviewed by a peer.

The reviewer can be anyone (other than the original test author) that
has the required experience with both the spec under test and with the
test [format][format] and [style][style] guidelines. Review must
happen in public, but the exact review location is flexible. In
particular if a vendor is submitting tests that have already been
reviewed in their own review system, that review may be carried
forward, as long as the original review is clearly linked in the
GitHub pull request.

To assist with test reviews, a [review checklist][review-checklist]
is available.

## Review Tools

All new code submissions must use the GitHub pull request
workflow. The GitHub UI for code review may be used, but other tools
may also be used as long as the review is clearly linked.

### Critic

[Critic][critic] is a code review tool that is frequently used for
reviewing web-platform-tests submissions. Although it has a steeper
learning curve than the GitHub tools, it has more features that aid in
conducting non-trivial reviews.

If you want to use Critic to review code, visit the [homepage][critic]
and log (authentication is via GitHub). On the homepage, click "Add
Filter". In the resulting dialog, select the web-platform-tests
repository and add the path of the folder(s) where you want to review
code, e.g. `/` to review any submissions or `XMLHttpRequest/` to
review only submissions in the `XHMLHttpRequest` directory. Ensure that
your email address is added so that you receive notifications of new
reviews matching your filters, and activity on existing reviews.

## Labels

Pull requests get automatically labelled in the GitHub repository. Check
out the [list of labels in Github][issues]
to see the open pull requests for a given specification or a given Working Group.

## Status

The
[web-platform-tests dashboard](http://testthewebforward.org/dashboard/#all)
shows the number of open review requests, and can be filtered by testsuite.

[format]: ./test-format-guidelines.html
[style]: ./test-style-guidelines.html
[review-checklist]: ./review-checklist.html
[issues]: https://github.com/w3c/web-platform-tests/issues
[critic]: https://critic.hoppipolla.co.uk
