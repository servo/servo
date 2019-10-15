// Import a remote origin script.
import('https://{{domains[www1]}}:{{ports[https][0]}}/workers/modules/resources/referrer-checker.py')
    .catch(error => postMessage(`Import failed: ${error}`));
