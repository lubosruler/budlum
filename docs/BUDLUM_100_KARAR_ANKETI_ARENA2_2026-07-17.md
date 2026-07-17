# Budlum — Alınmış Tüm Kararlar için 100 Soruluk Anket (ARENA2, 2026-07-17)

> **Amaç:** Budlum'un Phase 0.06'dan Phase 9'a kadar alınmış tüm stratejik ve teknik kararlarını tek bir ankette toplamak. Her soru teknik detay + parantez içinde teknik olmayan açıklama içerir. Seçenek sayısı 3-5, tüm olası cevapları kapsayacak şekilde. Cevap anahtarı kullanıcı tarafından verilecek.

> **Kaynaklar:** `docs/PHASE0.06_PLAN.md`, `PHASE0.08_PLAN.md`, `PHASE0.10_PLAN.md`, `PHASE0.378_*`, `PHASE0.42_PLAN.md`, `PHASE1_RAPOR.md`, `MAINNET_READINESS.md`, `BUDLUM_CONSTITUTION.md`, `RD_SOCIALFI_DWEB_VISION.md`, `PERSONAS.md`, `THREAT_MODEL.md`, `ORG_ROADMAP_AUDIT.md`, `PHASE8.9_ANALIZ_A1.md`, `PHASE9_VIZYON_KOD_CELISKI_DENETIM_ARENAX_2026-07-17.md`, `STATUS_ONLINE.md`, `CLAUDE.md`, `ARENA_AI.md`, `AUDIT_CHECKLIST.md`, `BUG_BOUNTY.md`, `MAINNET_GENESIS_CEREMONY.md`, `config/*.toml`, `src/core/chain_config.rs`, `src/crypto/pkcs11.rs`, `budzero/bud-isa/src/lib.rs`.

---

## Q1 — Phase sistemi isimlendirmesi
**Teknik:** TUR/ADIM serisi Phase sistemine taşındı (`ADIM1=Phase1`, `Tur1=Phase0`, `Tur10=Phase0.30`, `Tur25=Phase0.60`). Formül `t<10 → 0.02×(t-1)`, `t≥10 → 0.30+0.02×(t−10)`. (Repo genelinde TUR/ADIM kelimeleri Phase ile değiştirildi, eski dal adları hariç)
**(Non-teknik: Artık Tur ve Adım demiyoruz, her şeyi Phase diye çağırıyoruz — karışıklık bitsin)**
- A) Phase sistemi doğru, TUR/ADIM tamamen kaldırılsın
- B) Eski TUR/ADIM isimleri de paralel kalsın (çift isim)
- C) Sadece TUR kaldırılsın, ADIM kalsın
- D) Hiçbir şey değişmesin, eski isimler kalsın

## Q2 — Budlumdevnet dokunulmazlığı
**Teknik:** `github.com/budlum-xyz/budlumdevnet` main HEAD `6613219a`, son push 2026-07-11 21:11 UTC'den beri dokunulmadı. Budlum repo'sunun temelini aldığı eski kod sabit kalmalı. (Eski kodun değişmediğini CI ile doğruluyoruz)
**(Non-teknik: Eski temel kodu bozmayacağız, yeni işler ayrı repo'da)**
- A) Budlumdevnet kesinlikle dokunulmamalı, sadece okunmalı
- B) Küçük düzeltmeler budlumdevnet'e de uygulanabilir
- C) Budlumdevnet tamamen budlum'a merge edilsin

## Q3 — Force-push yasağı
**Teknik:** `git push --force`, `--force-with-lease` kesin yasak. Conflict durumunda `git pull --rebase` + normal push. Shallow clone sorunu `git fetch --unshallow`. (STATUS.md §4.2)
**(Non-teknik: Zorla üstüne yazmak yok, çakışırsa nazikçe birleştir)**
- A) Force-push kesin yasak kalsın
- B) --force-with-lease serbest olsun
- C) Force-push serbest, hızlı ilerleyelim

## Q4 — Workflow dosyası push yasağı
**Teknik:** Bot token `workflows: write` izni yok, `.github/workflows/ci.yml` değişiklikleri kullanıcı manuel ekler. (STATUS.md §4.3, STATUS_ONLINE F8)
**(Non-teknik: CI ayarlarını robotlar değiştiremez, sadece insanlar)**
- A) Yasak kalsın, kullanıcı eklesin
- B) Botlara workflows izni verilsin
- C) Workflow dosyaları tamamen kaldırılsın

## Q5 — Kanıt standardı (SHA/dosya:satır/CI-job)
**Teknik:** Her iddia `git cat-file -t <sha>`, `grep -n`, `gh api .../check-runs` ile kanıtlanmadan audit'e yazılmaz. Kanıtsız commit referansı yasak. (STATUS.md §4.1)
**(Non-teknik: Laf değil, kanıt — her iddianın ekran görüntüsü gibi)**
- A) Kanıt zorunlu kalsın
- B) Kanıt isteğe bağlı olsun
- C) Kanıt gerekmez, güven esastır

## Q6 — Permissionless mimari ve whitelist yasağı
**Teknik:** `STORAGE_OPERATOR`, `RELAYER`, `PROVER`, `VALIDATOR` rolleri `PermissionlessRegistry` ile, stake tabanlı, whitelist/onay yok. `PoaMembershipRegistry` ayrı, B.U.D. asla PoA'ya dokunmaz. (CLAUDE.md §2)
**(Non-teknik: Kimseyi dışlamıyoruz, para yatıran herkes girebilir)**
- A) Permissionless kalsın, whitelist kesinlikle yasak
- B) Whitelist eklenebilir (kurumsal istek)
- C) Sadece davetliler girebilsin

## Q7 — VerifyMerkle production gate (Phase 4 = Z-B Commit 3.5)
**Teknik:** `bud-isa::Opcode::VerifyMerkle=0x1E`, `is_experimental()=false` oldu (önce `true` idi). 1-depth, 2-depth, 64-depth STARK testleri yeşil, `proves_verify_merkle_valid_64_depth` ignore'dan çıkarıldı, BudZero job yeşil. (PHASE0.06_PLAN aslında Phase 0.44)
**(Non-teknik: Zorlu kriptografi bulmacası çözüldü, artık depolama kanıtı gerçek)**
- A) Gate açık kalsın (mevcut, VerifyMerkle prod'da aktif)
- B) Gate kapalı kalsın (eski, sadece test)
- C) Gate sadece testnet'te açık, mainnet'te kapalı

## Q8 — B.U.D. Faz 3: Merkle proof zorunlu
**Teknik:** `src/domain/storage_deal.rs` `open_deal` artık `merkle_proof: Option<Vec<u8>>` ve `storage_root: Option<Hash32>` zorunlu, `ProofEnvelope` bincode deserialize ile format-validasyon. (Phase 0.10 planı aslında Faz 3)
**(Non-teknik: Depolama anlaşması açarken kriptografik kanıt göstermek zorunlu)**
- A) Zorunlu kalsın (gerçek Proof-of-Storage)
- B) Opsiyonel kalsın (interim challenge yeterli)
- C) Tamamen kaldırılsın

## Q9 — B.U.D. Faz 4: GlobalBlockHeader.storage_root
**Teknik:** `BlockHeader` / `GlobalBlockHeader` içine `storage_root` eklendi, block hash'e dahil, genesis'te `hash(empty)`. (PHASE0.06_PLAN §4.4)
**(Non-teknik: Her blokta tüm depolama verisinin özeti var)**
- A) storage_root block hash'e dahil kalsın
- B) storage_root ayrı bir sidecar'da tutulsun, block hash'e dahil olmasın
- C) storage_root tamamen kaldırılsın

## Q10 — ConsensusStateV2 migration iskeleti
**Teknik:** `StateSnapshotV2` `schema_version=3`, `registry`, `liveness`, `tokenomics` + atomik `tokenomics_burn` (timed_burn + burn_reserve_address + team_vesting) taşır, `#[serde(default)]` ile eski schema-2 uyumlu, `from_snapshot_v2` atomik restore. (STATUS.md, PHASE0.378)
**(Non-teknik: Eski cüzdan yedekleri yeni sürümde de açılsın)**
- A) V2 migration iskeleti korunsun, atomik restore şart
- B) V1'e geri dönülsün
- C) Migration otomatik değil manuel olsun

## Q11 — BLS/PQ HSM policy (Phase 0.378 B1)
**Teknik:** `src/crypto/pkcs11.rs` gerçek PKCS#11 HSM, `cryptoki 0.12.0` + `secrecy 0.10.3` (`SecretString`), `MechanismType::new_vendor_defined` fail-closed, `VendorDefinedMechanism::new::<()>()`, `CInitializeArgs::new(OS_LOCKING_OK)`, `EddsaParams::new(Pure)`. Disk `ValidatorKeys` mainnet'te reddedilir. (F1-F10 rapor V14)
**(Non-teknik: Anahtarlar donanım kasasında, diskte çıplak duramaz)**
- A) Sadece gerçek HSM, disk yasak (mevcut)
- B) Mock HSM de kabul edilsin (geliştirme kolaylığı)
- C) Disk de kabul edilsin (operatör tercihi)

