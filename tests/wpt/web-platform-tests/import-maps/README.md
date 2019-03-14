Tests for [Import Maps](https://github.com/WICG/import-maps).

Because the spec itself is still under development and there are ongoing spec
discussions, the tests are all tentative.

Also, some tests are based on Chromium's behavior which reflects an older
version of import maps spec ("package name maps" around May 2018), and have
dependency to Chromium's implementation (internals.resolveModuleSpecifier).
These dependencies should be removed, once the spec matures.
