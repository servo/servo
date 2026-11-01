function runOffsetIframeTest(iframeSrc)
{
    async_test(function(t) {
        const iframe = document.getElementById("iframe");

        window.addEventListener("message", t.step_func(function(event) {
            if (event.data === "observing") {
                waitForNotification(t, function() {
                    iframe.contentWindow.postMessage("report", "*");
                });
                return;
            }
            assert_true(event.data.isIntersecting);
            t.done();
        }));

        iframe.onload = t.step_func(function() {
            waitForNotification(t, function() {
                iframe.contentWindow.postMessage("observe", "*");
            });
        });

        iframe.src = iframeSrc;
    }, "Fully-visible target in an offset iframe is intersecting");
}
