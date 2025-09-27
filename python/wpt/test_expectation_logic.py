#!/usr/bin/env python3

"""
Test the corrected expectation logic for tests without expectation files
"""

import os
import sys
import tempfile

# Add the directory containing ohos_webdriver_test to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from ohos_webdriver_test import OHOSWebDriverController


def test_no_expectations_logic():
    """Test that tests without expectation files are handled correctly."""
    print("Testing No Expectations Logic")
    print("=" * 40)

    controller = OHOSWebDriverController()

    # Test 1: Test without expectation file that PASSES
    print("\n1. Test without expectations that PASSES:")
    test_path = "fake/test/that/passes.html"

    # Mock result: test passes
    mock_result_pass = {"status": "PASS", "passCount": 3, "failCount": 0, "failingTests": []}

    analysis = controller.compare_with_expectations(test_path, mock_result_pass)

    print(f"   Test expected: {analysis['test_expected']}")
    print(f"   Test actual: {analysis['test_actual']}")
    print(f"   Test matches expectation: {analysis['test_matches_expectation']}")
    print(f"   Has expectations file: {analysis['has_expectations']}")
    print(f"   Summary: {analysis['summary']}")

    assert analysis["test_expected"] == "PASS", "Should expect PASS by default"
    assert analysis["test_actual"] == "PASS", "Test should be PASS"
    assert analysis["test_matches_expectation"] == True, "Should match expectation"
    assert analysis["has_expectations"] == False, "Should have no expectations file"
    assert analysis["summary"] == "EXPECTED", "Should be EXPECTED"
    print("   ✓ PASS without expectations → EXPECTED")

    # Test 2: Test without expectation file that FAILS
    print("\n2. Test without expectations that FAILS:")

    # Mock result: test fails
    mock_result_fail = {
        "status": "FAIL",
        "passCount": 2,
        "failCount": 1,
        "failingTests": [{"name": "Some subtest", "error": "assertion failed"}],
    }

    analysis = controller.compare_with_expectations(test_path, mock_result_fail)

    print(f"   Test expected: {analysis['test_expected']}")
    print(f"   Test actual: {analysis['test_actual']}")
    print(f"   Test matches expectation: {analysis['test_matches_expectation']}")
    print(f"   Has expectations file: {analysis['has_expectations']}")
    print(f"   Unexpected subtests: {len(analysis['unexpected_subtests'])}")
    print(f"   Summary: {analysis['summary']}")

    assert analysis["test_expected"] == "PASS", "Should expect PASS by default"
    assert analysis["test_actual"] == "FAIL", "Test should be FAIL"
    assert analysis["test_matches_expectation"] == False, "Should not match expectation"
    assert analysis["has_expectations"] == False, "Should have no expectations file"
    assert len(analysis["unexpected_subtests"]) == 1, "Should have 1 unexpected subtest"
    assert analysis["summary"] == "UNEXPECTED_SUBTEST_RESULTS", "Should be UNEXPECTED_SUBTEST_RESULTS"
    print("   ✓ FAIL without expectations → UNEXPECTED_SUBTEST_RESULTS")

    # Test 3: Test without expectation file that TIMES OUT
    print("\n3. Test without expectations that TIMES OUT:")

    # Mock result: test times out
    mock_result_timeout = {"status": "TIMEOUT", "passCount": 0, "failCount": 0, "failingTests": []}

    analysis = controller.compare_with_expectations(test_path, mock_result_timeout)

    print(f"   Test expected: {analysis['test_expected']}")
    print(f"   Test actual: {analysis['test_actual']}")
    print(f"   Test matches expectation: {analysis['test_matches_expectation']}")
    print(f"   Has expectations file: {analysis['has_expectations']}")
    print(f"   Summary: {analysis['summary']}")

    assert analysis["test_expected"] == "PASS", "Should expect PASS by default"
    assert analysis["test_actual"] == "TIMEOUT", "Test should be TIMEOUT"
    assert analysis["test_matches_expectation"] == False, "Should not match expectation"
    assert analysis["has_expectations"] == False, "Should have no expectations file"
    assert analysis["summary"] == "UNEXPECTED_FAIL", "Should be UNEXPECTED_FAIL"
    print("   ✓ TIMEOUT without expectations → UNEXPECTED_FAIL")

    # Test 4: Test with expectation file that expects FAIL
    print("\n4. Test WITH expectations that expects FAIL:")

    # Create a temporary .ini file
    with tempfile.NamedTemporaryFile(mode="w", suffix=".ini", delete=False) as f:
        f.write("[test-with-expectations.html]\nexpected = FAIL\n")
        temp_ini_path = f.name

    # Mock the get_expected_path method to return our temp file
    original_get_expected_path = controller.get_expected_path

    def mock_get_expected_path(test_path):
        return temp_ini_path

    controller.get_expected_path = mock_get_expected_path

    try:
        test_path_with_exp = "fake/test-with-expectations.html"

        mock_result_fail_expected = {
            "status": "FAIL",
            "passCount": 0,
            "failCount": 1,
            "failingTests": [{"name": "Expected failure", "error": "this was expected"}],
        }

        analysis = controller.compare_with_expectations(test_path_with_exp, mock_result_fail_expected)

        print(f"   Test expected: {analysis['test_expected']}")
        print(f"   Test actual: {analysis['test_actual']}")
        print(f"   Test matches expectation: {analysis['test_matches_expectation']}")
        print(f"   Has expectations file: {analysis['has_expectations']}")
        print(f"   Summary: {analysis['summary']}")

        assert analysis["test_expected"] == "FAIL", "Should expect FAIL from .ini file"
        assert analysis["test_actual"] == "FAIL", "Test should be FAIL"
        assert analysis["test_matches_expectation"] == True, "Should match expectation"
        assert analysis["has_expectations"] == True, "Should have expectations file"
        assert analysis["summary"] == "EXPECTED", "Should be EXPECTED"
        print("   ✓ FAIL with FAIL expectation → EXPECTED")

    finally:
        controller.get_expected_path = original_get_expected_path
        os.unlink(temp_ini_path)

    print("\n" + "=" * 40)
    print("✅ All expectation logic tests passed!")
    print("\nSummary:")
    print("- Tests without .ini files are expected to PASS completely")
    print("- Any failure in such tests is classified as unexpected")
    print("- Tests with .ini files use their explicit expectations")


if __name__ == "__main__":
    test_no_expectations_logic()
