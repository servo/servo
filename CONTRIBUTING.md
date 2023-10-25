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

Head over to [Servo Starters](https://starters.servo.org/) to find
good tasks to start with. If you come across words or jargon that do not make
sense, please check [the glossary](docs/glossary.md) first. If there's no
matching entry, please make a pull request to add one with the content `TODO`
so we can correct that!

See [`HACKING_QUICKSTART.md`](docs/HACKING_QUICKSTART.md) for more information
on how to start working on Servo.

## Pull request checklist

- Branch from the master branch and, if needed, rebase to the current master
  branch before submitting your pull request. If it doesn't merge cleanly with
  master you may be asked to rebase your changes.

- Commits should be as small as possible, while ensuring that each commit is
  correct independently (i.e., each commit should compile and pass tests). 

- Commits should be accompanied by a Developer Certificate of Origin
  (http://developercertificate.org) sign-off, which indicates that you (and
  your employer if applicable) agree to be bound by the terms of the
  [project license](LICENSE). In git, this is the `-s` option to `git commit`.

- If your patch is not getting reviewed or you need a specific person to review
  it, you can @-reply a reviewer asking for a review in the pull request or a
  comment, or you can ask for a review in [the Servo chat](https://servo.zulipchat.com/).

- Add tests relevant to the fixed bug or new feature.  For a DOM change this
  will usually be a web platform test; for layout, a reftest.  See our [testing
  guide](https://github.com/servo/servo/wiki/Testing) for more information.

For specific git instructions, see [GitHub workflow 101](https://github.com/servo/servo/wiki/Github-workflow).

## Running tests in pull requests

When you push to a pull request, GitHub automatically checks that your changes have no compile, lint, or tidy errors.

To run unit tests or Web Platform Tests against a pull request, you can mention [@bors-servo](https://github.com/bors-servo) in a comment, or add one or more labels to your pull request:

| comment | label |
|---|---|
| `@bors-servo try`<br>`@bors-servo try=full` | `T-full` |
| `@bors-servo try=wpt-2013`<br>`@bors-servo try=wpt`<sup>1</sup> | `T-linux-wpt-2013` |
| `@bors-servo try=wpt-2020` | `T-linux-wpt-2020` |
| `@bors-servo try=linux` | `T-linux-wpt-2020`<sup>2</sup> |
| `@bors-servo try=macos` | `T-macos` |
| `@bors-servo try=windows` | `T-windows` |

1. this will become equivalent to `try=wpt-2020` at some point
2. unlike `try=linux`, this runs WPT tests too, not just unit tests

## Conduct

Servo Code of Conduct is published at <https://servo.org/coc/>.

## Communication

Servo contributors frequent the [Servo Zulip chat](https://servo.zulipchat.com/).

## Technical Steering Committee

Technical oversight of the Servo Project is provided by the
[Technical Steering Committee](https://github.com/servo/project/blob/master/governance/tsc/README.md).

