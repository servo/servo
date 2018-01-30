print = function(o) {
    console.log(o);
    if (window.dump) {
      window.dump(o + '\n');
    }
}

function formatLine(name, t) {
  print("[PERF]," + name + "," + t);
}

function printPerfTiming() {
  print("[PERF] perf block start")
  formatLine("testcase", window.location);
  formatLine("title", document.title.replace(/,/g, "&#44;"));
  formatLine("navigationStart", performance.timing.navigationStart);
  formatLine("unloadEventStart", performance.timing.unloadEventStart);
  formatLine("unloadEventEnd", performance.timing.unloadEventEnd);
  formatLine("redirectStart", performance.timing.redirectStart);
  formatLine("redirectEnd", performance.timing.redirectEnd);
  formatLine("fetchStart", performance.timing.fetchStart);
  formatLine("domainLookupStart", performance.timing.domainLookupStart);
  formatLine("domainLookupEnd", performance.timing.domainLookupEnd);
  formatLine("connectStart", performance.timing.connectStart);
  formatLine("connectEnd", performance.timing.connectEnd);
  formatLine("secureConnectionStart", performance.timing.secureConnectionStart);
  formatLine("requestStart", performance.timing.requestStart);
  formatLine("responseStart", performance.timing.responseStart);
  formatLine("responseEnd", performance.timing.responseEnd);
  formatLine("domLoading", performance.timing.domLoading);
  formatLine("domInteractive", performance.timing.domInteractive);
  formatLine("domContentLoadedEventStart", performance.timing.domContentLoadedEventStart);
  formatLine("domContentLoadedEventEnd", performance.timing.domContentLoadedEventEnd);
  formatLine("domComplete", performance.timing.domComplete);
  formatLine("loadEventStart", performance.timing.loadEventStart);
  formatLine("loadEventEnd", performance.timing.loadEventEnd);
  print("[PERF] perf block end")
}

if (document.readyState === "complete") { 
    printPerfTiming()
    window.close();
} else {
    window.addEventListener('load', function () {
	window.setTimeout(printPerfTiming, 0);
    });
    var timeout = 5;
    window.setTimeout(function() {
        print("[PERF] Timeout after " + timeout + " min. Force stop");
        printPerfTiming();
        window.close();
    }, timeout * 60 * 1000)
}
