use std::borrow::BorrowMut;

use clarity_repl::clarity::stacks_common::types::StacksEpochId;
use clarity_repl::{
    clarity::{ast::ContractAST, ClarityVersion},
    repl::{ClarityCodeSource, ClarityContract, ContractDeployer, Session, SessionSettings},
};
use clarity_vs_wasm::{reverse_buff32, add128};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wasmer_compiler_singlepass::Singlepass;

fn rust_add(c: &mut Criterion) {
    c.bench_function("add: rust", |b| {
        b.iter(|| {
            black_box(add128(black_box(42), black_box(12345)));
        })
    });
}

fn clarity_add(c: &mut Criterion) {
    // Setup the session with the Clarity contract first
    let mut session = Session::new(SessionSettings::default());
    let contract_source = r#"
(define-read-only (add (x int) (y int))
    (+ x y)
)
    "#
    .to_string();

    let contract = ClarityContract {
        name: "add".to_string(),
        code_source: ClarityCodeSource::ContractInMemory(contract_source),
        clarity_version: ClarityVersion::Clarity2,
        epoch: StacksEpochId::latest(),
        deployer: ContractDeployer::Address(
            "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM".to_string(),
        ),
    };

    let mut ast: Option<ContractAST> = None;
    session
        .deploy_contract(&contract, None, false, None, &mut ast)
        .unwrap();
    session
        .eval(
            "(contract-call? 'ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.add add 42 12345)"
                .to_string(),
            None,
            false,
        )
        .unwrap();

    c.bench_function("add: clarity", |b| {
        b.iter(|| {
            session
                .eval(
                    "(contract-call? 'ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.add add 42 12345)"
                        .to_string(),
                    None,
                    false,
                )
                .unwrap();
        })
    });
}

fn wasmer_add(c: &mut Criterion) {
    c.bench_function("add: wasmer", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasm.wat");
        let mut store = wasmer::Store::default();
        let module = wasmer::Module::new(&store, module_wat).unwrap();
        let import_object = wasmer::imports! {};
        let instance = wasmer::Instance::new(&mut store, &module, &import_object).unwrap();
        let wasm_add = instance.exports.get_function("add").unwrap();

        b.iter(|| {
            wasm_add
                .call(
                    &mut store,
                    &[wasmer::Value::I32(42), wasmer::Value::I32(12345)],
                )
                .unwrap();
        })
    });
}

fn wasmer_singlepass_add(c: &mut Criterion) {
    c.bench_function("add: wasmer singlepass", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasm.wat");
        let compiler = Singlepass::new();
        let mut store = wasmer::Store::new(compiler);
        let module = wasmer::Module::new(&store, module_wat).unwrap();
        let import_object = wasmer::imports! {};
        let instance = wasmer::Instance::new(&mut store, &module, &import_object).unwrap();
        let wasm_add = instance.exports.get_function("add").unwrap();

        b.iter(|| {
            wasm_add
                .call(
                    &mut store,
                    &[wasmer::Value::I32(42), wasmer::Value::I32(12345)],
                )
                .unwrap();
        })
    });
}

fn wasmtime_add(c: &mut Criterion) {
    c.bench_function("add: wasmtime", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasm.wat");
        let engine = wasmtime::Engine::default();
        let mut store = wasmtime::Store::new(&engine, ());
        let module = wasmtime::Module::new(&engine, module_wat).unwrap();
        let instance = wasmtime::Instance::new(&mut store.borrow_mut(), &module, &[]).unwrap();
        let wasm_add = instance.get_func(&mut store.borrow_mut(), "add").unwrap();

        b.iter(|| {
            let mut results = [wasmtime::Val::I32(0)];
            wasm_add
                .call(
                    &mut store.borrow_mut(),
                    &[wasmtime::Val::I32(42), wasmtime::Val::I32(12345)],
                    &mut results,
                )
                .unwrap();
        })
    });
}

