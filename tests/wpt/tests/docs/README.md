# Project documentation tooling

The documentation for the web-platform-tests project is built using [the Sphinx
documentation generator](http://www.sphinx-doc.org). [The GitHub Actions
service](https://github.com/features/actions) is configured to automatically
update the public website each time changes are merged to the repository.

## Local Development

If you would like to build the site locally, follow these instructions.

1. Install the system dependencies. The free and open source software tools
   [Python](https://www.python.org/) and [Git](https://git-scm.com/) are
   required. Each website has instructions for downloading and installing on a
   variety of systems.
2. Download the source code. Clone this repository using the `git clone`
   command.
3. Install the Python dependencies. Run the following command in a terminal:

       pip install -r docs/requirements.txt

4. Build the documentation:

       ./wpt build-docs
