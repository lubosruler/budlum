# RFC — Verifier Registry Birleştirme Değerlendirmesi

**Madde:** mainnet-hazirligi-talimati.md #16
**Durum:** Değerlendirme taslağı — kullanıcı kararı bekliyor
**Tarih:** 2026-07-19
**Hazırlayan:** ARENAX

---

## 1. Mevcut Durum

Budlum'da farklı roller için kullanılan registry'ler:

| Registry | Modül | Rol ID | Kayıt Mekanizması |
|----------|-------|--------|-------------------|
| `PermissionlessRegistry` | `src/registry/permissionless.rs` | RoleId 1-5 | Stake ile kayıt |
| `PoaMembershipRegistry` | `src/registry/poa_membership.rs` | — | KYC + admin onayı |
| `AiRegistry` | `src/ai/registry.rs` | RoleId 6 (AI_VERIFIER) | Model kayıt + stake |

### Mevcut Roller (RoleId)
```
VALIDATOR = 1
VERIFIER = 2
RELAYER = 3
PROVER = 4
STORAGE_OPERATOR = 5
AI_VERIFIER = 6
```

### Kullanım Alanları
- **Validator (1):** PoS consensus, liveness tracking
- **Verifier (2):** ZK proof verification (genel)
- **Relayer (3):** Cross-domain message relay
- **Prover (4):** ZK proof generation
- **Storage Operator (5):** B.U.D. storage deals
- **AI Verifier (6):** AI inference attestation

## 2. Birleştirme Gerekli mi?

### Mevcut yapı ZATEN birleşik
`PermissionlessRegistry` generic `(RoleId, Address)` key ile tüm rolleri tek bir registry'de tutuyor. AI Verifier (RoleId 6) da bu registry'ye eklenmiş. Bu, CLAUDE.md'deki "rol bazlı genişletilebilir tasarım" ilkesine uygun.

### Potansiyel Sorunlar
1. **DeEd Master Verifiers:** `budlum-xyz/budlumdevnet` reposunda ayrı bir verifier sistemi var. Bu, ana repodaki registry ile uyumlu mu?
2. **SocialFi Content Validator:** NFT/content doğrulama için ayrı bir validator seti gerekebilir.
3. **Budlum Go Supply Chain:** Supply chain attester'ları ayrı bir registry kullanıyor olabilir.

### Öneri
Mevcut `PermissionlessRegistry` yapısı zaten birleşik. Farklı kullanım alanları için yeni `RoleId`'ler eklenerek genişletilebilir:

| Önerilen Rol | ID | Kullanım Alanı | Durum |
|--------------|-----|----------------|-------|
| `BUD_STORAGE_NODE` | 7 | B.U.D. storage node (opsiyonel, STORAGE_OPERATOR'dan farklı) | Taslak |
| `CONTENT_VALIDATOR` | 8 | SocialFi content doğrulama | Taslak |
| `SUPPLY_CHAIN_ATTESTER` | 9 | Budlum Go supply chain | Taslak |

## 3. Karar

**Mevcut yapı birleştirmeye ihtiyaç duymuyor** — zaten tek bir generic registry kullanıyor. Yeni kullanım alanları için yeni RoleId eklemek yeterli.

**Bilinçli borç:** DeEd verifier'ları ve Budlum Go attester'ları ayrı repolarda. Ana repodaki PermissionlessRegistry ile entegrasyon mainnet v2 planı.

---

*Bu RFC kullanıcı kararı bekliyor: mevcut yapı yeterli mi, yoksa ek entegrasyon gerekli mi?*
