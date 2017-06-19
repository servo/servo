// Testing importScripts()
function log(w) { this.postMessage(w) }
function f() { log("FAIL") }
function p() { log("PASS") }

["", "?type=", "?type=x", "?type=x/x"].forEach(function(urlpart) {
  try {
    importScripts("resources/js.py" + urlpart)
  } catch(e) {
    (e.name == "NetworkError") ? p() : log("FAIL (no NetworkError exception): " + urlpart)
  }

})
importScripts("resources/js.py?type=text/javascript&outcome=p")
importScripts("resources/js.py?type=text/ecmascript&outcome=p")
importScripts("resources/js.py?type=text/ecmascript;blah&outcome=p")
log("END")
