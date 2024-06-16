import sys


if __name__ == "__main__":
    import cProfile
    import pstats

    import pytest  # noqa: F401

    script = sys.argv[1:] if len(sys.argv) > 1 else ["empty.py"]
    cProfile.run("pytest.cmdline.main(%r)" % script, "prof")
    p = pstats.Stats("prof")
    p.strip_dirs()
    p.sort_stats("cumulative")
    print(p.print_stats(500))
