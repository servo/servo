async function createTranslator(options = {}) {
  if (!options.monitor) {
    const availability = await Translator.availability(options);
    assert_implements_optional(
        availability !== 'unavailable',
        'Translator is not available for the given options');
  }
  await test_driver.bless();
  return await Translator.create(options);
}
