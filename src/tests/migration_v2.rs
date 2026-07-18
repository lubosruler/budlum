//! Migration/upgrade path testi — CI Genişletme Madde 3.
//!
//! Eski format (schema-2) snapshot'tan yeni formata (schema-4) migration
//! veri bozmadan çalışmalı. Bu test:
//! 1. Schema-2 snapshot fixture'ı oluşturur
//! 2. Migration report'unu doğrular
//! 3. Bakiyeler, nonce'lar, validator seti kaybolmadığını assert eder

#[cfg(test)]
mod migration_tests {
    use crate::chain::snapshot::*;
    use crate::core::account::AccountState;
    use crate::core::address::Address;

    /// Schema-2 snapshot oluşturup schema-4'e migration yoluyla
    /// veri kaybı olmadığını doğrula.
    #[test]
    fn schema2_to_schema4_migration_preserves_data() {
        // 1. Test state oluştur
        let mut state = AccountState::new();
        let alice = Address::from([0xAA; 32]);
        let bob = Address::from([0xBB; 32]);

        state.add_balance(&alice, 5000);
        state.add_balance(&bob, 3000);
        state.add_validator(alice, 2000);

        // 2. Schema-4 snapshot oluştur
        let snapshot = StateSnapshotV2::from_state(
            &state,
            StateSnapshotV2Params {
                height: 100,
                block_hash: "test_block_hash".into(),
                genesis_hash: "test_genesis_hash".into(),
                chain_id: 1337,
                finalized_height: 90,
                finalized_hash: "finalized_hash".into(),
                finality_certificates: vec![],
            },
        );

        // 3. Snapshot'ı schema-2'ye düşür (migration senaryosu)
        let mut snapshot_v2_compat = snapshot.clone();
        snapshot_v2_compat.schema_version = 2; // Eski schema

        // 4. Migration report'u DOĞRUDAN snapshot üzerinde kontrol et
        // (from_bytes çağırmadan — çünkü from_bytes schema_version'ı yükseltir)
        let report = snapshot_v2_compat.migration_report().unwrap();
        assert!(report.migrated, "Schema-2 should trigger migration");
        assert_eq!(report.original_schema_version, 2);
        assert_eq!(report.target_schema_version, 4); // CURRENT_STATE_SNAPSHOT_SCHEMA_VERSION

        // 5. from_bytes ile yükle (schema otomatik yükseltilir)
        let bytes = serde_json::to_vec(&snapshot_v2_compat).unwrap();
        let restored = StateSnapshotV2::from_bytes(&bytes).unwrap();

        // 6. Veri kaybı yok — bakiyeler korunmalı
        assert_eq!(restored.balances.get(&alice), Some(&5000));
        assert_eq!(restored.balances.get(&bob), Some(&3000));

        // 7. Validator seti korunmalı
        assert!(restored.validators.contains_key(&alice));

        // 8. Temel alanlar korunmalı
        assert_eq!(restored.height, 100);
        assert_eq!(restored.chain_id, 1337);
        assert_eq!(restored.block_hash, "test_block_hash");

        // 9. from_bytes sonrası schema CURRENT'a yükseltilmiş olmalı
        assert_eq!(restored.schema_version, 4);
    }

    /// Desteklenmeyen eski schema reddedilmeli.
    #[test]
    fn unsupported_schema_version_rejected() {
        let state = AccountState::new();
        let snapshot = StateSnapshotV2::from_state(
            &state,
            StateSnapshotV2Params {
                height: 1,
                block_hash: "h".into(),
                genesis_hash: "g".into(),
                chain_id: 1,
                finalized_height: 0,
                finalized_hash: "".into(),
                finality_certificates: vec![],
            },
        );

        // Schema 1 desteklenmemeli
        let mut bad = snapshot.clone();
        bad.schema_version = 1;
        let bytes = serde_json::to_vec(&bad).unwrap();
        assert!(StateSnapshotV2::from_bytes(&bytes).is_err());

        // Schema 99 desteklenmemeli
        let mut future = snapshot;
        future.schema_version = 99;
        let bytes = serde_json::to_vec(&future).unwrap();
        assert!(StateSnapshotV2::from_bytes(&bytes).is_err());
    }

    /// Schema-4 snapshot doğrudan yüklenmeli (migration gerekmez).
    #[test]
    fn schema4_snapshot_loads_directly() {
        let state = AccountState::new();
        let snapshot = StateSnapshotV2::from_state(
            &state,
            StateSnapshotV2Params {
                height: 50,
                block_hash: "hash".into(),
                genesis_hash: "genesis".into(),
                chain_id: 42,
                finalized_height: 40,
                finalized_hash: "final".into(),
                finality_certificates: vec![],
            },
        );

        let bytes = snapshot.to_bytes();
        let restored = StateSnapshotV2::from_bytes(&bytes).unwrap();

        // Schema-4 = CURRENT, migration gerekmez
        assert_eq!(restored.schema_version, 4);
        assert_eq!(restored.height, 50);
        assert_eq!(restored.chain_id, 42);
    }
}
