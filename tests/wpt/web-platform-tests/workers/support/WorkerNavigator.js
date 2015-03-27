var obj = new Object();
obj.appName    = navigator.appName;
obj.appVersion = navigator.appVersion;
obj.platform   = navigator.platform;
obj.userAgent  = navigator.userAgent;
obj.onLine     = navigator.onLine;

postMessage(obj);
