function createPassFail(condition, test, cleanup, cleanupParam) {
    var div = document.querySelector("#passfail")
    var para = document.createElement("p")
    var pass = document.createElement("button")
    var fail = document.createElement("button")
    var style = "font-family: monospace"
    para.innerHTML = condition
        + ', press the PASS button;'
        + ' otherwise press the FAIL button.',
    pass.innerHTML = "PASS"
    fail.innerHTML = "FAIL"
    pass.setAttribute("style", style)
    fail.setAttribute("style", style)
    pass.addEventListener("click", function () {
        clearPassFail()
        cleanup(cleanupParam)
        test.done()
    }, false)
    fail.addEventListener("click", function () {
        clearPassFail()
        cleanup(cleanupParam)
        test.force_timeout()
        test.set_status(test.FAIL)
        test.done()
    }, false)
    document.body.appendChild(div)
    div.appendChild(para)
    div.appendChild(pass)
    div.appendChild(fail)
}
function clearPassFail() {
    document.querySelector("#passfail").innerHTML = ""
}
function closeNotifications(notifications) {
    for (var i=0; i < notifications.length; i++) {
        notifications[i].close()
    }
}
function hasNotificationPermission() {
    Notification.requestPermission()
    if (Notification.permission != "granted") {
        alert("TEST NOT RUN. Change your browser settings so that"
            + " notifications for this origin are allowed, and then re-run"
            + " this test.")
        return false
    }
    return true
}
