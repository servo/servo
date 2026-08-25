/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use criterion::*;

fn create_data(number_of_pixels: usize) -> Vec<u8> {
    (0..=number_of_pixels)
        .map(|i| {
            let i = (i % 255) as u8;
            [i, i, i, i]
        })
        .flatten()
        .collect()
}

fn bench(c: &mut Criterion) {
    let data = create_data(1_000_000);

    c.bench_function("unmultiply_inplace", move |b| {
        b.iter_batched(
            || data.clone(),
            |mut data| pixels::unmultiply_inplace::<false>(&mut data),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
