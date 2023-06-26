function assert_barProps(barPropObjects, visible) {
  let lastBarProp = undefined;
  for (const currentBarProp of barPropObjects) {
    assert_not_equals(currentBarProp, lastBarProp, "BarBrop objects of different properties are identical");
    assert_equals(currentBarProp.visible, visible, "a BarProp's visible is wrong");
    lastBarProp = currentBarProp;
  }
}

function assert_identical_barProps(barProps, w, oldBarPropObjects, visible) {
  barProps.map(val => w[val]).map((val, index) => {
    assert_equals(val, oldBarPropObjects[index], "BarProp identity not preserved");
  });
  assert_barProps(oldBarPropObjects, visible);
}

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe")),
        frameW = frame.contentWindow,
        barProps = ["locationbar", "menubar", "personalbar", "scrollbars", "statusbar", "toolbar"],
        barPropObjects = barProps.map(val => frameW[val]);

  assert_barProps(barPropObjects, true);
  frame.remove();
  assert_identical_barProps(barProps, frameW, barPropObjects, false);
  t.step_timeout(() => {
    assert_identical_barProps(barProps, frameW, barPropObjects, false);
    t.done();
  }, 0);
}, "BarBrop objects of a nested Window");

async_test(t => {
  const openee = window.open("/common/blank.html"),
        barProps = ["locationbar", "menubar", "personalbar", "scrollbars", "statusbar", "toolbar"],
        barPropObjects = barProps.map(val => openee[val]);

  // This is used to demonstrate that the Document is replaced while the global object (not the
  // global this object) stays the same
  openee.tiedToGlobalObject = openee.document;

  assert_barProps(barPropObjects, true);
  openee.onload = t.step_func(() => {
    assert_own_property(openee, "tiedToGlobalObject");
    assert_not_equals(openee.tiedToGlobalObject, openee.document);

    assert_identical_barProps(barProps, openee, barPropObjects, true);

    openee.onunload = t.step_func(() => {
      assert_identical_barProps(barProps, openee, barPropObjects, true);
      t.step_timeout(() => {
        assert_identical_barProps(barProps, openee, barPropObjects, false);
        t.done();
      }, 0);
    });

    openee.close();
    assert_identical_barProps(barProps, openee, barPropObjects, true);
  });
}, "BarProp objects of an auxiliary Window");
