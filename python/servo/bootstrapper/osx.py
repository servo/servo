# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function, unicode_literals

import os
import re
import subprocess
import sys
import tempfile
import traceback
try:
    from urllib2 import urlopen
except ImportError:
    from urllib.request import urlopen

from distutils.version import StrictVersion

from base import BaseBootstrapper
from packages import OSX_BREW, OSX_PORT

XCODE_APP_STORE = 'macappstore://itunes.apple.com/app/id497799835?mt=12'

HOMEBREW_BOOTSTRAP = 'https://raw.githubusercontent.com/Homebrew/install/master/install'

MACPORTS_URL = {'11': 'https://distfiles.macports.org/MacPorts/MacPorts-2.3.4-10.11-ElCapitan.pkg',
                '10': 'https://distfiles.macports.org/MacPorts/MacPorts-2.3.4-10.10-Yosemite.pkg',
                '9': 'https://distfiles.macports.org/MacPorts/MacPorts-2.3.4-10.9-Mavericks.pkg',
                '8': 'https://distfiles.macports.org/MacPorts/MacPorts-2.3.4-10.8-MountainLion.pkg',
                '7': 'https://distfiles.macports.org/MacPorts/MacPorts-2.3.4-10.7-Lion.pkg', }

MACPORTS_CLANG_PACKAGE = 'clang-3.3'

RE_CLANG_VERSION = re.compile('Apple (?:clang|LLVM) version (\d+\.\d+)')

APPLE_CLANG_MINIMUM_VERSION = StrictVersion('4.2')

XCODE_REQUIRED = '''
Xcode is required to build Servo. Please complete the install of Xcode
through the App Store.

It's possible Xcode is already installed on this machine but it isn't being
detected. This is possible with developer preview releases of Xcode, for
example. To correct this problem, run:

  `xcode-select --switch /path/to/Xcode.app`.

e.g. `sudo xcode-select --switch /Applications/Xcode.app`.
'''

XCODE_REQUIRED_LEGACY = '''
You will need to download and install Xcode to build Servo.

Please complete the Xcode download and then relaunch this script.
'''

XCODE_NO_DEVELOPER_DIRECTORY = '''
xcode-select says you don't have a developer directory configured. We think
this is due to you not having Xcode installed (properly). We're going to
attempt to install Xcode through the App Store. If the App Store thinks you
have Xcode installed, please run xcode-select by hand until it stops
complaining and then re-run this script.
'''

XCODE_COMMAND_LINE_TOOLS_MISSING = '''
The Xcode command line tools are required to build Servo.
'''

INSTALL_XCODE_COMMAND_LINE_TOOLS_STEPS = '''
Perform the following steps to install the Xcode command line tools:

    1) Open Xcode.app
    2) Click through any first-run prompts
    3) From the main Xcode menu, select Preferences (Command ,)
    4) Go to the Download tab (near the right)
    5) Install the "Command Line Tools"

When that has finished installing, please relaunch this script.
'''

UPGRADE_XCODE_COMMAND_LINE_TOOLS = '''
An old version of the Xcode command line tools is installed. You will need to
install a newer version in order to compile Servo. If Xcode itself is old,
its command line tools may be too old even if it claims there are no updates
available, so if you are seeing this message multiple times, please update
Xcode first.
'''

PACKAGE_MANAGER_INSTALL = '''
We will install the %s package manager to install required packages.

You will be prompted to install %s with its default settings. If you
would prefer to do this manually, hit CTRL+c, install %s yourself, ensure
"%s" is in your $PATH, and relaunch bootstrap.
'''

PACKAGE_MANAGER_PACKAGES = '''
We are now installing all required packages via %s. You will see a lot of
output as packages are built.
'''

PACKAGE_MANAGER_OLD_CLANG = '''
We require a newer compiler than what is provided by your version of Xcode.

We will install a modern version of Clang through %s.
'''

PACKAGE_MANAGER_CHOICE = '''
Please choose a package manager you'd like:
1. Homebrew
2. MacPorts
Your choice:
'''