## Q12 — Vendor mechanism CLI wiring (F3)
**Teknik:** `--pkcs11-bls-mechanism` / `--pkcs11-pq-mechanism` `commands.rs:223-226,687-693` parse ediyor, `Pkcs11Signer::with_vendor_mechanisms()` var, `main.rs:485` artık `new()` + `with_vendor_mechanisms()` ile wire edildi. (Rapor F3 fix)
**(Non-teknik: Donanımın özel imza modu artık CLI'dan seçilebiliyor)**
- A) Wire kalsın (config → signer)
- B) Wire kaldırılsın, sadece software fallback
- C) Vendor mechanism tamamen kaldırılsın

## Q13 — README roadmap ve test sayısı prose
**Teknik:** Badge otomasyonu `cargo test --lib` sayısını `README.md` içindeki `tests-XX%20lib` rozeti ile otomatik güncelliyor, prose da `531 → 538 → 539 → 546` manuel tazelendi. (F6 fix)
**(Non-teknik: Test sayısı rozeti ve yazıda aynı olmalı)**
- A) Rozet otomatik, prose manuel tazeleme (mevcut)
- B) Prose da otomatik olsun
- C) Test sayısı hiç yazmasın, sadece badge

## Q14 — Persona paketleri (user-devnet/developer/enterprise-poa)
**Teknik:** `config/personas/{user-devnet,developer,enterprise-poa}.toml` + `docs/PERSONAS.md` uyumluluk matrisi, aynı `budlum-core` binary farklı config ile farklı persona. (Phase 0.398)
**(Non-teknik: Aynı program, farklı ayarla kullanıcı/dev/kurum modu)**
- A) Persona sistemi kalsın
- B) Tek persona yeterli (developer)
- C) Persona sistemi kaldırılsın, herkes aynı config

## Q15 — Prometheus latency histogram wiring (Phase 2 §2.5)
**Teknik:** `src/core/metrics.rs` `Metrics` içinde `chain_height`, `blocks_produced`, `rpc_request_duration` histogram, `/metrics` text encoding test ile doğrulandı (`test_metrics_initialization_and_encoding`).
**(Non-teknik: Sistem ne kadar hızlı, metriklerle görüyoruz)**
- A) Histogram wiring kalsın
- B) Sadece counter yeterli, histogram kaldırılsın
- C) Metrics tamamen kaldırılsın

## Q16 — Per-IP quota / RPC rate limiting (Phase 2 §2.6)
**Teknik:** `src/rpc/server.rs` `is_per_ip_rate_limited` per-minute pencere, 10k IP bellek tavan, `X-Real-IP` sadece `trusted_proxies` set ise. Test `test_per_ip_rate_limiting` eklendi.
**(Non-teknik: Bir IP çok istek atarsa yavaşlat)**
- A) Per-IP rate limit kalsın
- B) Sadece global rate limit yeterli
- C) Rate limit tamamen kaldırılsın

## Q17 — Fuzzing + dependency audit + SBOM (Phase 2 §2.7-2.8)
**Teknik:** `fuzz/` (5 target: block_deserialize, consensus_validate, snapshot_deserialize, transaction_deserialize), `cargo audit`, `cargo cyclonedx` SBOM, `scripts/audit-deps.sh`, `scripts/generate-sbom.sh`, CI `supply-chain-extra` job. (Phase 0.40 §1.7)
**(Non-teknik: Kod bulanık test ve bağımlılık taraması ile sağlamlaşıyor)**
- A) Fuzz + audit + SBOM tam kalsın
- B) Sadece audit yeterli
- C) Hepsi kaldırılsın

## Q18 — Bug bounty programı (Phase 2 §2.9)
**Teknik:** `docs/BUG_BOUNTY.md` kapsam, ödül seviyeleri, iletişim kanalı, immunefi benzeri. Harici audit öncesi bug bounty ile başla (Phase 0.40 §1.5 kararı C).
**(Non-teknik: Hacker'ları ödüllendir, açığı dışarıdan bulsunlar)**
- A) Bug bounty ile başla (mevcut karar)
- B) Doğrudan harici firma audit'i
- C) Hiçbiri, self-audited yeterli

## Q19 — Mainnet genesis config ve fail-closed guard'lar (Phase 3 §3.1)
**Teknik:** `config/mainnet-genesis.json` + `mainnet.toml` + `test_mainnet_genesis_json_matches_code` hash+state_root+validator_set_hash eşitliği, `test_mainnet_genesis_hash_matches_documented_constant` absolute assert `02166d370613fc70e5beb47e4d1ef48e5ccad93eb0f4b8bd5edfe5787a7f98fc`. Placeholder peer guard `first_placeholder_peer` marker listesi `dummy, placeholder, 203.0.113., .example`. (F7, F9)
**(Non-teknik: Gerçek genesis dosyası yoksa veya sahte bootnode varsa node başlamasın)**
- A) Fail-closed guard'lar kalsın (mevcut)
- B) Guard'lar uyarı versin ama devam etsin (fail-open)
- C) Guard'lar kaldırılsın

## Q20 — Docker + systemd + runbook (Phase 3 §3.2-3.3)
**Teknik:** `Dockerfile`, `docker-compose.yml`, `docs/operations/PRODUCTION_RUNBOOK.md` §8 genesis hash tablosu, `operations/VALIDATOR_ONBOARDING.md`, `NETWORK_HARDENING.md`, `ARCHIVE_AND_BACKUP.md`.
**(Non-teknik: Node'u konteynerde ve servis olarak nasıl çalıştırırsın)**
- A) Docker+systemd+runbook tam kalsın
- B) Sadece binary yeterli, docker kaldırılsın
- C) Hepsi kaldırılsın

## Q21 — Network hardening (Phase 3 §3.4)
**Teknik:** `src/network/` p2p max_peers, peer_rate_limit_per_minute, `peer_manager` token bucket, banned_peers persist, mdns_enabled false mainnet, `PeerManager` security config. (Q5 guard)
**(Non-teknik: Ağ saldırıya dayanıklı olsun)**
- A) Hardening tam kalsın
- B) Sadece max_peers yeterli
- C) Hardening kaldırılsın

## Q22 — Validator onboarding flow (Phase 3 §3.5)
**Teknik:** `src/registry/permissionless.rs` stake == registration, `sync_validator_registration`, `upsert_stake`, `bond_relayer`, `bond_prover`, `bond_storage_operator`, RPC `bud_registryRegister`, `bud_registryBondRelayer/Prover`, `bud_registryActiveMembers`. (CLAUDE.md)
**(Non-teknik: Stake yatıran otomatik validator olur)**
- A) Stake==registration kalsın
- B) Ayrı manual kayıt adımı eklensin
- C) Sadece PoA whitelist ile validator olunur

## Q23 — B.U.D. interim retrieval challenge dokümantasyonu (Phase 3 §3.6)
**Teknik:** `docs/BUD_INTERIM.md` interim challenge sadece byte-range testi, gerçek Proof-of-Storage değil, `RetrievalChallenge`/`Response`/`Outcome` interim, Faz 3 gerçek PoS için VerifyMerkle açık. (Faz 2 compat)
**(Non-teknik: Şimdilik basit kontrol, gerçek kriptografik kanıt sonra)**
- A) Interim challenge dokümante kalsın, gerçek PoS Faz 3'te
- B) Interim challenge kaldırılsın, direkt gerçek PoS
- C) Interim challenge gerçek PoS gibi pazarlansın

## Q24 — B.U.D. Faz 5 economics accounting (Phase 1 devamı)
**Teknik:** `Blockchain::accrue_storage_operator_rewards` (fee_per_epoch * epochs), `finalize_missed_storage_challenges` slashed bond total + burned, `StorageEconomicsEvent` + `ChainHandle::get_storage_economics_events/summary`, ChainActor otomatik bakım (block üretim/doğrulama sonrası reward accrual + challenge issuance + missed finalization).
**(Non-teknik: Depolama operatörleri ne kadar kazandı, kim ceza yedi, olay kaydı)**
- A) Economics accounting tam kalsın
- B) Sadece reward, slashing olmasın
- C) Economics tamamen kaldırılsın

## Q25 — Constitution §1: Content & Moderation — Community Voting
**Teknik:** Reported content validator/governance oylaması ile. (BUDLUM_CONSTITUTION.md §1)
**(Non-teknik: Sakıncalı içeriği topluluk oylasın)**
- A) Community voting kalsın
- B) Merkezi moderasyon olsun
- C) Hiç moderasyon olmasın

## Q26 — Constitution §1: Right to be Forgotten — Hard Pruning (F1)
**Teknik:** `NftBurn` transaction ile linked B.U.D. data fiziksel silinir, `NftRegistry::burn` cid döndürür, `StorageRegistry::prune_content(cid,epoch)` deal'leri expired yapar manifest'i siler, `NodeCommand::StoragePrune{cid:[u8;32]}` + `ContentStore::delete` fiziksel chunk silme, `NetworkMessage::StoragePrune` gossip ile full P2P (Q-X1 full_p2p_prune). Log `Hard Prune Triggered` dürüstleştirildi. (F1 fix 5322e00 + b65f058)
**(Non-teknik: NFT yakarsan verin gerçekten silinir — unutulma hakkı)**
- A) Hard pruning tam P2P (consensus+local+network broadcast) kalsın
- B) Sadece registry silme yeterli, fiziksel silme olmasın
- C) Hard pruning tamamen kaldırılsın

