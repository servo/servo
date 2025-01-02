"""
This is the script that is actually frozen into an executable: simply executes
pytest main().
"""

if __name__ == "__main__":
    import sys

    import pytest

    sys.exit(pytest.main())
