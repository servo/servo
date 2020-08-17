import fnmatch
import os
import re
import shutil
import sys
import uuid

from .base import Step, StepRunner
from .tree import Commit

here = os.path.abspath(os.path.dirname(__file__))


def copy_wpt_tree(tree, dest, excludes=None, includes=None):
    """Copy the working copy of a Tree to a destination directory.

    :param tree: The Tree to copy.
    :param dest: The destination directory"""
    if os.path.exists(dest):
        assert os.path.isdir(dest)

    shutil.rmtree(dest)

    os.mkdir(dest)

    if excludes is None:
        excludes = []

    excludes = [re.compile(fnmatch.translate(item)) for item in excludes]

    if includes is None:
        includes = []

    includes = [re.compile(fnmatch.translate(item)) for item in includes]

    for tree_path in tree.paths():
        if (any(item.match(tree_path) for item in excludes) and
            not any(item.match(tree_path) for item in includes)):
            continue

        source_path = os.path.join(tree.root, tree_path)
        dest_path = os.path.join(dest, tree_path)

        dest_dir = os.path.dirname(dest_path)
        if not os.path.isdir(source_path):
            if not os.path.exists(dest_dir):
                os.makedirs(dest_dir)
            shutil.copy2(source_path, dest_path)

    for source, destination in [("testharness_runner.html", ""),
                                ("testdriver-vendor.js", "resources/")]:
        source_path = os.path.join(here, os.pardir, source)
        dest_path = os.path.join(dest, destination, os.path.basename(source))
        shutil.copy2(source_path, dest_path)


class UpdateCheckout(Step):
    """Pull changes from upstream into the local sync tree."""

    provides = ["local_branch"]

    def create(self, state):
        sync_tree = state.sync_tree
        state.local_branch = uuid.uuid4().hex
        sync_tree.update(state.sync["remote_url"],
                         state.sync["branch"],
                         state.local_branch)
        sync_path = os.path.abspath(sync_tree.root)
        if sync_path not in sys.path:
            from update import setup_paths
            setup_paths(sync_path)

    def restore(self, state):
        assert os.path.abspath(state.sync_tree.root) in sys.path
        Step.restore(self, state)


class GetSyncTargetCommit(Step):
    """Find the commit that we will sync to."""

    provides = ["sync_commit"]

    def create(self, state):
        if state.target_rev is None:
            #Use upstream branch HEAD as the base commit
            state.sync_commit = state.sync_tree.get_remote_sha1(state.sync["remote_url"],
                                                                state.sync["branch"])
        else:
            state.sync_commit = Commit(state.sync_tree, state.rev)

        state.sync_tree.checkout(state.sync_commit.sha1, state.local_branch, force=True)
        self.logger.debug("New base commit is %s" % state.sync_commit.sha1)


class UpdateManifest(Step):
    """Update the manifest to match the tests in the sync tree checkout"""

    provides = ["manifest_path", "test_manifest"]

    def create(self, state):
        from manifest import manifest
        state.manifest_path = os.path.join(state.metadata_path, "MANIFEST.json")
        state.test_manifest = manifest.load_and_update(state.sync["path"],
                                                       state.manifest_path,
                                                       "/",
                                                       write_manifest=True)


class CopyWorkTree(Step):
    """Copy the sync tree over to the destination in the local tree"""

    def create(self, state):
        copy_wpt_tree(state.sync_tree,
                      state.tests_path,
                      excludes=state.path_excludes,
                      includes=state.path_includes)


class CreateSyncPatch(Step):
    """Add the updated test files to a commit/patch in the local tree."""

    def create(self, state):
        if not state.patch:
            return

        local_tree = state.local_tree
        sync_tree = state.sync_tree

        local_tree.create_patch("web-platform-tests_update_%s" % sync_tree.rev,
                                "Update %s to revision %s" % (state.suite_name, sync_tree.rev))
        test_prefix = os.path.relpath(state.tests_path, local_tree.root)
        local_tree.add_new(test_prefix)
        local_tree.add_ignored(sync_tree, test_prefix)
        updated = local_tree.update_patch(include=[state.tests_path,
                                                   state.metadata_path])
        local_tree.commit_patch()

        if not updated:
            self.logger.info("Nothing to sync")


class SyncFromUpstreamRunner(StepRunner):
    """(Sub)Runner for doing an upstream sync"""
    steps = [UpdateCheckout,
             GetSyncTargetCommit,
             UpdateManifest,
             CopyWorkTree,
             CreateSyncPatch]