## Q27 — Constitution §1: Content Portability
**Teknik:** NFT transferinde content otomatik yeni sahibin profiline ve SocialFi feed'ine taşınır, `ownership` map güncellenir. (Constitution §1, RD_SOCIALFI)
**(Non-teknik: NFT'yi başkasına verirsen içeriğin de onun profiline taşınır)**
- A) Portability kalsın
- B) Transferde içerik eski sahibinde kalsın
- C) Transfer yasak olsun

## Q28 — Constitution §2: Social Recovery — No recovery
**Teknik:** HSM key kaybolursa account ve data sonsuza kadar kilitli, recovery yok, maximum security. (Constitution §2)
**(Non-teknik: Anahtarı kaybedersen geçmiş olsun, kurtarma yok)**
- A) No recovery kalsın (max güvenlik)
- B) Social recovery eklensin (arkadaşların kurtarsın)
- C) Merkezi recovery (email ile)

## Q29 — Constitution §2: BNS Disputes — First Come First Served
**Teknik:** `.bud` isim hakları kayıt anında absolute, trademark arbitration yok, `BnsRegistry` first-come. (Constitution §2)
**(Non-teknik: Önce gelen alır, marka davası yok)**
- A) First come first served kalsın
- B) Trademark arbitration eklensin
- C) İsimler açık artırmayla satılsın

## Q30 — Constitution §2: Privacy — Selective Encryption
**Teknik:** Her SocialFi post için kullanıcı Public veya Encrypted seçer. (Constitution §2)
**(Non-teknik: Her gönderiyi açık veya şifreli seçebiliyorsun)**
- A) Selective encryption kalsın
- B) Her şey public olsun
- C) Her şey encrypted olsun

## Q31 — Constitution §3: Spam Protection — Fee per post
**Teknik:** Her SocialFi etkileşimi (NftMint) tx fee içerir, `tx.fee` saturating_sub. (Constitution §3)
**(Non-teknik: Her gönderi biraz ücretli, spam engellenir)**
- A) Fee per post kalsın
- B) Ücretsiz gönderi
- C) Aylık abonelik

## Q32 — Constitution §3: Longevity — Permanent by default
**Teknik:** Data NFT yakılana kadar ağda kalır, `DealStatus::Active` → `Expired` sadece `deal_end_epoch` veya `burn` ile. (Constitution §3)
**(Non-teknik: Silmediğin sürece verin sonsuza kadar durur)**
- A) Permanent by default kalsın
- B) 1 yıl sonra otomatik silinsin
- C) Kullanıcı süre seçsin

## Q33 — Constitution §3: Self-Hosting Option
**Teknik:** Kullanıcı yıllık storage rent ödemek istemezse local B.U.D. node ile self-host edebilir, `MobileConfig` batarya/Wi-Fi dostu, `ShardManager` self-host önceliği. (Constitution §3, Phase 5 §5.2)
**(Non-teknik: Kendi telefonunda/bilgisayarında saklayabilirsin)**
- A) Self-hosting seçeneği kalsın
- B) Sadece profesyonel operatörler saklasın
- C) Self-host yasaklansın

## Q34 — Constitution §3: Rewards — Storage Provider Heavy
**Teknik:** Yeni $BUD ihracının çoğunluğu B.U.D. operatörlerine (Storage Proofs), `accrue_storage_operator_rewards` + `pending_bud_boost_share` ağırlıklı dağıtım. (Constitution §3)
**(Non-teknik: En çok ödül depolayanlara)**
- A) Provider heavy kalsın
- B) Validator heavy olsun
- C) Eşit dağıtım

## Q35 — Constitution §3: Advertising & Highlighting Model — 4/16/80 split (F4)
**Teknik:** `NftBoost {nft_id, amount}`: `bud_share=4%`, `creator_share=16%`, `protocol_share=80%`. Executor'da `pending_bud_boost_share+=bud_share`, `creator.balance+=creator_share`, `protocol_share` burn_reserve/treasury'ye (treasury_pool) veya burn. Blockchain'de `distribute_bud_boost_share` fee_per_epoch ağırlığına göre + dust ilk operatöre. (F4 fix 5322e00, 7f054d7, 6dd66e5 config-driven treasury)
**(Non-teknik: Bir içeriği öne çıkarırsan %4 depolayanlara, %16 yaratıcıya, %80 ekibe/hazineye)**
- A) 4/16/80 kalsın (mevcut, weighted + dust first)
- B) 10/30/60 olsun
- C) %100 creator'a gitsin
- D) %100 yakılsın
- E) %4 B.U.D. + %16 creator + %80 yeni ayrı TREASURY_ADDRESS (config'den)

## Q36 — Constitution §3: Treasury & %80 Ekip Havuzu (Q-X4)
**Teknik:** Protocol_share %80 ekip havuzu, `burn_reserve_address` veya yeni `TREASURY_ADDRESS` (config `table treasury { address=... }`), multi-sig single/governance, RPC `bud_treasuryBalance` yok/olsun, event emit. (Q-X4 treasury_pool)
**(Non-teknik: %80'lik havuz ekibin kasası, nasıl yönetilsin?)**
- A) burn_reserve_address treasury olarak kullanılsın
- B) Yeni ayrı TREASURY_ADDRESS tanımla, config'den okunur
- C) team_vesting adresi kullanılsın
- D) %80 direk yakılsın, treasury yok
- E) %80 + %4 operatör yoksa tamamı treasury'ye

## Q37 — Constitution §3: Social Ranking — Luminance (Işık Şiddeti)
**Teknik:** NFT `luminance=1000 mcd` (1 cd) başlar, `NftUpdateLight {delta_mcd}` owner-only, +0.0006 cd >30s view, +0.005 cd 5/5 spark, -0.0006 <1s, -0.003 darken, %10 yıllık decay, UI threshold 0.1 cd. (Constitution §3, RD_SOCIALFI)
**(Non-teknik: İçerik ışık saçıyor, beğenildikçe parlıyor)**
- A) Luminance algoritması kalsın
- B) Sadece beğeni sayısı kullanılsın
- C) Ranking tamamen kaldırılsın

## Q38 — Constitution §3: Content Mobility — Digital Bud
**Teknik:** NFT'ler "Dijital Tomurcuk", transfer ile authority ve future earnings yeni sahibe geçer, `transfer(id, from, to)` ownership map günceller. (Constitution §3)
**(Non-teknik: Tomurcuk başkasına geçince çiçeği o büyütür)**
- A) Digital Bud mobility kalsın
- B) Transferde earning eski sahibinde kalsın
- C) NFT'ler transfer edilemesin (soulbound)

## Q39 — Constitution §3: Interoperability — Universal Bridge Hub
**Teknik:** Unified ecosystem interface, `HubRegistry`, `budget hub_register`, `Universal Relayer` master translator, EVM, Solana vb. (Constitution §4, Phase 6)
**(Non-teknik: Her zincirdeki uygulama tek yerden kayıt oluyor)**
- A) Hub açık kayıt (democratic) kalsın
- B) Hub sadece davetliler
- C) Hub kaldırılsın

## Q40 — Constitution §3: Zero-Fee Inbound Bridge
**Teknik:** Budlum'a inbound transferde upfront $BUD yok, kaynak zincir veya relayer fee gelen varlıktan düşer, `Zero-Fee Inbound Bridge` (Constitution §3)
**(Non-teknik: Dışarıdan gelirken $BUD'un yoksa sorun değil, gelen paradan keseriz)**
- A) Zero-fee inbound kalsın
- B) Inbound için de $BUD gerekli olsun
- C) Inbound tamamen yasak

## Q41 — Constitution §4: Relayer Incentives
**Teknik:** Relayer'lar protokol tarafından $BUD mint ile ödüllendirilir, inbound bridge'de gelen varlığın küçük kısmı fee olarak alır. (Constitution §4, relayer_liveness.rs)
**(Non-teknik: Köprücüler çevirdikleri için ödül alır)**
- A) Relayer mint reward + asset fee kalsın
- B) Sadece mint reward
- C) Sadece asset fee

## Q42 — Constitution §5: AI Layer — Permissioned & Monetized
**Teknik:** `AiOfferData`, `AiPurchaseData` transaction tipleri, user-to-AI data market, explicit permission + payment in $BUD. (Constitution §5, RD_SOCIALFI Q3)
**(Non-teknik: Verini AI'ya satmak istersen izin verip para alıyorsun)**
- A) Permissioned & monetized kalsın
- B) AI erişimi ücretsiz ve açık olsun
- C) AI erişimi tamamen yasak

