use clarity_repl::clarity::stacks_common::types::StacksEpochId;
use clarity_repl::{
    clarity::{ast::ContractAST, ClarityVersion},
    repl::{ClarityCodeSource, ClarityContract, ContractDeployer, Session, SessionSettings},
};
use clarity_vs_wasmer::{add, reverse_buff32};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wasmer::{imports, Instance, Module, Store, Value};

fn rust_add(c: &mut Criterion) {
    c.bench_function("add: rust", |b| {
        b.iter(|| {
            black_box(add(black_box(42), black_box(12345)));
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

fn wasm_add(c: &mut Criterion) {
    c.bench_function("add: wasm", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasmer.wat");
        let mut store = Store::default();
        let module = Module::new(&store, module_wat).unwrap();
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object).unwrap();
        let wasm_add = instance.exports.get_function("add").unwrap();

        b.iter(|| {
            wasm_add
                .call(&mut store, &[Value::I32(42), Value::I32(12345)])
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

fn wasm_reverse_buff32(c: &mut Criterion) {
    c.bench_function("reverse: wasm", |b| {
        let module_wat = include_str!("../pkg/clarity_vs_wasmer.wat");
        let mut store = Store::default();
        let module = Module::new(&store, module_wat).unwrap();
        let import_object = imports! {};
        let instance = Instance::new(&mut store, &module, &import_object).unwrap();
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
                        Value::I32(buffer.len() as i32),
                        Value::I32(0),
                        Value::I32(buffer.len() as i32),
                    ],
                )
                .unwrap();
        })
    });
}

criterion_group!(add_benches, clarity_add, wasm_add, rust_add);
criterion_group!(
    reverse_benches,
    clarity_reverse_buff32,
    wasm_reverse_buff32,
    rust_reverse_buff32
);
criterion_main!(reverse_benches, add_benches);
