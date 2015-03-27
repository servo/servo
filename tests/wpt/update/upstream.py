import os
import re
import subprocess
import sys
import urlparse

from wptrunner.update.sync import LoadManifest
from wptrunner.update.tree import get_unique_name
from wptrunner.update.base import Step, StepRunner, exit_clean, exit_unclean

from .tree import Commit, GitTree, Patch
import github
from .github import GitHub


def rewrite_patch(patch, strip_dir):
    """Take a Patch and convert to a different repository by stripping a prefix from the
    file paths. Also rewrite the message to remove the bug number and reviewer, but add
    a bugzilla link in the summary.

    :param patch: the Patch to convert
    :param strip_dir: the path prefix to remove
    """

    if not strip_dir.startswith("/"):
        strip_dir = "/%s"% strip_dir

    new_diff = []
    line_starts = ["diff ", "+++ ", "--- "]
    for line in patch.diff.split("\n"):
        for start in line_starts:
            if line.startswith(start):
                new_diff.append(line.replace(strip_dir, "").encode("utf8"))
                break
        else:
            new_diff.append(line)

    new_diff = "\n".join(new_diff)

    assert new_diff != patch

    return Patch(patch.author, patch.email, rewrite_message(patch), new_diff)

def rewrite_message(patch):
    rest = patch.message.body

    if patch.message.bug is not None:
        return "\n".join([patch.message.summary,
                          patch.message.body,
                          "",
                          "Upstreamed from https://bugzilla.mozilla.org/show_bug.cgi?id=%s" %
                          patch.message.bug])

    return "\n".join([patch.message.full_summary, rest])


class SyncToUpstream(Step):
    """Sync local changes to upstream"""

    def create(self, state):
        if not state.kwargs["upstream"]:
            return

        if not isinstance(state.local_tree, GitTree):
            self.logger.error("Cannot sync with upstream from a non-Git checkout.")
            return exit_clean

        try:
            import requests
        except ImportError:
            self.logger.error("Upstream sync requires the requests module to be installed")
            return exit_clean

        if not state.sync_tree:
            os.makedirs(state.sync["path"])
            state.sync_tree = GitTree(root=state.sync["path"])

        kwargs = state.kwargs
        with state.push(["local_tree", "sync_tree", "tests_path", "metadata_path",
                         "sync"]):
            state.token = kwargs["token"]
            runner = SyncToUpstreamRunner(self.logger, state)
            runner.run()


class CheckoutBranch(Step):
    """Create a branch in the sync tree pointing at the last upstream sync commit
    and check it out"""

    provides = ["branch"]

    def create(self, state):
        self.logger.info("Updating sync tree from %s" % state.sync["remote_url"])
        state.branch = state.sync_tree.unique_branch_name(
            "outbound_update_%s" % state.test_manifest.rev)
        state.sync_tree.update(state.sync["remote_url"],
                               state.sync["branch"],
                               state.branch)
        state.sync_tree.checkout(state.test_manifest.rev, state.branch, force=True)


class GetLastSyncCommit(Step):
    """Find the gecko commit at which we last performed a sync with upstream."""

    provides = ["last_sync_path", "last_sync_commit"]

    def create(self, state):
        self.logger.info("Looking for last sync commit")
        state.last_sync_path = os.path.join(state.metadata_path, "mozilla-sync")
        with open(state.last_sync_path) as f:
            last_sync_sha1 = f.read().strip()

        state.last_sync_commit = Commit(state.local_tree, last_sync_sha1)

        if not state.local_tree.contains_commit(state.last_sync_commit):
            self.logger.error("Could not find last sync commit %s" % last_sync_sha1)
            return exit_clean

        self.logger.info("Last sync to web-platform-tests happened in %s" % state.last_sync_commit.sha1)


class GetBaseCommit(Step):
    """Find the latest upstream commit on the branch that we are syncing with"""

    provides = ["base_commit"]

    def create(self, state):
        state.base_commit = state.sync_tree.get_remote_sha1(state.sync["remote_url"],
                                                            state.sync["branch"])
        self.logger.debug("New base commit is %s" % state.base_commit.sha1)


class LoadCommits(Step):
    """Get a list of commits in the gecko tree that need to be upstreamed"""

    provides = ["source_commits"]

    def create(self, state):
        state.source_commits = state.local_tree.log(state.last_sync_commit,
                                                    state.tests_path)

        update_regexp = re.compile("Bug \d+ - Update web-platform-tests to revision [0-9a-f]{40}")

        for i, commit in enumerate(state.source_commits[:]):
            if update_regexp.match(commit.message.text):
                # This is a previous update commit so ignore it
                state.source_commits.remove(commit)
                continue

            if commit.message.backouts:
                #TODO: Add support for collapsing backouts
                raise NotImplementedError("Need to get the Git->Hg commits for backouts and remove the backed out patch")

            if not commit.message.bug:
                self.logger.error("Commit %i (%s) doesn't have an associated bug number." %
                             (i + 1, commit.sha1))
                return exit_unclean

        self.logger.debug("Source commits: %s" % state.source_commits)

