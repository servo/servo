from setuptools import setup

setup(
    name="eggsample-spam",
    install_requires="eggsample",
    entry_points={"eggsample": ["spam = eggsample_spam"]},
    py_modules=["eggsample_spam"],
)
