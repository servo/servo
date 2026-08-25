# Command-line utility scripts

Sometimes you may want to add a script to the repository that's meant to be
used from the command line, not from a browser (e.g., a script for generating
test files). If you want to ensure (e.g., for security reasons) that such
scripts won't be handled by the HTTP server, but will instead only be usable
from the command line, then place them in either:

* the `tools` subdir at the root of the repository, or

* the `tools` subdir at the root of any top-level directory in the repository
  which contains the tests the script is meant to be used with

Any files in those `tools` directories won't be handled by the HTTP server;
instead the server will return a 404 if a user navigates to the URL for a file
within them.

If you want to add a script for use with a particular set of tests but there
isn't yet any `tools` subdir at the root of a top-level directory in the
repository containing those tests, you can create a `tools` subdir at the root
of that top-level directory and place your scripts there.

For example, if you wanted to add a script for use with tests in the
`notifications` directory, create the `notifications/tools` subdir and put your
script there.