class MovePatches(Step):
    """Convert gecko commits into patches against upstream and commit these to the sync tree."""

    provides = ["commits_loaded"]

    def create(self, state):
        state.commits_loaded = 0

        strip_path = os.path.relpath(state.tests_path,
                                     state.local_tree.root)
        self.logger.debug("Stripping patch %s" % strip_path)

        for commit in state.source_commits[state.commits_loaded:]:
            i = state.commits_loaded + 1
            self.logger.info("Moving commit %i: %s" % (i, commit.message.full_summary))
            patch = commit.export_patch(state.tests_path)
            stripped_patch = rewrite_patch(patch, strip_path)
            try:
                state.sync_tree.import_patch(stripped_patch)
            except:
                print patch.diff
                raise
            state.commits_loaded = i

class RebaseCommits(Step):
    """Rebase commits from the current branch on top of the upstream destination branch.

    This step is particularly likely to fail if the rebase generates merge conflicts.
    In that case the conflicts can be fixed up locally and the sync process restarted
    with --continue.
    """

    provides = ["rebased_commits"]

    def create(self, state):
        self.logger.info("Rebasing local commits")
        continue_rebase = False
        # Check if there's a rebase in progress
        if (os.path.exists(os.path.join(state.sync_tree.root,
                                        ".git",
                                        "rebase-merge")) or
            os.path.exists(os.path.join(state.sync_tree.root,
                                        ".git",
                                        "rebase-apply"))):
            continue_rebase = True

        try:
            state.sync_tree.rebase(state.base_commit, continue_rebase=continue_rebase)
        except subprocess.CalledProcessError:
            self.logger.info("Rebase failed, fix merge and run %s again with --continue" % sys.argv[0])
            raise
        state.rebased_commits = state.sync_tree.log(state.base_commit)
        self.logger.info("Rebase successful")

class CheckRebase(Step):
    """Check if there are any commits remaining after rebase"""

    def create(self, state):
        if not state.rebased_commits:
            self.logger.info("Nothing to upstream, exiting")
            return exit_clean

class MergeUpstream(Step):
    """Run steps to push local commits as seperate PRs and merge upstream."""

    provides = ["merge_index", "gh_repo"]

    def create(self, state):
        gh = GitHub(state.token)
        if "merge_index" not in state:
            state.merge_index = 0

        org, name = urlparse.urlsplit(state.sync["remote_url"]).path[1:].split("/")
        if name.endswith(".git"):
            name = name[:-4]
        state.gh_repo = gh.repo(org, name)
        for commit in state.rebased_commits[state.merge_index:]:
            with state.push(["gh_repo", "sync_tree"]):
                state.commit = commit
                pr_merger = PRMergeRunner(self.logger, state)
                rv = pr_merger.run()
                if rv is not None:
                    return rv
            state.merge_index += 1

class UpdateLastSyncCommit(Step):
    """Update the gecko commit at which we last performed a sync with upstream."""

    provides = []

    def create(self, state):
        self.logger.info("Updating last sync commit")
        with open(state.last_sync_path, "w") as f:
            f.write(state.local_tree.rev)
        # This gets added to the patch later on

class MergeLocalBranch(Step):
    """Create a local branch pointing at the commit to upstream"""

    provides = ["local_branch"]

    def create(self, state):
        branch_prefix = "sync_%s" % state.commit.sha1
        local_branch = state.sync_tree.unique_branch_name(branch_prefix)

        state.sync_tree.create_branch(local_branch, state.commit)
        state.local_branch = local_branch

class MergeRemoteBranch(Step):
    """Get an unused remote branch name to use for the PR"""
    provides = ["remote_branch"]

    def create(self, state):
        remote_branch = "sync_%s" % state.commit.sha1
        branches = [ref[len("refs/heads/"):] for sha1, ref in
                    state.sync_tree.list_remote(state.gh_repo.url)
                    if ref.startswith("refs/heads")]
        state.remote_branch = get_unique_name(branches, remote_branch)


class PushUpstream(Step):
    """Push local branch to remote"""
    def create(self, state):
        self.logger.info("Pushing commit upstream")
        state.sync_tree.push(state.gh_repo.url,
                             state.local_branch,
                             state.remote_branch)

class CreatePR(Step):
    """Create a PR for the remote branch"""

    provides = ["pr"]

    def create(self, state):
        self.logger.info("Creating a PR")
        commit = state.commit
        state.pr = state.gh_repo.create_pr(commit.message.full_summary,
                                           state.remote_branch,
                                           "master",
                                           commit.message.body if commit.message.body else "")


class PRAddComment(Step):
    """Add an issue comment indicating that the code has been reviewed already"""
    def create(self, state):
        state.pr.issue.add_comment("Code reviewed upstream.")


class MergePR(Step):
    """Merge the PR"""

    def create(self, state):
        self.logger.info("Merging PR")
        state.pr.merge()


class PRDeleteBranch(Step):
    """Delete the remote branch"""

    def create(self, state):
        self.logger.info("Deleting remote branch")
        state.sync_tree.push(state.gh_repo.url, "", state.remote_branch)


class SyncToUpstreamRunner(StepRunner):
    """Runner for syncing local changes to upstream"""
    steps = [LoadManifest,
             CheckoutBranch,
             GetLastSyncCommit,
             GetBaseCommit,
             LoadCommits,
             MovePatches,
             RebaseCommits,
             CheckRebase,
             MergeUpstream,
             UpdateLastSyncCommit]


class PRMergeRunner(StepRunner):
    """(Sub)Runner for creating and merging a PR"""
    steps = [
        MergeLocalBranch,
        MergeRemoteBranch,
        PushUpstream,
        CreatePR,
        PRAddComment,
        MergePR,
        PRDeleteBranch,
    ]
