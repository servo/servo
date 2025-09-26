window.changeEventPromise = function changeEventPromise(preference, t) {
    return Promise.race([
        new Promise(resolve => {
            navigator.preferences[preference].onchange = resolve;
        }),
        new Promise((resolve, reject) => {
            t.step_timeout(() => {
                reject(`Change event for ${preference} preference not fired.`);
            }, 500);
        })
    ]);
}