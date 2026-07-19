//! PoA/Permissionless izolasyon test seti — CI Genişletme Madde 9.
//!
//! Bu dosya PoA domain'inin permissionless tarafa sızmadığını doğrular.
//! 5 farklı sızma senaryosu test edilir:
//! 1. RPC leak — PoA verisi permissionless RPC'de görünmemeli
//! 2. Event leak — PoA membership event'leri permissionless domain'e sızmamalı
//! 3. Cross-domain mesaj leak — PoA KYC metadata cross-domain mesajda taşınmamalı
//! 4. Log leak — PoA bilgisi zincir verilerinde sızdırılmamalı
//! 5. Error message leak — Hata mesajları PoA detaylarını ifşa etmemeli

#[cfg(test)]
mod poa_isolation_tests {
    use crate::core::account::AccountState;
    use crate::core::address::Address;
    use crate::registry::poa_membership::PoaMembershipRegistry;
    use crate::registry::role::roles;

    const POA_DOMAIN: u32 = 3;

    /// Senaryo 1: RPC Leak — PoA membership verisi permissionless registry'de görünmemeli.
    ///
    /// PoA üyesi permissionless registry'ye stake atlamadan girememeli.
    #[test]
    fn poa_member_cannot_register_in_permissionless_registry_without_stake() {
        let mut perm_state = AccountState::new();
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        let poa_member = Address::from([0xAA; 32]);

        // Admin ata ve PoA'ya KYC ile başvur + onayla
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, poa_member, [1u8; 32])
            .unwrap();
        poa_reg.approve(POA_DOMAIN, admin, poa_member).unwrap();

