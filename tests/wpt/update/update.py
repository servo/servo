# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os

from wptrunner.update.base import Step, StepRunner
from wptrunner.update.update import LoadConfig, SyncFromUpstream, UpdateMetadata
from wptrunner.update.tree import NoVCSTree

from .tree import GitTree, HgTree, GeckoCommit
from .upstream import SyncToUpstream

class LoadTrees(Step):
    """Load gecko tree and sync tree containing web-platform-tests"""

    provides = ["local_tree", "sync_tree"]

    def create(self, state):
        if os.path.exists(state.sync["path"]):
            sync_tree = GitTree(root=state.sync["path"])
        else:
            sync_tree = None

        if GitTree.is_type():
            local_tree = GitTree(commit_cls=GeckoCommit)
        elif HgTree.is_type():
            local_tree = HgTree(commit_cls=GeckoCommit)
        else:
            local_tree = NoVCSTree()

        state.update({"local_tree": local_tree,
                      "sync_tree": sync_tree})


class UpdateRunner(StepRunner):
    """Overall runner for updating web-platform-tests in Gecko."""
    steps = [LoadConfig,
             LoadTrees,
             SyncToUpstream,
             SyncFromUpstream,
             UpdateMetadata]
