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

## Previews

The website [http://w3c-test.org](http://w3c-test.org) exists to help
contributors demonstrate their proposed changes to others. If you are [a GitHub
collaborator](https://help.github.com/en/articles/permission-levels-for-a-user-account-repository)
on WPT, then the content of your pull requests will be available at
`http://w3c-test.org/submissions/{{pull request ID}}`, where "pull request ID"
is the numeric identifier for the pull request.

For example, a pull request at https://github.com/web-platform-tests/wpt/pull/3
has a pull request ID `3`. Its contents can be viewed at
http://w3c-test.org/submissions/3.

If you are *not* a GitHub collaborator, then your submission may be made
available if a collaborator makes the following comment on your pull request:
"w3c-test:mirror".

Previews are not created automatically for non-collaborators because the WPT
server will execute Python code in the mirrored submissions. Collaborators are
encouraged to enable the preview by making the special comment only if they
trust the authors not to submit malicious code.

[repo]: https://github.com/web-platform-tests/wpt/
[github flow]: https://guides.github.com/introduction/flow/
[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[matrix]: https://app.element.io/#/room/#wpt:matrix.org
