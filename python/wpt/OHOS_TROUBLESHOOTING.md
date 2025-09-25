# OHOS WebDriver Test Troubleshooting Guide

## Common Connection Issues

### Error: "No connection could be made because the target machine actively refused it"

This error typically occurs when the WebDriver server connection fails. Here are the potential causes and solutions:

## Prerequisites Check

Before running tests, ensure the following are set up correctly:

### 1. Device Connection
- **OHOS device connected via USB**
- **Device in developer mode** with USB debugging enabled
- **HDC (HarmonyOS Device Connector) installed** and in PATH

Check device connection:
```bash
hdc list targets
```
Should show connected devices, not `[Empty]`.

### 2. HDC Installation
The `hdc` command must be available in your PATH. This comes with the OHOS SDK.

Test HDC:
```bash
hdc --version
```

### 3. Servo WebDriver Server
The Servo application on OHOS must have WebDriver support compiled in and running.

## Execution Order Fix

**Important**: The script now properly sets up HDC port forwarding **before** attempting to connect to the WebDriver server. The corrected order is:

1. Kill existing servo instances
2. **Set up HDC port forwarding** (`hdc fport tcp:7000 tcp:7000`)
3. **Set up HDC reverse forwarding** (`hdc rport tcp:8000 tcp:8000`)
4. Start servo application
5. Create WebDriver session

## Troubleshooting Steps

### Step 1: Check Device Connection
```bash
hdc list targets
```
If empty, check USB connection and device settings.

### Step 2: Check Port Forwarding
The script automatically sets up port forwarding, but you can manually verify:

```bash
# WebDriver port (device -> host)
hdc fport tcp:7000 tcp:7000

# WPT server port (host -> device)  
hdc rport tcp:8000 tcp:8000
```

### Step 3: Check Servo Installation
Verify servo is installed on the OHOS device:
```bash
hdc shell pm list package | grep servo
```
Should show `org.servo.servo`.

### Step 4: Manual WebDriver Test
Try creating a WebDriver session manually:
```bash
curl -X POST http://127.0.0.1:7000/session \
  -H "Content-Type: application/json" \
  -d '{"capabilities": {"alwaysMatch": {"browserName": "servo"}}}'
```

### Step 5: Check Servo Logs
View servo logs on the device:
```bash
hdc shell hilog | grep servo
```

## Running Tests with Diagnostics

Use verbose mode to see detailed connection information:
```bash
python ohos_webdriver_test.py --test css/cssom-view/CaretPosition-001.html --verbose
```

## Port Configuration

Default ports:
- **WebDriver**: 7000
- **WPT Server**: 8000

Change if needed:
```bash
python ohos_webdriver_test.py --test your-test.html --webdriver-port 7001 --wpt-server-port 8001
```

## Common Issues

### "HDC command not found"
- Install OHOS SDK
- Add HDC to system PATH
- Restart terminal

### "No OHOS devices connected"
- Check USB cable
- Enable developer mode on device
- Enable USB debugging
- Trust computer on device

### "Session not created"
- Servo may not have WebDriver support
- Servo may have crashed during startup
- Port forwarding may have failed

### "WPT server failed to start"
- Check if port 8000 is already in use
- Ensure WPT tests directory exists
- Verify Python WPT tools are installed

## Enhanced Error Reporting

The script now provides detailed error information including:
- HDC command output and return codes
- Device connection status
- Port forwarding success/failure details
- WebDriver session creation logs

Use `--verbose` flag to see all diagnostic information.