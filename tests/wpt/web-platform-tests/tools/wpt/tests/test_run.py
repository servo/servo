import mock

import pytest

from tools.wpt import run


@pytest.mark.parametrize("platform", ["Windows", "Linux", "Darwin"])
def test_check_environ_fail(platform):
    m_open = mock.mock_open(read_data=b"")

    with mock.patch.object(run, "open", m_open):
        with mock.patch.object(run.platform, "uname",
                               return_value=(platform, "", "", "", "", "")):
            with pytest.raises(run.WptrunError) as excinfo:
                run.check_environ("foo")

    assert "wpt make-hosts-file" in excinfo.value.message
