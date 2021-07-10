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

## Pull Request Checklist

- Branch from the master branch and, if needed, rebase to the current master
  branch before submitting your pull request. If it doesn't merge cleanly with
  master you may be asked to rebase your changes.

- Commits should be as small as possible, while ensuring that each commit is
  correct independently (i.e., each commit should compile and pass tests). 

- Commits should be accompanied by a Developer Certificate of Origin
  (http://developercertificate.org) sign-off, which indicates that you (and
  your employer if applicable) agree to be bound by the terms of the
  [project license](LICENSE.md). In git, this is the `-s` option to `git commit`

- If your patch is not getting reviewed or you need a specific person to review
  it, you can @-reply a reviewer asking for a review in the pull request or a
  comment, or you can ask for a review in [the Servo chat](https://servo.zulipchat.com/).

- Add tests relevant to the fixed bug or new feature.  For a DOM change this
  will usually be a web platform test; for layout, a reftest.  See our [testing
  guide](https://github.com/servo/servo/wiki/Testing) for more information.

For specific git instructions, see [GitHub workflow 101](https://github.com/servo/servo/wiki/Github-workflow).

## Conduct

In all Servo-related forums, we follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). For escalation or moderation issues, please contact a member of the Servo Technical Steering Committee instead of the Rust moderation team.


## Communication

Servo contributors frequent the [Servo Zulip chat](https://servo.zulipchat.com/).

## Technical Steering Committee

Technical oversight of the Servo Project is provided by the Technical Steering Committee,
comprised of:

- [Alan Jeffrey](https://github.com/asajeffrey)
- [Anthony Ramine](https://github.com/nox)
- [Connor Brewster](https://github.com/cbrewster)
- [Cheng-You Bai](https://github.com/cybai)
- [Diane Hosfelt](https://github.com/avadacatavra)
- [Dzmitry Malyshau](https://github.com/kvark)
- [Emilio Cobos Álvarez](https://github.com/emilio)
- [Fernando Jiménez Moreno](https://github.com/ferjm)
- [Gregory Terzian](https://github.com/gterzian)
- [Jack Moffitt](https://github.com/metajack)
- [James Graham](https://github.com/jgraham)
- [Josh Matthews](https://github.com/jdm)
- [Keith Yeung](https://github.com/KiChjang)
- [Lars Bergstrom](https://github.com/larsbergstrom)
- [Manish Goregaokar](https://github.com/Manishearth)
- [Martin Robinson](https://github.com/mrobinson)
- [Patrick Walton](https://github.com/pcwalton)
- [Paul Rouget](https://github.com/paulrouget)
- [Simon Sapin](https://github.com/SimonSapin)