NO_PACKAGE_MANAGER_WARNING = '''
It seems you don't have any supported package manager installed.
'''

PACKAGE_MANAGER_EXISTS = '''
Looks like you have %s installed. We will install all required packages via %s.
'''

MULTI_PACKAGE_MANAGER_EXISTS = '''
It looks like you have multiple package managers installed.
'''

# May add support for other package manager on os x.
PACKAGE_MANAGER = {'Homebrew': 'brew',
                   'MacPorts': 'port'}

PACKAGE_MANAGER_CHOICES = ['Homebrew', 'MacPorts']

PACKAGE_MANAGER_BIN_MISSING = '''
A package manager is installed. However, your current shell does
not know where to find '%s' yet. You'll need to start a new shell
to pick up the environment changes so it can be found.

Please start a new shell or terminal window and run this
bootstrapper again.

If this problem persists, you will likely want to adjust your
shell's init script (e.g. ~/.bash_profile) to export a PATH
environment variable containing the location of your package
manager binary. e.g.

    export PATH=/usr/local/bin:$PATH
'''

BAD_PATH_ORDER = '''
Your environment's PATH variable lists a system path directory (%s)
before the path to your package manager's binaries (%s).
This means that the package manager's binaries likely won't be
detected properly.

Modify your shell's configuration (e.g. ~/.profile or
~/.bash_profile) to have %s appear in $PATH before %s. e.g.

    export PATH=%s:$PATH

Once this is done, start a new shell (likely Command+T) and run
this bootstrap again.
'''


