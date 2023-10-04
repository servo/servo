import pathlib
import re

import setuptools


root_dir = pathlib.Path(__file__).parent

description = "An implementation of the WebSocket Protocol (RFC 6455 & 7692)"

long_description = (root_dir / 'README.rst').read_text(encoding='utf-8')

# PyPI disables the "raw" directive.
long_description = re.sub(
    r"^\.\. raw:: html.*?^(?=\w)",
    "",
    long_description,
    flags=re.DOTALL | re.MULTILINE,
)

exec((root_dir / 'src' / 'websockets' / 'version.py').read_text(encoding='utf-8'))

packages = ['websockets', 'websockets/legacy', 'websockets/extensions']

ext_modules = [
    setuptools.Extension(
        'websockets.speedups',
        sources=['src/websockets/speedups.c'],
        optional=not (root_dir / '.cibuildwheel').exists(),
    )
]

setuptools.setup(
    name='websockets',
    version=version,
    description=description,
    long_description=long_description,
    url='https://github.com/aaugustin/websockets',
    author='Aymeric Augustin',
    author_email='aymeric.augustin@m4x.org',
    license='BSD',
    classifiers=[
        'Development Status :: 5 - Production/Stable',
        'Environment :: Web Environment',
        'Intended Audience :: Developers',
        'License :: OSI Approved :: BSD License',
        'Operating System :: OS Independent',
        'Programming Language :: Python',
        'Programming Language :: Python :: 3',
        'Programming Language :: Python :: 3.7',
        'Programming Language :: Python :: 3.8',
        'Programming Language :: Python :: 3.9',
        'Programming Language :: Python :: 3.10',
    ],
    package_dir = {'': 'src'},
    package_data = {'websockets': ['py.typed']},
    packages=packages,
    ext_modules=ext_modules,
    include_package_data=True,
    zip_safe=False,
    python_requires='>=3.7',
    test_loader='unittest:TestLoader',
)