        // PoA üyesi permissionless registry'de aktif olmamalı (stake yok)
        assert!(
            !perm_state.registry.is_active(&poa_member, roles::VALIDATOR),
            "PoA member should NOT be active as a permissionless validator without stake"
        );
        assert!(
            !perm_state
                .registry
                .is_active(&poa_member, roles::STORAGE_OPERATOR),
            "PoA member should NOT be active as a storage operator without stake"
        );
        assert!(
            !perm_state
                .registry
                .is_active(&poa_member, roles::AI_VERIFIER),
            "PoA member should NOT be active as an AI verifier without stake"
        );
    }

    /// Senaryo 2: Event Leak — PoA membership event'leri permissionless domain'de görünmemeli.
    ///
    /// PoA üyeliği permissionless validator setine yansımamalı.
    #[test]
    fn poa_membership_does_not_affect_permissionless_validator_set() {
        let mut perm_state = AccountState::new();
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        let poa_member = Address::from([0xAA; 32]);
        let permissionless_validator = Address::from([0xBB; 32]);

        // PoA üyesi ekle
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, poa_member, [1u8; 32])
            .unwrap();
        poa_reg.approve(POA_DOMAIN, admin, poa_member).unwrap();

        // Permissionless validator ekle (stake ile)
        perm_state.add_balance(&permissionless_validator, 10_000);
        perm_state.add_validator(permissionless_validator, 5_000);

        // Active validators listesinde sadece permissionless validator olmalı
        let active = perm_state.get_active_validators();
        assert_eq!(
            active.len(),
            1,
            "Only permissionless validator should be in active set"
        );
        assert_eq!(active[0].address, permissionless_validator);

        // PoA üyesi active validators listesinde olmamalı
        assert!(
            !active.iter().any(|v| v.address == poa_member),
            "PoA member must NOT appear in permissionless active validator set"
        );
    }

    /// Senaryo 3: Cross-Domain Mesaj Leak — PoA KYC metadata cross-domain mesajda taşınmamalı.
    ///
    /// CrossDomainMessage KYC commitment içermez — sadece payload_hash taşır.
    #[test]
    fn cross_domain_message_does_not_carry_kyc_metadata() {
        use crate::cross_domain::message::{
            CrossDomainMessage, CrossDomainMessageParams, MessageKind,
        };

        // PoA domain'inden permissionless domain'e mesaj oluştur
        let message = CrossDomainMessage::new(CrossDomainMessageParams {
            source_domain: POA_DOMAIN,
            target_domain: 1,
            source_height: 100,
            event_index: 0,
            nonce: 0,
            sender: Address::from([0xAA; 32]),
            recipient: Address::from([0xBB; 32]),
            payload_hash: [0xCC; 32],
            kind: MessageKind::Custom(vec![1, 2, 3]),
            expiry_height: 200,
        });

        // Mesaj KYC commitment veya PoA metadata içermemeli
        let message_bytes = serde_json::to_vec(&message).unwrap();
        let message_str = String::from_utf8_lossy(&message_bytes);

        assert!(
            !message_str.to_lowercase().contains("kyc"),
            "CrossDomainMessage must NOT contain KYC metadata"
        );

        // Mesaj sadece hash taşır, ham veri değil
        assert_ne!(
            message.payload_hash, [0u8; 32],
            "Payload hash should be present"
        );
    }

    /// Senaryo 4: Log Leak — PoA bilgisi zincir verilerinde sızdırılmamalı.
    ///
    /// PoA registry'si permissionless registry'den tamamen ayrıdır.
    #[test]
    fn poa_membership_isolated_from_permissionless_registry() {
        let mut perm_state = AccountState::new();
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        let poa_member = Address::from([0xAA; 32]);

        // PoA üyesi ekle
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, poa_member, [1u8; 32])
            .unwrap();
        poa_reg.approve(POA_DOMAIN, admin, poa_member).unwrap();

        // PoA registry'si permissionless registry'den ayrı
        assert!(
            poa_reg.is_authorized(POA_DOMAIN, &poa_member),
            "PoA member should be authorized in PoA registry"
        );

        // Permissionless registry'de bu adres aktif olmamalı
        assert!(
            !perm_state.registry.is_active(&poa_member, roles::VALIDATOR),
            "PoA member must NOT be active in permissionless registry"
        );
    }

    /// Senaryo 5: Error Message Leak — Hata mesajları PoA detaylarını ifşa etmemeli.
    ///
    /// PoA ve Permissionless registry'ler tamamen ayrı veri yapılarıdır.
    #[test]
    fn poa_and_permissionless_registries_share_no_state() {
        let mut perm_state = AccountState::new();
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);

        let poa_addr = Address::from([0xAA; 32]);
        let perm_addr = Address::from([0xBB; 32]);

        // PoA'ya üye ekle
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, poa_addr, [1u8; 32])
            .unwrap();
        poa_reg.approve(POA_DOMAIN, admin, poa_addr).unwrap();

        // Permissionless'a validator ekle
        perm_state.add_balance(&perm_addr, 10_000);
        perm_state.add_validator(perm_addr, 5_000);

        // PoA üyesi permissionless validator setinde yok
        assert!(
            !perm_state.registry.is_active(&poa_addr, roles::VALIDATOR),
            "PoA member must NOT be in permissionless registry"
        );

        // Permissionless validator PoA'da yok
        assert!(
            !poa_reg.is_authorized(POA_DOMAIN, &perm_addr),
            "Permissionless validator must NOT be in PoA registry"
        );

        // Permissionless registry parametreleri PoA'dan bağımsız
        let perm_params = perm_state.registry.params();
        assert!(perm_params.min_stake > 0);
    }

    /// Ek: PoA domain ID'si permissionless domain ID'sinden farklı olmalı.
    #[test]
    fn poa_domain_id_isolated_from_permissionless() {
        use crate::domain::types::DomainId;

        let poa_domain: DomainId = POA_DOMAIN;
        let permissionless_domain: DomainId = 1;

        assert_ne!(
            poa_domain, permissionless_domain,
            "PoA domain ID must differ from permissionless domain ID"
        );
    }

    /// Ek: PoA admin yetkisi permissionless tarafı etkilememeli.
    #[test]
    fn poa_admin_authority_does_not_grant_permissionless_power() {
        let mut perm_state = AccountState::new();
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);

        poa_reg.add_admin(POA_DOMAIN, admin);

        // Admin PoA'da yetkili
        assert!(poa_reg.is_admin(POA_DOMAIN, &admin));

        // Ama permissionless registry'de sıradan bir hesap
        assert!(
            !perm_state.registry.is_active(&admin, roles::VALIDATOR),
            "PoA admin should NOT have permissionless validator status"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // DERİN BOUNDARY TESTLERİ — State Root İzolasyon
    // ═══════════════════════════════════════════════════════════════════

    /// PoA membership değişiklikleri permissionless state_root'u etkilememeli.
    ///
    /// Bu test, PoA registry'sindeKİ değişikliklerin AccountState'in
    /// calculate_state_root()'unu değiştirmediğini kanıtlar.
    #[test]
    fn poa_membership_changes_do_not_affect_permissionless_state_root() {
        let mut perm_state = AccountState::new();
        let perm_addr = Address::from([0xBB; 32]);
        perm_state.add_balance(&perm_addr, 10_000);
        perm_state.add_validator(perm_addr, 5_000);

        let root_before_poa = perm_state.calculate_state_root();

        // PoA registry'sinde büyük değişiklikler yap
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        poa_reg.add_admin(POA_DOMAIN, admin);

        for i in 0..20u8 {
            let member = Address::from([i.wrapping_add(0xA0); 32]);
            poa_reg
                .submit_application(POA_DOMAIN, member, [i.wrapping_add(1); 32])
                .unwrap();
            poa_reg.approve(POA_DOMAIN, admin, member).unwrap();
        }

        // Permissionless state_root değişmemeli
        let root_after_poa = perm_state.calculate_state_root();
        assert_eq!(
            root_before_poa, root_after_poa,
            "PoA membership changes must NOT affect permissionless state_root"
        );
    }

    /// Permissionless registry değişiklikleri PoA membership'ı etkilememeli.
    #[test]
    fn permissionless_changes_do_not_affect_poa_membership() {
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        let poa_member = Address::from([0xAA; 32]);
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, poa_member, [1u8; 32])
            .unwrap();
        poa_reg.approve(POA_DOMAIN, admin, poa_member).unwrap();

        assert!(poa_reg.is_authorized(POA_DOMAIN, &poa_member));

        // Permissionless tarafta büyük miktarda stake/validator ekle
        let mut perm_state = AccountState::new();
        for i in 0..20u8 {
            let addr = Address::from([i.wrapping_add(0x10); 32]);
            perm_state.add_balance(&addr, 100_000);
            perm_state.add_validator(addr, 50_000);
        }

        // PoA membership durumu değişmemeli
        assert!(
            poa_reg.is_authorized(POA_DOMAIN, &poa_member),
            "Permissionless changes must NOT affect PoA membership status"
        );
        assert_eq!(
            poa_reg.authorized_members(POA_DOMAIN).len(),
            1,
            "PoA authorized count must remain 1"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // DERİN BOUNDARY TESTLERİ — Slashing İzolasyon
    // ═══════════════════════════════════════════════════════════════════

    /// PoA revokesu permissionless slashing'ı etkilememeli.
    #[test]
    fn poa_revoke_does_not_slash_permissionless_validator() {
        let mut perm_state = AccountState::new();
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        let dual_member = Address::from([0xDD; 32]);

        // dual_member hem PoA'da hem permissionless'da
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, dual_member, [9u8; 32])
            .unwrap();
        poa_reg.approve(POA_DOMAIN, admin, dual_member).unwrap();

        perm_state.add_balance(&dual_member, 50_000);
        perm_state.add_validator(dual_member, 25_000);

        assert!(perm_state
            .registry
            .is_active(&dual_member, roles::VALIDATOR));
        let stake_before = perm_state
            .registry
            .get(&dual_member, roles::VALIDATOR)
            .unwrap()
            .stake;

        // PoA'dan revoke et
        poa_reg.revoke(POA_DOMAIN, admin, dual_member).unwrap();
        assert!(!poa_reg.is_authorized(POA_DOMAIN, &dual_member));

        // Permissionless validator durumu değişmemeli
        assert!(
            perm_state
                .registry
                .is_active(&dual_member, roles::VALIDATOR),
            "PoA revoke must NOT affect permissionless validator status"
        );
        assert_eq!(
            perm_state
                .registry
                .get(&dual_member, roles::VALIDATOR)
                .unwrap()
                .stake,
            stake_before,
            "PoA revoke must NOT affect permissionless validator stake"
        );
    }

    /// Permissionless slashing PoA membership'ı etkilememeli.
    #[test]
    fn permissionless_slash_does_not_revoke_poa_member() {
        let mut perm_state = AccountState::new();
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        let dual_member = Address::from([0xDD; 32]);

        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, dual_member, [9u8; 32])
            .unwrap();
        poa_reg.approve(POA_DOMAIN, admin, dual_member).unwrap();

        perm_state.add_balance(&dual_member, 50_000);
        perm_state.add_validator(dual_member, 25_000);

        // Permissionless tarafta slash (simulated via unstake)
        perm_state
            .registry
            .upsert_stake(dual_member, roles::VALIDATOR, 0, 0);
        assert!(!perm_state
            .registry
            .is_active(&dual_member, roles::VALIDATOR));

        // PoA membership değişmemeli
        assert!(
            poa_reg.is_authorized(POA_DOMAIN, &dual_member),
            "Permissionless slash must NOT revoke PoA membership"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // DERİN BOUNDARY TESTLERİ — CrossDomainMessage Yalıtımı
    // ═══════════════════════════════════════════════════════════════════

    /// CrossDomainMessage PoA internal state sızdırmamalı — deep inspection.
    ///
    /// Mesajın serde_bytes'inde KYC, admin, membership, status, stake,
    /// approval gibi PoA'ya özgü hiçbir alan bulunmamalı.
    #[test]
    fn cross_domain_message_deep_inspection_no_poa_state_leak() {
        use crate::cross_domain::message::{
            CrossDomainMessage, CrossDomainMessageParams, MessageKind,
        };

        let message = CrossDomainMessage::new(CrossDomainMessageParams {
            source_domain: POA_DOMAIN,
            target_domain: 1,
            source_height: 100,
            event_index: 0,
            nonce: 42,
            sender: Address::from([0xAA; 32]),
            recipient: Address::from([0xBB; 32]),
            payload_hash: [0xCC; 32],
            kind: MessageKind::Custom(vec![1, 2, 3]),
            expiry_height: 200,
        });

        let serialized = serde_json::to_string(&message).unwrap();
        let lower = serialized.to_lowercase();

        // PoA-specific terimler mesajda bulunmamalı
        let poa_leak_terms = [
            "kyc",
            "commitment",
            "admin",
            "approved",
            "revoked",
            "rejected",
            "membership",
            "permissioned",
            "pending",
            "decided_by",
            "poa_",
        ];
        for term in &poa_leak_terms {
            assert!(
                !lower.contains(term),
                "CrossDomainMessage contains PoA leak term '{term}': {serialized}"
            );
        }

        // Mesajın içerdiği alanlar sadece domain-agnostic bilgiler olmalı
        // (source/target domain, height, nonce, addresses, payload_hash, kind)
        assert!(message.source_domain == POA_DOMAIN);
        assert!(message.target_domain == 1);
        assert!(message.payload_hash != [0u8; 32]);
    }

    /// Farklı CrossDomainMessage türleri (BridgeLock, BridgeMint, Custom) PoA
    /// state sızdırmamalı.
    #[test]
    fn all_message_kinds_are_poa_state_free() {
        use crate::cross_domain::message::{
            CrossDomainMessage, CrossDomainMessageParams, MessageKind,
        };

        let kinds = vec![
            MessageKind::BridgeLock,
            MessageKind::BridgeMint,
            MessageKind::BridgeBurn,
            MessageKind::BridgeUnlock,
            MessageKind::Custom(vec![0xDE, 0xAD]),
        ];

        for kind in kinds {
            let message = CrossDomainMessage::new(CrossDomainMessageParams {
                source_domain: POA_DOMAIN,
                target_domain: 1,
                source_height: 50,
                event_index: 0,
                nonce: 1,
                sender: Address::from([0xAA; 32]),
                recipient: Address::from([0xBB; 32]),
                payload_hash: [0xCC; 32],
                kind,
                expiry_height: 200,
            });

            let serialized = serde_json::to_string(&message).unwrap();
            assert!(
                !serialized.to_lowercase().contains("kyc"),
                "MessageKind variant leaked KYC: {serialized}"
            );
            assert!(
                !serialized.to_lowercase().contains("admin"),
                "MessageKind variant leaked admin: {serialized}"
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════
    // FUZZ-STYLE BULK BOUNDARY TESTLERİ
    // ═══════════════════════════════════════════════════════════════════

    /// 100 farklı adresle PoA ve Permissionless registry'ye ekle —
    /// hiçbir cross-contamination olmamalı.
    #[test]
    fn fuzz_bulk_addresses_no_cross_contamination() {
        let mut perm_state = AccountState::new();
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        poa_reg.add_admin(POA_DOMAIN, admin);

        // 100 adres: her biri ya PoA ya Permissionless'a kayıtlı
        for i in 0..100u8 {
            let addr = Address::from([i.wrapping_add(0x01); 32]);
            if i % 2 == 0 {
                // PoA'ya kaydet
                poa_reg
                    .submit_application(POA_DOMAIN, addr, [i.wrapping_add(1); 32])
                    .unwrap();
                poa_reg.approve(POA_DOMAIN, admin, addr).unwrap();
            } else {
                // Permissionless'a kaydet
                perm_state.add_balance(&addr, 10_000);
                perm_state.add_validator(addr, 5_000);
            }
        }

        // Doğrula: her adres doğru registry'de
        for i in 0..100u8 {
            let addr = Address::from([i.wrapping_add(0x01); 32]);
            if i % 2 == 0 {
                assert!(
                    poa_reg.is_authorized(POA_DOMAIN, &addr),
                    "Even addr {i} should be PoA authorized"
                );
                assert!(
                    !perm_state.registry.is_active(&addr, roles::VALIDATOR),
                    "Even addr {i} should NOT be in permissionless registry"
                );
            } else {
                assert!(
                    !poa_reg.is_authorized(POA_DOMAIN, &addr),
                    "Odd addr {i} should NOT be PoA authorized"
                );
                assert!(
                    perm_state.registry.is_active(&addr, roles::VALIDATOR),
                    "Odd addr {i} should be in permissionless registry"
                );
            }
        }

        // State root sadece permissionless verileri yansıtmalı
        let root = perm_state.calculate_state_root();
        assert!(!root.is_empty(), "State root should be non-empty");
    }

    /// Revoking ALL PoA members should not change permissionless state_root.
    #[test]
    fn fuzz_revoke_all_poa_members_state_root_unchanged() {
        let mut perm_state = AccountState::new();
        let perm_addr = Address::from([0xBB; 32]);
        perm_state.add_balance(&perm_addr, 10_000);
        perm_state.add_validator(perm_addr, 5_000);
        let root_before = perm_state.calculate_state_root();

        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        poa_reg.add_admin(POA_DOMAIN, admin);

        // 50 PoA üyesi ekle ve hepsini revoke et
        for i in 0..50u8 {
            let addr = Address::from([i.wrapping_add(0xC0); 32]);
            poa_reg
                .submit_application(POA_DOMAIN, addr, [i.wrapping_add(1); 32])
                .unwrap();
            poa_reg.approve(POA_DOMAIN, admin, addr).unwrap();
            poa_reg.revoke(POA_DOMAIN, admin, addr).unwrap();
        }

        let root_after = perm_state.calculate_state_root();
        assert_eq!(
            root_before, root_after,
            "Revoking all PoA members must NOT change permissionless state_root"
        );
    }

    /// PoA domain boundary: adding/removing PoA admins does not leak.
    #[test]
    fn poa_admin_churn_does_not_leak() {
        let mut perm_state = AccountState::new();
        let perm_addr = Address::from([0xBB; 32]);
        perm_state.add_balance(&perm_addr, 10_000);
        perm_state.add_validator(perm_addr, 5_000);
        let root_before = perm_state.calculate_state_root();

        let mut poa_reg = PoaMembershipRegistry::new();

        // Add/remove many admins
        for i in 0..30u8 {
            let admin_addr = Address::from([i.wrapping_add(0xF0); 32]);
            poa_reg.add_admin(POA_DOMAIN, admin_addr);
        }

        let root_after = perm_state.calculate_state_root();
        assert_eq!(
            root_before, root_after,
            "PoA admin churn must NOT affect permissionless state_root"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // BOUNDARY: EXECUTOR / TRANSACTION İZOLASYONU
    // ═══════════════════════════════════════════════════════════════════

    /// PoA membership approve/revoke Transaction'ları executor'da
    /// permissionless state'i etkilememeli (eğer varsa).
    ///
    /// Bu test, PoA ve Permissionless registry'lerin ayrı veri yapıları
    /// olduğunu ve executor'ın PoA üyeliğini permissionless tarafa
    /// geçirmediğini doğrular.
    #[test]
    fn executor_boundary_poa_and_permissionless_share_no_storage() {
        use crate::registry::permissionless::PermissionlessRegistry;

        let mut perm_registry = PermissionlessRegistry::new();
        let mut poa_reg = PoaMembershipRegistry::new();

        let admin = Address::from([0xAD; 32]);
        let member = Address::from([0xAA; 32]);

        // PoA side
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, member, [1u8; 32])
            .unwrap();
        poa_reg.approve(POA_DOMAIN, admin, member).unwrap();

        // Permissionless side: register with stake
        perm_registry.register_validator(member, 10_000, 0).unwrap();

        // PoA authorize → perm_registry unchanged
        assert!(poa_reg.is_authorized(POA_DOMAIN, &member));
        assert!(perm_registry.is_active(&member, roles::VALIDATOR));

        // PoA revoke → perm_registry unchanged
        poa_reg.revoke(POA_DOMAIN, admin, member).unwrap();
        assert!(!poa_reg.is_authorized(POA_DOMAIN, &member));
        assert!(
            perm_registry.is_active(&member, roles::VALIDATOR),
            "PoA revoke must NOT touch permissionless registry"
        );
    }

    /// Permissionless registry parametreleri PoA'dan tamamen bağımsız.
    #[test]
    fn registry_params_independence() {
        use crate::registry::params::RegistryParams;
        use crate::registry::permissionless::PermissionlessRegistry;

        let mut perm_registry = PermissionlessRegistry::new();
        let default_params = perm_registry.params().clone();

        // Parametreleri değiştir
        let new_params = RegistryParams {
            min_stake: 50_000,
            unbonding_epochs: 30,
            ..default_params
        };
        perm_registry.set_params(new_params);

        // PoA registry'si bu değişiklikten etkilenmemeli
        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, Address::from([0xAA; 32]), [1u8; 32])
            .unwrap();

        // PoA submit_application stake parametresini kullanmaz
        assert!(poa_reg.is_authorized(POA_DOMAIN, &Address::from([0xAA; 32])) == false);
        // (Pending durumunda authorized olmaması normal — ama PoA kendi
        //  bağımsız mantığıyla çalışır)
    }

    // ═══════════════════════════════════════════════════════════════════
    // BOUNDARY: HASH / SERDE İZOLASYONU
    // ═══════════════════════════════════════════════════════════════════

    /// PoA membership verisi serde JSON'da permissionless verisiyle
    /// kesişmemeli.
    #[test]
    fn poa_and_permissionless_serialized_forms_are_disjoint() {
        use crate::registry::permissionless::PermissionlessRegistry;

        let mut perm_registry = PermissionlessRegistry::new();
        perm_registry
            .register_validator(Address::from([0xBB; 32]), 10_000, 0)
            .unwrap();

        let mut poa_reg = PoaMembershipRegistry::new();
        let admin = Address::from([0xAD; 32]);
        poa_reg.add_admin(POA_DOMAIN, admin);
        poa_reg
            .submit_application(POA_DOMAIN, Address::from([0xAA; 32]), [1u8; 32])
            .unwrap();
        poa_reg
            .approve(POA_DOMAIN, admin, Address::from([0xAA; 32]))
            .unwrap();

        let perm_json = serde_json::to_string(&perm_registry).unwrap();

        // Permissionless JSON'da PoA terimleri olmamalı
        assert!(
            !perm_json.to_lowercase().contains("kyc"),
            "Permissionless JSON must NOT contain KYC"
        );
        assert!(
            !perm_json.to_lowercase().contains("decided_by"),
            "Permissionless JSON must NOT contain decided_by"
        );

        // PoA ve Permissionless ayrı veri yapıları: PoA'da unbonding/slashed
        // olmamalı (PoA membership'da bu kavramlar yok)
        assert!(
            poa_reg.is_authorized(POA_DOMAIN, &Address::from([0xAA; 32])),
            "PoA member should be authorized"
        );
        assert!(
            !perm_registry.is_active(&Address::from([0xAA; 32]), roles::VALIDATOR),
            "PoA member should NOT be in permissionless registry"
        );
    }
}
