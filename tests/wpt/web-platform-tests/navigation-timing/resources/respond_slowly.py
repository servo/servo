import time


def main(request, response):
    head = b"""<script>
    let navigationTiming = performance.getEntriesByType('navigation')[0];
    let originalResponseEnd = navigationTiming.responseEnd;
    let originalDuration = navigationTiming.duration;
    function checkResponseEnd() {
        let responseEndDuringLoadEvent = navigationTiming.responseEnd;
        let durationDuringLoadEvent = navigationTiming.duration;
        setTimeout(function() {
            parent.postMessage([
                originalResponseEnd,
                originalDuration,
                responseEndDuringLoadEvent,
                durationDuringLoadEvent,
                navigationTiming.responseEnd,
                navigationTiming.duration], '*');
        }, 0);
    }
    </script><body onload='checkResponseEnd()'>"""
    response.headers.set(b"Content-Length", str(len(head) + 1000))
    response.headers.set(b"Content-Type", b"text/html")
    response.write_status_headers()
    response.writer.write_content(head)
    for i in range(100):
        response.writer.write_content(b"1234567890")
        time.sleep(0.01)
