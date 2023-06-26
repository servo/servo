window.assertSpeculationRulesIsSupported = () => {
  assert_implements(
      'supports' in HTMLScriptElement,
      'HTMLScriptElement.supports must be supported');
  assert_implements(
      HTMLScriptElement.supports('speculationrules'),
      '<script type="speculationrules"> must be supported');
};
