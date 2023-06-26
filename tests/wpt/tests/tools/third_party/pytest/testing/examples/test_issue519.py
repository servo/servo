from _pytest.pytester import Pytester


def test_519(pytester: Pytester) -> None:
    pytester.copy_example("issue_519.py")
    res = pytester.runpytest("issue_519.py")
    res.assert_outcomes(passed=8)
