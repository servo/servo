This folder contains tests and support files for smoke testing popular plugins against the current pytest version.

The objective is to gauge if any intentional or unintentional changes in pytest break plugins.

As a rule of thumb, we should add plugins here:

1. That are used at large. This might be subjective in some cases, but if answer is yes to
   the question: *if a new release of pytest causes pytest-X to break, will this break a ton of test suites out there?*.
2. That don't have large external dependencies: such as external services.

Besides adding the plugin as dependency, we should also add a quick test which uses some
minimal part of the plugin, a smoke test. Also consider reusing one of the existing tests if that's
possible.
