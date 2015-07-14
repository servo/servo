if (window.testRunner)
    testRunner.dumpAsText();

function log(msg) {
    var txt = document.createTextNode(msg);
    document.querySelector('body').appendChild(txt);
    document.querySelector('body').appendChild(document.createElement('br'));
}

function injectPolicy(policy) {
    var meta = document.createElement('meta');
    meta.setAttribute('http-equiv', 'Content-Security-Policy');
    meta.setAttribute('content', policy);
    document.head.appendChild(meta);
}

if (!document.securityPolicy)
    log('FAIL document.securityPolicy is not defined.')
