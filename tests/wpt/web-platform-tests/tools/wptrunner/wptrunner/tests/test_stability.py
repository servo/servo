import sys
from os.path import dirname, join

sys.path.insert(0, join(dirname(__file__), "..", ".."))

from wptrunner import stability


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
