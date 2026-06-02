async function createTranslator(options) {
  await test_driver.bless();
  return await Translator.create(options);
}
