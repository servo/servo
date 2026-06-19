(function () {
  var status = document.getElementById("script-status");
  var log = document.getElementById("probe-log");

  if (status) {
    status.textContent = "main.js ran";
  }

  if (log) {
    log.textContent += "\nmain.js executed";
    log.textContent += "\nlocation.href = " + location.href;
  }

  console.log("[buttugly-probe] main.js executed");
})();
