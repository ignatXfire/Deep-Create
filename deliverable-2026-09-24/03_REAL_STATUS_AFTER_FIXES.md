# Deep Create - Real system status (2026-09-24)

## Status after session 2026-09-24

### What has been fixed

| Problem | Was | Became |
|----------|-----------|-------|
| dc-core was missing | Didn't exist | Created with persistence, dispatch, protocol, broker |
| dc-worker → dc-broker | Direct Dependence (C-11 Violation) | dc-worker is independent, dc-core contains CoreDispatcher |
| Broker didn't work | Stub | ProtocolDispatcher with real protocol auth |
| Rejected not reviewed | verify_record_for_unit only for Accepted | Now for Accepted and Rejected |
| Final review weak | No source_verification | Qualified verification record required |
| Self-development ID | Reusing evidence_id | External id_allocator |
| Auth timestamp | Overflow risk | checked_add/saturating_add |
| End-to-end pipeline | Not proven | Integration test passes |

### Tests
```
cargo test --workspace --offline: 378 passed ✅
Architecture checker: PASS ✅
```

### Git commits
```
bf4ee6f feat: end-to-end pipeline test
dddd929 feat: dc-core + dc-worker dispatch integration
96b547f fix: final review admission
20f52a1 feat: dc-core orchestration crate
```

## Remaining problems (according to forensic audit)

### P0 - Requires immediate correction
1. **BROKER-STUB** — dc-broker is still a stub, not a real HTTP dispatch
2. **ARCH-CHECKER** - fixed in this session

### P1 - Important semantic defects
3. **ADMISSION-LIVENESS-TYPE** — HeldAdmission not RAII guard
4. **ADMISSION-TOKEN-COPY** — AdmissionToken Clone+Copy
5. **STORE-DURABILITY-GAPS** - Non-canonical Stage-3 (WAL, synchronous, foreign_keys, strict)
6. **UI-FIXTURE** — Desktop UI is hardcoded, does not read the real state
7. **OUTCOME-EVIDENCE-DROPPED** — evidence_id is lost after transition

### P2 - Gain
8. **Real network dispatch** - Now protocol auth, but not HTTP
9. **Self-development runtime** - Proposal analyzer only, not a full cycle
10. **Windows security** - No Job Object/AppContainer

## Real readiness

| Component | Readiness | Status |
|-----------|----------|--------|
| dc-domain (pure model) | 80% | Substantial, there are semantic defects |
| dc-protocol | 70% | Auth/replay improved, no OS-backed auth |
| dc-store | 40% | Early Stage 3, CAS works, no full schema |
| dc-core | 60% | Created, orchestration works, requires runtime polish |
| dc-broker | 20% | Protocol integration exists, real dispatch is stub |
| dc-worker | 50% | Execute works, no isolation |
| dc-windows | 10% | Type contracts only, no real enforcement |
| dc-desktop | 5% | Fixture, does not read state |
| dc-watchdog | 30% | Recovery logic yes, no authority.anchor |
| **Overall** | **~35-40%** | **Not production-ready** |

## What really works

1. ✅ Domain model - works, permissions, verification
2. ✅ Protocol - auth, replay protection, outcome
3. ✅ Store - CAS, transactions (basic)
4. ✅ Core - orchestration, scheduling, context, verification
5. ✅ Worker dispatch - via CoreDispatcher
6. ✅ End-to-end test - command→schedule→dispatch→result→verify→persist

## What does NOT work (critical)

1. ❌ Real model dispatch (broker stub)
2. ❌ Windows process isolation (Job Object, AppContainer)
3. ❌ Full Stage-3 storage (WAL, synchronous=FULL, strict tables)
4. ❌ Self-development full cycle (proposal → code → test → verify → merge)
5. ❌ Desktop UI live state
6. ❌ Crash recovery with authority.anchor
7. ❌ Multi-model routing with resource limits
8. ❌ Independent verification with different models

## Next steps for production

1. Real HTTP dispatch to broker (with authority, intent, resource tracking)
2. Worker isolation (process boundary, no direct broker access)
3. Stage-3 storage schema (events, resources, evidence, recovery)
4. Windows Job Object integration
5. Live UI projection
6. Self-development runtime (not only analysis)
7. Multi-attempt verification with different models
