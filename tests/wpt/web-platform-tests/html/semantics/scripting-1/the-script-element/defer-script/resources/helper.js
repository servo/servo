window.result = [];
function log(msg) {
  window.result.push(msg);
}
function checkIfReachedBodyEnd() {
  const endelement = document.getElementById("bodyend");
  // `<pre id="bodyend">End</pre>` is needed at the end of HTML.
  if (endelement && endelement.textContent === "End") {
    log("EndOfBody");
    endelement.textContent = "Detected";
  }
}
function logScript(msg) {
  checkIfReachedBodyEnd();
  log(msg);
}
document.addEventListener("DOMContentLoaded", function() { logScript("DOMContentLoaded"); });
