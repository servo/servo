import argparse
import browser
import sys

def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument('browser', choices=['firefox', 'chrome'],
                        help='name of web browser product')
    parser.add_argument('component', choices=['browser', 'webdriver'],
                        help='name of component')
    parser.add_argument('-d', '--destination',
                        help='filesystem directory to place the component')
    return parser


def run(venv, **kwargs):
    browser = kwargs["browser"]
    destination = kwargs["destination"]

    if destination is None:
        if venv:
            if kwargs["component"] == "browser":
                destination = venv.path
            else:
                destination = venv.bin_path
        else:
            raise argparse.ArgumentError(None,
                                         "No --destination argument, and no default for the environment")

    install(browser, kwargs["component"], destination)


def install(name, component, destination):
    if component == 'webdriver':
        method = 'install_webdriver'
    else:
        method = 'install'

    subclass = getattr(browser, name.title())
    sys.stdout.write('Now installing %s %s...\n' % (name, component))
    getattr(subclass(), method)(dest=destination)


if __name__ == '__main__':
    args = parser.parse_args()
    run(None, **vars(args))
