# Contributing to Servo

Servo welcomes contribution from everyone. Here are the guidelines if you are
thinking of helping us:


## Contributions

Contributions to Servo or its dependencies should be made in the form of GitHub
pull requests. Each pull request will be reviewed by a core contributor
(someone with permission to land patches) and either landed in the main tree or
given feedback for changes that would be required. All contributions should
follow this format, even those from core contributors.

Should you wish to work on an issue, please claim it first by commenting on
the GitHub issue that you want to work on it. This is to prevent duplicated
efforts from contributors on the same issue.

Head over to [Servo Starters](http://servo.github.io/servo-starters/) to find
good tasks to start with. If you come across words or jargon that do not make
sense, please check [the glossary](docs/glossary.md) first. If there's no
matching entry, please make a pull request to add one with the content `TODO`
so we can correct that!

See [`HACKING_QUICKSTART.md`](docs/HACKING_QUICKSTART.md) for more information
on how to start working on Servo.

## Pull Request Checklist

- Branch from the master branch and, if needed, rebase to the current master
  branch before submitting your pull request. If it doesn't merge cleanly with
  master you may be asked to rebase your changes.

- Commits should be as small as possible, while ensuring that each commit is
  correct independently (i.e., each commit should compile and pass tests). 

- If your patch is not getting reviewed or you need a specific person to review
  it, you can @-reply a reviewer asking for a review in the pull request or a
  comment, or you can ask for a review in `#servo` on `irc.mozilla.org`.

- Add tests relevant to the fixed bug or new feature.  For a DOM change this
  will usually be a web platform test; for layout, a reftest.  See our [testing
  guide](https://github.com/servo/servo/wiki/Testing) for more information.

For specific git instructions, see [GitHub workflow 101](https://github.com/servo/servo/wiki/Github-workflow).

## Conduct

In all Servo-related forums, we follow the [Rust Code of Conduct](http://www.rust-lang.org/conduct.html). For escalation or moderation issues, please contact Jack (jmoffitt@mozilla.com) and Lars (larsberg@mozilla.com) instead of the Rust moderation team.


## Communication

Servo contributors frequent the `#servo` channel on [`irc.mozilla.org`](https://wiki.mozilla.org/IRC).

You can also join the [`dev-servo` mailing list](https://lists.mozilla.org/listinfo/dev-servo).
