from .. import stability

def test_is_inconsistent():
    assert stability.is_inconsistent({"PASS": 10}, 10) is False
    assert stability.is_inconsistent({"PASS": 9}, 10) is True
    assert stability.is_inconsistent({"PASS": 9, "FAIL": 1}, 10) is True
    assert stability.is_inconsistent({"PASS": 8, "FAIL": 1}, 10) is True


def test_find_slow_status():
    assert stability.find_slow_status({
        "longest_duration": {"TIMEOUT": 10},
        "timeout": 10}) is None
    assert stability.find_slow_status({
        "longest_duration": {"CRASH": 10},
        "timeout": 10}) is None
    assert stability.find_slow_status({
        "longest_duration": {"ERROR": 10},
        "timeout": 10}) is None
    assert stability.find_slow_status({
        "longest_duration": {"PASS": 1},
        "timeout": 10}) is None
    assert stability.find_slow_status({
        "longest_duration": {"PASS": 81},
        "timeout": 100}) == "PASS"
    assert stability.find_slow_status({
        "longest_duration": {"TIMEOUT": 10, "FAIL": 81},
        "timeout": 100}) == "FAIL"
    assert stability.find_slow_status({
        "longest_duration": {"SKIP": 0}}) is None


def test_get_steps():
    logger = None

    steps = stability.get_steps(logger, 0, 0, [])
    assert len(steps) == 0

    steps = stability.get_steps(logger, 0, 0, [{}])
    assert len(steps) == 0

    repeat_loop = 1
    flag_name = 'flag'
    flag_value = 'y'
    steps = stability.get_steps(logger, repeat_loop, 0, [
                                {flag_name: flag_value}])
    assert len(steps) == 1
    assert steps[0][0] == "Running tests in a loop %d times with flags %s=%s" % (
        repeat_loop, flag_name, flag_value)

    repeat_loop = 0
    repeat_restart = 1
    flag_name = 'flag'
    flag_value = 'n'
    steps = stability.get_steps(logger, repeat_loop, repeat_restart, [
                                {flag_name: flag_value}])
    assert len(steps) == 1
    assert steps[0][0] == "Running tests in a loop with restarts %d times with flags %s=%s" % (
        repeat_restart, flag_name, flag_value)

    repeat_loop = 10
    repeat_restart = 5
    steps = stability.get_steps(logger, repeat_loop, repeat_restart, [{}])
    assert len(steps) == 2
    assert steps[0][0] == "Running tests in a loop %d times" % repeat_loop
    assert steps[1][0] == (
        "Running tests in a loop with restarts %d times" % repeat_restart)