## Q43 — Constitution §6: Physical Hardware — Plug & Play storage
**Teknik:** $BUD ile satın alınan pre-configured physical nodes, mobile high-priority storage node. (Constitution §6)
**(Non-teknik: Fiziksel kutu al tak çalıştır depolama)**
- A) Physical node satışı kalsın
- B) Sadece yazılım, donanım yok
- C) Donanım zorunlu olsun

## Q44 — Constitution §7: Verified Status & Sub-BNS & Emergency Halt
**Teknik:** Premium BNS yıllık yüksek ödeme ile Verified badge, sub-domains parent-controlled (x.ayaz.bud), DAO Halt topluluk oylaması ile chain'i geçici durdurabilir, $BUD ile storage access boost. (Constitution §7)
**(Non-teknik: Rozet, alt isimler ebeveynde, acil durumda zinciri durdurma)**
- A) Hepsi kalsın (premium verified, parent sub-BNS, DAO halt, boost)
- B) Sadece verified kalsın
- C) Hiçbiri olmasın

## Q45 — Phase 2 §2.3: ConsensusStateV2 migration hook
**Teknik:** `Blockchain::collect_block_transactions` + `apply_block_effects` + snapshot V2 → V3 migration test `test_snapshot_v2_migration_roundtrip_with_tokenomics_burn`. (ARENA1 5548c42)
**(Non-teknik: Eski yedekler yeni sürüme sorunsuz geçsin)**
- A) Migration hook testli kalsın
- B) Migration manuel olsun
- C) Migration kaldırılsın, sıfırdan genesis

## Q46 — Phase 2: Multi-validator permissionless E2E (5548c42)
**Teknik:** `src/tests/permissionless_e2e.rs` 3 validator (v1,v2,absentee) stake→register, çoklu epoch blok üretimi, absentee liveness slashing döngüsü.
**(Non-teknik: 3 doğrulayıcı ile gerçek ağ simülasyonu)**
- A) Multi-validator E2E kalsın
- B) Sadece single-validator E2E yeterli
- C) E2E testler kaldırılsın

## Q47 — Phase 2: Liveness — liveness_max_missed_epochs = 20 (38adeec)
**Teknik:** `RegistryParams::default()` içinde `liveness_max_missed_epochs` 10'dan 20'ye çıkarıldı, transient network blip toleransı için mainnet kararı. (ARENA1 13:15 UTC)
**(Non-teknik: Biraz internet kesilse hemen cezalandırma yok, 20 epoch tolerans)**
- A) 20 epoch kalsın (mevcut mainnet kararı)
- B) 10 epoch'a geri dön
- C) 5 epoch daha katı olsun
- D) Liveness slashing tamamen kapatılsın

## Q48 — Tokenomics — Vesting cliff/duration ve BUD_UNIT 6 decimals
**Teknik:** `src/tokenomics/mod.rs` `BUD_UNIT=1_000_000` (6 ondalık), `VestingSchedule::unlocked_at` linear from start, cliff anında `total*cliff/duration` açılır (örn. 60 epoch cliff, 200 duration → 250e9). Test `bud(1_000_000)/4` kilitli. (ARENA1 893ffdc, ARENA2 920e9fe fix)
**(Non-teknik: Token hakedişi uçurumdan sonra birikmiş olarak açılır)**
- A) Linear from start + cliff'te birikmiş açılma kalsın (mevcut)
- B) Cliff'te 0 açılsın, sonra linear
- C) Vesting tamamen kaldırılsın

## Q49 — Phase 2: Prometheus + RPC rate limiting + snapshot V2 roundtrip (Phase 8.9 → 2.5/2.6)
**Teknik:** `test_metrics_initialization_and_encoding`, `test_per_ip_rate_limiting`, `test_snapshot_v2_migration_roundtrip_with_tokenomics_burn` eklendi (ARENA1 5548c42)
**(Non-teknik: Metrik, hız limiti ve yedek dönüşüm testleri)**
- A) Bu 3 test kalsın
- B) Sadece metrics ve rate limit kalsın, snapshot kaldırılsın
- C) Hepsi kaldırılsın

## Q50 — Phase 8.9 / Q5: Dummy bootnode fail-closed guard ve DNS seeds
**Teknik:** `MAINNET_BOOTNODES = ["/ip4/203.0.113.10/...", ...]` RFC5737 TEST-NET-3, `MAINNET_DNS_SEEDS = ["_dnsaddr.placeholder-seed-1.mainnet.budlum.network", ...]`, `PLACEHOLDER_PEER_MARKERS=["dummy","placeholder","203.0.113.",".example"]`, `first_placeholder_peer()` marker arar, mainnet'te DIAL edilmez CRITICAL exit 1. Test `test_placeholder_peer_detection_blocks_dummy_mainnet_entries` + F7 güçlendirme derlenmiş sabitlerin placeholder yakalanması. (893ffdc, F7 fix)
**(Non-teknik: Sahte adresle mainnet açılmasın, törene kadar kapalı)**
- A) Fail-closed guard kalsın, placeholder'lar törene kadar bloklasın
- B) Guard uyarı versin ama mainnet açılsın (fail-open)
- C) Guard kaldırılsın

## Q51 — Phase 8: forbid(unsafe_code) (G1 ADIM8 3.3)
**Teknik:** `src/lib.rs:1` `#![forbid(unsafe_code)]` + `#![allow(warnings)]` (user-decided, dead_code gizler). First-party 0 unsafe temiz taban, ikinci kanıt katmanı `cargo geiger` job. (PHASE8.9_ANALIZ_A1 F10)
**(Non-teknik: Güvensiz kod yasak, herkes güvenli kod yazacak)**
- A) forbid(unsafe_code) kalsın
- B) allow(unsafe_code) olsun, performans için
- C) Hiçbir şey olmasın

## Q52 — Phase 8 / G2: pedantic+nursery ratchet baseline 191
**Teknik:** `cargo clippy --all-targets -W clippy::pedantic -W clippy::nursery` → 191 uyarı/20 lint (uninlined_format_args 106, cast_precision_loss 14, cast_sign_loss 10), baseline `.github/clippy-extra-baseline.txt=191`, ARTARSA CI fail, düşürme bilinçli PR'la. (STATUS_ONLINE 263)
**(Non-teknik: Kod titizlik seviyesi 191'de sabit, artarsa alarm)**
- A) Ratchet 191 kalsın, artarsa fail
- B) Baseline 0 olsun, tüm pedantic temizlensin
- C) Pedantic/nursery tamamen kapatılsın

## Q53 — Phase 8 / G3: udeps unused dependency kapısı (Dalga 14)
**Teknik:** `cargo-udeps --locked`, `scripts/check-udeps.sh` gerçek format parse ("unused dependencies:" + ağaç parse → paket:dep), kanaryalı self-test, baseline `.github/udeps-baseline.txt` 4 bulgu (budlum-core:chrono, group, bud-node:serde_json, bud-proof:p3-uni-stark) sıfır hit grep kanıtlı, ratchet yeni satır → fail. (dbc99b0 ARENA2)
**(Non-teknik: Kullanılmayan kütüphane varsa CI kızsın)**
- A) Udeps ratchet kalsın
- B) Udeps tamamen kaldırılsın
- C) Udeps sadece bilgi versin, gate olmasın

## Q54 — Phase 8.5 / G11: geiger unsafe görünürlük
**Teknik:** `scripts/check-geiger.sh` kanaryalı (first-party unsafe FAIL / deps bilgi / temiz PASS) + geiger job supply-chain-extra'da, G1 forbid(unsafe_code)'dan bağımsız ikinci kanıt katmanı, third-party unsafe rapora düşer. (STATUS_ONLINE 440)
**(Non-teknik: Bizim kodda güvensiz kod yok, başkasının kodunda varsa raporlansın)**
- A) Geiger first-party 0 kanıt kalsın
- B) Geiger tamamen kaldırılsın
- C) Third-party unsafe de FAIL olsun

## Q55 — Phase 8.5 / G14: bud-e2e-invariants isim-kilitli job
**Teknik:** `bud-e2e-invariants` job 9 invariant + 3 e2e, `scripts/check-bud-e2e.sh` isim kanaryasıyla ZORUNLU, vacuous-gate koruması invariant silinir/yeniden adlandırılırsa cargo test yeşil kalsa bile FAIL. (STATUS_ONLINE 351)
**(Non-teknik: Kritik testlerin ismi değişirse CI yakalasın)**
- A) İsim kanaryası kalsın
- B) İsim kanaryası kaldırılsın, sadece test sayısına bak
- C) E2E job tamamen kaldırılsın

## Q56 — Phase 8 / G7: CODEOWNERS kritik yollar
**Teknik:** `/src/consensus/`, `/src/crypto/`, `/src/rpc/`, `/config/` → @lubosruler @eurymedee, org team kurulana kadar catch-all aynı ikili. (STATUS_ONLINE 353)
**(Non-teknik: Kritik dosyalara dokununca sahipleri onaylasın)**
- A) CODEOWNERS kalsın
- B) CODEOWNERS kaldırılsın, herkes her yere dokunabilsin
- C) Daha geniş team ekle

