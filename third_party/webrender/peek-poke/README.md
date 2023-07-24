# Peeks, Pokes, and Pointers

Peek from and poke structures into byte slices.

## Benchmark

Below are the benchmark results of comparison between `peek-poke` and `bincode` serializing and deserializing same `struct`:
```
struct MyPeekPokeStruct {
    a: u8,
    b: u16,
    c: MyPeekPokeEnum,
    d: Option<usize>,
}

enum MyPeekPokeEnum {
    Variant1,
    Variant2(u16),
}
```

```
Benchmarking struct::serialize/peek_poke::poke_into: Collecting 100 samples in                                                                                struct::serialize/peek_poke::poke_into
                        time:   [2.7267 ns 2.7321 ns 2.7380 ns]

Benchmarking struct::serialize/bincode::serialize: Collecting 100 samples in est                                                                                struct::serialize/bincode::serialize
                        time:   [31.264 ns 31.326 ns 31.389 ns]

Benchmarking struct::deserialize/peek_poke::peek_from: Collecting 100 samples                                                                                 struct::deserialize/peek_poke::peek_from
                        time:   [5.3544 ns 5.3672 ns 5.3817 ns]

Benchmarking struct::deserialize/bincode::deserialize: Collecting 100 samples in                                                                                struct::deserialize/bincode::deserialize
                        time:   [25.155 ns 26.439 ns 27.661 ns]
```

You can run benchmarks by running following command:
```
cargo bench
```

## License
[license]: #license

Licensed under either of
- Apache License, Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (http://opensource.org/licenses/MIT)

at your option.

see [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as about, without any additional terms or conditions.
