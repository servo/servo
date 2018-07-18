import pytest


@pytest.fixture
def create_file(tmpdir_factory):
    def inner(filename):
        fh = tmpdir_factory.mktemp("tmp").join(filename)
        fh.write(filename)

        return fh

    inner.__name__ = "create_file"
    return inner


@pytest.fixture
def create_files(tmpdir_factory):
    def inner(filenames):
        filelist = []
        tmpdir = tmpdir_factory.mktemp("tmp")
        for filename in filenames:
            fh = tmpdir.join(filename)
            fh.write(filename)
            filelist.append(fh)

        return filelist

    inner.__name__ = "create_files"
    return inner
