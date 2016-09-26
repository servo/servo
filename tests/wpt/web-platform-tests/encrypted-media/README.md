## Preparing to run tests
The following steps may be necessary when running test from a new server/origin for the first time.
* Some implementations and/or tests may require consent.
  When running on such clients, manually run a test to trigger the consent request and choose to persist the consent.
* Some of the tests, such as *-retrieve-*, use pop-ups.
  It may be necessary to run these tests manually and choose to always allow pop-ups on the origin.
  Related test failures may appear as: "Cannot set property 'onload' of undefined"