## Q57 — Phase 8 / G6: trivy image (docker-smoke.yml)
**Teknik:** `trivy image` budlum-core:smoke-test (vuln+secret+misconfig, CRITICAL/HIGH=fail+ignore-unfixed), `docker image inspect` imza kanıtı. (STATUS_ONLINE 361)
**(Non-teknik: Docker imajında güvenlik açığı varsa CI durdur)**
- A) Trivy image kalsın
- B) Trivy sadece bilgi versin, fail olmasın
- C) Trivy kaldırılsın

## Q58 — Phase 8.5 / G27: zizmor workflow güvenlik lint'i + G5 persist-credentials sertleştirme
**Teknik:** zizmor v1.27.0 sürüm+sha256 pinli, `scripts/check-zizmor.sh` kanaryalı (pull_request_target+head-checkout→FAIL, temiz→PASS), repo-lint job'ında kapı, 0-bulgu politikası, `persist-credentials: false` sertleştirme. (STATUS_ONLINE 363-365)
**(Non-teknik: GitHub workflow dosyaları güvenli mi diye robot kontrol)**
- A) Zizmor 0-bulgu kalsın
- B) Zizmor uyarı versin ama geçsin
- C) Zizmor kaldırılsın

## Q59 — Phase 8.4 / G4: modul-bazlı coverage altyapısı (ratchet 64.00→60.00)
**Teknik:** `cargo llvm-cov` `coverage/cov.json`, `scripts/check-coverage.sh` kanaryalı, baseline `.github/coverage-baseline` (64.00 → 60.00), düşerse fail, `coverage/` artifact 30 gün. (Phase 8.4 Dalga 12, Phase 8.9 Dalga 7b)
**(Non-teknik: Testler kodun %60'ını kapsasın, düşerse alarm)**
- A) Coverage ratchet kalsın (60.00)
- B) Coverage sadece bilgi, gate olmasın
- C) Coverage kaldırılsın

## Q60 — Phase 8.10/8.11: actionlint + buf + genesis schema + branch protection (Repo Lint)
**Teknik:** `actionlint` workflow lint, `buf build+lint+breaking` (`.git#branch=origin/main` fix F8), `check_genesis_schema.py` mainnet/testnet/devnet JSON schema `base_fee + bud_tokenomics` var mı, zizmor. (510a510, 4c46f64)
**(Non-teknik: Workflow, protobuf, genesis dosyaları doğru formatta mı)**
- A) Repo Lint tam kalsın
- B) Sadece actionlint kalsın
- C) Repo Lint kaldırılsın

## Q61 — Phase 9 F2: MainnetActivation wire vs kaldır vs config-driven (Q-X2)
**Teknik:** `bud-isa::MainnetActivation {verify_merkle_enabled:bool}` + `decode_for_mainnet()` + `MainnetActivationRequired` error, VM `decode_instruction(raw, mainnet_mode)` içinde `is_verify_merkle_enabled()` env var `BUDLUM_VERIFY_MERKLE` (config `features.verify_merkle` true default) → `full()` (gate open) vs `default()` (closed). ARENA3 bayrak kaldır önerisi, ARENA2 wire, son karar config-driven (6dd66e5). (F2 fix)
**(Non-teknik: VerifyMerkle mainnet'te açık mı kapalı mı, config'den mi kontrol edilsin?)**
- A) Config-driven kalsın (features.verify_merkle + env var, default true=open)
- B) Bayrak tamamen kaldırılsın, her zaman açık (dürüst doc)
- C) Bayrak hep kapalı kalsın, sadece testnet'te açık
- D) Config-driven ama default false (staged rollout, ceremony'de true yapılır)

## Q62 — Phase 9 F2: Genesis ceremony'de verify_merkle flip (Q-X2 devamı)
**Teknik:** `MAINNET_GENESIS_CEREMONY.md` §6 bootnodes + DNS seeds placeholder, şimdi `verify_merkle` flip de ceremony checklist'e eklenmeli mi? `GENESIS_FLIP_CHECKLIST.md` cross-ref.
**(Non-teknik: Tören günü VerifyMerkle'yi de açıyoruz diye listeye yazalım mı?)**
- A) Evet, ceremony checklist'e verify_merkle flip ekle
- B) Hayır, sadece config yeterli
- C) Ceremony dokümanı tamamen kaldır

## Q63 — Phase 9 F3: Vendor mechanism wiring (Q-X3)
**Teknik:** `Pkcs11Signer::with_vendor_mechanisms(Option<String>, Option<String>)` parse `"0x..."` hex veya decimal, `MechanismType::new_vendor_defined` fail-closed, `VendorDefinedMechanism::new::<()>(mech_type, None)`, `try_vendor_sign` BLS/PQ için. (V14 fix)
**(Non-teknik: Donanımın özel imza modunu kullanmak artık mümkün)**
- A) Vendor mechanism wire kalsın (mevcut)
- B) Wire kaldırılsın, sadece software fallback
- C) Vendor mechanism tamamen kaldırılsın, sadece Ed25519 HSM

## Q64 — Phase 9 F4: Boost weighted distribution (Q-X4 devamı)
**Teknik:** `pending_bud_boost_share` biriktir, `distribute_bud_boost_share(boost_share)` aktif deal'lerin `fee_per_epoch` ağırlığına göre dağıt, `share = boost_share * weight / total_weight`, dust ilk deal operatörüne, aktif deal yoksa burn. `add_balance`. Test `boost_share_distributes_by_deal_fee_weight_with_dust_to_first` ve `boost_without_active_deals_burns_share_and_drains_pool` mühür. (7f054d7, eb45388, aa9cfcd fix op1/op2 51/52)
**(Non-teknik: Boost'tan gelen %4 depolayanlara ne kadar iş yapıyorlarsa o kadar)**
- A) Weighted + dust first kalsın (mevcut)
- B) Eşit dağıtım (her operatöre aynı)
- C) Stake ağırlıklı dağıtım (ne kadar stake o kadar pay)
- D) Tamamı ilk operatöre

## Q65 — Phase 9 F5: Genesis persistence `let _ =` → `tracing::error!` (High bulgu)
**Teknik:** `blockchain.rs:503-504` `insert_block` + `save_last_hash` dead_code path + `:2843` reorg sonrası `save_last_hash` ve `load_state` artık `if let Err(e) => tracing::error!`. 267 total `let _ =` içinde kritik yollar sertleştirildi, Option/rx.await bilinçli dokunulmadı. (ARENA3 7/16 High, F5 fix)
**(Non-teknik: Diske yazarken hata olursa sessizce yutma, bağır)**
- A) Error log ile görünür kalsın (mevcut)
- B) Sessiz yutma geri gelsin (`let _ =`)
- C) Panic olsun (fail-fast)

## Q66 — Phase 9 F6: Test-count prose stale (badge vs README)
**Teknik:** Badge `tests-538/539/546%20lib` otomatik bot (`chore(badge): tests rozeti -> N lib (CI kanitli, SHA)`), prose `README.md:114` ve `MAINNET_READINESS.md` manuel tazeleme, bot loop guard'lı self-commit. (F6 fix)
**(Non-teknik: Rozet otomatik, yazıdaki sayı elle düzeltiliyor)**
- A) Badge otomatik, prose manuel (mevcut)
- B) Prose da otomatik olsun
- C) Sadece badge yeterli, prose kaldırılsın

## Q67 — Phase 9 F7: Guard test strength regression (F7)
**Teknik:** `test_placeholder_peer_detection_blocks_dummy_mainnet_entries` synthetic dummy + compiled `MAINNET_BOOTNODES` ve `MAINNET_DNS_SEEDS` placeholder yakalama assert'i (derlenmiş sabitler). (F7 fix, c953049 regresyonu)
**(Non-teknik: Gerçek mainnet adresleri sahte mi diye test eden test güçlendirildi)**
- A) Güçlendirilmiş test kalsın (synthetic + compiled)
- B) Sadece synthetic yeterli
- C) Test tamamen kaldırılsın

## Q68 — Phase 9 F8: CI buf breaking step non-main dallarda kırık
**Teknik:** `ci.yml:442` `buf breaking --against '.git#branch=main'` → `'.git#branch=origin/main'` (local main ref yokken repo lint kırmızı, job 87812434426 kanıtı). Token `workflows` izni yok, ARENA2 fix. (F8)
**(Non-teknik: Dalda iken de CI doğru kontrol etsin)**
- A) origin/main fix kalsın
- B) main kalsın (eski, dallarda kırmızı)
- C) Buf breaking tamamen kaldırılsın

## Q69 — Phase 9 F9: Genesis hash constant unasserted + drift
**Teknik:** `config/mainnet.toml:5` hash `02166d370613fc70e5beb47e4d1ef48e5ccad93eb0f4b8bd5edfe5787a7f98fc` (eski `9bf07f9f...` drifted), `test_mainnet_genesis_hash_matches_documented_constant` absolute assert, `PRODUCTION_RUNBOOK.md` §8.2 ve `mainnet.toml` comment sync. (F9 fix 5fb7215, 4aa616f)
**(Non-teknik: Genesis hash'i dokümanda yazanla kod aynı mı diye test)**
- A) Absolute assert kalsın (drift yakalar, mevcut)
- B) Sadece JSON==code eşitliği yeterli (relative)
- C) Genesis hash hiç dokümante edilmesin

