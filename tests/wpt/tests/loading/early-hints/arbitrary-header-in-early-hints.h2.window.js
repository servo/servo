test(() => {
    const test_url = "resources/arbitrary-header-in-early-hints.h2.py";
    window.location.replace(new URL(test_url, window.location));
});
