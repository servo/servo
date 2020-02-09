import os

from .. import metadata, products

from .base import Step, StepRunner


class GetUpdatePropertyList(Step):
    provides = ["update_properties"]

    def create(self, state):
        state.update_properties = products.load_product_update(state.config, state.product)


class UpdateExpected(Step):
    """Do the metadata update on the local checkout"""

    def create(self, state):
        if state.sync_tree is not None:
            sync_root = state.sync_tree.root
        else:
            sync_root = None

        metadata.update_expected(state.paths,
                                 state.serve_root,
                                 state.run_log,
                                 update_properties=state.update_properties,
                                 rev_old=None,
                                 full_update=state.full_update,
                                 sync_root=sync_root,
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
            message = "Update %s expected data to revision %s" % (state.suite_name, sync_tree.rev)
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
    steps = [GetUpdatePropertyList,
             UpdateExpected,
             CreateMetadataPatch]
