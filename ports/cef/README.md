How to test:

1. Go to http://cefbuilds.com/ and download a binary for your OS/arch
2. Unpack the (huge) archive
3. Create both Debug and Release build types for CEF (```./build.sh [Debug|Release]```)
4. Build servo
5. Run a CEF-based executable with the embedding crate preloaded:
	* Linux: ```LD_LIBRARY_PATH=/path/to/cef-bin-unpack-dir/out/$build_type LD_PRELOAD=/path/to/servo/build/libembedding-*.so [CEF EXE]```
6. Enjoy CEF-powered crashes

Notes:
* Running with the Debug build in GDB is EXTREMELY slow on startup. Only use this if you are actively debugging an unimplemented CEF interaction.
