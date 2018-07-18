"""
This is the script that is actually frozen into an executable: simply executes
py.test main().
"""

if __name__ == "__main__":
    import sys
    import pytest

    sys.exit(pytest.main())
