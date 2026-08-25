# mypy: allow-untyped-defs

import os

from .. import metadata

from .base import Step, StepRunner


class UpdateExpected(Step):
    """Do the metadata update on the local checkout"""

    def create(self, state):
        metadata.update_expected(state.paths,
                                 state.run_log,
                                 update_properties=state.product.update_properties,
                                 full_update=state.full_update,
                                 disable_intermittent=state.disable_intermittent,
                                 update_intermittent=state.update_intermittent,
                                 remove_intermittent=state.remove_intermittent)


class CreateMetadataPatch(Step):
    """Create a patch/commit for the metadata checkout"""

    def create(self, state):
        if not state.patch:
            return

        local_tree = state.local_tree
        sync_tree = state.sync_tree

        if sync_tree is not None:
            name = "web-platform-tests_update_%s_metadata" % sync_tree.rev
            message = f"Update {state.suite_name} expected data to revision {sync_tree.rev}"
        else:
            name = "web-platform-tests_update_metadata"
            message = "Update %s expected data" % state.suite_name

        local_tree.create_patch(name, message)

        if not local_tree.is_clean:
            metadata_paths = [manifest_path["metadata_path"]
                              for manifest_path in state.paths.itervalues()]
            for path in metadata_paths:
                local_tree.add_new(os.path.relpath(path, local_tree.root))
            local_tree.update_patch(include=metadata_paths)
            local_tree.commit_patch()


class MetadataUpdateRunner(StepRunner):
    """(Sub)Runner for updating metadata"""
    steps = [UpdateExpected,
             CreateMetadataPatch]
