# Budlum101 — İç Teknik Referans ve Karar Ekleri

**Yayın modeli:** Public Budlum101 kitabına eşlik eden internal teknik ek.
Bu belge kaynak/commit/CI kanıtları, migration ayrıntıları, threat model,
operasyon kararları ve açık teknik borçlar için kullanılır.

## Kapsam
- Kaynak ve CI kanıt indeksi
- Protocol/state ownership matrisi
- Workspace extraction karar kayıtları
- Transaction V4 canonical signing manifesti
- Snapshot schema-4 trust/migration ayrıntıları
- Bridge/EVM negatif test matrisi
- V19 durability fail-stop tasarımı
- HSM, ceremony, audit ve launch gate kayıtları

## Karar matrisi — imzalama ve admission

| Karar | Teknik sonuç | Kanıt ihtiyacı |
|---|---|---|
| V4 canonical signing | Type payload alanları imza preimage’ine explicit girer | Her payload variant için mutation reject testi |
| Legacy non-genesis red | Eski imzalar admission’da kabul edilmez | RPC/mempool/block admission regresyonları |
| Explicit signature version | Wire/proto/serde transaction sürümü görünür olur | Transport roundtrip + historical policy testi |

## Snapshot schema-4 denetim matrisi

- Digest field manifesti isim bazlı pinlenmelidir.
- BNS ve B.U.D. `StorageRegistry` type ownershipi crate cutover sonrası digest
  kapsamından düşmemelidir.
- Legacy schema yükleme, eski digest doğrulanmadan schema-4’e sessiz yükseltme
  yapmamalıdır.
- Trust policy snapshot’ın saldırgan tarafından değiştirilebilir alanı değil,
  dışarıdan güvenilen loader/config policy olmalıdır.

## V19 durability kararları

Production critical persistence başarısızlığı fail-stop davranışına gider.
Bridge transfer/replay, verified QC ve finality certificate/canonical height
birbirinden bağımsız “logla devam et” yazıları değildir. Staged state → atomic
batch → flush → memory publish sırası, test failure injection ile kanıtlanır.

## Workspace extraction kanıtları

| Katman | Canonical sahiplik | Core rolü |
|---|---|---|
| `budlum-primitives` | Address + canonical hash primitives | path dependency + re-export transition |
| `B.U.D/` | ContentId, manifest, params, deal/challenge registry | executor/RPC/snapshot adapter |
| `BNS/` | Name registry, record, resolution | account/snapshot/chain actor adapter |
| `budlum-core` | consensus, chain, execution, network, persistence | orchestration ve integration |

## CI güvenilirlik ilkesi

Bir green job yalnız o jobun çalıştığını gösterir. Mainnet kabulü için:

1. testin doğru production path’i çağırdığı,
2. negatif mutation’ın gerçekten rejection ürettiği,
3. gate scriptinin test silinince fail ettiği,
4. dependency/lockfile/workspace sınırlarının derlendiği,
5. CI run linki ve commit SHA’nın kaydedildiği

ayrı ayrı doğrulanmalıdır.

## Kaynak kanıt sınıflandırması

| Kaynak türü | Kitaptaki kullanım | Tek başına yeterli mi? |
|---|---|---|
| Constitution / RFC | hedef, normatif karar, kabul edilen tasarım | Hayır; kod/test/operasyonla bağlanır |
| README / module README | modül sınırı ve kullanıcı iletişimi | Hayır; stale olasılığı vardır |
| Rust source | uygulanmış davranış ve type ownership | Hayır; test/CI olmadan regression olabilir |
| Unit/integration test | belirli invariant veya lifecycle | Hayır; production path eşdeğerliği denetlenir |
| CI run | belirli SHA’da job sonucu | Hayır; test mantığını tek başına kanıtlamaz |
| Runbook / ceremony | operasyonel süreç | Hayır; gerçek rehearsal/evidence gerekir |

## Budlum101 yazım kontrol listesi

- Her “uygulanmıştır” cümlesi source/test/CI kanıt sınıfı ile eşleştirilir.
- Her “hedeflenir” cümlesi RFC/Constitution/plan kaynağına bağlanır.
- Production/mainnet/audit iddiaları ayrı kanıt kapısı olmadan kullanılmaz.
- Teknik olmayan kutu, teknik iddiayı basitleştirir ama genişletmez.
- B.U.D. interim retrieval ve VerifyMerkle production gate farkı her ilgili
  bölümde korunur.
- PoA permissioned kuralının permissionless domainlere sızmadığı açıkça yazılır.