fn wasmtime_interpreted_add(c: &mut Criterion) {
    c.bench_function("add: wasmtime interpreted", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasm.wat");
        let mut config = wasmtime::Config::new();
        config.cranelift_opt_level(wasmtime::OptLevel::None);
        let engine = wasmtime::Engine::new(&config).unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        let module = wasmtime::Module::new(&engine, module_wat).unwrap();
        let instance = wasmtime::Instance::new(&mut store.borrow_mut(), &module, &[]).unwrap();
        let wasm_add = instance.get_func(&mut store.borrow_mut(), "add").unwrap();

        b.iter(|| {
            let mut results = [wasmtime::Val::I32(0)];
            wasm_add
                .call(
                    &mut store.borrow_mut(),
                    &[wasmtime::Val::I32(42), wasmtime::Val::I32(12345)],
                    &mut results,
                )
                .unwrap();
        })
    });
}

//---------- REVERSE BUFF32 ----------

fn rust_reverse_buff32(c: &mut Criterion) {
    c.bench_function("reverse: rust", |b| {
        b.iter(|| {
            black_box(reverse_buff32(black_box(vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ])));
        })
    });
}

fn clarity_reverse_buff32(c: &mut Criterion) {
    // Setup the session with the Clarity contract first
    let mut session = Session::new(SessionSettings::default());
    let contract_source = r#"
;; Generate a permutation of a given 32-byte buffer, appending the element at target-index to hash-output.
;; The target-index decides which index in hash-input gets appended to hash-output.
(define-read-only (inner-reverse (target-index uint) (hash-input (buff 32)))
    (unwrap-panic
        (replace-at?
            (unwrap-panic
                (replace-at?
                    hash-input
                    target-index
                    (unwrap-panic (element-at? hash-input (- u31 target-index)))))
            (- u31 target-index)
            (unwrap-panic (element-at? hash-input  target-index)))))

;; Reverse the byte order of a 32-byte buffer.  Returns the (buff 32).
(define-read-only (reverse-buff32 (input (buff 32)))
    (fold inner-reverse
        (list u31 u30 u29 u28 u27 u26 u25 u24 u23 u22 u21 u20 u19 u18 u17 u16)
        input))
    "#.to_string();

    let contract = ClarityContract {
        name: "clarity-bitcoin".to_string(),
        code_source: ClarityCodeSource::ContractInMemory(contract_source),
        clarity_version: ClarityVersion::Clarity2,
        epoch: StacksEpochId::latest(),
        deployer: ContractDeployer::Address(
            "ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM".to_string(),
        ),
    };

    let mut ast: Option<ContractAST> = None;
    session
        .deploy_contract(&contract, None, false, None, &mut ast)
        .unwrap();

    c.bench_function("reverse: clarity", |b| {
        b.iter(|| {
            session.eval("(contract-call? 'ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM.clarity-bitcoin reverse-buff32 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20)".to_string(), None, false)
                .unwrap();
        })
    });
}

fn wasmer_reverse_buff32(c: &mut Criterion) {
    c.bench_function("reverse: wasmer", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasm.wat");
        let mut store = wasmer::Store::default();
        let module = wasmer::Module::new(&store, module_wat).unwrap();
        let import_object = wasmer::imports! {};
        let instance = wasmer::Instance::new(&mut store, &module, &import_object).unwrap();
        let wasm_reverse_buff32 = instance.exports.get_function("reverse_buff32").unwrap();

        // Prepare the buffer to be passed to the wasm function
        let memory = instance.exports.get_memory("memory").unwrap();
        let buffer = (1..=32).collect::<Vec<u8>>();
        let memory_view = memory.view(&store);
        for (i, byte) in buffer.iter().enumerate() {
            memory_view.write(i as u64, &[*byte]).unwrap();
        }

        b.iter(|| {
            // Call the function with a pointer to the result buffer, a pointer to the start of the input buffer and the length of the buffer.
            wasm_reverse_buff32
                .call(
                    &mut store,
                    &[
                        wasmer::Value::I32(buffer.len() as i32),
                        wasmer::Value::I32(0),
                        wasmer::Value::I32(buffer.len() as i32),
                    ],
                )
                .unwrap();
        })
    });
}

