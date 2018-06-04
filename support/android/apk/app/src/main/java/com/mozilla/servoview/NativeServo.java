package com.mozilla.servoview;

public class NativeServo {
    public native String version();
    public native void init(String url,
                            WakeupCallback wakeup,
                            ReadFileCallback readfile,
                            ServoCallbacks callbacks,
                            int width, int height, boolean log);
    public native void performUpdates();
    public native void resize(int width, int height);
    public native void reload();
    public native void stop();
    public native void goBack();
    public native void goForward();
    public native void loadUri(String uri);
    public native void scrollStart(int dx, int dy, int x, int y);
    public native void scroll(int dx, int dy, int x, int y);
    public native void scrollEnd(int dx, int dy, int x, int y);
    public native void click(int x, int y);

    NativeServo() {
        System.loadLibrary("c++_shared");
        System.loadLibrary("simpleservo");
    }

    public interface ReadFileCallback {
        byte[] readfile(String file);
    }

    public interface WakeupCallback {
        void wakeup();
    }

    public interface ServoCallbacks {
        void flush();
        void onLoadStarted();
        void onLoadEnded();
        void onTitleChanged(String title);
        void onUrlChanged(String url);
        void onHistoryChanged(boolean canGoBack, boolean canGoForward);
        void onAnimatingChanged(boolean animating);
    }
}
