# Windows GPU & Windowed Rendering Hardware Validation

## 1. Context & Rationale

CI for Windows runners typically executes in headless / software emulation modes without access to physical GPU hardware or native desktop compositors (Desktop Window Manager / DWM). As a result, critical hardware paths—such as DirectX/ANGLE surface presentation, WGL context creation, high-DPI scaling, and multi-monitor swapchains—require dedicated validation.

This document establishes the hardware verification checklist and test harness for Windows embedders.

---

## 2. Test Matrix

| ID | Category | Test Case | Target Configuration | Expected Outcome |
| :--- | :--- | :--- | :--- | :--- |
| **GPU-01** | Context Creation | WGL OpenGL Core Profile | Intel UHD / NVIDIA / AMD | Native GL context initializes without software fallback |
| **GPU-02** | Context Creation | ANGLE EGL Direct3D 11 | Windows 10/11 D3D11 Feature Level 11_0 | EGL display initializes via ANGLE backend |
| **GPU-03** | Presentation | Native Window (`HWND`) Composition | 60Hz/120Hz/144Hz displays with DWM enabled | Tear-free presentation synchronized with VSync |
| **GPU-04** | Offscreen (OSR) | FBO to DXGI Shared Handle | Headless / embedded DirectComposition | Shared texture handle resolves in parent swapchain |
| **DPI-01** | High DPI | Per-Monitor V2 DPI awareness | 100%, 150%, 200% Windows display scaling | Web layout resizes cleanly; device pixel ratio scales |
| **DPI-02** | High DPI | Dynamic monitor drag | Moving window between 100% and 200% displays | WM_DPICHANGED triggers layout relayout without blur |
| **INP-01** | Input | Windows Pointer Messages | Touchscreen / Precision Touchpad | Smooth pinch-to-zoom and kinetic inertia scrolling |

---

## 3. Manual Hardware Validation Checklist

Execute the following verification steps on physical Windows hardware before major releases:

### Step 1: Run Servoshell in Windowed Mode
```cmd
target\release\servoshell.exe --pref dom.webgl.enabled=true https://webglsamples.org/aquarium/aquarium.html
```
- Verify: FPS counter maintains 60+ FPS on discrete GPU.
- Verify: GPU engine shows "3D" / "GPU 0" active in Task Manager Performance tab.

### Step 2: High-DPI Scaling Test
```cmd
target\release\servoshell.exe https://devicepixelratio.com
```
- Move window between monitors with different scale factors (e.g. 100% vs 175%).
- Verify that text remains crisp and devicePixelRatio updates dynamically.

### Step 3: Offscreen Rendering Test
```cmd
target\release\servoshell.exe --headless --screenshot output.png https://servo.org
```
- Inspect `output.png` to verify pixel-perfect software/FBO rendering.

---

## 4. Hardware Verification Automation Script

Run `etc/ci/windows_hw_check.py` on a hardware test runner to log GPU adapter details, driver versions, and display modes:

```cmd
python etc\ci\windows_hw_check.py
```
