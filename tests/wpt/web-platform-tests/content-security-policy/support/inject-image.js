// This script block will trigger a violation report.
var i = document.createElement('img');
i.src = '/content-security-policy/support/fail.png';
document.body.appendChild(i);
log("TEST COMPLETE");