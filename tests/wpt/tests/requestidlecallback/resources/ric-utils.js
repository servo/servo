function getDeadlineForNextIdleCallback() {
    return new Promise(
        resolve =>
            requestIdleCallback(deadline => resolve(deadline.timeRemaining()))
    );
}

function getPendingRenderDeadlineCap() {
    return 1000 / 60;
}

function getRICRetryCount() {
    return 10;
}