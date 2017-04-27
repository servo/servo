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
import android.view.View;
import android.view.WindowManager;
import android.webkit.URLUtil;

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
        try {
            extractAssets();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }

        final Intent intent = getIntent();
        if (intent != null && intent.getAction().equals(Intent.ACTION_VIEW)) {
            final String url = intent.getDataString();
            if (url != null && URLUtil.isValidUrl(url)) {
                Log.d(LOGTAG, "Received url "+url);
                set_url(url);
            }
        }

        JSONObject preferences = loadPreferences();
        boolean keepScreenOn = preferences.optBoolean("shell.keep_screen_on.enabled", false);
        mFullScreen = !preferences.optBoolean("shell.native-titlebar.enabled", false);
        String orientation = preferences.optString("shell.native-orientation", "both");

        // Handle orientation preference
        if (orientation.equalsIgnoreCase("portrait")) {
            setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_PORTRAIT);
        }
        else if (orientation.equalsIgnoreCase("landscape")) {
            setRequestedOrientation(ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE);
        }

        super.onCreate(savedInstanceState);

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

    private boolean needsToExtractAssets(String path) {
        SharedPreferences prefs = PreferenceManager.getDefaultSharedPreferences(this);
        int version = BuildConfig.VERSION_CODE;

        if (!new File(path).exists()) {
            // Assets folder doesn't exist, resources need to be copied
            prefs.edit().putInt(PREF_KEY_RESOURCES_SYNC, version).apply();
            return true;
        }

        if (version != prefs.getInt(PREF_KEY_RESOURCES_SYNC, -1)) {
            // Also force a reextract when the version changes and the resources may be updated
            // This can be improved by generating a hash or version number of the resources
            // instead of using version code of the app
            prefs.edit().putInt(PREF_KEY_RESOURCES_SYNC, version).apply();
            return true;
        }
        return false;
    }

    private File getAppDataDir() {
        File file = getExternalFilesDir(null);
        return file != null ? file : getFilesDir();
    }
    /**
     * extracts assets/ in the APK to /sdcard/servo.
     */
    private void extractAssets() throws IOException {
        String path = getAppDataDir().getAbsolutePath();
        if (!needsToExtractAssets(path)) {
            return;
        }

        ZipFile zipFile = null;
        File targetDir = new File(path);
        try {
            zipFile = new ZipFile(this.getApplicationInfo().sourceDir);
            for (Enumeration<? extends ZipEntry> e = zipFile.entries(); e.hasMoreElements(); ) {
                ZipEntry entry = e.nextElement();
                if (entry.isDirectory() || !entry.getName().startsWith("assets/")) {
                    continue;
                }
                File targetFile = new File(targetDir, entry.getName().substring("assets/".length()));
                targetFile.getParentFile().mkdirs();
                byte[] tempBuffer = new byte[(int)entry.getSize()];
                BufferedInputStream is = null;
                FileOutputStream os = null;
                try {
                    is = new BufferedInputStream(zipFile.getInputStream(entry));
                    os = new FileOutputStream(targetFile);
                    is.read(tempBuffer);
                    os.write(tempBuffer);
                } finally {
                    try {
                        if (is != null) {
                            is.close();
                        }
                        if (os != null) {
                            os.close();
                        }
                    } catch (Exception ex) {
                        Log.e(LOGTAG, Log.getStackTraceString(ex));
                    }
                }
            }
        } finally {
            try {
                if (zipFile != null) {
                    zipFile.close();
                }
            } catch (Exception e) {
                Log.e(LOGTAG, Log.getStackTraceString(e));
            }
        }
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
