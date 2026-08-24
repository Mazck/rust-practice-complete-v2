use criterion::{Criterion, criterion_group, criterion_main};
use production_lab::Telemetry;
use std::hint::black_box;

fn parse_benchmark(c: &mut Criterion) {
    let payload = br#"{"device_id":"d-1","message_id":"m-1","value":21.5,"unit":"C"}"#;
    c.bench_function("parse telemetry json", |bench| {
        bench.iter(|| {
            let value: Telemetry = serde_json::from_slice(black_box(payload)).unwrap();
            black_box(value);
        });
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
