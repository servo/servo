# mypy: allow-untyped-defs

import argparse
from . import browser

latest_channels = {
    'android_weblayer': 'dev',
    'android_webview': 'dev',
    'firefox': 'nightly',
    'chrome': 'nightly',
    'chrome_android': 'dev',
    'chromium': 'nightly',
    'edgechromium': 'dev',
    'safari': 'preview',
    'servo': 'nightly',
    'webkitgtk_minibrowser': 'nightly'
}

channel_by_name = {
    'stable': 'stable',
    'release': 'stable',
    'beta': 'beta',
    'dev': 'dev',
    'canary': 'canary',
    'nightly': latest_channels,
    'preview': latest_channels,
    'experimental': latest_channels,
}

channel_args = argparse.ArgumentParser(add_help=False)
channel_args.add_argument('--channel', choices=channel_by_name.keys(),
                          default='nightly', action='store',
                          help='''
Name of browser release channel (default: nightly). "stable" and "release" are
synonyms for the latest browser stable release; "beta" is the beta release;
"dev" is only meaningful for Chrome (i.e. Chrome Dev); "nightly",
"experimental", and "preview" are all synonyms for the latest available
development or trunk release. (For WebDriver installs, we attempt to select an
appropriate, compatible version for the latest browser release on the selected
channel.) This flag overrides --browser-channel.''')


def get_parser():
    parser = argparse.ArgumentParser(
        parents=[channel_args],
        description="Install a given browser or webdriver frontend.")
    parser.add_argument('browser', choices=['firefox', 'chrome', 'chromium', 'servo', 'safari'],
                        help='name of web browser product')
    parser.add_argument('component', choices=['browser', 'webdriver'],
                        help='name of component')
    parser.add_argument('--download-only', action="store_true",
                        help="Download the selected component but don't install it")
    parser.add_argument('--rename', action="store", default=None,
                        help="Filename, excluding extension for downloaded archive "
                        "(only with --download-only)")
    parser.add_argument('-d', '--destination',
                        help='filesystem directory to place the component')
    parser.add_argument('--revision', default=None,
                        help='Chromium revision to install from snapshots')
    return parser


def get_channel(browser, channel):
    channel = channel_by_name[channel]
    if isinstance(channel, dict):
        channel = channel.get(browser)
    return channel


def run(venv, **kwargs):
    import logging
    logger = logging.getLogger("install")

    browser = kwargs["browser"]
    destination = kwargs["destination"]
    channel = get_channel(browser, kwargs["channel"])

    if channel != kwargs["channel"]:
        logger.info("Interpreting channel '%s' as '%s'", kwargs["channel"], channel)

    if destination is None:
        if venv:
            if kwargs["component"] == "browser":
                destination = venv.path
            else:
                destination = venv.bin_path
        else:
            raise argparse.ArgumentError(None,
                                         "No --destination argument, and no default for the environment")

    if kwargs["revision"] is not None and browser != "chromium":
        raise argparse.ArgumentError(None, "--revision flag cannot be used for non-Chromium browsers.")

    install(browser, kwargs["component"], destination, channel, logger=logger,
            download_only=kwargs["download_only"], rename=kwargs["rename"],
            revision=kwargs["revision"])


def install(name, component, destination, channel="nightly", logger=None, download_only=False,
            rename=None, revision=None):
    if logger is None:
        import logging
        logger = logging.getLogger("install")

    prefix = "download" if download_only else "install"
    suffix = "_webdriver" if component == 'webdriver' else ""

    method = prefix + suffix

    browser_cls = getattr(browser, name.title())
    logger.info('Now installing %s %s...', name, component)
    kwargs = {}
    if download_only and rename:
        kwargs["rename"] = rename
    if revision:
        kwargs["revision"] = revision

    path = getattr(browser_cls(logger), method)(dest=destination, channel=channel, **kwargs)
    if path:
        logger.info('Binary %s as %s', "downloaded" if download_only else "installed", path)
