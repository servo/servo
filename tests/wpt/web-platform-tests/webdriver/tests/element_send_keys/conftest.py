import pytest


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
