cd build
../configure
export DISPLAY=:1.0
export RUST_TEST_TASKS=1
make tidy && make -j2 && make check-servo && make check-content && make check-ref-cpu
