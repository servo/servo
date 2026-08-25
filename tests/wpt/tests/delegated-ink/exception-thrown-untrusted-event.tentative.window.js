let presenter = navigator.ink.requestPresenter();
let style = { color: "red", diameter: 3 };
let evt = new PointerEvent("pointerdown", {clientX: 10, clientY: 10});
presenter.then( function(p) {
  test(() => {
    assert_throws_dom("NotAllowedError", function() {
      p.updateInkTrailStartPoint(evt, style);
    }, "NotAllowedError is expected due to untrusted event.");
  }, "Expected a NotAllowedError to be thrown due to untrusted event.");
})