# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import shutil
import sys
import uuid

from .. import testloader

from base import Step, StepRunner
from tree import Commit

here = os.path.abspath(os.path.split(__file__)[0])

bsd_license = """W3C 3-clause BSD License

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

* Redistributions of works must retain the original copyright notice, this
  list of conditions and the following disclaimer.

* Redistributions in binary form must reproduce the original copyright
  notice, this list of conditions and the following disclaimer in the
  documentation and/or other materials provided with the distribution.

* Neither the name of the W3C nor the names of its contributors may be
  used to endorse or promote products derived from this work without
  specific prior written permission.


THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.
"""


def copy_wpt_tree(tree, dest):
    """Copy the working copy of a Tree to a destination directory.

    :param tree: The Tree to copy.
    :param dest: The destination directory"""
    if os.path.exists(dest):
        assert os.path.isdir(dest)

    shutil.rmtree(dest)
    os.mkdir(dest)

    for tree_path in tree.paths():
        source_path = os.path.join(tree.root, tree_path)
        dest_path = os.path.join(dest, tree_path)

        dest_dir = os.path.split(dest_path)[0]
        if not os.path.isdir(source_path):
            if not os.path.exists(dest_dir):
                os.makedirs(dest_dir)
            shutil.copy2(source_path, dest_path)

    for source, destination in [("testharness_runner.html", ""),
                                ("testharnessreport.js", "resources/")]:
        source_path = os.path.join(here, os.pardir, source)
        dest_path = os.path.join(dest, destination, os.path.split(source)[1])
        shutil.copy2(source_path, dest_path)

    add_license(dest)


def add_license(dest):
    """Write the bsd license string to a LICENSE file.

    :param dest: Directory in which to place the LICENSE file."""
    with open(os.path.join(dest, "LICENSE"), "w") as f:
        f.write(bsd_license)

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
        if not sync_path in sys.path:
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


class LoadManifest(Step):
    """Load the test manifest"""

    provides = ["manifest_path", "test_manifest"]

    def create(self, state):
        from manifest import manifest
        state.manifest_path = os.path.join(state.metadata_path, "MANIFEST.json")
        state.test_manifest = manifest.Manifest("/")


class UpdateManifest(Step):
    """Update the manifest to match the tests in the sync tree checkout"""

    def create(self, state):
        from manifest import manifest, update
        update.update(state.sync["path"], state.test_manifest)
        manifest.write(state.test_manifest, state.manifest_path)


class CopyWorkTree(Step):
    """Copy the sync tree over to the destination in the local tree"""

    def create(self, state):
        copy_wpt_tree(state.sync_tree,
                      state.tests_path)


class CreateSyncPatch(Step):
    """Add the updated test files to a commit/patch in the local tree."""

    def create(self, state):
        if state.no_patch:
            return

        local_tree = state.local_tree
        sync_tree = state.sync_tree

        local_tree.create_patch("web-platform-tests_update_%s" % sync_tree.rev,
                                "Update %s to revision %s" % (state.suite_name, sync_tree.rev))
        local_tree.add_new(os.path.relpath(state.tests_path,
                                           local_tree.root))
        updated = local_tree.update_patch(include=[state.tests_path,
                                                   state.metadata_path])
        local_tree.commit_patch()

        if not updated:
            self.logger.info("Nothing to sync")


class SyncFromUpstreamRunner(StepRunner):
    """(Sub)Runner for doing an upstream sync"""
    steps = [UpdateCheckout,
             GetSyncTargetCommit,
             LoadManifest,
             UpdateManifest,
             CopyWorkTree,
             CreateSyncPatch]
