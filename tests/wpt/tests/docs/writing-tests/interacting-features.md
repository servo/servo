# Testing interactions between features

Web platform features do not exist in isolation. Often, testing the interaction between two features is necessary in tests.
To support this, many directories contain libraries which are intended to be used from other directories.

These are not WPT server features, but are canonical usage of one feature intended for other features to test against.
This allows the tests for a feature to be decoupled as much as possible from the specifics of another feature which it should integrate with.

## Web Platform Feature Testing Support Libraries

### Common

There are several useful utilities in the `/common/` directory

### Cookies

Features which need to test their interaction with cookies can use the scripts in `cookies/resources` to control which cookies are set on a given request.

### Permissions Policy

Features which integrate with Permissions Policy can make use of the `permissions-policy.js` support library to generate a set of tests for that integration.

### Reporting

Testing integration with the Reporting API can be done with the help of the common report collector. This service will collect reports sent from tests and provides an API to retrieve them. See documentation at `reporting/resources/README.md`.
