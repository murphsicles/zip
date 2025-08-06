use criterion::{Criterion, criterion_group, criterion_main};
use zip::blockchain::TransactionManager;
use zip::storage::ZipStorage;

fn bench_pre_create_utxos(c: &mut Criterion) {
    let storage = Arc::new(ZipStorage::new().unwrap());
    let tx_manager = TransactionManager::new(Arc::clone(&storage));

    c.bench_function("pre_create_utxos", |b| {
        b.iter(|| tx_manager.pre_create_utxos("parent_txid", 0, 1000000, 100, 1000));
    });
}

fn bench_build_payment_tx(c: &mut Criterion) {
    let storage = Arc::new(ZipStorage::new().unwrap());
    let tx_manager = TransactionManager::new(Arc::clone(&storage));

    let script = Script::default();
    c.bench_function("build_payment_tx", |b| {
        b.iter(|| tx_manager.build_payment_tx(script.clone(), 10000, 1000));
    });
}

criterion_group!(benches, bench_pre_create_utxos, bench_build_payment_tx);
criterion_main!(benches);
