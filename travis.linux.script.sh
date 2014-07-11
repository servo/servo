cd build
../configure
make tidy && make -j2 && make check-servo
