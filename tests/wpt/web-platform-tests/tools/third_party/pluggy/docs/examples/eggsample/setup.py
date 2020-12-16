from setuptools import setup, find_packages

setup(
    name="eggsample",
    install_requires="pluggy>=0.3,<1.0",
    entry_points={"console_scripts": ["eggsample=eggsample.host:main"]},
    packages=find_packages(),
)
