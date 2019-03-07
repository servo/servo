import os
import subprocess

import mock

from .. import vcs


def test_git_for_path_no_git():
    this_dir = os.path.dirname(__file__)
    with mock.patch(
            "subprocess.check_output",
            side_effect=subprocess.CalledProcessError(1, "foo")):
        assert vcs.Git.for_path(this_dir, "/", this_dir) is None
