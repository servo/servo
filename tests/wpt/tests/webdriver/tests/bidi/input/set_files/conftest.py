import os
import pytest


@pytest.fixture
def create_files(tmpdir_factory):
    def create_files(paths):
        """Create files in a temporary folder.

        Note: the fixture doesn't work with absolute paths, since all the files
        should be created inside the temporary folder.

        :param paths: list of file names which should be used for creating files
        :return: list of final paths for created files
        """

        filelist = []
        tmpdir = tmpdir_factory.mktemp("wdspec-")

        for path in paths:
            # Make path separators system compatible.
            path = os.path.join(*path.split("/"))
            dir_path = os.path.join(tmpdir, os.path.dirname(path))
            if not os.path.exists(dir_path):
                os.makedirs(dir_path)

            full_path = os.path.join(tmpdir, path)
            open(full_path, 'w').close()
            filelist.append(full_path)

        return filelist

    return create_files
