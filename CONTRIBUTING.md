# Contributing to Servo

Servo welcomes contribution from everyone. Here are the guidelines if you are
thinking of helping us:


## Contributions

Contributions to Servo or its dependencies should be made in the form of GitHub
pull requests. Each pull request will be reviewed by a core contributor
(someone with permission to land patches) and either landed in the main tree or
given feedback for changes that would be required. All contributions should
follow this format, even those from core contributors.

If you're looking for easy bugs, have a look at the [E-Easy issue tag](https://github.com/mozilla/servo/issues?labels=E-easy&page=1&state=open) on GitHub.

## Pull Request Checklist

- Branch from the master branch and, if needed, rebase to the current master
  branch before submitting your pull request. If it doesn't merge cleanly with
  master you may be asked to rebase your changes.

- Don't put submodule updates in your pull request unless they are to landed
  commits.

- If your patch is not getting reviewed or you need a specific person to review
  it, you can @-reply a reviewer asking for a review in the pull request or a
  comment, or you can ask for a review in `#servo` on `irc.mozilla.org`.

- Add tests relevant to the fixed bug or new feature.  For a DOM change this
  will usually be a content test; for layout, a reftest.  See our [testing
  guide](https://github.com/mozilla/servo/wiki/Testing) for more information.

For specific git instructions, see [GitHub & Critic PR handling 101](https://github.com/mozilla/servo/wiki/Github-&-Critic-PR-handling-101).

## Conduct

We follow the [Rust Code of Conduct](https://github.com/mozilla/rust/wiki/Note-development-policy#wiki-conduct).


## Communication

Servo contributors frequent the `#servo` channel on [`irc.mozilla.org`](https://wiki.mozilla.org/IRC).

You can also join the [`dev-servo` mailing list](https://lists.mozilla.org/listinfo/dev-servo).
