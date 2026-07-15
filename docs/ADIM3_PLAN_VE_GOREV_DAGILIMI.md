# ADIM3 — Plan & Görev Dağılımı (Yeniden Derlenmiş)

> **Not:** Orijinal `ADIM3_PLAN_VE_GOREV_DAGILIMI.md` force-push/silinme nedeniyle
> repoda bulunamadı. Bu dosya `docs/MAINNET_READINESS.md` §ADIM3, commit mesajları
> (`0.1`–`0.4`, `3.6`, escrow) ve `STATUS_ONLINE.md` kanıtlarından **2026-07-15**
> tarihinde ARENA2 tarafından yeniden derlenmiştir.

**HEAD baz:** `44fe0f0` (CI yeşil)
**Aktif dal:** `main` (force-push yasak)
**AI üyeleri:** ARENA1 (kod), ARENA2 (denetim/koordinasyon — bu oturum), ARENA3 (çekirdek/ZK)

---

## 0. ADIM3 güvenlik borçları (öncelikli)

| # | Görev | Sahip (tarihsel) | Durum |
|---|-------|------------------|-------|
| 0.1 | `StorageAttestationFinalityAdapter` PoS/Bft `cert.verify()` | ARENA1+ARENA2 | ✅ `49b6b46`/`65d0446` |
| 0.2 | `storage_open_challenge` / `answer` imza zorunluluğu | ARENA1 | ✅ `aa8feab` |
| 0.3 | `bud_storageActiveOperators` hayalet RPC | ARENA2 docs | 🟡 docs only — RPC OPEN |
| 0.4 | Mock HSM kaldırıldı, sadece PKCS#11 | ARENA1+ARENA2 | ✅ `433ab58` |

## 1. ADIM3 Mainnet v1 lansman paketi (`MAINNET_READINESS.md`)

| # | Görev | Durum | Önerilen sahip |
|---|-------|-------|----------------|
| 3.1 | Mainnet genesis config + deterministik test genişletmesi | ✅ DONE (ARENA3 JSON + ARENA2 hash/runbook) | ARENA3+ARENA2 |
| 3.2 | Docker mainnet default + systemd smoke | 🟡 kısmi | ARENA2/3 |
| 3.3 | PRODUCTION_RUNBOOK mainnet genesis hash + seed nodes | 🟡 hash yazıldı, seed ceremony boş | ARENA2 |
| 3.4 | Network hardening (rate limit stress, p2p) | 🟡 kısmi | ARENA3 |
| 3.5 | Validator onboarding E2E (stake+register) | ❌ OPEN | ARENA1 |
| 3.6 | `BUD_INTERIM.md` | ✅ DONE | ARENA2 `5321c28` |

## 2. B.U.D. yan paket (ADIM3 ile örtüşen)

| # | Görev | Durum |
|---|-------|-------|
| F5 escrow | `open_storage_deal_with_escrow` + RPC sync | ✅ `f2b8075`+`44fe0f0` |
| F4 storage_root | `GlobalBlockHeader.storage_root` | ✅ (önceki oturum) |
| F3 VerifyMerkle | production gate | 🔒 ADIM4 |
| F6 BNS/.bud | isimlendirme | 🔒 ADIM5+ |

## 3. İş akışı (kullanıcı kuralı)

1. **Aşama 1:** AI'lar `STATUS_ONLINE.md` üzerinden konuşur, görev paylaşır.
2. **Aşama 2:** Başka AI commit attı mı kontrol → sonra commit.
3. **Aşama 3:** CI yeşil olana kadar durulmaz; yanlış commit'ler `STATUS_ONLINE` + PR/commit yorumlarıyla tartışılır.
4. Force-push yok. Workflow dosyası push yok. Kanıtsız SHA yok.

## 4. Org roadmap kapsamı (dürüst)

| Kaynak | Kapsam durumu |
|--------|----------------|
| `budlum-xyz/Budlum` Research Roadmap (kodlanabilir) | Büyük ölçüde ADIM1–2 ile kapalı / tooling ready |
| `budlum-xyz/BudZero` Phase 0–9 | Büyük ölçüde; VerifyMerkle Z-B hâlâ experimental |
| `budlum-xyz/B.U.D.` Faz 1–2–4–5 | iskelet+ekonomi main'de; Faz 3/6 açık |
| External audit / TLA+ / Privacy / AI layer | **Bitmedi** — checklist/process only |
| Budlumdevnet / Budlumdevnet2 | Temel + tarihsel roadmap; aktif monorepo = `budlum` |

**Sonuç cümlesi:** "Tüm org roadmap'i bitirdik" **DEĞİL**. Mainnet v1 lansman paketi (ADIM3 3.1–3.5) + VerifyMerkle (ADIM4) + harici audit (ADIM5) hâlâ açık.


---

## 5. Kullanıcı karar kaydı (2026-07-15, ARENA2 oturumu)

| Karar | Seçim |
|-------|-------|
| Sıradaki öncelik | **§3.1 Mainnet genesis config + deterministik test** |
| VerifyMerkle Z-B | **Sonra** (ADIM4; ADIM3 lansman önce) |
| AI koordinasyonu | Önce `STATUS_ONLINE` yanıtı, kod bir sonraki "devam"da |
| Token | Kullanıcı: yenilendi / tek kullanımlık |

**Aktif kuyruk:** 3.1 → (0.3 veya 3.2/3.3) → 3.4/3.5 → ADIM4 VerifyMerkle


## 6. §3.1 kapanış kaydı

- ARENA3 `e012803`: `config/*-genesis.json` + deterministic tests (CI yeşil)
- ARENA2 follow-up: JSON↔code hash tests, `print_genesis_hash`, runbook §8
- Mainnet genesis hash: `16a60f4883768590b79e4f2f4abbf10ff24d4d4815069f4d98909740152f668e`
- Placeholder addresses; ceremony keys later; bootnodes empty until ceremony
