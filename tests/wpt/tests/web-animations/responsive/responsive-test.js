class ResponsiveTest {
  constructor(target, property, keyframes) {
    this.property = property;
    this.target = target;
    this.duration = 1000;
    this.anim = target.animate(keyframes, this.duration);
    this.anim.pause();
  }

  get ready() {
    return new Promise(resolve => {
      this.anim.ready.then(resolve);
    });
  }

  set underlyingValue(value) {
    this.target.style[this.property] = value;
  }

  set inheritedValue(value) {
    this.target.parentElement.style[this.property] = value;
  }

  // The testCases are of the form:
  // [{at: <fractional_progress>, is: <computed style> }, ...]
  assertResponsive(testCases) {
    for (let i = 0; i < testCases.length; i++) {
      const testCase = testCases[i];
      this.anim.currentTime = this.duration * testCase.at;
      assert_equals(getComputedStyle(this.target)[this.property], testCase.is,
                    `${this.property} at ${testCase.at}`);
    }
  }
}

// Creates a test that allows setting the underlying style of the target
// element or its parent.
// Options are of the form:
//   property: required property in camelcase form as used in the
//   web animation API.
//   from: optional starting keyframe as a string.
//   to: optional ending keyframe as a string.
function createResponsiveTest(test, options) {
  const parent = document.createElement('div');
  const target = document.createElement('div');
  document.body.appendChild(parent);
  parent.appendChild(target);
  const property = options.property;
  const keyframes = [];
  const createKeyframe = (value) => {
    const keyframe = {};
    keyframe[property] = value;
    return keyframe;
  }
  if (options.from) {
    keyframes.push(createKeyframe(options.from));
  }
  if (options.to) {
    keyframes.push(createKeyframe(options.to));
  }
  test.add_cleanup(() => {
    parent.remove();
  });
  return new ResponsiveTest(target, property, keyframes);
}
