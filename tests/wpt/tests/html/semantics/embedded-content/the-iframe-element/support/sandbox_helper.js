function IsSandboxSupported() {
    if ('sandbox' in document.createElement('iframe')) {
        return true;
    }
    return false;
}

function DisableTestForNonSupportingBrowsers() {
    //check if sandbox is supported by the browser
    if (!IsSandboxSupported()) {
        document.getElementById('testframe').innerHTML = "FAIL: Your browser does not support the sandbox attribute on the iframe element.";
        document.getElementById('testframe').style.color = "Red";
    }
}
