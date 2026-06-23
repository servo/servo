# Severin Python wheel download

Severin's first Python distribution path is a direct GitHub Release asset:

```text
severin-<version>-<python-tag>-<abi-tag>-linux_x86_64.whl
```

It is built in the existing Debian 12 release environment for Linux x86_64 and the exact CPython minor version available there. It is experimental and CPython-version-specific; it is not published to PyPI and is not a cross-platform Python package yet.

## Offline install

1. Download the matching `.whl` from the repository's Releases page.
2. Install from the local file without dependency resolution:

   ```bash
   python3 -m pip install --user --no-deps ./severin-<version>-<python-tag>-<abi-tag>-linux_x86_64.whl
   ```

3. Verify import through CPython's normal extension importer:

   ```bash
   python3 -c 'import severin; print(severin.App)'
   ```

The wheel embeds the native `severin` extension in-process. It does not use PyPI, does not download dependencies, does not run Cargo on the user's machine, does not require this repository checkout, and does not launch a helper process, server, port, socket, or localhost IPC bridge.
