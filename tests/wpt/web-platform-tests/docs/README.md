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
3. Install the Python dependencies. Run the following command in a terminal
   from the "docs" directory of the WPT repository:

       pip install -r requirements.txt

4. Build the documentation. Windows users should execute the `make.bat` batch
   file. GNU/Linux and macOS users should use the `make` command.
