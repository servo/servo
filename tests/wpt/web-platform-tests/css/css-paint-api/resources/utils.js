
// Register a property, and interpolate its value to the halfway point.
function registerAndInterpolateProperty(options) {
  CSS.registerProperty({
    name: options.name,
    syntax: `${options.syntax} | none`,
    initialValue: 'none',
    inherits: false
  });
  let animation = options.on.animate([
    { [options.name]: options.from },
    { [options.name]: options.to }
  ], 1000);
  animation.currentTime = 500;
  animation.pause();
}

// Apply a paint worklet to 'target' which verifies that the worklet-side value
// of a set of properties is what we expect.
//
// The 'expected' parameter is an object where each key is the name of a
// property to check, and each corresponding value is an array with the expected
// (serialized) values for that property.
function expectWorkletValues(target, expected) {
  const workletName = 'registered-property-value';

  // Wrap any single values in an array. This makes it possible to omit the
  // array if there is only one value.
  const ensureArray = x => x.constructor === Array ? x : [x];
  expected = Object.entries(expected).map(([k, v]) => [k, ensureArray(v)])
                                     .map(x => ({[x[0]]: x[1]}))
                                     .reduce((a, b) => Object.assign(a, b), {});

  target.style.setProperty('width', '100px');
  target.style.setProperty('height', '100px');
  target.style.setProperty('background-image', `paint(${workletName})`);

  const worklet = `
    const expectedData = ${JSON.stringify(expected)};
    const expectedKeys = Object.keys(expectedData).sort();
    registerPaint('${workletName}', class {
      static get inputProperties() { return expectedKeys; }
      paint(ctx, geom, styleMap) {
        let serialize = (v) => '[' + v.constructor.name + ' ' + v.toString() + ']';
        let actual = expectedKeys.map(k => styleMap.getAll(k).map(serialize).join(', ')).join(' | ');
        let expected = expectedKeys.map(k => expectedData[k].join(', ')).join(' | ');
        ctx.strokeStyle = (actual === expected) ? 'green' : 'red';
        ctx.lineWidth = 4;
        ctx.strokeRect(0, 0, geom.width, geom.height);
      }
    });`

  importWorkletAndTerminateTestAfterAsyncPaint(CSS.paintWorklet, worklet);
}

// Like expectWorkletValues, but can only test a single property.
function expectWorkletValue(target, property, expected) {
  expectWorkletValues(target, { [property]: expected });
}
