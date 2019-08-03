import argparse
import browser
import sys


latest_channels = {
    'firefox': 'nightly',
    'chrome': 'dev',
    'chrome_android': 'dev',
    'edgechromium': 'dev',
    'safari': 'preview',
    'servo': 'nightly'
}

channel_by_name = {
    'stable': 'stable',
    'release': 'stable',
    'beta': 'beta',
    'nightly': latest_channels,
    'dev': latest_channels,
    'preview': latest_channels,
    'experimental': latest_channels,
    'canary': 'canary',
}


def get_parser():
    parser = argparse.ArgumentParser(description="""Install a given browser or webdriver frontend.

    For convenience the release channel of the browser accepts various spellings,
    but we actually support at most three variants; whatever the latest development
    release is (e.g. Firefox nightly or Chrome dev), the latest beta release, and
    the most recent stable release.""")
    parser.add_argument('browser', choices=['firefox', 'chrome', 'servo'],
                        help='name of web browser product')
    parser.add_argument('component', choices=['browser', 'webdriver'],
                        help='name of component')
    parser.add_argument('--channel', choices=channel_by_name.keys(),
                        default="nightly", help='Name of browser release channel. '
                        '"stable" and "release" are synonyms for the latest browser stable release,'
                        '"nightly", "dev", "experimental", and "preview" are all synonyms for '
                        'the latest available development release. For WebDriver installs, '
                        'we attempt to select an appropriate, compatible, version for the '
                        'latest browser release on the selected channel.')
    parser.add_argument('-d', '--destination',
                        help='filesystem directory to place the component')
    return parser


def get_channel(browser, channel):
    channel = channel_by_name[channel]
    if isinstance(channel, dict):
        channel = channel.get(browser)
    return channel


def run(venv, **kwargs):
    browser = kwargs["browser"]
    destination = kwargs["destination"]
    channel = get_channel(browser, kwargs["channel"])

    if channel != kwargs["channel"]:
        print("Interpreting channel '%s' as '%s'" % (kwargs["channel"],
                                                     channel))

    if destination is None:
        if venv:
            if kwargs["component"] == "browser":
                destination = venv.path
            else:
                destination = venv.bin_path
        else:
            raise argparse.ArgumentError(None,
                                         "No --destination argument, and no default for the environment")

    install(browser, kwargs["component"], destination, channel)


def install(name, component, destination, channel="nightly", logger=None):
    if logger is None:
        import logging
        logger = logging.getLogger("install")

    if component == 'webdriver':
        method = 'install_webdriver'
    else:
        method = 'install'

    subclass = getattr(browser, name.title())
    sys.stdout.write('Now installing %s %s...\n' % (name, component))
    path = getattr(subclass(logger), method)(dest=destination, channel=channel)
    if path:
        sys.stdout.write('Binary installed as %s\n' % (path,))
