# Baseline Metrics

## Current State (Pre-Improvement)
- Total source modules: 15
- Modules with test blocks: 13
- Total test functions: ~20 (estimated, mostly constructors)
- Meaningful behavior coverage: <5%
- cargo test status: passes (minimal tests)

## Target State
- All core modules (Groups A-D) have behavior-focused tests
- cargo test: 100% pass
- Test quality: behavior-focused, not implementation-coupled
