// META: title=tests for navigator.userAgentData

test(t => {
  const brands = navigator.userAgentData.brands;
  assert_true(brands.every(brand => brand.brand.length < 32),
    "No brand should be longer than 32 characters.");
});
