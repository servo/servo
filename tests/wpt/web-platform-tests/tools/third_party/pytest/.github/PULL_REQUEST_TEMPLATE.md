Thanks for submitting a PR, your contribution is really appreciated!

Here's a quick checklist that should be present in PRs:

- [ ] Add a new news fragment into the changelog folder
  * name it `$issue_id.$type` for example (588.bug)
  * if you don't have an issue_id change it to the pr id after creating the pr
  * ensure type is one of `removal`, `feature`, `bugfix`, `vendor`, `doc` or `trivial`
  * Make sure to use full sentences with correct case and punctuation, for example: "Fix issue with non-ascii contents in doctest text files."
- [ ] Target: for `bugfix`, `vendor`, `doc` or `trivial` fixes, target `master`; for removals or features target `features`;
- [ ] Make sure to include reasonable tests for your change if necessary

Unless your change is a trivial or a documentation fix (e.g.,  a typo or reword of a small section) please:

- [ ] Add yourself to `AUTHORS`, in alphabetical order;
