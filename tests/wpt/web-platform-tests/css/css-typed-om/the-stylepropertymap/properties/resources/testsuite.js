const gTestSyntax = {
  '0': {
    description: 'unitless zero',
    set: true,
    examples: [
      new CSSUnitValue(0, 'number'),
    ],
  },
  '<length>': {
    description: 'a length',
    get: true,
    set: true,
    examples: [
      new CSSUnitValue(0, 'px'),
      new CSSUnitValue(-3.14, 'em'),
      new CSSUnitValue(3.14, 'cm'),
    ],
  },
  '<percentage>': {
    description: 'a percent',
    get: true,
    set: true,
    examples: [
      new CSSUnitValue(0, 'percent'),
      new CSSUnitValue(-3.14, 'percent'),
      new CSSUnitValue(3.14, 'percent'),
    ],
  },
  '<time>': {
    description: 'a time',
    get: true,
    set: true,
    examples: [
      new CSSUnitValue(0, 's'),
      new CSSUnitValue(-3.14, 'ms'),
      new CSSUnitValue(3.14, 's'),
    ],
  },
  '<ident>': {
    description: 'a CSSKeywordValue',
    set: true,
    get: true,
    // user-specified examples
    examples: null,
  },
};

function testGet(propertyName, values, description) {
  test(t => {
    let element = createDivWithStyle(t);
    let styleMap = element.attributeStyleMap;

    for (const styleValue of values) {
      element.style[propertyName] = styleValue.toString();

      getComputedStyle(element); // Force a style recalc.
      const result = styleMap.get(propertyName);
      assert_style_value_equals(result, styleValue);
    }
  }, `Can get ${description} from '${propertyName}'`);
}

function testSet(propertyName, values, description) {
  test(t => {
    let element = createDivWithStyle(t);
    let styleMap = element.attributeStyleMap;

    for (const styleValue of values) {
      styleMap.set(propertyName, styleValue);

      getComputedStyle(element); // Force a style recalc.
      assert_equals(element.style[propertyName], styleValue.toString());
    }
  }, `Can set '${propertyName}' to ${description}`);
}

function testSetInvalid(propertyName, values, description) {
  test(t => {
    let element = createDivWithStyle(t);
    let styleMap = element.attributeStyleMap;

    for (const styleValue of values) {
      assert_throws(new TypeError(), () => styleMap.set(propertyName, styleValue));
    }
  }, `Setting '${propertyName}' to ${description} throws TypeError`);
}

function runPropertyTests(propertyName, testCases) {
  let productionsTested = new Set();

  for (const testCase of testCases) {
    const syntax = gTestSyntax[testCase.specified];
    if (!syntax)
      throw new Error(`'${testCase.specified}' is not a valid production`);

    const examples = testCase.examples || syntax.examples;
    if (!examples)
      throw new Error(`'${testCase.specified}' tests require explicit examples`);

    if (syntax.get)
      testGet(propertyName, examples, syntax.description);
    if (syntax.set)
      testSet(propertyName, examples, syntax.description);

    productionsTested.add(testCase.specified);
  }

  // Also test that styleMap.set rejects invalid CSSStyleValues.
  for (const [production, syntax] of Object.entries(gTestSyntax)) {
    if (!productionsTested.has(production)) {
      if (syntax.set && syntax.examples)
        testSetInvalid(propertyName, syntax.examples, syntax.description);
    }
  }
}
