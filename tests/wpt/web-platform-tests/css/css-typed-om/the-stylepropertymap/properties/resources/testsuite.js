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

function testGetSet(propertyName, values, description) {
  testGet(propertyName, values, description);
  testSet(propertyName, values, description);
}

function runPropertyTests(propertyName, testCases) {
  for (const testCase of testCases) {
    if (testCase.specified == '0') {
      testSet(propertyName, [
        new CSSUnitValue(0, 'number'),
      ], 'unitless zero');
    } else if (testCase.specified === '<length>') {
      testGetSet(propertyName, [
        new CSSUnitValue(0, 'px'),
        new CSSUnitValue(-3.14, 'em'),
        new CSSUnitValue(3.14, 'cm'),
      ], 'a length CSSUnitValue');
    } else if (testCase.specified == '<percentage>') {
      testGetSet(propertyName, [
        new CSSUnitValue(0, 'percent'),
        new CSSUnitValue(-3.14, 'percent'),
        new CSSUnitValue(3.14, 'percent'),
      ], 'a percent CSSUnitValue');
    } else if (testCase.specified == '<ident>') {
      if (!testCase.examples) {
        throw new Error('<ident> tests require examples');
      }

      testGetSet(propertyName, testCase.examples,
        'a CSSKeywordValue');
    }
  }
}
