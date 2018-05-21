package com.mozilla.servo;
import android.annotation.TargetApi;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.ActivityInfo;
import android.content.pm.PackageInfo;
import android.os.Bundle;
import android.os.Environment;
import android.os.Handler;
import android.os.PowerManager;
import android.preference.PreferenceManager;
import android.util.Log;
import android.view.SurfaceView;
import android.view.View;
import android.view.WindowManager;
import android.webkit.URLUtil;
import android.widget.FrameLayout;

import com.mozilla.servo.BuildConfig;

import org.json.JSONException;
import org.json.JSONObject;

import java.io.BufferedInputStream;
import java.io.BufferedReader;
import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.PrintStream;
import java.lang.System;
import java.util.Enumeration;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;


public class MainActivity extends android.app.NativeActivity {
    private static final String LOGTAG = "Servo";
    private boolean mFullScreen = false;
    private static final String PREF_KEY_RESOURCES_SYNC = "res_sync_v";

    static {
        Log.i(LOGTAG, "Loading the NativeActivity");

        // Libaries should be loaded in reverse dependency order
        System.loadLibrary("c++_shared");
        System.loadLibrary("servo");
    }

    @Override
    public void onCreate(Bundle savedInstanceState) {
        final Intent intent = getIntent();
        if (intent != null && intent.getAction().equals(Intent.ACTION_VIEW)) {
            final String url = intent.getDataString();
            if (url != null && URLUtil.isValidUrl(url)) {
                Log.d(LOGTAG, "Received url "+url);
                set_url(url);
            }
        }

        JSONObject preferences = loadPreferences();

        boolean keepScreenOn = false;

        if (BuildConfig.FLAVOR.contains("vr")) {
            // Force fullscreen mode and keep screen on for VR experiences.
            keepScreenOn = true;
            mFullScreen = true;
        }
        else {
            keepScreenOn = preferences.optBoolean("shell.keep_screen_on.enabled", false);
            mFullScreen = !preferences.optBoolean("shell.native-titlebar.enabled", false);

            String orientation = preferences.optString("shell.native-orientation", "both");

            // Handle orientation preference
            if (orientation.equalsIgnoreCase("portrait")) {
                setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_PORTRAIT);
            }
            else if (orientation.equalsIgnoreCase("landscape")) {
                setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE);
            }
        }

        super.onCreate(savedInstanceState);

        // NativeActivity ignores the Android view hierarchy because it’s designed
        // to take over the surface from the window to directly draw to it.
        // Inject a custom SurfaceView in order to support adding views on top of the browser.
        // (e.g. Native Banners, Daydream GVRLayout or other native views)
        getWindow().takeSurface(null);
        FrameLayout layout = new FrameLayout(this);
        layout.setLayoutParams(new FrameLayout.LayoutParams(FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT));
        SurfaceView nativeSurface = new SurfaceView(this);
        nativeSurface.getHolder().addCallback(this);
        layout.addView(nativeSurface, new FrameLayout.LayoutParams(FrameLayout.LayoutParams.MATCH_PARENT, FrameLayout.LayoutParams.MATCH_PARENT));
        setContentView(layout);

        // Handle keep screen on preference
        if (keepScreenOn) {
            keepScreenOn();
        }

        // Handle full screen preference
        if (mFullScreen) {
            addFullScreenListener();
        }
    }

    @Override
    protected void onStop() {
        Log.d(LOGTAG, "onStop");
        super.onStop();
    }

    @Override
    protected void onPause() {
        Log.d(LOGTAG, "onPause");
        super.onPause();
    }

    @Override
    protected void onResume() {
        Log.d(LOGTAG, "onPause");
        if (mFullScreen) {
            setFullScreen();
        }
        super.onResume();
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus && mFullScreen) {
            setFullScreen();
        }
    }

    // keep the device's screen turned on and bright.
    private void keepScreenOn() {
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
    }

    // Dim toolbar and make the view fullscreen
    private void setFullScreen() {
        int flags = View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                    | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                    | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                    | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION // Hides navigation bar
                    | View.SYSTEM_UI_FLAG_FULLSCREEN; // Hides status bar
        if( android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.KITKAT) {
            flags |= getImmersiveFlag();
        } else {
            flags |= View.SYSTEM_UI_FLAG_LOW_PROFILE;
        }
        getWindow().getDecorView().setSystemUiVisibility(flags);
    }

    @TargetApi(19)
    private int getImmersiveFlag() {
        return View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY;
    }

    private void addFullScreenListener() {
        View decorView = getWindow().getDecorView();
        decorView.setOnSystemUiVisibilityChangeListener(
            new View.OnSystemUiVisibilityChangeListener() {
                public void onSystemUiVisibilityChange(int visibility) {
                    if ((visibility & View.SYSTEM_UI_FLAG_FULLSCREEN) == 0) {
                        setFullScreen();
                    }
                }
            });
    }

    private String loadAsset(String file) {
        InputStream is = null;
        BufferedReader reader = null;
        try {
            is = getAssets().open(file);
            reader = new BufferedReader(new InputStreamReader(is));
            StringBuilder result = new StringBuilder();
            String line;
            while ((line = reader.readLine()) != null) {
                result.append(line).append('\n');
            }
            return result.toString();
        } catch (IOException e) {
            Log.e(LOGTAG, Log.getStackTraceString(e));
            return null;
        }
        finally {
            try {
                if (reader != null) {
                    reader.close();
                }
                if (is != null) {
                    is.close();
                }
            } catch (Exception e) {
                Log.e(LOGTAG, Log.getStackTraceString(e));
            }
        }
    }

    private JSONObject loadPreferences() {
        String json = loadAsset("prefs.json");
        try {
            return new JSONObject(json);
        } catch (JSONException e) {
            Log.e(LOGTAG, Log.getStackTraceString(e));
            return new JSONObject();
        }
    }

    private File getAppDataDir() {
        File file = getExternalFilesDir(null);
        return file != null ? file : getFilesDir();
    }

    private void set_url(String url) {
        try {
            File file = new File(getAppDataDir() + "/android_params");
            if (!file.exists()) {
                file.createNewFile();
            }
            PrintStream out = new PrintStream(new FileOutputStream(file, false));
            out.println("# The first line here should be the \"servo\" argument (without quotes) and the");
            out.println("# last should be the URL to load.");
            out.println("# Blank lines and those beginning with a '#' are ignored.");
            out.println("# Each line should be a separate parameter as would be parsed by the shell.");
            out.println("# For example, \"servo -p 10 http://en.wikipedia.org/wiki/Rust\" would take 4");
            out.println("# lines (the \"-p\" and \"10\" are separate even though they are related).");
            out.println("servo");
            out.println("-w");
            String absUrl = url.replace("file:///storage/emulated/0/", "/sdcard/");
            out.println(absUrl);
            out.flush();
            out.close();
        } catch (Exception e) {
            Log.e(LOGTAG, Log.getStackTraceString(e));
        }
    }
}
