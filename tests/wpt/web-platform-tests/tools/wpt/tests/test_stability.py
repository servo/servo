from tools.wpt import stability

def test_is_inconsistent():
    assert stability.is_inconsistent({"PASS": 10}, 10) is False
    assert stability.is_inconsistent({"PASS": 9}, 10) is True
    assert stability.is_inconsistent({"PASS": 9, "FAIL": 1}, 10) is True
    assert stability.is_inconsistent({"PASS": 8, "FAIL": 1}, 10) is True
