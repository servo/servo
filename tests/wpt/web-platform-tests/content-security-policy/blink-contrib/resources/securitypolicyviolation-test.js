window.jsTestIsAsync = true;

document.addEventListener('securitypolicyviolation', function handleEvent(e) {
    window.e = e;
    for (key in expectations)
        shouldBe('window.e.' + key, JSON.stringify(expectations[key]));
    finishJSTest();
});

window.addEventListener('load', function() {
    debug('Kicking off the tests:');
    run();
});
