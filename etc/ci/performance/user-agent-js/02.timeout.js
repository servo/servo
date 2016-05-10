var timeout = 5;
window.setTimeout(function(){
  console.log("[PERF] Timeout after " + timeout + " min. Force stop");
  printPerfTiming();
  window.close();
}, timeout * 60 * 1000)
