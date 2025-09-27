#!/usr/bin/env python3

"""
Test the subtest name matching improvements for scrollIntoView-fixed.html
"""

import sys
import os

# Add the directory containing ohos_webdriver_test to the path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from ohos_webdriver_test import OHOSWebDriverController


def test_scrollintoview_name_matching():
    """Test that subtest names match correctly for scrollIntoView-fixed.html."""
    print("Testing Subtest Name Matching for scrollIntoView-fixed.html")
    print("=" * 60)

    controller = OHOSWebDriverController()

    # Test the specific problematic case
    test_path = "css/cssom-view/scrollIntoView-fixed.html"
    expectations = controller.load_test_expectations(test_path)

    print(f"Test: {test_path}")
    print(f"Test expected: {expectations['test_expected']}")
    print(f"Subtest expectations count: {len(expectations['subtest_expectations'])}")

    # Show the actual expectations from the INI file
    print("\nExpectations from INI file:")
    for expected_name, status in expectations["subtest_expectations"].items():
        normalized = controller._normalize_expectation_name(expected_name)
        print(f"  Original: '{expected_name}' -> Normalized: '{normalized}' (expected: {status})")

    # Simulate the problematic parsed names from the actual test output
    problematic_names = [
        "[Box A] scrollIntoView from unscrollable position:fixed",
        "[Box B] scrollIntoView from unscrollable position:fixed in iframeassert_approx_equals: must scroll outer window [scrollX] expected 100 +/- 20 but got 0runTests/<",
        "[Box C] scrollIntoView from scrollable position:fixed",
        "[Box D] scrollIntoView from scrollable position:fixed in iframeassert_approx_equals: must scroll outer window [scrollX] expected 740 +/- 20 but got 0runTests/<",
    ]

    print("\nTesting name matching:")
    print("-" * 40)

    for parsed_name in problematic_names:
        print(f"\nParsed name: '{parsed_name}'")
        normalized_parsed = controller._normalize_subtest_name(parsed_name)
        print(f"Normalized: '{normalized_parsed}'")

        # Try to find a match
        matched_expectation = None
        expected_status = None

        for expected_name in expectations["subtest_expectations"]:
            normalized_expected = controller._normalize_expectation_name(expected_name)

            if (
                normalized_expected == normalized_parsed
                or normalized_expected in normalized_parsed
                or normalized_parsed in normalized_expected
                or expected_name in parsed_name
                or parsed_name in expected_name
            ):
                matched_expectation = expected_name
                expected_status = expectations["subtest_expectations"][expected_name]
                break

        if matched_expectation:
            print(f"✓ MATCHED with: '{matched_expectation}' (expected: {expected_status})")
        else:
            print("✗ NO MATCH FOUND")

    # Test the full comparison logic
    print(f"\n" + "=" * 60)
    print("Testing Full Comparison Logic")
    print("=" * 60)

    # Simulate a test result similar to the actual output
    mock_result = {
        "status": "FAIL",
        "passCount": 0,
        "failCount": 4,
        "failingTests": [
            {"name": "[Box A] scrollIntoView from unscrollable position:fixed", "error": "Some error"},
            {
                "name": "[Box B] scrollIntoView from unscrollable position:fixed in iframeassert_approx_equals: must scroll outer window [scrollX] expected 100 +/- 20 but got 0runTests/<",
                "error": "assert_approx_equals error",
            },
            {"name": "[Box C] scrollIntoView from scrollable position:fixed", "error": "Some error"},
            {
                "name": "[Box D] scrollIntoView from scrollable position:fixed in iframeassert_approx_equals: must scroll outer window [scrollX] expected 740 +/- 20 but got 0runTests/<",
                "error": "assert_approx_equals error",
            },
        ],
    }

    analysis = controller.compare_with_expectations(test_path, mock_result)

    print(f"Test matches expectation: {analysis['test_matches_expectation']}")
    print(f"Expected failures: {len(analysis['expected_failures'])}")
    print(f"Unexpected subtests: {len(analysis['unexpected_subtests'])}")
    print(f"Summary: {analysis['summary']}")

    print(f"\nExpected failures:")
    for failure in analysis["expected_failures"]:
        print(f"  - {failure['name']}")
        print(f"    (matched with: {failure.get('matched_expectation', 'N/A')})")

    print(f"\nUnexpected subtests:")
    for unexpected in analysis["unexpected_subtests"]:
        print(f"  - {unexpected['name']}")
        print(f"    Expected: {unexpected['expected']}, Got: {unexpected['actual']}")

    # Final assessment
    print(f"\n" + "=" * 60)
    expected_to_match = 2  # Box B and Box D should match expectations
    actually_matched = len(analysis["expected_failures"])

    if actually_matched == expected_to_match:
        print("✅ SUCCESS: Correct number of subtests matched expectations!")
        print(f"   Expected to match: {expected_to_match}")
        print(f"   Actually matched: {actually_matched}")
    else:
        print("❌ ISSUE: Subtest matching is still not working correctly")
        print(f"   Expected to match: {expected_to_match}")
        print(f"   Actually matched: {actually_matched}")


if __name__ == "__main__":
    test_scrollintoview_name_matching()
