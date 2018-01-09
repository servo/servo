REM install pypy using choco
REM redirect to a file because choco install python.pypy is too noisy. If the command fails, write output to console
choco install python.pypy > pypy-inst.log 2>&1 || (type pypy-inst.log & exit /b 1)
set PATH=C:\tools\pypy\pypy;%PATH% # so tox can find pypy
echo PyPy installed
pypy --version
