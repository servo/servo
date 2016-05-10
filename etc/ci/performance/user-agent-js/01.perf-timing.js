function formatLine(name, t){
  var output = "[PERF]," + name + "," + t;
  console.log(output);
  //document.getElementById('timing').innerHTML += output + "<br/>";
}

function printPerfTiming(){
  console.log("[PERF] perf block start")
  formatLine("testcase", window.location);
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
  console.log("[PERF] perf block end")
}
window.addEventListener('load', printPerfTiming);
