# Status Online — Aktif iletişim kanalı (AI birliği)

**Amaç:** İki (veya daha fazla) AI'ın anlık olarak ne yaptığını, ne yapacağını,
karar taleplerini ve engelleri burada paylaşması. Statik denetim kayıtları
(`docs/STATUS.md`) ve şema (`docs/AI_BIRLIGI.md`) buraya girmez — bu
dosya **çalışma günlüğü**.

**Format:** serbest, ama timestamp'li ve AI-handle imzalı. Eski entry
"resolved" notuyla kalır (audit trail).

**Yazan:** her iki AI (handle + tarih).
**Okuyan:** her iki AI + kullanıcı (PR'a bakıyor olabilir).

---

## Entry şablonu

```markdown
### [YYYY-MM-DD HH:MM UTC+3] <handle> — <özet>

**Durum:** [devam ediyor | tamamlandı | engel | karar bekliyor]
**Kapsam:** [Tur X §Y | kod | docs | test | infra]
**Aksiyon:** ne yaptım / ne yapıyorum
**Kanıt:** [commit SHA | dosya | komut çıktısı]
**Sonraki adım:** ne yapacağım
**Engel (varsa):** ne durduruyor / ne karar bekliyor
```

---

## 2026-07-14 — kuruluş günü

### [2026-07-14 19:00 UTC+3] arena-agent — dosya şeması kuruldu

**Durum:** tamamlandı
**Kapsam:** docs (AI birliği şeması)
**Aksiyon:** `docs/DEVIR_RAPORU.md` → `docs/AI_BIRLIGI.md` rename + 4. bölümde
DEVIR_RAPORU'nun tamamı korunmuş + §1 AI üyeleri tablosu (2 → 4+ dinamik)
+ §2 iletişim kanalları ayrımı + §3 aktif iş akışı + §6 kesin kurallar.
`docs/STATUS.md` "tek-ajan devir" → "statik denetim" perspektifine
güncellendi (HEAD `39e30c7` referansı, 8 commit listesi).
`docs/STATUS_ONLINE.md` (bu dosya) yeni oluşturuldu — aktif iletişim
kanalı.

**Kanıt:** `git status` → 3 dosya değişti: rename + 2 modify (AI_BIRLIGI rename
zaten git mv ile staged).

**Sonraki adım:** kullanıcı doğrulaması → commit + push.

**Engel:** kullanıcı "the-plan yerine ekledim" / "oraya budlum zip ekledim"
diye belirsizlik yarattı — `/tmp/the-plan-check/` lokal kopya mevcut, içinde
TUR14_PLAN + TUR14_5_PLAN + claude-fable-5.md var (PR'a eklenmedi, sadece
referans olarak kullanıldı). zip değil, açılmış dizin. Beklemede.

### [2026-07-14 19:40 UTC+3] arena-agent — ADIM 1 geçişi ve budlum-xyz Yol Haritası Doğrulaması (eski adı: Tur 14 / PR #9 push'landı)

**Durum:** tamamlandı (PR #9 oluşturuldu: `https://github.com/lubosruler/budlum/pull/9`)
**Kapsam:** ADIM 1 (eski Tur 14 / B.U.D. Server Sistemi) & budlum-xyz Org Yol Haritası Senkronizasyonu
**Aksiyon:**
1. Kullanıcı talimatıyla **"tur" terminolojisi kaldırılmış**, ilk adımımız resmi olarak **ADIM 1** (`ADIM 1 = eski Tur 14 + Tur 14.5 B.U.D. Server Sistemi`) olarak tanımlanmıştır ("artık ADIM1 DİYE GEÇECEK tur demek yok").
2. `github.com/budlum-xyz` organizasyonundaki 4 depoda (`Budlum`, `BudZero`, `B.U.D.`, `budlum.com`) yer alan tüm yol haritası maddeleri incelendi ve `lubosruler/budlum` (`main` HEAD `e20c414` / `39e30c7`) koduyla eşleştirildi.
3. B.U.D. (Broad Universal Database / Merkeziyetsiz Depolama Sunucu Sistemi) kod tabanı (`src/domain/storage_params.rs`, `src/domain/storage_deal.rs`, `src/storage/content_id.rs`, `src/storage/manifest.rs`, `src/rpc/api.rs`, `src/rpc/server.rs`, `src/tests/bud_e2e.rs`) incelendi ve **ADIM 1 (Faz 1-2 ve Faz 5 deal/challenge iskeleti)** kapsamında `main` dalında (`39e30c7`) hayatta olduğu doğrulandı.
4. `AI_BIRLIGI.md`, `STATUS.md`, `STATUS_ONLINE.md` ve `ORG_ROADMAP_AUDIT.md` belgelerinde ADIM 1 tanımı ve budlum-xyz yol haritası kapsayıcılığı güncellendi; PR #9 olarak push'landı (`081c4f9`).

**Kanıt:** `github.com/budlum-xyz/B.U.D.` `BUD_Merkeziyetsiz_Depolama_Vizyonu.md` (§0-§11) API sorgusu ile okundu; `main` commit `39e30c7` (7 storage RPC + 3-aktör E2E + 9 invariant) doğrulandı. PR #9 (`gh pr view 9`).
**Sonraki adım:** Diğer AI ajanları ile koordine olarak **ADIM 1** ve sonraki adımların geliştirme akışını sürdürmek.
**Engel:** Yok. (Önceki 19:05 engeli kullanıcı netleştirmesiyle çözüldü).

---

## Çözülmüş entry'ler

### [2026-07-14 19:05 UTC+3] arena-agent — [resolved] bekleme

**Durum:** resolved (`ADIM 1` adı altında `main` branch HEAD `39e30c7` commitleri teyit edilerek çözüldü)
**Kapsam:** repo state
**Aksiyon:** eklenen dosyalar hakkında kullanıcıdan net bilgi bekleniyordu. Kullanıcı `tur14 ile başlayacağız. B.U.D. yani server sistemimiz commıtlerde ve diğer belgelerde var ancak forge push yüzünden silindi, buradan başlıyoruz ve ilk adımımız artık ADIM1 DİYE GEÇECEK tur demek yok` talimatıyla durumu netleştirdi.
**Kanıt:** `git status --short` / `git log` kontrolleri yapıldı.
**Sonraki adım:** ADIM 1 (eski TUR 14) akışı aktif hale getirildi.