## Q70 — Phase 9 F10: `#![allow(warnings)]` + `forbid(unsafe_code)`
**Teknik:** `src/lib.rs:1` `#![allow(warnings)]` user-decided, dead_code görünürlüğünü kapatır, denetim manuel grep ile, `#![forbid(unsafe_code)]` bağımsız. (F10 note, ⚪)
**(Non-teknik: Uyarıları sustur ama güvensiz kodu yasakla)**
- A) allow(warnings) + forbid(unsafe_code) kalsın (mevcut)
- B) allow(warnings) kaldırılsın, tüm uyarılar gösterilsin
- C) forbid(unsafe_code) kaldırılsın

## Q71 — Phase 0.378 Gap Matrix ve Execution Plan
**Teknik:** `PHASE0.378_GAP_MATRIX.md` ve `EXECUTION_PLAN.md` BLS/PQ key protection, finality live-path, ConsensusStateV2 notları, external audit checklist, README roadmap, DEVIR raporu. (Phase 0.378)
**(Non-teknik: Mainnet öncesi borç listesi)**
- A) Gap matrix güncel tutulup kapatılsın
- B) Gap matrix kaldırılsın
- C) Gap matrix sadece dış denetçiye verilsin

## Q72 — Phase 0.42: Mainnet Launch (2 alt-tur: 0.43 devnet pilot + harici audit, 0.438 audit kabul + launch)
**Teknik:** Phase 0.40 önkoşul 7 iş paketi (BLS/PQ HSM, B.U.D. Faz 1-2, finality live-path, ConsensusStateV2, external audit checklist, README roadmap, fuzzing/audit/SBOM). Phase 0.43: storage-operator.toml, 3+ bağımsız operatör, E2E smoke, permissionless kayıt testi, 1 hafta monitoring + audit firması seçimi/kickoff. Phase 0.438: AUDIT_REPORT, mainnet.toml Config V2 strict, governance/budzkvm_contract/pruning=false, ORG_ROADMAP_AUDIT §4b. (PHASE0.42_PLAN.md)
**(Non-teknik: Önce önkoşulları bitir, sonra devnet pilot + audit, sonra mainnet)**
- A) 2 alt-turlu plan kalsın (mevcut)
- B) Tek turda direkt mainnet
- C) Mainnet hiç yapılmasın, sonsuza kadar devnet

## Q73 — Phase 8.9 Analiz: Bitmiş/kırık/çürümüş/placeholder ayrımı
**Teknik:** Matris 3 kova: (A) kırık/çürümüş → bu süreçte düzeltilecek; (B) dokümante-placeholder kodu sağlam → ceremony gününe kadar boşluk değil; (C) kullanıcı-taraflı fiziksel kalemler (7.1 genesis keys, 7.2 bootnode gerçekleri, 7.3 HSM donanımı, M7 dış denetim) → tooling/şablon/fail-closed guard/checklist/hash-freeze kapatılacak, kalan "ceremony günü input listesi" dokümanında toplanacak. (PHASE8.9_ANALIZ_A1.md §1)
**(Non-teknik: Neyi şimdi, neyi tören günü yapacağız net ayıralım)**
- A) 3 kova ayrımı kalsın
- B) Her şey şimdi bitirilsin (ceremony yok)
- C) Her şey ceremony'ye ertelensin

## Q74 — Phase 1 Rapor: B.U.D. Faz 1-2 + Faz 5 iskeleti PR #6
**Teknik:** PR #6 HEAD `39e30c7` 8 commit: ARENA_AI.md adaptasyon, STATUS.md, 4 kayıp PR kurtarma, finality_live_path revert, Phase 0.38 Rust iskeleti `ConsensusKind::StorageAttestation(StorageDomainParams)` + `STORAGE_OPERATOR=RoleId(5)` + `ContentId`+`of_subrange` + `ContentManifest`+`ShardRef`+`manifest_id_from_shards` + `StorageDeal`+`StorageRegistry`+`RetrievalChallenge/Response/Outcome/Result` + 7 storage RPC + 3-aktör E2E + 9 ekip-bağımsızlık invariant. (PHASE1_RAPOR.md, STATUS.md)
**(Non-teknik: Depolama alanı için ilk altyapı bitti)**
- A) Phase 1 kapsamı doğru, PR #6 merge kalsın
- B) Phase 1 fazla büyük, küçültülsün
- C) Phase 1 tamamen geri alınsın

## Q75 — Phase 0.06 aslında Phase 0.44: VerifyMerkle gate (detaylı)
**Teknik:** `is_experimental()=false` tüm opcode'lar production-ready, `decode_for_mainnet` + `MainnetActivationRequired` + 3 test (default reject, full allow, other bypass), `tur119_verify_merkle_disabled_in_production` kaldır/güncelle, `GlobalBlockHeader.storage_root` block hash'e dahil. (docs/PHASE0.06_PLAN.md aslında 0.44)
**(Non-teknik: Doğrulama işi bitti mi?)**
- A) VerifyMerkle gate açık kalsın (mevcut)
- B) Gate kapalı kalsın
- C) Gate sadece experimental feature ile açık

## Q76 — Phase 0.08 aslında 0.46: Universal Relayer + Mobile B.U.D. Light Node
**Teknik:** `ExternalChain` enum (Ethereum, Solana, Bitcoin), `ExternalTransaction` Budlum cüzdanı ile imzalanıp relayer dış zincire basar, RPC `bud_relayerPrepareExternalTx`, `MobileConfig` batarya/Wi-Fi dostu limitler, `ShardManager` self-host önceliği, `Node::run` NftBurn worker `store.delete(cid)`. (PHASE0.08_PLAN.md)
**(Non-teknik: Telefonun da depolama düğümü olsun, dış zincirlere köprü)**
- A) Relayer + mobile light node kalsın
- B) Sadece relayer kalsın, mobile kaldırılsın
- C) İkisi de kaldırılsın

## Q77 — Phase 0.10 aslında 0.48: B.U.D. Gateway + Relayer EVM Proofs + SocialFi Feed
**Teknik:** Gateway `.bud` ismini HTML/Media'ya çeviren proxy `bud_gatewayFetchContent`, `RelayerExternalResult` receipt proof, SocialFi feed NFT sahipliğine dayalı SQL/Index, mainnet bootnodes tören sonrası P2P, Eco-Frontend proto Hub web. (PHASE0.10_PLAN.md)
**(Non-teknik: .bud ismini tarayıcıda aç, dış zincir sonucunu doğrula, akış göster)**
- A) Gateway + proofs + feed kalsın
- B) Sadece gateway kalsın
- C) Hepsi kaldırılsın

## Q78 — Phase 8.9 Dalga 1+2+3 (README 509→522, C1 dangling, .gitignore sbom, fuzz check)
**Teknik:** Dalga 1 küçük README 509→522 + L113 452→522 + C1 dangling Q1, .gitignore+sbom.cdx.json + cargo-fuzz metadata; Dalga 2 Q2+Q3 belge birleştirme/silme + genesis hash freeze checklist; Dalga 3 Q5 dummy-bootnode fail-closed guard + key-üretim script + allocations/validators JSON schema + ceremony input list tek dosya. (PHASE8.9_ANALIZ_A1.md §5)
**(Non-teknik: Önce küçük temizlik, sonra belge birleştirme, sonra güvenlik bekçisi)**
- A) Dalga planı kalsın
- B) Tüm dalgalar tek seferde yapılsın
- C) Dalga planı iptal

## Q79 — Phase 8.9 kalan ADIM 8.5 maddeleri (P1.1/P1.7/miri, geiger, semver-checks, cosign SBOM-signing, KAT vectors, dudect, PKCS#11 mock negative tests, X-Real-IP spoofing, zizmor, branch protection)
**Teknik:** ADIM8 3.3 Faz1 tamamlandıktan sonra P1/P2 ve ADIM8.5 eksikleri listesi (STATUS_ONLINE 2026-07-16 19:45 ARENA3). (PHASE8.9)
**(Non-teknik: Güvenlik için daha neler ekleyeceğiz listesi)**
- A) ADIM8.5 maddeleri tek tek kapatılsın
- B) Hepsi ertelensin, Phase 10'a
- C) Sadece miri ve geiger kalsın

## Q80 — MAINNET_READINESS MR-1..MR-10 (Phase 8.9 Dalga 5 sonrası)
**Teknik:** MR tablosu: Phase 8 full closure ADIM8-TALIMAT-1 (12 tasks) + ADIM8.5 add-ons (miri, geiger, semver-checks, cosign SBOM-signing, KAT vectors, dudect, PKCS#11 mock negative tests, X-Real-IP spoofing, zizmor, branch protection) + uploads talimat + CI kapıları. (MAINNET_READINESS.md MR-2)
**(Non-teknik: Mainnet'e hazır mıyız kontrol listesi)**
- A) MR listesi güncel tutulup yeşil olmadan mainnet yok
- B) MR listesi bilgi amaçlı, mainnet kararı ayrı
- C) MR listesi kaldırılsın

