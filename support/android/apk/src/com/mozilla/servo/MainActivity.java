package com.mozilla.servo;
import android.app.NativeActivity;
import android.content.Intent;
import android.os.Bundle;
import android.util.Log;
import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileNotFoundException;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.PrintStream;
import java.lang.System;
import java.util.Enumeration;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;


public class MainActivity extends android.app.NativeActivity {
    private static final String LOGTAG="servo_wrapper";
    static {
        Log.i(LOGTAG, "Loading the NativeActivity");
        System.loadLibrary("main");
    }

    private void set_url(String url) {
        try {
            PrintStream out = new PrintStream(new FileOutputStream("/sdcard/servo/android_params"));
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
        } catch (FileNotFoundException e) {
        }
    }

    @Override
    public void onCreate(Bundle savedInstanceState) {
        if (needsToExtractAssets()) {
            try {
                extractAssets();
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        }
        super.onCreate(savedInstanceState);
        final Intent intent = getIntent();
        if (intent.getAction().equals(Intent.ACTION_VIEW)) {
            final String url = intent.getDataString();
            Log.d(LOGTAG, "Received url "+url);
            set_url(url);
        }
    }

    @Override
    protected void onStop() {
        super.onStop();  // Always call the superclass method first

        Log.d(LOGTAG, "got onStop; finishing servo activity");
        finish();

        // Glutin and the Java wrapper libraries that we use currently do not support restoring
        // Servo after Android has sent it to the background, as the resources were reclaimed.
        // Until we either address that in glutin or move to a library that supports recreating
        // the native resources after being restored, we just forcibly shut Servo down when it
        // is sent to the background.
        int pid = android.os.Process.myPid();
        android.os.Process.killProcess(pid);
        System.exit(0);
    }

    private boolean needsToExtractAssets() {
        // todo: also force a reextract when the resources are updated.
        return !(new File("/sdcard/servo").exists());
    }

    /**
     * extracts assets/ in the APK to /sdcard/servo.
     */
    private void extractAssets() throws IOException {
        ZipFile zipFile = null;
        File targetDir = new File("/sdcard/servo");
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
                    if (is != null) is.close();
                    if (os != null) os.close();
                }
            }
        } finally {
            if (zipFile != null) zipFile.close();
        }
    }
}
