# Submitting Tests

Test submission is via the typical [GitHub workflow][github flow]:

* Fork the [GitHub repository][repo] (and make sure you're still relatively in
sync with it if you forked a while ago)

* Create a branch for your changes. Being a key of effective Git flow, it is
strongly recommended that the **topic branch** tradition be followed here,
i.e. the branch naming convention is based on the "topic" you will be working
on, e.g. `git checkout -b topic-name`

* Make your changes

* Run the `lint` script in the root of your checkout to detect common
  mistakes in test submissions. This will also be run after submission
  and any errors will prevent your PR being accepted. If it detects an
  error that forms an essential part of your test, edit the list of
  exceptions stored in `tools/lint/lint.whitelist`.

* Commit your changes.

* Push your local branch to your GitHub repository.

* Using the GitHub UI create a Pull Request for your branch.

* When you get review comments, make more commits to your branch to
  address the comments.

* Once everything is reviewed and all issues are addressed, your pull
  request will be automatically merged.

For detailed guidelines on setup and each of these steps, please refer to the
[Github Test Submission](../appendix/github-intro) documentation.

Hop on to the [mailing list][public-test-infra] or [IRC][]
([webclient][web irc], join channel `#testing`) if you have an issue.  There is
no need to announce your review request, as soon as you make a Pull Request
GitHub will inform interested parties.

## Previews

The website [wpt-submissions.live](http://wpt-submissions.live) exists to help
contributors demonstrate their proposed changes to others. If your pull request
is open and has the GitHub label `pull-request-has-preview`, then it will be
available at `http://wpt-submissions.live/{{pull request ID}}`, where "pull
request ID" is the numeric identifier for the pull request.

For example, a pull request at https://github.com/web-platform-tests/wpt/pull/3
has a pull request ID `3`. Once that has been assigned the
`pull-request-has-preview` label, then its contents can be viewed at
http://wpt-submissions.live/3.

If you are [a GitHub
collaborator](https://help.github.com/en/articles/permission-levels-for-a-user-account-repository)
on WPT, the label and the preview will be created automatically. Because the
WPT server will execute Python code in the mirrored submissions, previews are
not created automatically for non-collaborators. Collaborators are encouraged
to enable the preview by adding the label, provided they trust the authors not
to submit malicious code.

[repo]: https://github.com/web-platform-tests/wpt/
[github flow]: https://guides.github.com/introduction/flow/
[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[IRC]: irc://irc.w3.org:6667/testing
[web irc]: http://irc.w3.org
