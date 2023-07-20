## Generate WAT (Wasm text format)

```sh
wasm-pack build --target web
```

This generates ./pkg/:

```sh
pkg
├── README.md
├── clarity_vs_wasmer.d.ts
├── clarity_vs_wasmer.js
├── clarity_vs_wasmer_bg.wasm
├── clarity_vs_wasmer_bg.wasm.d.ts
└── package.json
```

We can generate the text-based Wasm (WAT) file from the .wasm file using:

```sh
wasm2wat pkg/clarity_vs_wasmer_bg.wasm -o clarity_vs_wasmer.wat
```

## Run Benchmarks

```sh
cargo bench
```

## Results

Keep in mind this is a very rough test, just to get an idea of how much speedup is potentially possible by using a bytecode like WebAssembly. Note that Wasm does not support 128-bit integers, so the Wasm code is doing a 64-bit addition while the Clarity and Rust versions are doing 128-bit addition. Two different Wasm runtimes are tested: [wasmer](https://github.com/wasmerio/wasmer) and [wasmtime](https://github.com/bytecodealliance/wasmtime). For each of these, we try the fast, unoptimized version (`singlepass` for wasmer, and `OptLevel::None` for wasmtime) as well as the default, optimized versions.

Running this on my M1 Mac, these are the results I see:

- `add`
  - clarity: 53.064 µs
  - wasmer singlepass: 370.82 ns (142x faster)
  - wasmer: 371.92 ns (142x faster)
  - wasmtime interpreter: 51.003 ns (1039x faster)
  - wasmtime: 43.827 ns (1209x faster)
  - rust: 1.0058 ns (53,000x faster)
- `reverse_buff32`
  - clarity: 171.07 µs
  - wasmer singlepass: 232.59 ns (735x faster)
  - wasmer: 213.06 ns (802x faster)
  - wasmtime interpreter: 52.597 ns (3,250x faster)
  - wasmtime: 48.302 ns (3,540x faster)
  - rust: 21.973 ns (7,800x faster)
