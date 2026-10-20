# Android Embedding Guide: ServoView AAR

## 1. Overview

Servo provides a first-class Android embedding library (`servoview`) that packages the Servo rendering engine, JNI bindings, and an Android `SurfaceView` implementation into an Android Archive (`.aar`) artifact.

This allows Android application developers to embed Servo as a drop-in web view alternative without coupling their codebase to `servoshell`.

---

## 2. Architecture

```
+-----------------------------------------------------------+
|                   Android Host Application                |
|                                                           |
|   Activity / Fragment                                     |
|     +-- org.servo.servoview.ServoView (SurfaceView)       |
|           |                                               |
|           +-- org.servo.servoview.Servo (Engine Handle)   |
+-----------------------------+-----------------------------+
                              | JNI
+-----------------------------v-----------------------------+
|                  libservoview.so / libservo.so             |
|                                                           |
|   - Native Activity / EGL Surface Management              |
|   - Webrender GPU Context (OpenGLES 3.0 / ANGLE)          |
|   - Spidermonkey JavaScript Engine & DOM Layout           |
|   - Android Asset / Storage Providers                     |
+-----------------------------------------------------------+
```

---

## 3. Building the AAR Artifact

To build the `servoview` AAR library from source:

1. Configure the Android NDK and SDK:
   ```bash
   export ANDROID_HOME=/path/to/android-sdk
   export NDK_HOME=/path/to/android-ndk
   ```

2. Build the Rust libraries for Android targets:
   ```bash
   ./mach build --android --target aarch64-linux-android --release
   ```

3. Assemble the AAR with Gradle:
   ```bash
   cd support/android/apk
   ./gradlew :servoview:assembleRelease
   ```

The compiled AAR is generated at:
`support/android/apk/servoview/build/outputs/aar/servoview-release.aar`

---

## 4. Consuming the AAR in an Android Application

### 4.1 Dependency Setup
In your application's `app/build.gradle.kts`:
```kotlin
dependencies {
    implementation(files("libs/servoview-release.aar"))
    // Or via Maven/local repository if published:
    // implementation("org.servo:servoview:0.1.0")
}
```

### 4.2 Layout XML
Add `ServoView` directly to your layout:
```xml
<?xml version="1.0" encoding="utf-8"?>
<FrameLayout xmlns:android="http://schemas.android.com/apk/res/android"
    android:layout_width="match_parent"
    android:layout_height="match_parent">

    <org.servo.servoview.ServoView
        android:id="@+id/servo_view"
        android:layout_width="match_parent"
        android:layout_height="match_parent" />

</FrameLayout>
```

### 4.3 Activity Integration
```kotlin
package com.example.browser

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import org.servo.servoview.ServoView

class MainActivity : AppCompatActivity() {
    private lateinit var servoView: ServoView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        servoView = findViewById(R.id.servo_view)
        
        // Initialize and load URL
        servoView.loadUrl("https://servo.org")
    }

    override fun onPause() {
        super.onPause()
        servoView.onPause()
    }

    override fun onResume() {
        super.onResume()
        servoView.onResume()
    }

    override fun onDestroy() {
        super.onDestroy()
        servoView.destroy()
    }
}
```

---

## 5. Security & Isolation on Android

Per `MULTI_PLATFORM_SANDBOX_DESIGN.md`:
- Roadmap, **not current behaviour**: for multi-process rendering, specify `android:isolatedProcess="true"` on the content process service in `AndroidManifest.xml`. Today Servo spawns content processes itself by re-executing `current_exe()` with no profile (see `ARCHITECTURE.md` §8.1), so the isolated-process UID/SELinux isolation described below only exists once that engine change lands — Servo neither creates nor confirms it.
- When that exists, the Android OS enforces UID isolation and an isolated SELinux domain, preventing direct file or network access from the renderer.
