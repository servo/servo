# Servo Browser (for Windows) — Installation & Packaging Guide

## Overview

**Servo Browser (for Windows)** is a fast, memory-safe, lightweight standalone web browser powered by the Servo web engine.

---

## 1. Quick Installation (PowerShell)

To install Servo Browser into your user profile with Desktop shortcuts, Start Menu integration, and Windows Default Browser support:

```powershell
# From the repository root or extracted package directory:
powershell -ExecutionPolicy Bypass -File .\support\packaging\windows\Install-ServoBrowser.ps1
```

### Installation Features:
1. **Target Directory:** `%LOCALAPPDATA%\Programs\ServoBrowser`
2. **Desktop Shortcut:** Creates `Servo Browser.lnk` on your Desktop.
3. **Start Menu Shortcut:** Creates `Servo Browser` in Start Menu.
4. **Command-Line Access:** Registers in `App Paths` so you can launch it via `Win + R` -> `servobrowser`.
5. **Windows Settings Integration:** Registers under `StartMenuInternet` and `RegisteredApplications` for HTTP/HTTPS associations, allowing you to choose Servo Browser as your Default Browser in Windows 10 & 11 Settings.
6. **Add/Remove Programs:** Adds an entry in Windows Settings > Installed Apps for one-click uninstallation.

---

## 2. Silent / Automated Installation

For scripted deployments or CI:

```powershell
powershell -ExecutionPolicy Bypass -File .\support\packaging\windows\Install-ServoBrowser.ps1 -Silent
```

---

## 3. Uninstallation

To uninstall cleanly:
* Either use Windows Settings > **Installed Apps** > **Servo Browser** > **Uninstall**.
* Or run:
  ```powershell
  powershell -ExecutionPolicy Bypass -File .\support\packaging\windows\Uninstall-ServoBrowser.ps1
  ```

---

## 4. Building the Standalone Setup Installer (.exe)

If you have [Inno Setup](https://jrsoftware.org/isdl.php) installed:

```powershell
iscc .\support\packaging\windows\ServoBrowser_Installer.iss
```

This compiles a single standalone executable:
`target\installer\Setup_ServoBrowser_v0.7.0.exe`

---

## 5. Command-Line Options

Servo Browser supports standard browser arguments:

```bash
# Open URL
servobrowser https://servo.org

# Private Browsing / Incognito mode (in-memory isolated storage)
servobrowser --incognito https://example.com

# Load an unpacked WebExtension (Manifest v2 / v3)
servobrowser --load-extension=C:\path\to\extension

# Origin Isolation (quarantine subdomains to separate event loops)
servobrowser --origin-isolation https://example.com
```
