use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use tempfile::TempDir;
use toonstoredb::ToonStore;

fn bench_put(c: &mut Criterion) {
    let mut group = c.benchmark_group("put");
    group.sample_size(50);
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("put_1kb", |b| {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();
        let data = vec![b'x'; 1024];
        
        b.iter(|| {
            black_box(db.put(&data).unwrap());
        });
    });
    group.finish();
}

fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");
    group.sample_size(50);
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("get_1kb", |b| {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();
        let data = vec![b'x'; 1024];
        
        // Pre-populate with 100 rows
        for _ in 0..100 {
            db.put(&data).unwrap();
        }
        
        b.iter(|| {
            black_box(db.get(50).unwrap());
        });
    });
    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed");
    group.sample_size(50);
    group.throughput(Throughput::Elements(1));
    
    group.bench_function("50_read_50_write", |b| {
        let dir = TempDir::new().unwrap();
        let db = ToonStore::open(dir.path()).unwrap();
        let data = vec![b'x'; 1024];
        
        // Pre-populate with 100 rows
        for _ in 0..100 {
            db.put(&data).unwrap();
        }
        
        let mut counter = 0u64;
        b.iter(|| {
            if counter.is_multiple_of(2) {
                black_box(db.get(counter % 100).ok());
            } else {
                black_box(db.put(&data).ok());
            }
            counter += 1;
        });
    });
    group.finish();
}

criterion_group!(benches, bench_put, bench_get, bench_mixed_workload);
criterion_main!(benches);

