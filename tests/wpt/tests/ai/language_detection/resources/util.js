async function createLanguageDetector(options = {}) {
  await test_driver.bless();
  return await LanguageDetector.create(options);
}
