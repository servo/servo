'use strict';

const installMayFinish = new Promise(resolve => {
    self.finishInstall = resolve;
});

let report = { installEventFired: false };

addEventListener('install', event => {
    report.installEventFired = true;
    let attemptUpdate = registration.update().catch(exception => {
        report.exception = exception.name;
    });
    event.waitUntil(Promise.all([installMayFinish, attemptUpdate]));
});

addEventListener('message', event => {
    if (event.data === 'finishInstall') {
        finishInstall();
    } else {
        event.source.postMessage(report);
    }
});
