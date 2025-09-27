# Corrected Expectation Analysis Logic

## Overview

Fixed the expectation analysis logic to properly handle tests without expectation files. The key insight is that **tests without .ini files are expected to PASS completely** - both the overall test and all subtests should pass.

## Problem

The original logic incorrectly classified tests without expectation files as "NO_EXPECTATIONS" rather than applying the implicit expectation that they should pass. This led to:

- 24 `UNEXPECTED_SUBTEST_RESULTS` that should have been correctly classified
- 25 `UNEXPECTED` results that were actually expected failures or timeouts
- Various `UNEXPECTED_FAIL` cases not being handled properly

## Solution

### Core Principle

**Tests without .ini expectation files have an implicit expectation of PASS for everything:**
- Overall test status: `expected = PASS`
- All subtests: `expected = PASS`

### Classification Logic

#### For Tests WITHOUT Expectation Files:

1. **Test passes, no subtest failures** → `EXPECTED`
2. **Test fails due to subtest failures** → `UNEXPECTED_SUBTEST_RESULTS`
3. **Test fails at test level (timeout, crash, etc.)** → `UNEXPECTED_FAIL`

#### For Tests WITH Expectation Files:

1. **Matches expectations** → `EXPECTED`
2. **Subtest mismatches** → `UNEXPECTED_SUBTEST_RESULTS`  
3. **Test level mismatches** → `UNEXPECTED_FAIL`, `UNEXPECTED_PASS`, or `UNEXPECTED`

## Code Changes

### 1. Updated Summary Logic

```python
# For tests without expectation files, everything is expected to PASS
if not has_expectations_file:
    if has_unexpected_subtests:
        # Subtests failed but should have passed
        analysis["summary"] = "UNEXPECTED_SUBTEST_RESULTS"
    elif not analysis["test_matches_expectation"]:
        # Test failed when it should have passed
        analysis["summary"] = "UNEXPECTED_FAIL"  
    else:
        # Test passed as expected
        analysis["summary"] = "EXPECTED"
```

### 2. Removed NO_EXPECTATIONS Category

- Removed `NO_EXPECTATIONS` from expectation counts
- Updated filtering logic in summary methods
- Tests without expectations are now properly classified as EXPECTED or UNEXPECTED_*

### 3. Subtest Analysis

The existing subtest logic was already correct:
```python
else:
    # No expectation means it should pass
    analysis["unexpected_subtests"].append(
        {"name": subtest_name, "expected": "PASS", "actual": "FAIL", ...}
    )
```

## Expected Results

### Before Fix:
```
Test without .ini file that fails:
Expected: PASS  
Actual: FAIL
Summary: NO_EXPECTATIONS  ❌ Wrong!
```

### After Fix:
```  
Test without .ini file that fails:
Expected: PASS
Actual: FAIL  
Summary: UNEXPECTED_FAIL  ✅ Correct!
```

### Or if subtests fail:
```
Test without .ini file with failing subtests:
Expected: PASS
Actual: FAIL
Summary: UNEXPECTED_SUBTEST_RESULTS  ✅ Correct!
```

## Impact

This should significantly reduce the number of incorrect expectation classifications:

- **UNEXPECTED_SUBTEST_RESULTS**: Now correctly identifies when subtests fail in tests that should fully pass
- **UNEXPECTED_FAIL**: Now correctly identifies when tests timeout/crash when they should pass  
- **EXPECTED**: Now correctly identifies when tests without expectations actually pass as expected

The classification now properly reflects WPT testing conventions where:
- **No .ini file** = Test should work perfectly (PASS)
- **.ini file present** = Test has known issues (use explicit expectations)

## Testing

Created `test_expectation_logic.py` to verify:
1. ✅ Tests without expectations that PASS → `EXPECTED`
2. ✅ Tests without expectations that FAIL → `UNEXPECTED_SUBTEST_RESULTS` or `UNEXPECTED_FAIL`  
3. ✅ Tests with expectations use explicit expectations → `EXPECTED`
4. ✅ No more `NO_EXPECTATIONS` classifications

This should resolve the expectation analysis issues you reported with the 24 UNEXPECTED_SUBTEST_RESULTS and 25 UNEXPECTED cases.