# mypy: allow-untyped-defs

import os
import subprocess
from unittest import mock

from .. import utils


def test_git_for_path_no_git():
    this_dir = os.path.dirname(__file__)
    with mock.patch(
            "subprocess.check_output",
            side_effect=subprocess.CalledProcessError(1, "foo")):
        assert utils.git(this_dir) is None
