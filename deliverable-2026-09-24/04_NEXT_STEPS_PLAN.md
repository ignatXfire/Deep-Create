# Deep Create - Plan for further work (2026-09-24)

## Status after corrections

### Corrected in this session

| Problem | File | Status |
|----------|-----------|--------|
| P1-OUTCOME-EVIDENCE-DROPPED | dc-watchdog/src/recovery.rs | ✅ Corrected |
| P1-ADMISSION-TOKEN-COPY | dc-windows/src/admission.rs | ✅ Corrected |
| P1-ADMISSION-LIVENESS-TYPE | dc-windows/src/admission.rs | ✅ Corrected |
| ARCH-CHECKER | tools/check-architecture.mjs | ✅ Corrected |

### Already fixed before

| Problem | Status |
|----------|--------|
| dc-core was missing | ✅ Created |
| dc-worker → dc-broker dependency | ✅ Removed |
| Rejected verifier | ✅ Corrected |
| Final review | ✅ Corrected |
| Self-development ID | ✅ Corrected |
| Auth timestamp | ✅ Corrected |
| End-to-end test | ✅ Working |

## Remaining critical tasks

### P0 - Block production

1. **BROKER-STUB** - dc-broker does not do real HTTP dispatch
   - Needed: add reqwest/ureq, a real HTTP client
   - Blocks: any real work with models
   - Difficulty: Medium (2-3 hours)

2. **Real Process Isolation** - Windows Job Object/AppContainer
   - Need: real Win32 API calls
   - Blocks: safe code execution
   - Difficulty: Very high (1-2 days)

### P1 - Important semantic defects

3. **STORE-DURABILITY-GAPS** - Non-canonical Stage-3
   - Already have: WAL, synchronous=FULL, foreign_keys, STRICT
   - Need: full scheme events/resources/evidence/recovery
   - Difficulty: High (4-6 hours)

4. **UI-FIXTURE** - Desktop UI hardcoded
   - Needed: reading real status from C-03/C-14
   - Difficulty: Medium (2-3 hours)

5. **Self-development runtime** - Proposal analyzer only
   - Required: full cycle proposal → code → test → verify → merge
   - Difficulty: Very high (2-3 days)

### P2 - Gain

6. **Multi-model routing** — No model selection by capability
7. **Resource limits** - No tokens/cost tracking
8. **Crash recovery** - authority.anchor not implemented
9. **Independent verification** - No multi-model verify

## Recommended operating procedure

### Stage 1: Real broker (today)
1. Add reqwest to workspace
2. Implement real HTTP dispatch in dc-broker
3. Add timeout/retry/circuit breaker
4. Tests with mock server

### Stage 2: Storage Stage 3 (tomorrow)
1. Events table
2. Resources table  
3. Evidence table
4. Recovery procedures
5. Crash tests

### Stage 3: Windows security (day after tomorrow)
1. Job Object creation
2.AppContainer profile
3. Process isolation tests

### Stage 4: UI + Self-development (3-4 days)
1. Live UI projection
2. Self-development runtime
3.Multi-model verification

## Readiness metrics

| Component | Was | Became | Goal |
|-----------|-----------|-------|------|
| dc-domain | 80% | 80% | 90% |
| dc-protocol | 70% | 70% | 85% |
| dc-store | 40% | 45% | 80% |
| dc-core | 60% | 60% | 85% |
| dc-broker | 20% | 20% | 70% |
| dc-worker | 50% | 50% | 75% |
| dc-windows | 10% | 25% | 60% |
| dc-desktop | 5% | 5% | 50% |
| dc-watchdog | 30% | 40% | 70% |
| **Overall** | **35%** | **40%** | **75%** |

## Risks

1. **Time**: Full implementation requires 1-2 weeks
2. **Difficulty**: Windows security requires low-level Win32
3. **Testing**: Need real models for integration tests
4. **Regressions**: Each change can break 378 tests

## Conclusion

The system has a solid architectural foundation, but requires significant development for production. Critical path: broker → storage → security → UI/self-dev.
