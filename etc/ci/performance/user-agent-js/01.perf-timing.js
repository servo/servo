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
  var entries = performance.getEntriesByName(window.location);
  for (entry in entries) {
    for (key in entries[entry]) {
      if (typeof entries[entry][key] === "number") {
	formatLine(key, entries[entry][key]);
      }
    }
  }
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
