#!/bin/sh
if test ! -e rust-toolchain; then
    echo "Please run script in servo git repository"
    exit
fi

echo "Syncing Rust dependencies..."
git clone https://github.com/flatpak/flatpak-builder-tools.git && \
cd flatpak-builder-tools/cargo && \
python3 ./flatpak-cargo-generator.py ../../Cargo.lock -t -o ../../data/cargo-sources.json && \
cd ../..
rm -rf flatpak-builder-tools

echo "Syncing Rush nightly version..."

VER=`cat rust-toolchain | cut -c 9-`

arr1="https://static.rust-lang.org/dist/${VER}/rust-nightly-aarch64-unknown-linux-gnu.tar.xz"
arr2="https://static.rust-lang.org/dist/${VER}/rust-nightly-x86_64-unknown-linux-gnu.tar.xz"
arr3="https://static.rust-lang.org/dist/${VER}/rustc-dev-nightly-aarch64-unknown-linux-gnu.tar.xz"
arr4="https://static.rust-lang.org/dist/${VER}/rustc-dev-nightly-x86_64-unknown-linux-gnu.tar.xz"

OUTPUT="data/org.servo.Servo.json"

cp "data/org.servo.Servo.template.json" $OUTPUT
i=1
for str in $arr1 $arr2 $arr3 $arr4; do
	sed -i "s|_REPLACE_${i}_|$str|g" $OUTPUT
	sha256=`curl "$str".sha256 | cut -c "-64"`
	sed -i "s|_REPLACE_SHA_${i}_|$sha256|g" $OUTPUT
	i=`expr $i + 1`
done
