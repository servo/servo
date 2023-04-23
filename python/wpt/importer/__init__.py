# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os
import sys

from .tree import GitTree, GeckoCommit

from wptrunner.update import setup_logging, WPTUpdate  # noqa: F401
from wptrunner.update.base import Step, StepRunner, exit_unclean  # noqa: F401
from wptrunner.update.update import LoadConfig, SyncFromUpstream, UpdateMetadata  # noqa: F401
from wptrunner import wptcommandline  # noqa: F401


class LoadTrees(Step):
    """Load gecko tree and sync tree containing web-platform-tests"""

    provides = ["local_tree", "sync_tree"]

    def create(self, state):
        if os.path.exists(state.sync["path"]):
            sync_tree = GitTree(root=state.sync["path"])
        else:
            sync_tree = None

        assert GitTree.is_type()
        state.update({"local_tree": GitTree(commit_cls=GeckoCommit),
                      "sync_tree": sync_tree})


class UpdateRunner(StepRunner):
    """Overall runner for updating web-platform-tests in Gecko."""
    steps = [LoadConfig,
             LoadTrees,
             SyncFromUpstream,
             UpdateMetadata]


def run_update(**kwargs):
    logger = setup_logging(kwargs, {"mach": sys.stdout})
    updater = WPTUpdate(logger, runner_cls=UpdateRunner, **kwargs)
    return updater.run() != exit_unclean


def create_parser():
    parser = wptcommandline.create_parser_update()
    parser.add_argument("--layout-2020", "--with-layout-2020", default=False, action="store_true",
                        help="Use expected results for the 2020 layout engine")
    return parser


def check_args(kwargs):
    wptcommandline.set_from_config(kwargs)
    if hasattr(wptcommandline, 'check_paths'):
        wptcommandline.check_paths(kwargs)
    return kwargs
