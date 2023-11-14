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

When you push to a pull request, GitHub automatically checks that your changes have no compilation, lint, or tidy errors.

To run unit tests or Web Platform Tests against a pull request, add one or more of the labels below to your pull request. If you do not have permission to add labels to your pull request, add a comment on your bug requesting that they be added.

| Label | Effect |
|---|---|
| `T-full` | Unit tests: Linux, macOS, Windows<br>Layout tests: Linux, macOS<br>Legacy layout tests: Linux, macOS |
| `T-linux-wpt-2013` | Unit tests: Linux<br>Legacy layout tests: Linux |
| `T-linux-wpt-2020` | Unit tests: Linux<br>Layout tests: Linux |
| `T-macos` | Unit tests: macOS |
| `T-windows` | Unit tests: Windows |

## Conduct

Servo Code of Conduct is published at <https://servo.org/coc/>.

## Communication

Servo contributors frequent the [Servo Zulip chat](https://servo.zulipchat.com/).

## Technical Steering Committee

Technical oversight of the Servo Project is provided by the
[Technical Steering Committee](https://github.com/servo/project/blob/master/governance/tsc/README.md).

