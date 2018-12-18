let appendScript = (src, resolve) => {
    const script = document.createElement('script');
    script.type = 'text/javascript';
    script.src = src;
    script.onload = resolve;
    document.body.appendChild(script);
}

let xhrScript = src => {
    var xhr = new XMLHttpRequest();
    xhr.open("GET", src, false);
    xhr.send(null);
}

let waitForNextTask = () => {
    return new Promise(resolve => {
        step_timeout(resolve, 0);
    });
};

let waitForEventToFire = () => {
    return new Promise(resolve => {
        let waitForIt = function() {
            if (eventFired) {
                eventFired = false;
                resolve();
            } else {
                step_timeout(waitForIt, 0);
            }
        }
        step_timeout(waitForIt, 0);
    });
};

let clearBufferAndSetSize = size => {
    performance.clearResourceTimings();
    performance.setResourceTimingBufferSize(size);
}

let fillUpTheBufferWithSingleResource = src => {
    return new Promise(resolve => {
        // This resource gets buffered in the resource timing entry buffer.
        appendScript(src, resolve);
    });
};

let loadResource = src => {
    return new Promise(resolve => {
        appendScript(src, resolve);
    });
};

let fillUpTheBufferWithTwoResources = async src => {
    // These resources get buffered in the resource timing entry buffer.
    await loadResource(src);
    await loadResource(src + '?second');
};

