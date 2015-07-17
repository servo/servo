if (window.testRunner) {
    testRunner.dumpAsText();
    testRunner.dumpChildFramesAsText();
    testRunner.setXSSAuditorEnabled(true);
    testRunner.waitUntilDone();
}

function testMixedHeader(csp, xssProtection) {
    var params = [
        'q=<script>alert_assert(String.fromCharCode(0x58,0x53,0x53))<' + '/script>'
    ];
    if (csp != 'unset')
        params.push('csp=' + csp);

    if (xssProtection == 'allow')
        params.push('disable-protection=1');
    if (xssProtection == 'block')
        params.push('enable-full-block=1');
    if (xssProtection == 'filter')
        params.push('valid-header=2');
    if (xssProtection == 'invalid')
        params.push('malformed-header=1');

    var url = '/security/xssAuditor/resources/echo-intertag.pl?';
    url += params.join('&amp;');

    document.write('<p>Testing behavior when "reflected-xss" is set to ' + csp + ', and "X-XSS-Protection" is set to ' + xssProtection + '.');
    document.write('<iframe src="' + url + '"></iframe>');
}

function frameLoaded() {
    var frame = document.querySelector('iframe');
    try {
        alert_assert('Loaded ' + frame.contentWindow.location.href + ' into the IFrame.');
    } catch (e) {
        alert_assert('Loaded cross-origin frame.');
    }
    testRunner.notifyDone();
}

window.onload = frameLoaded;
