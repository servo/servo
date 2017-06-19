callback = arguments[arguments.length - 1];

function check_done() {
    if (!document.documentElement.classList.contains('reftest-wait')) {
        callback();
    } else {
        setTimeout(check_done, 50);
    }
}

if (document.readyState === 'complete') {
    check_done();
} else {
    addEventListener("load", check_done);
}
