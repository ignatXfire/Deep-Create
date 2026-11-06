//! Defect probes: success means current behavior was reproduced, not repaired.
#[cfg(test)]
mod tests {
    use dc_domain::ids::{EntityId, RowVersion, UnitRevision};
    use dc_domain::work::*;
    use dc_store::{EventRecord, SQLiteStore, StoreBackend, Stage3Backend};
    use dc_store::stage3::{EvidenceArtifact, JournalEvent, Stage3CanonicalBackend};
    use std::{path::Path, num::NonZeroU32};
    fn id(n: u128) -> EntityId {
        EntityId::from_u128(0x017f_22e2_79b0_7cc3_98c4_dc0c_0c07_398f + n).unwrap()
    }
    fn store() -> SQLiteStore {
        let mut s = SQLiteStore::new(":memory:");
        s.initialize(Path::new(":memory:")).unwrap(); s
    }
    #[test]
    fn fixed_event_duplicate_rejected_without_destroying_prior_record() {
        let mut s=store();
        s.write_bytes(&id(1), b"parent", dc_store::ExpectedVersion::MustNotExist).unwrap();
        let mut e=EventRecord { event_id:"same-id".into(), event_type:"audit".into(), entity_id:id(1), payload:b"first".to_vec(), created_at:1 };
        s.put_event(&e).unwrap(); e.payload=b"second".to_vec();
        assert!(s.put_event(&e).is_err());
        assert_eq!(s.events_for_entity(&id(1)).unwrap()[0].payload,b"first");
    }
    #[test]
    fn future_schema_ten_silently_relabelled_three() {
        let path=std::path::PathBuf::from(std::env::var("AUDIT_TMP").unwrap()).join("schema-ten.sqlite");
        { let mut s=SQLiteStore::new(path.to_str().unwrap()); s.initialize(&path).unwrap(); }
        { let c=rusqlite::Connection::open(&path).unwrap(); c.execute("UPDATE store_meta SET value='10' WHERE key='schema_version'",[]).unwrap(); }
        { let mut s=SQLiteStore::new(path.to_str().unwrap()); assert!(s.initialize(&path).is_ok()); }
        let c=rusqlite::Connection::open(&path).unwrap();
        let version:String=c.query_row("SELECT value FROM store_meta WHERE key='schema_version'",[],|r|r.get(0)).unwrap();
        assert_eq!(version,"3");
    }
    #[test]
    fn evidence_same_bytes_lose_second_entity_provenance() {
        let mut s=store();
        for n in [1,2] { s.write_bytes(&id(n), b"parent", dc_store::ExpectedVersion::MustNotExist).unwrap(); }
        let a=EvidenceArtifact::new(Some(id(1)),"text/log".into(),"author-A".into(),b"identical bytes".to_vec(),1);
        let b=EvidenceArtifact::new(Some(id(2)),"text/log".into(),"author-B".into(),b"identical bytes".to_vec(),2);
        assert_eq!(s.evidence_put(&a).unwrap(),s.evidence_put(&b).unwrap());
        assert_eq!(s.evidence_list_by_entity(&id(1)).unwrap().len(),1);
        assert!(s.evidence_list_by_entity(&id(2)).unwrap().is_empty());
    }
    #[test]
    fn journal_unsigned_sequence_breaks_head_order() {
        let mut s=store();
        for (name,seq) in [("one",1),("large",1u64<<63)] {
            s.journal_append(&JournalEvent::new(name.into(),"stream".into(),seq,"audit".into(),"tester".into(),b"bytes".to_vec(),1)).unwrap();
        }
        assert_eq!(s.journal_head_sequence("stream").unwrap(),Some(1));
        assert_eq!(s.journal_read_stream("stream").unwrap()[0].sequence,1u64<<63);
    }
    #[test]
    fn machine_pass_without_criterion_evidence_and_with_wrong_unit() {
        let scope=WorkScope {session_id:id(1),work_id:id(2),objective_id:id(3),objective_revision:UnitRevision::new(1).unwrap()};
        let unit=WorkUnitSpec { unit:UnitRef {unit_id:id(4),revision:UnitRevision::new(1).unwrap()},scope,parent:None,kind:WorkKind::Research,expected_output:OutputContract {export_name:"real-goal".into(),schema_version:NonZeroU32::new(1).unwrap(),fields:vec![]},obligation_ids:vec![id(12)],acceptance_criteria:vec![id(13)],bounds:WorkBounds {max_attempts:NonZeroU32::new(1).unwrap(),max_steps:NonZeroU32::new(2).unwrap(),stop_conditions:vec![StopCondition::OutputProduced]},exploration:None,stage:UnitStage::Ready,accepted_result:None,row_version:RowVersion::new(1).unwrap()};
        let wrong_unit=UnitRef {unit_id:id(9),revision:UnitRevision::new(1).unwrap()};
        let result=ResultPacket {result_id:id(5),attempt_id:id(6),scope,unit:wrong_unit,snapshot:DependencySnapshot {snapshot_id:id(8),scope,unit:wrong_unit,read_set:vec![],policy_versions:vec![]},fingerprint:[1;32],verification_policy:VersionedDependency {key:DependencyKey {kind:DependencyKind::VerificationCriteria,entity_id:id(10)},expected_revision:RowVersion::new(1).unwrap(),fingerprint:None},used_links:vec![],stage:ResultStage::Submitted,row_version:RowVersion::new(1).unwrap()};
        let outcome=dc_core::verification::run_machine_verification(&unit,&result,id(20)).unwrap();
        assert!(matches!(outcome,dc_core::verification::VerificationOutcome::Passed {..}));
    }
    #[cfg(windows)]
    #[test]
    fn impersonation_guard_is_movable_between_threads() {
        fn send_sync<T:Send+Sync>() {}
        send_sync::<dc_windows::named_pipe::ImpersonationGuard>();
    }
}
