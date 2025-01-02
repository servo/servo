function imageLoadedPromise(image) {
    return new Promise(function(resolve, reject) {
        if (image.complete)
            resolve();
        image.addEventListener("load", resolve, { once: true });
    });
}

function videoLoadedPromise(video) {
    return new Promise(function(resolve, reject) {
        if (video.readyState == 4)
            resolve();
        else {
            video.addEventListener("loadeddata", resolve, { once: true });
            video.addEventListener("error", reject, { once: true });
        }
    });
}

function waitForNFrames(count) {
    if (count <= 0)
         return Promise.reject(new TypeError("count should be greater than 0!"));

    return new Promise(resolve => {
        function tick() {
            (--count) ? requestAnimationFrame(tick) : resolve();
        }
        requestAnimationFrame(tick);
    });
}

function seekTo(video, time) {
    return new Promise(function(resolve, reject) {
        video.addEventListener("seeked", async function() {
            /* Work around flakiness in video players... */
            await waitForNFrames(3);
            resolve();
        }, { once: true });
        video.currentTime = time;
    });
}

function checkBoundingBox(actual, expected, fuzziness) {
    assert_equals(actual.constructor.name, "DOMRectReadOnly");
    assert_approx_equals(actual.left, expected.left, fuzziness);
    assert_approx_equals(actual.right, expected.right, fuzziness);
    assert_approx_equals(actual.top, expected.top, fuzziness);
    assert_approx_equals(actual.bottom, expected.bottom, fuzziness);
}

function checkPointsLieWithinBoundingBox(points, boundingBox) {
    for (point of points) {
        assert_between_inclusive(point.x, boundingBox.left, boundingBox.right);
        assert_between_inclusive(point.y, boundingBox.top, boundingBox.bottom);
    }
}

function checkPointIsNear(actual, expected, fuzzinessX, fuzzinessY) {
    assert_approx_equals(actual.x, expected.x, fuzzinessX);
    assert_approx_equals(actual.y, expected.y, fuzzinessY);
}

function checkPointsAreNear(actual, expected, fuzzinessX, fuzzinessY) {
    for (point of actual)
        checkPointIsNear(point, expected, fuzzinessX, fuzzinessY);
}