class OSXBootstrapper(BaseBootstrapper):
    def __init__(self, version, **kwargs):
        BaseBootstrapper.__init__(self, **kwargs)

        self.os_version = StrictVersion(version)

        self.packages_port = OSX_PORT
        self.packages_brew = OSX_BREW
        if self.os_version >= StrictVersion('10.11'):
            self.packages_brew += ["openssl"]

        print(self.packages_brew)

        if self.os_version < StrictVersion('10.7'):
            raise Exception('OS X 10.7 or above is required.')

        self.minor_version = version.split('.')[1]

        self.package_manager = self.ensure_package_manager()

        if self.package_manager == "homebrew":
            self.brew = self.which('brew')
        elif self.package_manager == "macports":
            self.port = self.which('port')
        else:
            sys.exit("No selected package manager")

    def ensure_system_packages(self):
        self.install_system_packages()

    def install_system_packages(self):
        self.ensure_xcode()
        self._update_package_manager()

        getattr(self, 'ensure_%s_system_packages' % self.package_manager)()

    def install_mobile_android_packages(self):
        raise NotImplementedError('Bootstrap support for Android not yet available.')

    def install_virtualenv(self):
        self.install_system_packages()

        if self.package_manager == "homebrew":
            self.run(['pip', 'install', 'virtualenv'])

    def ensure_xcode(self):
        # OS X 10.7 have Xcode come from the app store. However, users can
        # still install Xcode into any arbitrary location. We honor the
        # location of Xcode as set by xcode-select. This should also pick up
        # developer preview releases of Xcode, which can be installed into
        # paths like /Applications/Xcode5-DP6.app.
        select = self.which('xcode-select')
        try:
            output = self.check_output([select, '--print-path'],
                                       stderr=subprocess.STDOUT)
        except subprocess.CalledProcessError as e:
            # This seems to appear on fresh OS X machines before any Xcode
            # has been installed. It may only occur on OS X 10.9 and later.
            if b'unable to get active developer directory' in e.output:
                print(XCODE_NO_DEVELOPER_DIRECTORY)
                self._install_xcode_app_store()
                assert False  # Above should exit.

            output = e.output

        # This isn't the most robust check in the world. It relies on the
        # default value not being in an application bundle, which seems to
        # hold on at least Mavericks.
        if b'.app/' not in output:
            print(XCODE_REQUIRED)
            self._install_xcode_app_store()
            assert False  # Above should exit.

        # Once Xcode is installed, you need to agree to the license before you can
        # use it.
        try:
            output = self.check_output(['/usr/bin/xcrun', 'clang'],
                                       stderr=subprocess.STDOUT)
        except subprocess.CalledProcessError as e:
            if b'license' in e.output:
                xcodebuild = self.which('xcodebuild')
                try:
                    subprocess.check_call([xcodebuild, '-license'],
                                          stderr=subprocess.STDOUT)
                except subprocess.CalledProcessError as e:
                    if b'requires admin privileges' in e.output:
                        self.run_as_root([xcodebuild, '-license'])

        # Even then we're not done! We need to install the Xcode command line tools.
        # As of Mountain Lion, apparently the only way to do this is to go through a
        # menu dialog inside Xcode itself. We're not making this up.
        if not os.path.exists('/usr/bin/clang'):
            print(XCODE_COMMAND_LINE_TOOLS_MISSING)
            print(INSTALL_XCODE_COMMAND_LINE_TOOLS_STEPS)
            sys.exit(1)

        output = self.check_output(['/usr/bin/clang', '--version'])
        match = RE_CLANG_VERSION.search(output)
        if match is None:
            raise Exception('Could not determine Clang version.')

        version = StrictVersion(match.group(1))

        if version < APPLE_CLANG_MINIMUM_VERSION:
            print(UPGRADE_XCODE_COMMAND_LINE_TOOLS)
            print(INSTALL_XCODE_COMMAND_LINE_TOOLS_STEPS)
            sys.exit(1)

    def _install_xcode_app_store(self):
        subprocess.check_call(['open', XCODE_APP_STORE])
        print('Once the install has finished, please relaunch this script.')
        sys.exit(1)

    def _ensure_homebrew_packages(self, packages, extra_brew_args=[]):
        self.brew = self.which('brew')
        assert self.brew is not None
        cmd = [self.brew] + extra_brew_args

        installed = self.check_output(cmd + ['list']).split()

        printed = False

        for package in packages:
            if package in installed:
                continue

            if not printed:
                print("\nInstalling required packages via Homebrew...\n")
                printed = True

            print("Installing %s..." % package, end="")
            self.run(cmd + ['reinstall' if self.force else 'install', package])
            if package == "openssl":
                print("Linking %s..." % package, end="")
                self.run(cmd + ['link', '--force', package])

        return printed

    def ensure_homebrew_system_packages(self):
        packages = self.packages_brew

        self._ensure_homebrew_packages(packages)

    def _ensure_macports_packages(self, packages):
        self.port = self.which('port')
        assert self.port is not None

        installed = set(self.check_output([self.port, 'installed']).split())

        missing = [package for package in packages if package not in installed]
        if missing:
            print("\nInstalling required packages via MacPorts...\n")
            self.run_as_root([self.port, '-v', 'install'] + missing)

    def ensure_macports_system_packages(self):
        packages = self.packages_port

        self._ensure_macports_packages(packages)
        self.run_as_root([self.port, 'select', '--set', 'python', 'python27'])

    def ensure_package_manager(self):
        '''
        Search package mgr in sys.path, if none is found, prompt the user to install one.
        If only one is found, use that one. If both are found, prompt the user to choose
        one.
        '''
        # Read file with selected default package manager
        default_manager_file = os.path.join(os.path.abspath(os.curdir), ".servo", "default_package_manager")
        print(default_manager_file)
        default_manager = ""
        if os.path.exists(default_manager_file):
            default_manager = open(default_manager_file).read().strip()

        print(default_manager)

        installed = []
        for name, cmd in PACKAGE_MANAGER.iteritems():
            if default_manager == cmd and self.which(cmd) is not None:
                installed = [name]
                break
            elif self.which(cmd) is not None:
                installed.append(name)

        active_name, active_cmd = None, None

        if not installed:
            print(NO_PACKAGE_MANAGER_WARNING)
            choice = self.prompt_int(prompt=PACKAGE_MANAGER_CHOICE, low=1, high=2)
            active_name = PACKAGE_MANAGER_CHOICES[choice - 1]
            active_cmd = PACKAGE_MANAGER[active_name]
            getattr(self, 'install_%s' % active_name.lower())()
        elif len(installed) == 1:
            print(PACKAGE_MANAGER_EXISTS % (installed[0], installed[0]))
            active_name = installed[0]
            active_cmd = PACKAGE_MANAGER[active_name]
        else:
            print(MULTI_PACKAGE_MANAGER_EXISTS)
            choice = self.prompt_int(prompt=PACKAGE_MANAGER_CHOICE, low=1, high=2)

            active_name = PACKAGE_MANAGER_CHOICES[choice - 1]
            active_cmd = PACKAGE_MANAGER[active_name]

        # Ensure the active package manager is in $PATH and it comes before
        # /usr/bin. If it doesn't come before /usr/bin, we'll pick up system
        # packages before package manager installed packages and the build may
        # break.
        p = self.which(active_cmd)
        if not p:
            print(PACKAGE_MANAGER_BIN_MISSING % active_cmd)
            sys.exit(1)

        p_dir = os.path.dirname(p)
        for path in os.environ['PATH'].split(os.pathsep):
            if path == p_dir:
                break

            for check in ('/bin', '/usr/bin'):
                if path == check:
                    print(BAD_PATH_ORDER % (check, p_dir, p_dir, check, p_dir))
                    sys.exit(1)

        # Write default package manager to file
        try:
            if not os.path.exists(".servo"):
                os.makedirs(".servo")

            with open(default_manager_file, 'w') as dm:
                dm.write(active_cmd)
        except:
            traceback.print_exc()
            pass

        return active_name.lower()

    def install_homebrew(self):
        print(PACKAGE_MANAGER_INSTALL % ('Homebrew', 'Homebrew', 'Homebrew', 'brew'))
        bootstrap = urlopen(url=HOMEBREW_BOOTSTRAP, timeout=20).read()
        with tempfile.NamedTemporaryFile() as tf:
            tf.write(bootstrap)
            tf.flush()

            self.run(['ruby', tf.name])

    def install_macports(self):
        url = MACPORTS_URL.get(self.minor_version, None)
        if not url:
            raise Exception('We do not have a MacPorts install URL for your '
                            'OS X version. You will need to install MacPorts manually.')

        print(PACKAGE_MANAGER_INSTALL % ('MacPorts', 'MacPorts', 'MacPorts', 'port'))
        pkg = urlopen(url=url, timeout=300).read()
        with tempfile.NamedTemporaryFile(suffix='.pkg') as tf:
            tf.write(pkg)
            tf.flush()

            self.run_as_root(['installer', '-pkg', tf.name, '-target', '/'])

    def _update_package_manager(self):
        print("Updating package manager...")
        if self.package_manager == 'homebrew':
            self.run([self.brew, '-v', 'update'])
        else:
            assert self.package_manager == 'macports'
            self.run_as_root([self.port, 'selfupdate'])

    def _upgrade_package(self, package):
        self._ensure_package_manager_updated()

        if self.package_manager == 'homebrew':
            try:
                subprocess.check_output([self.brew, '-v', 'upgrade', package],
                                        stderr=subprocess.STDOUT)
            except subprocess.CalledProcessError as e:
                if b'already installed' not in e.output:
                    raise
        else:
            assert self.package_manager == 'macports'

            self.run_as_root([self.port, 'upgrade', package])

    def prompt_int(self, prompt, low, high, limit=5):
        ''' Prompts the user with prompt and requires an integer between low and high. '''
        valid = False
        while not valid and limit > 0:
            try:
                choice = int(raw_input(prompt))
                if not low <= choice <= high:
                    print("ERROR! Please enter a valid option!")
                    limit -= 1
                else:
                    valid = True
            except ValueError:
                print("ERROR! Please enter a valid option!")
                limit -= 1

        if limit > 0:
            return choice
        else:
            raise Exception("Error! Reached max attempts of entering option.")
