"""Shared constants for reproducible baseline/codegen benchmark runs."""

BASELINE_SHA = "c15fa1c254ea5b2bbabfe5f008d41f91e8297b89"
BENCH_RUSTFLAGS = "-C target-cpu=native"

CODEGEN_CONFIGS = {
    "x86_64-baseline": ("x86_64", "-C target-cpu=x86-64"),
    "x86_64-sse41": ("x86_64", "-C target-cpu=x86-64 -C target-feature=+sse4.1"),
    "x86_64-avx2": ("x86_64", "-C target-cpu=x86-64 -C target-feature=+avx2"),
    "aarch64-neon": ("aarch64", "-C target-cpu=generic -C target-feature=+neon"),
}
