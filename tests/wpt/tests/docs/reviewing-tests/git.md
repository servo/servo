# Working with Pull Requests as a reviewer

In order to do a thorough review,
it is sometimes desirable to have a local copy of the tests one wishes to review.

Reviewing tests also often results in wanting a few things to be changed.
Generally, the reviewer should ask the author to make the desired changes.
However, sometimes the original author does not respond to the requests,
or the changes are so trivial (e.g. fixing a typo)
that bothering the original author seems like a waste of time.

Here is how to do all that.

## Trivial cases

If it is possible to review the tests without a local copy,
but the reviewer still wants to make some simple tweaks to the tests before merging,
it is possible to do so via the Github web UI.

1. Open the pull request. E.g. https://github.com/web-platform-tests/wpt/pull/1234
2. Go to the ![Files changed](../assets/files-changed.png) view (e.g. https://github.com/web-platform-tests/wpt/pull/1234/files)
3. Locate the files you wish to change, and click the ![pencil](../assets/pencil-icon.png) icon in the upper right corner
4. Make the desired change
5. Write a commit message (including a good title) at the bottom
6. Make sure the ![Commit directly to the [name-of-the-PR-branch] branch.](../assets/commit-directly.png) radio button is selected.

   _Note: If the PR predates the introduction of this feature by Github,
   or if the author of the PR has disabled write-access by reviewers to the PR branch,
   this may not be available,
   and your only option would be to commit to a new branch, creating a new PR._
7. Click the ![Commit Changes](../assets/commitbtn.png) button.


## The Normal Way

This is how to import the Pull Request's branch into your existing local
checkout of the repository. If you don't have one, go [fork][fork],
[clone][clone], and [configure][configure] it.

1. Move into your local clone: `cd wherever-you-put-your-repo`
2. Add a remote for the PR author's repo: `git remote add <author-id> git://github.com/<author-id>/<repo-name>.git`
3. Fetch the PR: `git fetch <author-id> <name-of-the-PR-branch>`
4. Checkout that branch: `git checkout <name-of-the-PR-branch>`

   _The relevant `<author-id>`, `<repo-name>`, and `<name-of-the-PR-branch>` can be found by looking for this sentence in on the Github page of the PR:
   ![Add more commits by pushing to the name-of-the-PR-branch branch on author-id/repo-name.](../assets/more-commits.png)_

If all you meant to do was reviewing files locally, you're all set.
If you wish to make changes to the PR branch:

1. Make changes and [commit][commit] normally
2. Push your changes upstream: `git push <author-id> <name-of-the-PR-branch>`

   _Note: If the PR predates the introduction of this feature by Github,
   or if the author of the PR has disabled write-access by reviewers to the PR branch,
   this will not work, and you will need to use the alternative described below._

If, instead of modifying the existing PR, you wish to make a new one based on it:

1. Set up a new branch that contains the existing PR by doing one of the following:
   1. Create a new branch from the tip of the PR:
   `git branch <your-new-branch> <name-of-the-PR-branch> && git checkout <your-new-branch>`
   2. Create a new branch from `master` and merge the PR into it:
   `git branch <your-new-branch> master && git checkout <your-new-branch> && git merge <name-of-the-PR-branch>`
2. Make changes and [commit][commit] normally
3. Push your changes to **your** repo: `git push origin <your-new-branch>`
4. Go to the Github Web UI to [submit a new Pull Request][submit].

   _Note: You should also close the original pull request._

When you're done reviewing or making changes,
you can delete the branch: `git branch -d <name-of-the-PR-branch>`
(use `-D` instead of `-d` to delete a branch that has not been merged into master yet).

If you do not expect work with more PRs from the same author,
you may also discard your connection to their repo:
`git remote remove <author-id>`

[clone]: ../writing-tests/github-intro.html#clone
[commit]: ../writing-tests/github-intro.html#commit
[configure]: ../writing-tests/github-intro.html#configure-remote-upstream
[fork]: ../writing-tests/github-intro.html#fork-the-test-repository
[submit]: ../writing-tests/github-intro.html#submit
