if (window.testRunner) {
    testRunner.dumpAsText();
    testRunner.waitUntilDone();
}

function appendObjectElement(type) {
    window.onload = function() {
        var o = document.createElement('object');
        o.setAttribute('type', 'application/x-webkit-test-netscape');
        o.addEventListener('load', function() {
            console.log('FAIL: The object should have been blocked.');
            if (window.testRunner)
                testRunner.notifyDone();
        });
        o.addEventListener('error', function() {
            console.log('PASS: Error occurred, so load was correctly blocked.');
            if (window.testRunner)
                testRunner.notifyDone();
        });

        var p = document.createElement('param');
        p.setAttribute('value', 'http://127.0.0.1:8080/plugins/resources/mock-plugin.pl?' + type);
        p.setAttribute('name', type);

        o.appendChild(p);

        document.body.appendChild(o);
    };
}
