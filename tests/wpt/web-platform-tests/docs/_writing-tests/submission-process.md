---
layout: page
title: Submitting Tests
order: 16
---
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
[Github Test Submission][github-intro] documentation.

Hop on to the [mailing list][public-test-infra] or [IRC][]
([webclient][web irc], join channel `#testing`) if you have an issue.  There is
no need to announce your review request, as soon as you make a Pull Request
GitHub will inform interested parties.

[repo]: https://github.com/web-platform-tests/wpt/
[github-intro]: {{ site.baseurl }}{% link _appendix/github-intro.md %}
[github flow]: https://guides.github.com/introduction/flow/
[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[IRC]: irc://irc.w3.org:6667/testing
[web irc]: http://irc.w3.org
