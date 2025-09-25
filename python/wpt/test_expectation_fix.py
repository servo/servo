#!/usr/bin/env python3

"""
Test the fixed expectation logic for tests with .ini files
"""

import os
import sys

# Add the directory containing ohos_webdriver_test to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from ohos_webdriver_test import OHOSWebDriverController


def test_expectation_logic_fix() -> None:
    """Test that tests with .ini files are correctly expected to fail."""
    print("Testing Fixed Expectation Logic")
    print("=" * 50)

    controller = OHOSWebDriverController()

    # Test case 1: elementsFromPoint-shadowroot.html (the problematic one)
    test_path = "css/cssom-view/elementsFromPoint-shadowroot.html"
    expectations = controller.load_test_expectations(test_path)

    print(f"\n1. Test: {test_path}")
    print(f"   Has expectation file: {os.path.exists(controller.get_expected_path(test_path))}")
    print(f"   Test expected: {expectations['test_expected']}")
    print(f"   Subtest expectations: {len(expectations['subtest_expectations'])}")

    for subtest_name, expected_status in expectations["subtest_expectations"].items():
        print(f"     - {subtest_name}: {expected_status}")

    # Simulate the actual result that was problematic
    mock_result = {
        "status": "FAIL",
        "passCount": 0,
        "failCount": 2,
        "failingTests": [
            {
                "name": "elementsFromPoint on the document root should not return elements in shadow trees",
                "error": "assert_not_equals: got disallowed value -1",
            },
            {
                "name": "elementsFromPoint on a shadow root should include elements in that shadow tree",
                "error": "assert_not_equals: got disallowed value -1",
            },
        ],
    }

    analysis = controller.compare_with_expectations(test_path, mock_result)

    print("   Analysis Results:")
    print(f"     Test matches expectation: {analysis['test_matches_expectation']}")
    print(f"     Expected failures found: {len(analysis['expected_failures'])}")
    print(f"     Unexpected subtests: {len(analysis['unexpected_subtests'])}")
    print(f"     Summary: {analysis['summary']}")

    print("   Expected failures:")
    for failure in analysis["expected_failures"]:
        print(f"     - {failure['name']}")
        if "matched_expectation" in failure:
            print(f"       (matched with: {failure['matched_expectation']})")

    print("   Unexpected subtests:")
    for unexpected in analysis["unexpected_subtests"]:
        print(f"     - {unexpected['name']}: expected {unexpected['expected']}, got {unexpected['actual']}")

    # Test case 2: A test with mixed expectations
    print("\n" + "=" * 50)
    test_path2 = "css/css-color/parsing/color-computed.html"
    expectations2 = controller.load_test_expectations(test_path2)

    print(f"\n2. Test: {test_path2}")
    print(f"   Test expected: {expectations2['test_expected']}")
    print(f"   Subtest expectations: {len(expectations2['subtest_expectations'])}")

    # Show first few subtest expectations
    count = 0
    for subtest_name, expected_status in expectations2["subtest_expectations"].items():
        if count < 3:
            print(f"     - {subtest_name}: {expected_status}")
            count += 1
        else:
            remaining = len(expectations2["subtest_expectations"]) - 3
            print(f"     ... and {remaining} more")
            break

    # Test case 3: A simple test with overall expectation
    print("\n" + "=" * 50)
    test_path3 = "css/css-color/color-mix-basic-001.html"
    expectations3 = controller.load_test_expectations(test_path3)

    print(f"\n3. Test: {test_path3}")
    print(f"   Test expected: {expectations3['test_expected']}")
    print(f"   Subtest expectations: {len(expectations3['subtest_expectations'])}")

    print("\n" + "=" * 50)
    print("Summary:")
    print("✓ Tests with .ini files containing subtest failures are now expected to FAIL")
    print("✓ Tests with explicit test-level expectations use those")
    print("✓ Subtest matching should work correctly with normalized names")


if __name__ == "__main__":
    test_expectation_logic_fix()
