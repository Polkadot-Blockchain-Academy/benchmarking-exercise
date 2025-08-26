# FRAME Benchmarking Exercise

This repository contains a **benchmarking exercise** designed to teach FRAME pallet benchmarking concepts through a simplified Identity pallet implementation. The project demonstrates various benchmarking patterns and complexities in a Substrate runtime environment.

## Exercise Overview

Your task is to **implement the missing benchmarks** in the Identity pallet's benchmarking module. This exercise will teach you to:

1. **Analyze complexity patterns** - Understand different algorithmic complexities (linear, logarithmic, constant)
2. **Choose appropriate parameters** - Determine which complexity parameters are necessary for accurate benchmarking
3. **Implement benchmark scenarios** - Create proper setup, execution, and verification for benchmarks
4. **Understand storage patterns** - Compare different storage approaches and their performance implications

## Assignment Instructions

Navigate to [`pallets/identity/src/benchmarking.rs`](./pallets/identity/src/benchmarking.rs) and find the **two incomplete benchmarks**:

### 1. `clear_identity_inline_usage` Benchmark

```rust
#[benchmark]
fn clear_identity_inline_usage(
    b: Linear<1, { T::MaxFieldLength::get() }>, // TODO: determine if necessary
    j: Linear<0, { T::MaxJudgements::get() }>,  // TODO: determine if necessary
) {
    // TODO: implement
}
```

**Your Tasks:**
- **Analyze the complexity**: Examine the `clear_identity` extrinsic to understand its computational complexity when using inline storage
- **Determine parameters**: Decide which linear parameters (`b` for bytes, `j` for judgements) are actually necessary
- **Implement the benchmark**: Create a complete benchmark following the patterns from existing benchmarks
- **Add verification**: Include proper assertions to verify the benchmark correctness

### 2. `clear_identity_double_map_usage` Benchmark

```rust
#[benchmark]
fn clear_identity_double_map_usage(
    b: Linear<1, { T::MaxFieldLength::get() }>, // TODO: determine if necessary
    j: Linear<0, { T::MaxJudgements::get() }>,  // TODO: determine if necessary
) {
    // TODO: implement
}
```

**Your Tasks:**
- **Analyze the complexity**: Examine the `clear_identity` extrinsic to understand its computational complexity when using double map storage
- **Determine parameters**: Decide which linear parameters are actually necessary for this storage pattern
- **Implement the benchmark**: Create a complete benchmark demonstrating the difference from inline storage
- **Add verification**: Include proper assertions to verify the benchmark correctness

## Key Learning Objectives

- Complexity Analysis
- Storage Pattern Comparison
- Benchmarking Best Practices
    - **Proper setup**: Creating realistic pre-conditions for the benchmark
    - **Worst-case scenarios**: Testing the most expensive execution paths
    - **Comprehensive verification**: Ensuring benchmarks measure what they claim to measure

## Implementation Guidelines

### 1. Study Existing Benchmarks
Before implementing, examine the existing benchmarks in the file:
- `set_identity` - Shows linear complexity with bytes parameter
- `set_identity_update` - Shows linear complexity with both bytes and logarithmic complexity with judgements
- `provide_judgement_inline` - Shows logarithmic complexity with judgements
- `provide_judgement_double_map` - Shows linear complexity with bytes parameter but independent of judgements

### 2. Understand the `clear_identity` Extrinsic
Read [`pallets/identity/src/lib.rs`](./pallets/identity/src/lib.rs) to understand:
- How `clear_identity` works
- What storage operations it performs

### 3. Follow the Pattern
Each benchmark should include:
```rust
#[benchmark]
fn benchmark_name(/* parameters */) {
    // 1. Setup: Create test accounts and fund them
    // 2. Pre-conditions: Set up identity and judgements
    // 3. Execution: Call the extrinsic being benchmarked
    // 4. Verification: Assert the expected final state
}
```

### 4. Helper Functions
Use existing helper functions:
- `fund_account::<T>()` - Provides sufficient balance for operations
- `create_identity_info::<T>()` - Creates test identity data
- `whitelisted_caller()` or `account()` - Creates test accounts

## Testing Your Implementation

### Run Benchmark Tests
```bash
cargo test -p pallet-identity --features runtime-benchmarks
```

### Run All Tests
Run this to check whether your runtime compiles correctly.
```bash
cargo test --features runtime-benchmarks
```

### Check Code Quality
```bash
cargo +nightly fmt
cargo clippy -- -D warnings
```

## Expected Outcomes

After completing this exercise, you should understand:

1. **When parameters matter**: Why some benchmarks need `b` and `j` parameters while others don't
2. **Storage tradeoffs**: The performance implications of different storage patterns
3. **Complexity analysis**: How to identify and measure different algorithmic complexities
4. **Benchmark implementation**: How to write comprehensive, correct benchmarks

## Project Structure

```
pallets/identity/
├── src/
│   ├── lib.rs              # Pallet implementation with extrinsics
│   ├── benchmarking.rs     # 🎯 YOUR ASSIGNMENT - Complete the TODOs
│   ├── weights.rs          # Weight trait and implementations
│   ├── mock.rs             # Test runtime configuration
│   └── tests.rs            # Unit tests
└── Cargo.toml
```

## Educational Context

This Identity pallet is a **simplified version** of Substrate's Identity pallet, designed specifically for benchmarking education. It provides:

### Core Features
- Set/clear identity information with configurable fields
- Economic deposits to prevent spam
- Judgement system for identity validation

### Benchmarking Showcase
- **Linear complexity** - Operations scaling with data size
- **Logarithmic complexity** - Binary search operations
- **Storage pattern comparison** - BoundedVec vs DoubleMap performance
- **Economic operations** - Currency reservation, unreservation
- **Real-world scenarios** - Based on production Substrate patterns

## How to Run

### Individual Pallet Testing
```bash
# Test the identity pallet (primary focus)
cargo t -p pallet-identity

# Test with benchmarks
cargo test -p pallet-identity --features runtime-benchmarks
```

### Full Runtime Build and Testing
```bash
# Build runtime WASM
cargo build -p pba-runtime --release

# Test all pallets
cargo test --all

# Test all with benchmarks
cargo test --all --features runtime-benchmarks
```

### Running with Omni-Node
```bash
# Install required tools
cargo install polkadot-omni-node --locked
cargo install staging-chain-spec-builder --locked

# Create chain spec from WASM
chain-spec-builder create --runtime ./target/release/wbuild/pba-runtime/pba_runtime.wasm --relay-chain westend --para-id 1000 -t development default

# Run omni-node
polkadot-omni-node --chain chain_spec.json --dev-block-time 6000 --tmp
```

## Tips for Success

1. **Start by understanding**: Read the `clear_identity` extrinsic implementation first
2. **Study the patterns**: Look at existing benchmarks to understand the structure
3. **Test frequently**: Run tests after each change to catch issues early
4. **Think about complexity**: Consider what actually makes the operation more expensive
5. **Verify your work**: Ensure your benchmarks test what they claim to test

Good luck with your benchmarking implementation! This exercise will give you valuable hands-on experience with FRAME benchmarking concepts that are essential for production Substrate development.

## Extra Credits

Run the omni bencher and update the `weights.rs` file with your results.
```bash
cargo install frame-omni-bencher --locked
```