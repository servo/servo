import os
from setuptools import setup, find_packages


VERSION = '0.0.1'

install_requires = [
]

here = os.path.dirname(os.path.abspath(__file__))
# get documentation from the README and HISTORY
try:
    with open(os.path.join(here, 'README.rst')) as doc:
        readme = doc.read()
except:
    readme = ''

try:
    with open(os.path.join(here, 'HISTORY.rst')) as doc:
        history = doc.read()
except:
    history = ''

long_description = readme + '\n\n' + history

if __name__ == '__main__':
    setup(
        name='servo_tidy',
        version=VERSION,
        description='The tidy package of Servo',
        long_description=long_description,
        keywords='mozilla servo tidy ',
        author='The Servo Project Developers',
        author_email='dev-servo@lists.mozilla.org',
        url='https://github.com/servo/servo_tindy',
        packages=find_packages(exclude=['ez_setup', 'examples', 'tests']),
        package_data={},
        install_requires=install_requires,
        zip_safe=False,
    )