fn wasmer_singlepass_reverse_buff32(c: &mut Criterion) {
    c.bench_function("reverse: wasmer singlepass", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasm.wat");
        let compiler = Singlepass::new();
        let mut store = wasmer::Store::new(compiler);
        let module = wasmer::Module::new(&store, module_wat).unwrap();
        let import_object = wasmer::imports! {};
        let instance = wasmer::Instance::new(&mut store, &module, &import_object).unwrap();
        let wasm_reverse_buff32 = instance.exports.get_function("reverse_buff32").unwrap();

        // Prepare the buffer to be passed to the wasm function
        let memory = instance.exports.get_memory("memory").unwrap();
        let buffer = (1..=32).collect::<Vec<u8>>();
        let memory_view = memory.view(&store);
        for (i, byte) in buffer.iter().enumerate() {
            memory_view.write(i as u64, &[*byte]).unwrap();
        }

        b.iter(|| {
            // Call the function with a pointer to the result buffer, a pointer to the start of the input buffer and the length of the buffer.
            wasm_reverse_buff32
                .call(
                    &mut store,
                    &[
                        wasmer::Value::I32(buffer.len() as i32),
                        wasmer::Value::I32(0),
                        wasmer::Value::I32(buffer.len() as i32),
                    ],
                )
                .unwrap();
        })
    });
}

fn wasmtime_reverse_buff32(c: &mut Criterion) {
    c.bench_function("reverse: wasmtime", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasm.wat");
        let engine = wasmtime::Engine::default();
        let mut store = wasmtime::Store::new(&engine, ());
        let module = wasmtime::Module::new(&engine, module_wat).unwrap();
        let instance = wasmtime::Instance::new(&mut store.borrow_mut(), &module, &[]).unwrap();
        let wasm_reverse_buff32 = instance
            .get_func(&mut store.borrow_mut(), "reverse_buff32")
            .unwrap();

        // Prepare the buffer to be passed to the wasm function
        let memory = instance
            .get_memory(&mut store.borrow_mut(), "memory")
            .expect("failed to find `memory` export");
        let buffer = (1..=32).collect::<Vec<u8>>();
        let mut binding = store.borrow_mut();
        let memory_writer = memory.data_mut(&mut binding);
        for (i, byte) in buffer.iter().enumerate() {
            memory_writer[i] = *byte;
        }

        let mut results = [];
        b.iter(|| {
            wasm_reverse_buff32
                .call(
                    &mut store.borrow_mut(),
                    &[
                        wasmtime::Val::I32(buffer.len() as i32),
                        wasmtime::Val::I32(0),
                        wasmtime::Val::I32(buffer.len() as i32),
                    ],
                    &mut results,
                )
                .unwrap();
        })
    });
}

fn wasmtime_interpreted_reverse_buff32(c: &mut Criterion) {
    c.bench_function("reverse: wasmtime interpreted", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasm.wat");
        let mut config = wasmtime::Config::new();
        config.cranelift_opt_level(wasmtime::OptLevel::None);
        let engine = wasmtime::Engine::new(&config).unwrap();
        let mut store = wasmtime::Store::new(&engine, ());
        let module = wasmtime::Module::new(&engine, module_wat).unwrap();
        let instance = wasmtime::Instance::new(&mut store.borrow_mut(), &module, &[]).unwrap();
        let wasm_reverse_buff32 = instance
            .get_func(&mut store.borrow_mut(), "reverse_buff32")
            .unwrap();

        // Prepare the buffer to be passed to the wasm function
        let memory = instance
            .get_memory(&mut store.borrow_mut(), "memory")
            .expect("failed to find `memory` export");
        let buffer = (1..=32).collect::<Vec<u8>>();
        let mut binding = store.borrow_mut();
        let memory_writer = memory.data_mut(&mut binding);
        for (i, byte) in buffer.iter().enumerate() {
            memory_writer[i] = *byte;
        }

        let mut results = [];
        b.iter(|| {
            wasm_reverse_buff32
                .call(
                    &mut store.borrow_mut(),
                    &[
                        wasmtime::Val::I32(buffer.len() as i32),
                        wasmtime::Val::I32(0),
                        wasmtime::Val::I32(buffer.len() as i32),
                    ],
                    &mut results,
                )
                .unwrap();
        })
    });
}

criterion_group!(
    add_benches,
    clarity_add,
    wasmer_singlepass_add,
    wasmer_add,
    wasmtime_interpreted_add,
    wasmtime_add,
    rust_add
);
criterion_group!(
    reverse_benches,
    clarity_reverse_buff32,
    wasmer_singlepass_reverse_buff32,
    wasmer_reverse_buff32,
    wasmtime_interpreted_reverse_buff32,
    wasmtime_reverse_buff32,
    rust_reverse_buff32
);
criterion_main!(reverse_benches, add_benches);
