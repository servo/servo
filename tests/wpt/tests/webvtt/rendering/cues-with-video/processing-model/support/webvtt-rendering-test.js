window.addEventListener('load', async event => {
    if (!document.documentElement.classList.contains('reftest-wait'))
        return;

    let waitFor = (object, type) => {
        return new Promise(resolve => {
            object.addEventListener(type, resolve);
        }, { once: true });
    };

    let trackElement = document.querySelector('video > track[default]');
    if (!trackElement)
        return;

    if (trackElement.track.mode !== 'showing')
        trackElement.track.mode = 'showing';
    if (!trackElement.track.activeCues)
        await waitFor(trackElement.track, 'cuechange');

    document.documentElement.classList.remove('reftest-wait');
});