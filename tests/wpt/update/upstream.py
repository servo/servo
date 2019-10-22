from __future__ import print_function

import os
import re
import subprocess
import sys
import six.moves.urllib as urllib
from six.moves import input
from six import iteritems

from wptrunner.update.sync import UpdateCheckout
from wptrunner.update.tree import get_unique_name
from wptrunner.update.base import Step, StepRunner, exit_clean, exit_unclean

from .tree import Commit, GitTree, Patch
from .github import GitHub


def rewrite_patch(patch, strip_dir):
    """Take a Patch and  rewrite the message to remove the bug number and reviewer, but add
    a bugzilla link in the summary.

    :param patch: the Patch to convert
    """

    return Patch(patch.author, patch.email, rewrite_message(patch), None, patch.diff)

def rewrite_message(patch):
    if patch.merge_message and patch.merge_message.bug:
        bug = patch.merge_message.bug
    else:
        bug = patch.message.bug
    if bug is not None:
        return "\n".join([patch.message.summary,
                          patch.message.body,
                          "",
                          "Upstreamed from https://github.com/servo/servo/pull/%s [ci skip]" %
                          bug])

    return "\n".join([patch.message.full_summary, "%s\n[ci skip]\n" % patch.message.body])


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

class GetLastSyncData(Step):
    """Find the gecko commit at which we last performed a sync with upstream and the upstream
    commit that was synced."""

    provides = ["sync_data_path", "last_sync_commit", "old_upstream_rev"]

    def create(self, state):
        self.logger.info("Looking for last sync commit")
        state.sync_data_path = os.path.join(state.metadata_path, "mozilla-sync")
        items = {}
        with open(state.sync_data_path) as f:
            for line in f.readlines():
                key, value = [item.strip() for item in line.split(":", 1)]
                items[key] = value

        state.last_sync_commit = Commit(state.local_tree, items["local"])
        state.old_upstream_rev = items["upstream"]

        if not state.local_tree.contains_commit(state.last_sync_commit):
            self.logger.error("Could not find last sync commit %s" % last_sync_sha1)
            return exit_clean

        self.logger.info("Last sync to web-platform-tests happened in %s" % state.last_sync_commit.sha1)


class CheckoutBranch(Step):
    """Create a branch in the sync tree pointing at the last upstream sync commit
    and check it out"""

    provides = ["branch"]

    def create(self, state):
        self.logger.info("Updating sync tree from %s" % state.sync["remote_url"])
        state.branch = state.sync_tree.unique_branch_name(
            "outbound_update_%s" % state.old_upstream_rev)
        state.sync_tree.update(state.sync["remote_url"],
                               state.sync["branch"],
                               state.branch)
        state.sync_tree.checkout(state.old_upstream_rev, state.branch, force=True)


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

        update_regexp = re.compile("Update web-platform-tests to revision [0-9a-f]{40}")

        for i, commit in enumerate(state.source_commits[:]):
            if update_regexp.match(commit.message.text):
                # This is a previous update commit so ignore it
                state.source_commits.remove(commit)
                continue

            if commit.message.backouts:
                #TODO: Add support for collapsing backouts
                raise NotImplementedError("Need to get the Git->Hg commits for backouts and remove the backed out patch")

            if not commit.message.bug and not (commit.merge and commit.merge.message.bug):
                self.logger.error("Commit %i (%s) doesn't have an associated bug number." %
                             (i + 1, commit.sha1))
                return exit_unclean

        self.logger.debug("Source commits: %s" % state.source_commits)

class SelectCommits(Step):
    """Provide a UI to select which commits to upstream"""

    def create(self, state):
        if not state.source_commits:
            return

        while True:
            commits = state.source_commits[:]
            for i, commit in enumerate(commits):
                print("%i:\t%s" % (i, commit.message.summary))

            remove = input("Provide a space-separated list of any commits numbers to remove from the list to upstream:\n").strip()
            remove_idx = set()
            invalid = False
            for item in remove.split(" "):
                if not item:
                    continue
                try:
                    item = int(item)
                except:
                    invalid = True
                    break
                if item < 0 or item >= len(commits):
                    invalid = True
                    break
                remove_idx.add(item)

            if invalid:
                continue

            keep_commits = [(i,cmt) for i,cmt in enumerate(commits) if i not in remove_idx]
            #TODO: consider printed removed commits
            print("Selected the following commits to keep:")
            for i, commit in keep_commits:
                print("%i:\t%s" % (i, commit.message.summary))
            confirm = input("Keep the above commits? y/n\n").strip().lower()

            if confirm == "y":
                state.source_commits = [item[1] for item in keep_commits]
                break

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
            strip_count = strip_path.count('/')
            if strip_path[-1] != '/':
                strip_count += 1
            try:
                state.sync_tree.import_patch(stripped_patch, 1 + strip_count)
            except:
                print(patch.diff)
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

        org, name = urllib.parse.urlsplit(state.sync["remote_url"]).path[1:].split("/")
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

class UpdateLastSyncData(Step):
    """Update the gecko commit at which we last performed a sync with upstream."""

    provides = []

    def create(self, state):
        self.logger.info("Updating last sync commit")
        data = {"local": state.local_tree.rev,
                "upstream": state.sync_tree.rev}
        with open(state.sync_data_path, "w") as f:
            for key, value in iteritems(data):
                f.write("%s: %s\n" % (key, value))
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
        state.pr.issue.add_label("servo-export")


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
    steps = [GetLastSyncData,
             UpdateCheckout,
             CheckoutBranch,
             GetBaseCommit,
             LoadCommits,
             SelectCommits,
             MovePatches,
             RebaseCommits,
             CheckRebase,
             MergeUpstream,
             UpdateLastSyncData]


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