## Q81 — ORG_ROADMAP_AUDIT §4a 18 madde tablosu
**Teknik:** PR #6 CI yeşil, PR başlığı doğru, HEAD 39e30c7, StorageAttestation enum varyantı VAR, STORAGE_OPERATOR RoleId 5 VAR, content_id.rs, manifest.rs, storage_deal.rs, bud_e2e.rs VAR, docs/BUD/ kısmen, permissionless PoA izolasyon testi VAR, budlum.com URL YOK, admin/pause/freeze/owner hook YOK, B.U.D. upstream vizyon 495 satır VAR, 7 storage RPC VAR, PoA izolasyon bozulmadı. (ORG_ROADMAP_AUDIT.md §4a)
**(Non-teknik: Organizasyonun yol haritasındaki maddeler tek tek doğrulandı)**
- A) 18 madde audit tablosu kalsın ve güncel tutulsun
- B) Audit tablosu kaldırılsın
- C) Audit tablosu sadece dış denetçiye verilsin

## Q82 — B.U.D. data-sovereignty kuralı (Phase 0.39 plan §0.5)
**Teknik:** `open_deal` ve `open_challenge` permissionless, opener_bond >0 anti-spam, admin/pause/freeze/force hook kodu incelemesiyle YOK (`grep -n 'fn admin_\|fn pause_\|fn force_\|fn owner_\|fn freeze_'` boş). (PHASE0.42_PLAN.md §4.7)
**(Non-teknik: Kimseye muhtaç olmadan anlaşma açabilme)**
- A) Data-sovereignty kuralı kalsın (no admin hook)
- B) Admin hook eklenebilir (acil durum durdurma)
- C) Sadece takım açabilsin

## Q83 — PoA izolasyonu garantisi
**Teknik:** `STORAGE_OPERATOR` `PermissionlessRegistry` primitive'ini paylaşıyor, `PoaMembershipRegistry`ye dokunulmadı, `src/tests/permissionless.rs` PoA izolasyon testi sağlam (88-104). (CLAUDE.md §2, STATUS.md)
**(Non-teknik: Depolama operatörleri PoA'yı bozamaz)**
- A) PoA izolasyonu kesin korunsun
- B) PoA ve permissionless birleştirilsin
- C) PoA tamamen kaldırılsın

## Q84 — Slashing kalıcılığı + geçmiş (Phase 0.40 Görev 1) ve InvalidVoteTracker (Görev 2)
**Teknik:** `PermissionlessRegistry` `slashing_history: Vec<SlashingRecord>` `#[serde(default)]`, her ACTIONED rapor TEK slash yolunda geçmişe yazılır, `StateSnapshotV2` round-trip, `InvalidVoteTracker` EPOCH-bazlı sayaç kalıcı `AccountState.invalid_votes` + `StateSnapshotV2.invalid_votes`, threshold `max_invalid_votes_per_epoch=20` aşılınca `InvalidSignatureSpam`. (CLAUDE.md)
**(Non-teknik: Kimin ne zaman ceza yediği sonsuza kadar saklansın, çok yanlış oy atana da ceza)**
- A) Slashing history + InvalidVoteTracker kalsın
- B) Sadece slashing history kalsın, invalid vote kaldırılsın
- C) İkisi de kaldırılsın

## Q85 — Evidence spam koruması + Prover → L1 köprüsü (Model B tam açık)
**Teknik:** `submit_registry_slashing_report` reporter fee 10 (actionable iade, değilse yakılır), consensus-içi `reporter:None` fee yok, RPC provenance zorla `Unverified`. `submit_zk_proof(ZkProofSubmission)` kayıt ŞART DEĞİL, STARK kendini doğrular, `PROVER` rol sadece ÖDÜL için opsiyonel, `proof_submission_fee=10` geçerliyse iade geçersizse yakılır, `ProofClaimRegistry` ilk geçerli kazanır. (CLAUDE.md)
**(Non-teknik: İhbar eden de küçük ücret yatırsın, doğruysa geri alsın; kanıt sunmak için kayıt gerekmesin)**
- A) Fee + iade + yanma modeli kalsın
- B) Fee tamamen kaldırılsın, herkes ücretsiz ihbar etsin
- C) Fee iadesiz direkt yakılsın

## Q86 — B.U.D. Faz 5 economics fail-closed durumu
**Teknik:** Faz 5 ekonomi katmanı mainnet için fail-closed, Payer/Escrow ve bond escrow hazır olana kadar token basımı/yakımı devre dışı, `accrue_storage_operator_rewards` escrow needed log, slashed bond burn skip. (MAINNET_READINESS §1 Phase 1 tamamlananlar UYARI)
**(Non-teknik: Ekonomi kodu hazır ama gerçek para basma/yakma kapalı, emanet sistemi gelene kadar)**
- A) Fail-closed kalsın, escrow gelene kadar
- B) Fail-open olsun, likit bakiyeden yakma devam etsin
- C) Ekonomi tamamen kapatılsın

## Q87 — BudZero/BudZKVM derin denetim (BUDZERO_DERIN_DENETIM_ARENA3.md) ve TRACE_WIDTH=414 layout
**Teknik:** 7 crate, sıfır güvenlik açığı, `TRACE_WIDTH=414` layout dokümantasyonu + sütun çakışma boundary testi (Paket A), `Expr->ExprEF` type mismatch fix for Register LogUp, `Program CTL LogUp multiplicity fix` VerifyMerkle expansion rows excluded. (Phase 8.9 Dalga 3-5)
**(Non-teknik: ZK sanal makinesinin içi didik didik denetlendi)**
- A) Derin denetim raporları güncel tutulup sıfır açık korunsun
- B) Denetim raporları kaldırılsın
- C) Denetim sadece dış firmaya bırakılsın

## Q88 — Chaos engineering + disaster recovery (E2E + finality_live_path)
**Teknik:** `src/tests/disaster_recovery.rs` `test_chaos_v2_nft_burn_pruning_after_restart` NFT burn sonrası restart state tutarlılık, `finality_live_path.rs` 4 test, `finality_adversarial.rs` BLS vote gerçek anahtar çiftleriyle mock değil, `FinalityAggregator` ingest-time doğrulama + equivocation→slashing. (Phase 2 §1.3, Phase 8.9)
**(Non-teknik: Zincir çökse de kurtarma testi)**
- A) Chaos + disaster recovery testleri kalsın ve genişlesin
- B) Sadece unit test yeterli
- C) Chaos testleri kaldırılsın

## Q89 — BNS + Marketplace + Hub + Relayer modülleri (ADIM6)
**Teknik:** `src/{bns,gateway,hub,marketplace,nft,relayer}` + `BnsRegistry`, `MarketplaceRegistry`, `HubRegistry`, `NftRegistry`, `Relayer` permissionless kayıt, `bns_insufficient_payment` M4 fee gate, `HUB_REGISTER_MIN_FEE=100` M5 fee. (PHASE8.9_ANALIZ_A1 F1, CONSTITUTION §7)
**(Non-teknik: İsim sistemi, pazar yeri, hub ve köprücü modülleri)**
- A) BNS+Marketplace+Hub+Relayer tam kalsın
- B) Sadece BNS kalsın, diğerleri kaldırılsın
- C) Hepsi kaldırılsın

## Q90 — Mainnet genesis ceremony (MAINNET_GENESIS_CEREMONY.md) ve GENESIS_FLIP_CHECKLIST
**Teknik:** 7.1 genesis keys (Ed25519/BLS/Dilithium5), 7.2 bootnode gerçek multiaddr'lar ceremony'de replace, 7.3 HSM donanımı, 7.5 timeline, T-7/T-0/T+1 checklist, hash freeze `print_genesis_hash.rs` `MAINNET_HASH=02166d370613fc70e5beb47e4d1ef48e5ccad93eb0f4b8bd5edfe5787a7f98fc`, bootnodes placeholder `203.0.113.x` + `placeholder-seed-*` fail-closed. (Phase 7, ops/MAINNET_GENESIS_CEREMONY, Q5)
**(Non-teknik: Gerçek ağ açılış töreni prosedürü)**
- A) Ceremony prosedürü + fail-closed guard + hash freeze kalsın
- B) Ceremony olmadan direkt mainnet açılsın
- C) Ceremony sadece bir kişi yapsın, çoklu imza olmasın

## Q91 — Security Audit Hacker + Threat Model
**Teknik:** `docs/SECURITY_AUDIT_HACKER.md` StoragePrune P2P gossip ile tetiklenirse hacker sahte prune ile data silebilir → Fix: StoragePrune SADECE local Executor'dan sonra verified NftBurn ile. `THREAT_MODEL.md` + `BUG_BOUNTY.md`. (Phase 9 F1 Q-X1)
**(Non-teknik: Biri başkasının verisini silemesin diye koruma)**
- A) StoragePrune sadece local executor'dan kalsın (mevcut güvenlik)
- B) P2P'den de çağrılabilsin (hızlı yayılım ama riskli)
- C) Pruning tamamen kaldırılsın

