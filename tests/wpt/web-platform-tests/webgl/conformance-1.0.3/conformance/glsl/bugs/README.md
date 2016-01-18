BUGS
====

This folder is for GLSL tests that test driver specific bugs.

Most tests in other folders are fairly generic. While they might
only fail on specific drivers the tests themselves are designed
to test something in a generic way.

Tests in this folder on the otherhand are very targeted. They may
have very specific shaders that only fail under specific circumstances
on specific drivers.

An example might be if there was a driver that failed only when
and identifier was named "ABC". It makes no sense to have a generic
test that says "must allow ABC". A generic test would test some
subset of all possible identifiers not just one.

