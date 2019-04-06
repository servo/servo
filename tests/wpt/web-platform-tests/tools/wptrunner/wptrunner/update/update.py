import os
import sys

from .metadata import MetadataUpdateRunner
from .sync import SyncFromUpstreamRunner
from .tree import GitTree, HgTree, NoVCSTree

from .base import Step, StepRunner, exit_clean, exit_unclean
from .state import SavedState, UnsavedState

def setup_paths(sync_path):
    sys.path.insert(0, os.path.abspath(sync_path))
    from tools import localpaths  # noqa: flake8

class LoadConfig(Step):
    """Step for loading configuration from the ini file and kwargs."""

    provides = ["sync", "paths", "metadata_path", "tests_path"]

    def create(self, state):
        state.sync = {"remote_url": state.kwargs["remote_url"],
                      "branch": state.kwargs["branch"],
                      "path": state.kwargs["sync_path"]}

        state.paths = state.kwargs["test_paths"]
        state.tests_path = state.paths["/"]["tests_path"]
        state.metadata_path = state.paths["/"]["metadata_path"]

        assert os.path.isabs(state.tests_path)


class LoadTrees(Step):
    """Step for creating a Tree for the local copy and a GitTree for the
    upstream sync."""

    provides = ["local_tree", "sync_tree"]

    def create(self, state):
        if os.path.exists(state.sync["path"]):
            sync_tree = GitTree(root=state.sync["path"])
        else:
            sync_tree = None

        if GitTree.is_type():
            local_tree = GitTree()
        elif HgTree.is_type():
            local_tree = HgTree()
        else:
            local_tree = NoVCSTree()

        state.update({"local_tree": local_tree,
                      "sync_tree": sync_tree})


class SyncFromUpstream(Step):
    """Step that synchronises a local copy of the code with upstream."""

    def create(self, state):
        if not state.kwargs["sync"]:
            return

        if not state.sync_tree:
            os.mkdir(state.sync["path"])
            state.sync_tree = GitTree(root=state.sync["path"])

        kwargs = state.kwargs
        with state.push(["sync", "paths", "metadata_path", "tests_path", "local_tree",
                         "sync_tree"]):
            state.target_rev = kwargs["rev"]
            state.patch = kwargs["patch"]
            state.suite_name = kwargs["suite_name"]
            state.path_excludes = kwargs["exclude"]
            state.path_includes = kwargs["include"]
            runner = SyncFromUpstreamRunner(self.logger, state)
            runner.run()


class UpdateMetadata(Step):
    """Update the expectation metadata from a set of run logs"""

    def create(self, state):
        if not state.kwargs["run_log"]:
            return

        kwargs = state.kwargs
        with state.push(["local_tree", "sync_tree", "paths", "serve_root"]):
            state.run_log = kwargs["run_log"]
            state.ignore_existing = kwargs["ignore_existing"]
            state.stability = kwargs["stability"]
            state.patch = kwargs["patch"]
            state.suite_name = kwargs["suite_name"]
            state.product = kwargs["product"]
            state.config = kwargs["config"]
            state.extra_properties = kwargs["extra_property"]
            runner = MetadataUpdateRunner(self.logger, state)
            runner.run()


class RemoveObsolete(Step):
    """Remove metadata files that don't corespond to an existing test file"""

    def create(self, state):
        if not state.kwargs["remove_obsolete"]:
            return

        paths = state.kwargs["test_paths"]
        state.tests_path = state.paths["/"]["tests_path"]
        state.metadata_path = state.paths["/"]["metadata_path"]

        for url_paths in paths.itervalues():
            tests_path = url_paths["tests_path"]
            metadata_path = url_paths["metadata_path"]
            for dirpath, dirnames, filenames in os.walk(metadata_path):
                for filename in filenames:
                    if filename == "__dir__.ini":
                        continue
                    if filename.endswith(".ini"):
                        full_path = os.path.join(dirpath, filename)
                        rel_path = os.path.relpath(full_path, metadata_path)
                        test_path = os.path.join(tests_path, rel_path[:-4])
                        if not os.path.exists(test_path):
                            os.unlink(full_path)


class UpdateRunner(StepRunner):
    """Runner for doing an overall update."""
    steps = [LoadConfig,
             LoadTrees,
             SyncFromUpstream,
             RemoveObsolete,
             UpdateMetadata]


class WPTUpdate(object):
    def __init__(self, logger, runner_cls=UpdateRunner, **kwargs):
        """Object that controls the running of a whole wptupdate.

        :param runner_cls: Runner subclass holding the overall list of
                           steps to run.
        :param kwargs: Command line arguments
        """
        self.runner_cls = runner_cls
        self.serve_root = kwargs["test_paths"]["/"]["tests_path"]

        if not kwargs["sync"]:
            setup_paths(self.serve_root)
        else:
            if os.path.exists(kwargs["sync_path"]):
                # If the sync path doesn't exist we defer this until it does
                setup_paths(kwargs["sync_path"])

        if kwargs.get("store_state", False):
            self.state = SavedState(logger)
        else:
            self.state = UnsavedState(logger)
        self.kwargs = kwargs
        self.logger = logger

    def run(self, **kwargs):
        if self.kwargs["abort"]:
            self.abort()
            return exit_clean

        if not self.kwargs["continue"] and not self.state.is_empty():
            self.logger.error("Found existing state. Run with --continue to resume or --abort to clear state")
            return exit_unclean

        if self.kwargs["continue"]:
            if self.state.is_empty():
                self.logger.error("No sync in progress?")
                return exit_clean

            self.kwargs = self.state.kwargs
        else:
            self.state.kwargs = self.kwargs

        self.state.serve_root = self.serve_root

        update_runner = self.runner_cls(self.logger, self.state)
        rv = update_runner.run()
        if rv in (exit_clean, None):
            self.state.clear()

        return rv

    def abort(self):
        self.state.clear()
