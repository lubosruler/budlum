# Mainnet Karar Kaydı — 2026-07-22

**Yetkili:** Ayaz (karar sahibi) · **Kaydeden:** ARENA1
**Süreç:** G4 (Production runbook drill) öncesi açık karar noktaları ask_user ile çözüldü.
**Kural:** Kararlar CI'ı bypass edemez; sadece hangi kodun yazılacağını belirler. Her kararın uygulaması commit hash + CI linki ile kapatılır.

---

## D1 — Relayer Güven Modeli: **PERMISSIONLESS**

- **Seçim:** Herkes relayer çalıştırabilir.
- **Tetiklediği iş:** `src/bin/budlum-relayer.rs` (C3) production loop artık tek-operatör değil; permissionless tasarımla yazılacak.
- **Gereksinimler:**
  - Relayer kaydı + bond/stake (D4 unified Verifier Registry üzerinden).
  - Slashing mekanizması (griefing/fronting/yanlış-relay için).
  - Bridge contract tarafında relayer set'i open + challenge penceresi.
- **Risk:** Front-running, griefing, kötü-relay. Slashing parametreleri ayrı netleştirilecek.
- **Sinerji:** D4 (unify registry) ile doğrudan uyumlu — permissionless relayer stake-tabanlı registry'ye doğal oturur.

## D2 — Gizlilik Katmanı: **v1'DE DAHİL, POSEIDON İLE**

- **Seçim:** zkVM private witness katmanı mainnet v1'e dahil; field-native hash = **Poseidon** (Plonky3 stdlib ile en yaygın uyum).
- **Tetiklediği iş:** `bud-isa` yeni opcode ailesi (commit, nullifier-check, sum-conservation); `bud-vm`/`bud-proof` constraint set; `bud-state` note/UTXO veri modeli.
- **Bölüm 10 durum:** #1 (Poseidon) ✅ · #2 (note model) ✅ · #3 (view-key) ✅ · #4 (zamanlama: v1) ✅ · #5 (exec-time confidentiality) ✅ — **TÜM SORULAR ÇÖZÜLDÜ.**
- **Bölüm 10 madde 5 çözümü — Execution-time confidentiality:** **TEE, OPT-IN CÜZDAN ÖZELLİĞİ.** Sistem-genel zorunlu değil; cüzdan içinde kullanıcı açar. Toggle UX: *"Bu cüzdanın işlemleri TEE katmanıyla gizli kılınsın mı? → Evet (işlemleriniz biraz yavaşlar)."* Varsayılan kapalı. Hedef backend client-side TEE (laptop SGX — operatör yok, veri cihazdan çıkmaz); zayıf cihaz (mobile) için server-side TEE (AWS Nitro) fallback. FHE uzun vadeli roadmap maddesi (basit transfer-math için). HSM (G3/YubiHSM) ayrı — imzalama içindir, ağır hesaplama değil.
- **Bölüm 10 madde 2 çözümü — Note model:** **PARALEL İZOLE SUBTREE**. Gizli note'lar ayrı bir izole state alt-ağacında yaşar; mevcut account-model'e dokunmaz. Bölüm 7 izolasyon kuralıyla birebir uyumlu.
- **Bölüm 10 madde 3 çözümü — View-key:** **KULLANICI ÜRETİR + PAYLAŞIR** (Zcash deseni). Kullanıcı view key'i cüzdanında üretir/saklar; BDDK gibi yetkiliye manuel ibraz eder. Kamuya kapalı, yetkiliye açık.
- **İzolasyon (Bölüm 7):** Gizlilik opcode'ları yalnızca transfer ailesini kapsar; NFT/B.U.D./Pollen state'ine dokunmaz.

## D3 — Legacy Declared-Depth Proof: **TAMAMEN KALDIR**

- **Seçim:** Production ISA'dan legacy proof yolu silinir; sadece yeni bounded header-chain proof kullanılır.
- **Tetiklediği iş:** Legacy yolun referans edildiği tüm kod/test/path'ler tespit edilip kaldırılacak (önce bağımlılık doğrulaması).
- **Risk:** Backward-compat kopması (eski light-client proof'ları). Doğrulama yapılmadan silme commit atılmaz.
- **Kabul:** Legacy kod tamamen kaldırıldıysa mint-gate testleri de kaldırılır; aksi halde gerekçeyle kalır.

## D4 — Verifier Registry: **v1'DE BİRLEŞTİR**

- **Seçim:** Tek stake-tabanlı Verifier Registry; DeEd master verifiers + SocialFi content validator + relayer + supply-chain attester hepsi aynı "kim güvenilir, nasıl slash edilir" katmanını kullanır.
- **Tetiklediği iş:** Mevcut RoleId(8) tabanlı registry'nin 4 alanı kapsayıp kapsamadığı doğrulanacak → eksikse tek stake-tabanlı modele çekilecek.
- **Risk:** Yüksek efor + yeniden test. Mevcut `LUBOT_OPERATOR = RoleId(8)` dahil tüm rol tanımları korumalı.

---

## Uygulama Önceliği (ARENA1 önerisi)

1. **D2 gizlilik alt-soruları çözülünce** (Bölüm 10 #2/#3/#5) → opcode tasarım dokümanı + commit.
2. **D3** → bağımlılık doğrulaması → legacy kaldırma commit (orta efor, izole edilebilir).
3. **D4** → mevcut registry kapsam doğrulaması → birleştirme planı → uygulama (yüksek efor).
4. **D1** → D4'ten sonra relayer production loop (C3), slashing + bond ile (yüksek efor).

G2 (audit firması) ve G3 (HSM donanım) tedarik kararları olarak Ayaz'da bekliyor — kod ajanı seçemez.

---

*Bölüm 10 #2/#3/#5 çözüldükçe bu doküman güncellenir. Kararlar ask_user akışından alınmıştır (2026-07-22).*
