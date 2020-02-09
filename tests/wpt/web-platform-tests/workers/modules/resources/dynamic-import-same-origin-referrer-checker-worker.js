import('./referrer-checker.py')
    .catch(error => postMessage(`Import failed: ${error}`));
