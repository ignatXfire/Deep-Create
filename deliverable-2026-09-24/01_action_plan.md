# Action Plan — 2026-09-24

## Goal

Bring Deep Create to a real working system: integrate dc-core with persistence, protocol, worker, broker; confirm end-to-end flow; fix critical bugs.

## Priorities

### P0 - Blockers (without them there is no point in continuing)

1. **dc-core + dc-store integration** - persistence of commands, states, results
2. **dc-core + dc-protocol integration** - interprocess communication
3. **Fix Rejected verifier** - add unit-specific qualification
4. **Fix EntityId reuse** — proposal_id ≠ evidence_id

### P1 - Critical bugs

5. **WorkScope in proposal** - do not ignore scope in self-development
6. **Auth timestamp arithmetic** — checked/saturating arithmetic + future-skew bound
7. **Final review admission** - only from a qualified C-08 VerificationRecord

### P2 - Integration

8. **dc-core + dc-worker** — dispatch work units
9. **dc-core + dc-broker** - via protocol, not directly
10. **End-to-end test** — command → schedule → dispatch → result → verify → persist

## Action plan

### Step 1: dc-core + dc-store (persistence)

**Files:**
- `crates/dc-core/src/persistence.rs` (new)
- `crates/dc-core/src/lib.rs` (add module)

**Tasks:**
- Implement `CoreStore` trait for persistence
- Integrate with dc-store CAS
- Save commands, states, results
- Recovery after restart

### Step 2: dc-core + dc-protocol (interprocess communication)

**Files:**
- `crates/dc-core/src/protocol.rs` (new)
- `crates/dc-core/src/lib.rs` (add module)

**Tasks:**
- Implement DeepCreateCommand processing
- Sending results via protocol
- Integration with dc-protocol codec

### Step 3: Fix Rejected verifier

**Files:**
- `crates/dc-domain/src/work/result.rs`

**Tasks:**
- Add `verify_record_for_unit` for Rejected
- Check the qualifications of the verifier
- Unit-specific qualification for Rejected

### Step 4: Fix EntityId reuse

**Files:**
- `crates/dc-domain/src/self_development.rs`

**Tasks:**
- proposal_id = new trusted EntityId
- evidence_ids = [old evidence id]
- Do not reuse EntityId

### Step 5: WorkScope in proposal

**Files:**
- `crates/dc-domain/src/self_development.rs`

**Tasks:**
- Add WorkScope to proposal scope
- Do not ignore the `_scope` parameter

### Step 6: Auth timestamp arithmetic

**Files:**
- `crates/dc-protocol/src/auth.rs`

**Tasks:**
- checked/saturating arithmetic
- Future timestamp bound
- Boundary tests

### Step 7: Final review admission

**Files:**
- `crates/dc-domain/src/work/result.rs`

**Tasks:**
- ReviewedCompletion only from qualified C-08 VerificationRecord
- Method verification, qualification, criteria, required level, independence, policy revision

### Step 8: dc-core + dc-worker

**Files:**
- `crates/dc-core/src/dispatcher.rs` (new)
- `crates/dc-worker/src/lib.rs`

**Tasks:**
- Implement WorkDispatcher for dc-worker
- Dispatch work units via dc-core

### Step 9: dc-core + dc-broker

**Files:**
- `crates/dc-core/src/broker.rs` (new)
- `crates/dc-broker/src/lib.rs`

**Tasks:**
- Remove dc-worker → dc-broker dependency
- dc-core → dc-broker via protocol
- Real network dispatch (not STUB)

### Step 10: End-to-end test

**Files:**
- `crates/dc-core/tests/integration.rs` (new)

**Tasks:**
- Full cycle: command → schedule → dispatch → result → verify → persist
- Checking all components
- Checking recovery after restart

## Resources

- **Time**: 1-2 days for P0-P1, 1-2 weeks for P2
- **Tools**: cargo, git, node (architecture checker)
- **Limitations**: offline build, Windows host, bash shell

## Success criteria

- [ ] All tests pass (369+)
- [ ] Architecture checker PASS
- [ ] End-to-end test PASS
- [ ] Persistence works (restart recovery)
- [ ] Protocol integration works
- [ ] Critical bugs fixed

## Risks

1. **Integration complexity** - dc-core + dc-store + dc-protocol + dc-worker + dc-broker
2. **Time** - may take longer than expected
3. **Quality** - you need to maintain high code quality
4. **Testing** - you need to write tests for all new components

## Next steps after this plan

1. dc-store Stage 3 (migrations, WAL, encryption, recovery)
2. dc-windows runtime (Job Object, AppContainer, named pipe, DPAPI)
3. Self-development full runtime
4.Hardened canonical V1
