document._log = [];
window.addEventListener("error", function (ev) {
    document._log.push(ev.error.name);
});
window.addEventListener("load", function () {
    document._log = document._log.join(",");
});
