# Rust compilation benchmark — 2026-10-14

## Environment

- CPU: AMD Ryzen 9 9950X
- OS: Linux 6.18.3
- Rustc: Rust 1.98.1 (2026-09-01)
- Hyperfine: 1.21.0
- Linker: mold 2.42.1
- RUSTFLAGS: `-C link-arg=-fuse-ld=mold`

## Libraries

- Diesel 2.3.13
- SQLx 0.9.0

## Results

Use `hyperfine` with 1 warmup and 5 runs, remove the fastest and slowest runs, then average the remaining 3, lower is better.

Build time excludes dependencies.

15 tables and 37 queries from [Pagila](https://github.com/devrimgunduz/pagila/blob/d9610f790b723daad5411dd78664d170a58dafa2/pagila-schema.sql).

### Dev

| Library | Time(ms) | Relative |
| --- | ---: | ---: |
| SQLx | **241.7** | **1.00×** |
| placeholder-query | 354.0 | 1.46× |
| Diesel | 1119.1 | 4.63× |

### Release

| Library | Time(ms) | Relative |
| --- | ---: | ---: |
| SQLx | **417.5** | **1.00×** |
| placeholder-query | 973.8 | 2.33× |
| Diesel | 1171.1 | 2.81× |

## Reproduce

Run the benchmark script from the repository root.
Requires hyperfine, jq, and mold.

```sh
./benchmarks/compile/benchmark.sh
```

