use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use tempfile::TempDir;
use tooncache::ToonCache;

fn bench_cached_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("cached_get");
    group.sample_size(50);
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_1kb_cached", |b| {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 1000).unwrap();
        let data = vec![b'x'; 1024];

        // Pre-populate and warm cache
        let mut ids = Vec::new();
        for _ in 0..100 {
            ids.push(cache.put(&data).unwrap());
        }

        // Warm the cache
        for id in &ids {
            cache.get(*id).unwrap();
        }

        let mut counter = 0;
        b.iter(|| {
            black_box(cache.get(ids[counter % 100]).unwrap());
            counter += 1;
        });
    });

    group.finish();
}

fn bench_mixed_50_50(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed");
    group.sample_size(50);
    group.throughput(Throughput::Elements(1));

    group.bench_function("50_read_50_write_cached", |b| {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 1000).unwrap();
        let data = vec![b'x'; 1024];

        // Pre-populate
        let mut ids = Vec::new();
        for _ in 0..100 {
            ids.push(cache.put(&data).unwrap());
        }

        let mut counter = 0u64;
        b.iter(|| {
            if counter.is_multiple_of(2) {
                black_box(cache.get(ids[(counter as usize) % 100]).ok());
            } else {
                black_box(cache.put(&data).ok());
            }
            counter += 1;
        });
    });

    group.finish();
}

fn bench_cache_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_miss");
    group.sample_size(50);
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_1kb_miss", |b| {
        let dir = TempDir::new().unwrap();
        let cache = ToonCache::new(dir.path(), 10).unwrap(); // Small cache
        let data = vec![b'x'; 1024];

        // Pre-populate with more than cache size
        let mut ids = Vec::new();
        for _ in 0..100 {
            ids.push(cache.put(&data).unwrap());
        }

        let mut counter = 0;
        b.iter(|| {
            // Access pattern that guarantees misses
            black_box(cache.get(ids[counter % 100]).unwrap());
            counter += 1;
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cached_get,
    bench_mixed_50_50,
    bench_cache_miss
);
criterion_main!(benches);
