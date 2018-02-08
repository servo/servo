// Compares two CSSStyleValues to check if they're the same type
// and have the same attributes.
function assert_style_value_equals(a, b) {
  if (a == null || b == null) {
    assert_equals(a, b);
    return;
  }

  assert_equals(a.constructor.name, b.constructor.name);
  const className = a.constructor.name;
  switch (className) {
    case 'CSSKeywordValue':
      assert_equals(a.value, b.value);
      break;
    case 'CSSUnitValue':
      assert_approx_equals(a.value, b.value, 1e-6);
      assert_equals(a.unit, b.unit);
      break;
    case 'CSSMathSum':
    case 'CSSMathProduct':
    case 'CSSMathMin':
    case 'CSSMathMax':
      assert_style_value_array_equals(a.values, b.values);
      break;
    case 'CSSMathInvert':
    case 'CSSMathNegate':
      assert_style_value_equals(a.value, b.value);
      break;
    case 'CSSUnparsedValue':
      assert_style_value_array_equals(a, b);
      break;
    case 'CSSVariableReferenceValue':
      assert_equals(a.variable, b.variable);
      assert_style_value_equals(a.fallback, b.fallback);
      break;
    case 'CSSPositionValue':
      assert_style_value_equals(a.x, b.x);
      assert_style_value_equals(a.y, b.y);
      break;
    case 'CSSTransformValue':
      assert_style_value_array_equals(a, b);
      break;
    case 'CSSRotate':
      assert_style_value_equals(a.angle, b.angle);
      // fallthrough
    case 'CSSTranslate':
    case 'CSSScale':
      assert_style_value_equals(a.x, b.x);
      assert_style_value_equals(a.y, b.y);
      assert_style_value_equals(a.z, b.z);
      assert_style_value_equals(a.is2D, b.is2D);
      break;
    case 'CSSSkew':
      assert_style_value_equals(a.ax, b.ax);
      assert_style_value_equals(a.ay, b.ay);
      break;
    case 'CSSSkewX':
      assert_style_value_equals(a.ax, b.ax);
      break;
    case 'CSSPerspective':
      assert_style_value_equals(a.length, b.length);
      break;
    case 'CSSMatrixComponent':
      assert_matrix_approx_equals(a.matrix, b.matrix, 1e-6);
      break;
    case 'CSSURLImageValue':
      assert_equals(a.instrinsicWidth, b.instrinsicWidth);
      assert_equals(a.instrinsicHeight, b.instrinsicHeight);
      assert_equals(a.instrinsicRatio, b.instrinsicRatio);
      assert_equals(a.url, b.url);
      break;
    default:
      assert_equals(a, b);
      break;
  }
}

// Compares two arrays of CSSStyleValues to check if every element is equal
function assert_style_value_array_equals(a, b) {
  assert_equals(a.length, b.length);
  for (let i = 0; i < a.length; i++) {
    assert_style_value_equals(a[i], b[i]);
  }
}

const gValidUnits = [
  'number', 'percent', 'em', 'ex', 'ch',
  'ic', 'rem', 'lh', 'rlh', 'vw',
  'vh', 'vi', 'vb', 'vmin', 'vmax',
  'cm', 'mm', 'Q', 'in', 'pt',
  'pc', 'px', 'deg', 'grad', 'rad',
  'turn', 's', 'ms', 'Hz', 'kHz',
  'dpi', 'dpcm', 'dppx', 'fr',
];

// Creates a new div element with specified inline style |cssText|.
// The created element is deleted during test cleanup.
function createDivWithStyle(test, cssText) {
  let element = document.createElement('div');
  element.style = cssText || '';
  document.body.appendChild(element);
  test.add_cleanup(() => {
    element.remove();
  });
  return element;
}

// Creates a new div element with inline style |cssText| and returns
// its inline style property map.
function createInlineStyleMap(test, cssText) {
  return createElementWithInlineStyleMap(test, cssText)[1]
}
// Same as createInlineStyleMap but also returns the element itself.
function createElementWithInlineStyleMap(test, cssText) {
  let elem = createDivWithStyle(test, cssText);
  return [elem, elem.attributeStyleMap];
}

// Creates a new div element with inline style |cssText| and returns
// its computed style property map.
function createComputedStyleMap(test, cssText) {
  return createElementWithComputedStyleMap(test, cssText)[1];
}
// Same as createComputedStyleMap but also returns the element itself.
function createElementWithComputedStyleMap(test, cssText) {
  let elem = createDivWithStyle(test, cssText);
  return [elem, elem.computedStyleMap()];
}

// Creates a new style element with a rule |cssText| and returns
// its declared style property map.
function createDeclaredStyleMap(test, cssText) {
  return createRuleWithDeclaredStyleMap(test, cssText)[1];
}
// Same as createDeclaredStyleMap but also returns the rule itself.
function createRuleWithDeclaredStyleMap(test, cssText) {
  const style = document.createElement('style');
  document.head.appendChild(style);
  const rule = style.sheet.cssRules[style.sheet.insertRule('#test { ' + cssText + '}')];
  test.add_cleanup(() => {
    style.remove();
  });
  return [rule, rule.styleMap];
}

// Creates a new element with background image set to |imageValue|
// and returns a new Image element that can be used to attach
// event listeners regarding the image.
function loadImageResource(test, imageValue) {
  // Set a CSSURLImageValue on an element so it can be loaded.
  let styleMap = createInlineStyleMap(test, '');
  styleMap.set('background-image', imageValue);

  // add a new Image element to know if the image resource has been loaded
  let image = new Image();
  image.src = imageValue.url;
  return image;
}

function assert_matrix_approx_equals(actual, expected, epsilon) {
  assert_array_approx_equals(
      actual.toFloat64Array(), expected.toFloat64Array(), epsilon);
}
