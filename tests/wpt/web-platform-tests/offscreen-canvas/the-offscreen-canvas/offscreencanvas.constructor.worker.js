// spec link: https://html.spec.whatwg.org/#dom-offscreencanvas

importScripts("/resources/testharness.js");
importScripts("/common/canvas-tests.js");

var t1 = async_test("Test that calling OffscreenCanvas's constructor generates correct width and height.");
t1.step(function() {
    var offscreenCanvas = new OffscreenCanvas(100, 50);
    assert_equals(offscreenCanvas.width, 100);
    assert_equals(offscreenCanvas.height, 50);

    offscreenCanvas.width = 50;
    offscreenCanvas.height = 100;
    assert_equals(offscreenCanvas.width, 50);
    assert_equals(offscreenCanvas.height, 100);
    t1.done();
});

var t2 = async_test("Test that OffscreenCanvas constructor handles invalid arguments correctly in a worker");
t2.step(function() {
    var offscreenCanvas1 = new OffscreenCanvas(1, 1);

    offscreenCanvas1.width = null;
    offscreenCanvas1.height = null;
    assert_equals(offscreenCanvas1.width, 0);
    assert_equals(offscreenCanvas1.height, 0);

    assert_throws(new TypeError(), function() { new OffscreenCanvas(-1, -1); });

    var offscreenCanvas2 = new OffscreenCanvas(null, null);
    assert_equals(offscreenCanvas2.width, 0);
    assert_equals(offscreenCanvas2.height, 0);

    assert_throws(new TypeError(), function() { offscreenCanvas2.width = -1; });
    assert_throws(new TypeError(), function() { offscreenCanvas2.height = -1; });

    var obj = {Name: "John Doe", Age: 30};
    assert_throws(new TypeError(), function() { offscreenCanvas2.width = obj; });
    assert_throws(new TypeError(), function() { offscreenCanvas2.height = obj; });
    assert_throws(new TypeError(), function() { new OffscreenCanvas(obj, obj); });
    t2.done();
});

done();

