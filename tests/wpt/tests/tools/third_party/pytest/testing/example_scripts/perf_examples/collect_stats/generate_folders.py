# mypy: allow-untyped-defs
import argparse
import pathlib


HERE = pathlib.Path(__file__).parent
TEST_CONTENT = (HERE / "template_test.py").read_bytes()

parser = argparse.ArgumentParser()
parser.add_argument("numbers", nargs="*", type=int)


def generate_folders(root, elements, *more_numbers):
    fill_len = len(str(elements))
    if more_numbers:
        for i in range(elements):
            new_folder = root.joinpath(f"foo_{i:0>{fill_len}}")
            new_folder.mkdir()
            new_folder.joinpath("__init__.py").write_bytes(TEST_CONTENT)
            generate_folders(new_folder, *more_numbers)
    else:
        for i in range(elements):
            new_test = root.joinpath(f"test_{i:0<{fill_len}}.py")
            new_test.write_bytes(TEST_CONTENT)


if __name__ == "__main__":
    args = parser.parse_args()
    generate_folders(HERE, *(args.numbers or (10, 100)))
