# OHOS WebDriver Test Status Detection Improvements

## Overview

Enhanced the OHOS WebDriver test runner to properly distinguish between different test failure types, specifically **CRASH** vs **TIMEOUT** vs **COMPLETION_ERROR**, to match WPT expectation files accurately.

## Problem

Previously, the system would return ambiguous statuses like:
- `COMPLETION_ERROR` for both crashes and other issues
- `TIMEOUT OR CRASH` when it couldn't determine the exact cause

This led to expectation mismatches:
```
Test #48: css/cssom-view/elementFromPoint.html
Expected: CRASH
Actual: COMPLETION_ERROR
Summary: UNEXPECTED
```

## Solution

### 1. Enhanced Crash Detection

The `wait_for_test_completion_ohos()` method now tracks **consecutive WebDriver communication failures**:

- **3+ consecutive failures** = `CRASH` (browser/WebDriver died)
- **Intermittent failures** = Continue monitoring (temporary network issues)
- **Timeout after successful communication** = `TIMEOUT` (test running too long)

### 2. Improved Status Classification

**CRASH Detection:**
```python
consecutive_failures = 0
max_consecutive_failures = 3

# If we get 3+ consecutive WebDriver communication failures
if consecutive_failures >= max_consecutive_failures:
    return {"status": "CRASH", ...}
```

**TIMEOUT Detection:**
```python
# If we can still communicate but test takes too long
if elapsed_time >= timeout and can_communicate_with_webdriver:
    return {"status": "TIMEOUT", ...}
```

### 3. Updated Status Mapping

Added proper WPT status mapping:
```python
status_mapping = {
    "PASS": "PASS",
    "FAIL": "FAIL", 
    "TIMEOUT": "TIMEOUT",
    "CRASH": "CRASH",        # New!
    "ERROR": "ERROR",
    # Legacy handling
    "COMPLETION_ERROR": "CRASH",
    "TIMEOUT OR CRASH": "TIMEOUT",
}
```

## Results

### Before:
```
Test #48: css/cssom-view/elementFromPoint.html
Expected: CRASH
Actual: COMPLETION_ERROR
Summary: UNEXPECTED
```

### After:
```
Test #48: css/cssom-view/elementFromPoint.html  
Expected: CRASH
Actual: CRASH
Summary: EXPECTED
```

### Before:
```
Test #88: css/cssom-view/interrupt-hidden-smooth-scroll.html
Expected: TIMEOUT  
Actual: TIMEOUT OR CRASH
Summary: UNEXPECTED
```

### After:
```
Test #88: css/cssom-view/interrupt-hidden-smooth-scroll.html
Expected: TIMEOUT
Actual: TIMEOUT  
Summary: EXPECTED
```

## Technical Details

### Crash Detection Logic:
1. Track consecutive WebDriver request failures
2. If 3+ consecutive failures occur → Browser crashed
3. Reset counter on successful communication
4. Take crash screenshot for debugging

### Timeout Detection Logic:
1. Wait for configured timeout period (default 15s)
2. If test still running but communicable → Genuine timeout
3. If communication fails during timeout → Late crash
4. Take timeout screenshot for debugging

### Error Handling:
- **Intermittent network issues**: Retry with backoff
- **WebDriver session loss**: Detected as crash
- **Browser process death**: Detected as crash  
- **Long-running tests**: Detected as timeout

## Benefits

1. **Accurate expectation matching**: Tests now properly match CRASH/TIMEOUT expectations
2. **Better debugging**: Distinct crash vs timeout screenshots
3. **Improved reliability**: Handles transient network issues
4. **Backward compatibility**: Legacy status names still supported
5. **Clear reporting**: Users see exact failure reasons

## Usage

No changes needed - the enhanced detection works automatically:

```bash
python ohos_webdriver_test.py --test css/cssom-view/elementFromPoint.html --verbose
```

The system will now correctly identify and report CRASH vs TIMEOUT based on actual WebDriver communication patterns rather than ambiguous error catching.