## Q92 — CI root cause analysis (CI_ROOT_CAUSE_ANALYSIS_ARENA5.md) ve M5 VerifyMerkle raporu
**Teknik:** Kırmızı zincirlerin kök nedeni fmt/clippy'siz push (3 ardışık Format/Clippy kırmızısı: 9be811b, 749d27f/c953049, dbc99b0/c69e1c0), öneri `scripts/pre-push-check.sh` fmt+clippy+test. M5 raporu anti-sybil fee + L1 gerçek Merkle doğrulama + M4 regresyon. (Phase 8.9)
**(Non-teknik: Neden CI hep kırmızı oluyor, nasıl düzelir)**
- A) Pre-push-check script kullanılsın, CI kırmızı kök nedeni fmt/clippy'siz push
- B) CI'da fmt/clippy tamamen kaldırılsın
- C) CI her zaman yeşil sayılsın, kırmızı görmezden gelinsin

## Q93 — Bench baseline ve single_node internal_pipeline_tps
**Teknik:** `benches/micro/{merkle_scaling,merkle_update,sig_verify,timing_safe}` + `single_node/internal_pipeline_tps`, `docs/BENCH_BASELINE.md`, BudZero `proof_baseline.rs` proof süre/boyut JSON. (Phase 6, Phase 0.37)
**(Non-teknik: Ne kadar hızlı, ölçelim)**
- A) Bench baseline güncel tutulup ratchet edilsin
- B) Bench sadece bilgi, gate olmasın
- C) Bench tamamen kaldırılsın

## Q94 — Ops runbook'lar: HSM_BLS_PQ_POLICY, HSM_VENDOR_NATIVE_GUIDE, FINALITY_LIVE_PATH, MIGRATION_V2, NETWORK_HARDENING, PRODUCTION_RUNBOOK, SBOM, DEPENDENCY_AUDIT, BNS_MAINNET
**Teknik:** `docs/operations/` altında 11 runbook, her biri fail-closed §4, roller, hash freeze, minutes şablonu, HSM vendor BLS/PQ mechanism desteği. (Phase 2 §1.1, ORG_ROADMAP)
**(Non-teknik: Operatörler ne yapacak, adım adım kılavuz)**
- A) Tüm runbook'lar güncel kalsın ve ceremony'de kullanılsın
- B) Sadece PRODUCTION_RUNBOOK yeterli
- C) Runbook'lar kaldırılsın

## Q95 — Tokenomics: block_reward, annual_burn, validator_apy, metabolic_burn, BUD_UNIT, team_vesting
**Teknik:** `tokenomics/mod.rs` `block_reward=50`, `annual_burn_ratio=10%`, `validator_apy=5%`, `metabolic_burn=1%`, `BUD_UNIT=1e6`, vesting edge case testleri 6 ondalıklı hassasiyet, epoch ödül hesaplama. (893ffdc, ff29310, 920e9fe)
**(Non-teknik: Token nasıl dağıtılıyor, yakılıyor)**
- A) Mevcut tokenomics kalsın (50 ödül, %10 yakım, %5 APY, %1 metabolic)
- B) Ödül 100 olsun, yakım %5 olsun
- C) Tokenomics tamamen yeniden yazılsın

## Q96 — Liveness slashing enabled flag (Phase 0.34) ve InvalidVoteTracker threshold
**Teknik:** `maybe_observe_liveness_on_epoch_close` `RegistryParams::liveness_slashing_enabled` bayrağına göre ayrışır: true → gerçek slash (stake keser + jail), false (varsayılan) → rapor + tracing log, ekonomik etki yok. `slash()` her ihlalde jail ettiği için %1 downtime bile jail eder. `InvalidVoteTracker` threshold 20. (CLAUDE.md Phase 0.34, Phase 0.40 Görev 2)
**(Non-teknik: Çevrimdışı kalana ne kadar tolerans, şimdilik sadece izle)**
- A) Varsayılan kapalı kalsın (önce gözlemle, testnet'te doğrula, sonra aç)
- B) Varsayılan açık olsun, direkt jail
- C) Liveness slashing tamamen kaldırılsın

## Q97 — RPC + BNS + Hub + Storage RPC'leri (7 storage RPC + gateway)
**Teknik:** `src/rpc/api.rs:272-365` trait + `server.rs:1395-1818` impl, `bud_storageRegisterManifest`, `GetManifest`, `GetDealsByManifest/Shards`, `OpenChallenge`, `AnswerChallenge`, `GetOutcome`, `bud_storageOpenDeal` (VerifyMerkle ile), `ActiveOperators`, economics, `bud_gatewayFetchContent`, `bud_relayerPrepareExternalTx`, `bud_registry*`, `bud_submitZkProof`, `bud_submitSlashingReport`. (F1-F10 V2)
**(Non-teknik: Depolama, gateway, köprü, kayıt için RPC komutları)**
- A) Tüm RPC'ler kalsın (7 storage + gateway + relayer + registry)
- B) Sadece storage RPC'ler kalsın
- C) RPC'ler tamamen kaldırılsın

## Q98 — AI Birliği ve görev dağılımı (AI_BIRLIGI.md, ARENA_GOREV_DAGILIMI)
**Teknik:** Şema + tarih + görev ayrımı, aktif dal `arena/019f...`, 3 AI (ARENA1 Core, ARENA2 ZK/Build, ARENA3 HSM/Security, ARENAX denetim), STATUS_ONLINE protokolü, soru sorma zorunluluğu (ask_user), push onay bekleme, token harcama sınırsız derin analiz. (User 6 madde + ARENA_AI.md)
**(Non-teknik: AI'lar nasıl birlikte çalışacak, kurallar)**
- A) AI birliği şeması ve görev dağılımı kalsın
- B) Tek AI yeterli, birlik kaldırılsın
- C) AI'lar tamamen kaldırılsın, sadece insan

## Q99 — Mainnet readiness ve badge rozet otomasyonu
**Teknik:** `README.md` rozet `chore(badge): tests rozeti -> 546 lib (CI kanitli, SHA)` CI self-commit loop guard'lı, Q5 kararı yalnız sayı değişiminde yalnız main push'unda, `MAINNET_READINESS.md` §1 tablo 546 lib, `docs/STATUS_ONLINE.md` canlı, `PHASE8.9_ANALIZ_A1.md` ve `REPORTS_INDEX.md`. (Phase 8.4 Dalga 7b, F6)
**(Non-teknik: Test sayısı rozeti otomatik artıyor)**
- A) Badge otomasyonu kalsın (mevcut)
- B) Badge manuel olsun
- C) Badge tamamen kaldırılsın

## Q100 — Phase 9 final denetim ve Phase 8.9+ next steps (full P2P prune, config-driven verify_merkle, treasury_pool, ceremony flip)
**Teknik:** Phase 9 final denetim raporu `PHASE9_FINAL_DENETIM_ARENA3.md` 10 kapı yeşil, 660 test, sıfır stub, VerifyMerkle açık, F1-F10 kapanış, `PHASE9_VIZYON_KOD_CELISKI_DENETIM_ARENAX_2026-07-17.md` F1-F10, CI 13/13 yeşil baz `2acef45`, son ana dal 546 lib. Kullanıcı kararları: Q-X1 full_p2p_prune (proto + NetworkMessage::StoragePrune gossip), Q-X2 config_driven (features.verify_merkle + BUDLUM_VERIFY_MERKLE env), Q-X4 treasury_pool (protocol_share %80 burn_reserve/treasury, bud_share weighted + dust first, pending drain). (User son 3 soru cevabı: implement_p2p, add_ceremony, hold-dust+emit+new_treasury+single+no_rpc+parallel)
**(Non-teknik: Son denetim tamam, sırada tam P2P silme, config ile kapı kontrolü ve ekip kasası)**
- A) Full P2P prune + config-driven verify_merkle + treasury_pool (burn_reserve veya yeni TREASURY_ADDRESS, single sig, no RPC, event emit, hold dust) hepsi implemente edilsin, ceremony dokümanına verify_merkle flip eklensin, sonra mainnet readiness final (mevcut kullanıcı kararları)
- B) Sadece full P2P prune yapılsın, diğerleri ertelensin
- C) Hiçbiri yapılmasın, mevcut 546 lib yeşil haliyle mainnet'e gidilsin
- D) Phase 9 tamamen geri alınsın, Phase 8.9'a dönülsün
- E) Tüm Phase'lar iptal, sıfırdan yeni chain

---

**Son not:** Bu anketteki her madde için cevap anahtarını (A/B/C/D/E) işaretleyip yollayın, kalan teknik borçlar `docs/STATUS_ONLINE.md` ve `docs/PHASE9_FINAL_DENETIM_ARENA3.md` üzerinden kapatılacak. Force-push yasak, CI yeşili olmadan Phase 10'a geçilmez.

Co-authored-by: ARENA2 <arena2@budlum.ai>
