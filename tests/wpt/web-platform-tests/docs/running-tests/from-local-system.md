# Running Tests from the Local System

The tests are designed to be run from your local computer.

## System Setup

Running the tests requires `python`, `pip` and `virtualenv`, as well as updating
the system `hosts` file.

Note that Python 2.7 is required. Using Python 3 is not supported.

The required setup is different depending on your operating system.

### Linux Setup

If not already present, use the system package manager to install `python`,
`pip` and `virtualenv`.

On Debian or Ubuntu:

```bash
sudo apt-get install python python-pip virtualenv
```

### macOS Setup

The system-provided Python can be used, while `pip` and `virtualenv` can be
installed for the user only:

```bash
python -m ensurepip --user
export PATH="$PATH:$HOME/Library/Python/2.7/bin"
pip install --user virtualenv
```

To make the `PATH` change persistent, add it to your `~/.bash_profile` file or
wherever you currently set your PATH.

See also [additional setup required to run Safari](safari.md).

### Windows Setup

Download and install [Python 2.7](https://www.python.org/downloads). The
installer includes `pip` by default.

Add `C:\Python27` and `C:\Python27\Scripts` to your `%Path%`
[environment variable](http://www.computerhope.com/issues/ch000549.htm).

Finally, install `virtualenv`:

```bash
pip install virtualenv
```

The standard Windows shell requires that all `wpt` commands are prefixed
by the Python binary i.e. assuming `python` is on your path the server is
started using:

```bash
python wpt serve
```

#### Windows Subsystem for Linux

Optionally on Windows you can use the [Windows Subsystem for
Linux](https://docs.microsoft.com/en-us/windows/wsl/about) (WSL). If doing so,
installation and usage are similar to the Linux instructions. Be aware that WSL
may attempt to override `/etc/hosts` each time it is launched, which would then
require you to re-run [`hosts` File Setup](#hosts-file-setup). This behavior
[can be configured](https://docs.microsoft.com/en-us/windows/wsl/wsl-config#network).

### `hosts` File Setup

To get the tests running, you need to set up the test domains in your
[`hosts` file](http://en.wikipedia.org/wiki/Hosts_%28file%29%23Location_in_the_file_system).

On Linux, macOS or other UNIX-like system:

```bash
./wpt make-hosts-file | sudo tee -a /etc/hosts
```

And on Windows (this must be run in a PowerShell session with Administrator privileges):

```
python wpt make-hosts-file | Out-File $env:SystemRoot\System32\drivers\etc\hosts -Encoding ascii -Append
```

If you are behind a proxy, you also need to make sure the domains above are
excluded from your proxy lookups.

## Via the browser

The test environment can then be started using

    ./wpt serve

This will start HTTP servers on two ports and a websockets server on
one port. By default the web servers start on ports 8000 and 8443 and the other
ports are randomly-chosen free ports. Tests must be loaded from the
*first* HTTP server in the output. To change the ports,
create a `config.json` file in the wpt root directory, and add
port definitions of your choice e.g.:

```
{
  "ports": {
    "http": [1234, "auto"],
    "https":[5678]
  }
}
```

After your `hosts` file is configured, the servers will be locally accessible at:

http://web-platform.test:8000/<br>
https://web-platform.test:8443/ *

To use the web-based runner point your browser to:

http://web-platform.test:8000/tools/runner/index.html<br>
https://web-platform.test:8443/tools/runner/index.html *

This server has all the capabilities of the publicly-deployed version--see
[Running the Tests from the Web](from-web.md).

\**See [Trusting Root CA](../tools/certs/README.md)*

## Via the command line

Many tests can be automatically executed in a new browser instance using

    ./wpt run [browsername] [tests]

This will automatically load the tests in the chosen browser and extract the
test results. For example to run the `dom/historical.html` tests in a local
copy of Chrome:

    ./wpt run chrome dom/historical.html

Or to run in a specified copy of Firefox:

    ./wpt run --binary ~/local/firefox/firefox firefox dom/historical.html

For details on the supported products and a large number of other options for
customising the test run:

    ./wpt run --help

[A complete listing of the command-line arguments is available
here](command-line-arguments.md).

```eval_rst
.. toctree::
   :hidden:

   command-line-arguments
```

Additional browser-specific documentation:

```eval_rst
.. toctree::

  chrome
  chrome_android
  android_webview
  safari
  webkitgtk_minibrowser
```

For use in continuous integration systems, and other scenarios where regression
tracking is required, the command-line interface supports storing and loading
the expected result of each test in a test run. See [Expectations
Data](../../tools/wptrunner/docs/expectation) for more information on creating
and maintaining these files.
