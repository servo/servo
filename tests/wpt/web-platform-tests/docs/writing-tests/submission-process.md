# Submitting Tests

Test submission is via the typical [GitHub workflow][github flow]. For detailed
guidelines on setup and each of these steps, please refer to the [Github Test
Submission](github-intro) documentation.

* Fork the [GitHub repository][repo].

* Create a feature branch for your changes.

* Make your changes.

* Run the `lint` script in the root of your checkout to detect common
  mistakes in test submissions. There is [detailed documentation for the lint
  tool](lint-tool).

* Commit your changes.

* Push your local branch to your GitHub repository.

* Using the GitHub UI, create a Pull Request for your branch.

* When you get review comments, make more commits to your branch to
  address the comments.

* Once everything is reviewed and all issues are addressed, your pull
  request will be automatically merged.

We can sometimes take a little while to go through pull requests because we
have to go through all the tests and ensure that they match the specification
correctly. But we look at all of them, and take everything that we can.

Hop on to the [mailing list][public-test-infra] or [matrix
channel][matrix] if you have an issue.  There is no need to announce
your review request; as soon as you make a Pull Request, GitHub will
inform interested parties.

[repo]: https://github.com/web-platform-tests/wpt/
[github flow]: https://guides.github.com/introduction/flow/
[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[matrix]: https://app.element.io/#/room/#wpt:matrix.org
