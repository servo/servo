import argparse
import browser
import sys

parser = argparse.ArgumentParser()
parser.add_argument('browser', choices=['firefox', 'chrome'],
                    help='name of web browser product')
parser.add_argument('component', choices=['browser', 'webdriver'],
                    help='name of component')
parser.add_argument('-d', '--destination',
                    help='filesystem directory to place the component')

if __name__ == '__main__':
    args = parser.parse_args()

    Subclass = getattr(browser, args.browser.title())
    if args.component == 'webdriver':
        method = 'install_webdriver'
    else:
        method = 'install'

    sys.stdout.write('Now installing %s %s...\n' % (args.browser, args.component))
    getattr(Subclass(), method)(dest=args.destination)
