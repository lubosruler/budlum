# Budlum Bug Bounty Programı — ADIM2 Kararı (2.4=C)

**Durum:** Taslak (ADIM2)  
**Geçerlilik:** Mainnet v1 lansmanından itibaren  
**Yöneten:** ARENA1 / AI Birliği koordinasyonu  

---

## 1. Kapsam (In Scope)

| Bileşen | Kategori | Öncelik |
|---------|----------|---------|
| `src/consensus/` | Consensus bypass, double-spend, fork manipulation | Kritik |
| `src/chain/finality.rs` | BLS aggregate signature forgery, quorum bypass | Kritik |
| `src/crypto/pkcs11.rs` | HSM key extraction, mock→production leakage | Kritik |
| `src/cross_domain/bridge.rs` | Replay mint, lock bypass, asset drain | Kritik |
| `src/network/node.rs` | P2P DoS, eclipse attack, poisoned snapshot | Yüksek |
| `src/rpc/server.rs` | RPC auth bypass, rate limit bypass, spray OOM | Yüksek |
| `src/domain/storage_deal.rs` | Storage deal manipulation, challenge bypass | Yüksek |
| `budzero/bud-proof/` | ZK proof soundness break, invalid proof acceptance | Kritik |
| `budzero/bud-vm/` | VM escape, opcode abuse, gas manipulation | Yüksek |

## 2. Dışarıda (Out of Scope)

- Social engineering, phishing, fiziksel saldırılar
- Daha önce `STATUS_ONLINE.md`'de raporlanmış ve işlem görmüş bulgular
- Third-party dependency CVE'leri (aşağıdaki "Dependency Policy"ye bakın)
- Testnet / devnet-only konfigürasyonlar (mainnet spesifik olmalı)

## 3. Ödül Seviyeleri (Tahmini)

| Seviye | Örnek Bulgu | Ödül (USD) |
|--------|-------------|------------|
| **Kritik** | Consensus bypass, key extraction, bridge asset drain, ZK soundness break | $50,000–$100,000 |
| **Yüksek** | DoS with permanent state corruption, RPC auth bypass, P2P eclipse | $10,000–$25,000 |
| **Orta** | Rate limit bypass, information leak, economic manipulation | $2,500–$5,000 |
| **Düşük** | Best practice violation, documentation inconsistency | $500–$1,000 |

> **Not:** Ödüller ADIM4'te bug bounty platformu (immunefi.com, Q6 kararı: bug_bounty) ile entegre edildiğinde kesinleşecek. ADIM3 10-soru anket Q6: bug_bounty seçildi (harici firma değil, immunefi tipi platform).

**Güncelleme (ADIM4 Q6):** Immunefi entegrasyonu için `https://immunefi.com/bounty/budlum` taslak başvurusu hazırlanacak; kritik bulgular için $50k-$100k aralığı korunuyor. Mainnet self-audited olduğu için immunefi "medium" tier ile başlanacak, audit sonrası "high" tier'e yükseltilecek.

## 4. Raporlama Süreci

1. **Başvuru:** `security@budlum.network` (PGP key: `0xBUDLUM-SECURITY` — ADIM3'te yayınlanacak)
2. **Triage:** 72 saat içinde ilk yanıt
3. **Değerlendirme:** 14 gün içinde ön değerlendirme
4. **Düzeltme:** Kritik bulgular 30 gün içinde patch
5. **Açıklama:** Araştırmacı onayıyla coordinated disclosure (90 gün)

## 5. Kurallar

- Mainnet üzerinde test yapılmaz; testnet kopyası kullanılır.
- Veri kaybına veya servis kesintisine neden olunmaz.
- Bulgu raporlanmadan önce üçüncü tarafla paylaşılmaz.
- `AI_BIRLIGI.md` §6.1 (force-push yasağı) geçerlidir.

## 6. Dependency Policy

`cargo audit` tarafından tespit edilen CVE'ler (örneğin `protobuf`, `pqcrypto-*`, `ring`) bu program kapsamında **değerlendirilmez** — bunlar ayrı bir dependency upgrade ADIM'inde ele alınır. Ancak, bir dependency CVE'sinin Budlum spesifik bir exploit chain'ine dönüştürülebildiği kanıtlanırsa, "Kritik" seviyede değerlendirilebilir.

---

**Sonraki adım:** ADIM3'te immunefi.com veya benzeri bir platforma kayıt + PGP key yayınlama.
