## Generate WAT (WASM text format)

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

We can generate the text-based WASM (WAT) file from the .wasm file using:

```sh
wasm2wat pkg/clarity_vs_wasmer_bg.wasm -o clarity_vs_wasmer.wat
```

## Run Benchmarks

```sh
cargo bench
```

## Results

Keep in mind this is a very rough test, just to get an idea of how much speedup is potentially possible by using a bytecode like WebAssembly.

Running this on my M1 Mac, these are the results I see:

* `add`
  * clarity: 53.064 µs
  * wasm: 389.85 ns (135x faster)
  * rust: 1.0058 ns (53,000x faster)
* `reverse_buff32`
  * clarity: 171.07 µs
  * wasm: 213.06 ns (800x faster)
  * rust: 21.973 ns (7,800x faster)