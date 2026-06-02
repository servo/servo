window.onload = function() {
    let testWindow;
    if (opener) {
      testWindow = opener.top;
    } else {
      testWindow = top;
    }
    testWindow.postMessage(
        {location: location.href, referrer: document.referrer},
        "*");
}
