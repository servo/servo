function errorHandler(ev)
{
    document._errorReported.push("error");
}

document._errorReported = [];
window.addEventListener("error", errorHandler);
window.addEventListener("load", function () {
    document._errorReported = document._errorReported.join(",");
});
