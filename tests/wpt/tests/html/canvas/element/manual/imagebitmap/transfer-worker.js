addEventListener('message', evt => {
    postMessage(evt.data, [evt.data.bitmap]);
});
