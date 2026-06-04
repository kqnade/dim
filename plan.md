# Plan: Comprehensive Test Coverage for Dim

## Goal
Establish meaningful test coverage across all Dim modules using TDD principles, coordinated via swarm coding, with full QA infrastructure.

## Stage 1 — QA Infrastructure (test-suite-architect)
- Create `tests/` directory structure
- Set up test case tracking CSV
- Define quality gates (target: 80%+ coverage on core modules)
- Create test strategy document

## Stage 2 — Module Analysis & Contracts (swarm-coding prep)
- Group modules by dependency and testability
- Define public interfaces each worker must test
- Assign ownership boundaries

## Stage 3 — Parallel TDD Implementation (swarm-coding + test-driven-dev)
Deploy 4 coder agents in parallel on independent module groups:

| Agent | Module Group | Focus | Why Independent |
|-------|-------------|-------|-----------------|
| A | position, selection, buffer, undo | Pure data structures | Zero external deps, deterministic |
| B | file_io, config, terminal | I/O & configuration | File system and env deps, isolated |
| C | command, input, keymap | Command processing | Logic-heavy, mockable inputs |
| D | editor_state | Core editor logic | Most complex, builds on A's types |

Renderer, app, skk are excluded from initial pass — they need terminal/UI mocking and are harder to unit-test. They'll be Stage 4.

Each agent follows test-driven-dev:
1. Read module source
2. Identify untested public behaviors
3. Write ONE test → watch it fail (RED)
4. Add/fix minimal code → watch it pass (GREEN)
5. Repeat vertically (one test at a time)
6. Refactor only when GREEN

## Stage 4 — Integration & Validation
- Merge all agent branches
- Run full `cargo test`
- Calculate coverage metrics
- Generate QA report

## Quality Gates
- [ ] All modules in Groups A–D have `#[cfg(test)]` with meaningful coverage
- [ ] `cargo test` passes 100%
- [ ] No test tests implementation details (behavior-focused)
- [ ] Each test would survive internal refactor
