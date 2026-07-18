//! RLP (Recursive Length Prefix) — Ethereum Yellow Paper Appendix B.
//!
//! **Kaynak:** Ethereum resmi RLP spec'inden birebir (yeni kriptografi icat
//! EDİLMİYOR — kamuya açık, iyi-spec'li algoritma). In-tree impl (RFC Q3).
//!
//! **Verifier bağlamı (F10):** decode **strict canonical**'dır — non-canonical
//! encoding (örn. `0x81 0x00` `0x00` yerine, veya `0xb8 0x00...` uzunluk < 56)
//! `RlpError::NonCanonical` ile RED. Bu, encoding malleability saldırılarını
//! (aynı semantik veri → farklı RLP bytes → farklı keccak hash) kapatır.
//! Ethereum node'ları da strict'tir; relayer non-canonical proof gönderemez.
//!
//! # KAT (Known-Answer Tests)
//! `tests` modülü Ethereum well-known vector'larını içerir ( Appendix B örnekleri):
//! - `""` → `0x80`
//! - `"dog"` → `0x83646f67`
//! - `[]` → `0xc0`
//! - `["cat","dog"]` → `0xc8...`
//! - `0x0400` → `0x820400`

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RlpItem {
    /// Byte-string (Ethereum "string"). Boş = `vec![]`.
    Bytes(Vec<u8>),
    /// List of nested items.
    List(Vec<RlpItem>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RlpError {
    /// Byte-akışı bitti (beklenen byte yok).
    UnexpectedEnd,
    /// Trailing bytes (decode sonrası tüketilmemiş byte var — tek Item değil).
    TrailingBytes,
    /// Non-canonical encoding (malleability kapatma — RED).
    NonCanonical,
    /// Uzunluk alanı sıfır (geçersiz).
    ZeroLength,
}

impl std::fmt::Display for RlpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RlpError::UnexpectedEnd => write!(f, "rlp: unexpected end of input"),
            RlpError::TrailingBytes => write!(f, "rlp: trailing bytes after single item"),
            RlpError::NonCanonical => write!(f, "rlp: non-canonical encoding"),
            RlpError::ZeroLength => write!(f, "rlp: zero length-of-length"),
        }
    }
}

impl std::error::Error for RlpError {}

// ---------------------------------------------------------------------------
// ENCODE
// ---------------------------------------------------------------------------

/// RLP-encode an item. Çıktı her zaman canonical (deterministik).
pub fn encode(item: &RlpItem) -> Vec<u8> {
    match item {
        RlpItem::Bytes(b) => encode_bytes(b),
        RlpItem::List(items) => {
            let mut payload = Vec::new();
            for it in items {
                payload.extend(encode(it));
            }
            encode_length(payload.len(), 0xc0)
                .into_iter()
                .chain(payload)
                .collect()
        }
    }
}

fn encode_bytes(b: &[u8]) -> Vec<u8> {
    // Single byte in [0x00, 0x7f] → itself.
    if b.len() == 1 && b[0] <= 0x7f {
        return vec![b[0]];
    }
    // Empty or multi-byte → length-prefixed.
    let mut out = encode_length(b.len(), 0x80);
    out.extend_from_slice(b);
    out
}

/// Length-prefix: `offset + len` for short (<56), `offset+55+len_of_len` for long.
fn encode_length(length: usize, offset: u8) -> Vec<u8> {
    if length < 56 {
        vec![offset + length as u8]
    } else {
        let len_bytes = to_minimal_big_endian(length);
        let mut out = Vec::with_capacity(1 + len_bytes.len());
        out.push(offset + 55 + len_bytes.len() as u8);
        out.extend(len_bytes);
        out
    }
}

/// Minimal big-endian (leading-zero stripped). `0` → `[]` (burada length>55
/// bağlamında çağrılır, yani length ≥ 56 → ≥1 byte).
fn to_minimal_big_endian(n: usize) -> Vec<u8> {
    let mut v = Vec::new();
    let mut n = n;
    while n > 0 {
        v.push((n & 0xff) as u8);
        n >>= 8;
    }
    v.reverse();
    v
}

// ---------------------------------------------------------------------------
// DECODE (strict canonical)
// ---------------------------------------------------------------------------

/// Decode exactly one item; trailing bytes → `TrailingBytes` (strict).
pub fn decode(bytes: &[u8]) -> Result<RlpItem, RlpError> {
    let (item, consumed) = decode_item(bytes)?;
    if consumed != bytes.len() {
        return Err(RlpError::TrailingBytes);
    }
    Ok(item)
}

/// Decode one item starting at byte 0; returns (item, bytes_consumed).
fn decode_item(bytes: &[u8]) -> Result<(RlpItem, usize), RlpError> {
    if bytes.is_empty() {
        return Err(RlpError::UnexpectedEnd);
    }
    let first = bytes[0];

    if first <= 0x7f {
        // Single byte [0x00, 0x7f] → itself.
        Ok((RlpItem::Bytes(vec![first]), 1))
    } else if first <= 0xb7 {
        // Short string: [0x80, 0xb7] → length 0..55.
        let len = (first - 0x80) as usize;
        let start: usize = 1;
        let end = start.checked_add(len).ok_or(RlpError::UnexpectedEnd)?;
        if bytes.len() < end {
            return Err(RlpError::UnexpectedEnd);
        }
        let data = &bytes[start..end];
        // Canonical: single byte in [0x00,0x7f] must use the single-byte form,
        // not 0x81 0xNN. Yani `len==1 && data[0]<=0x7f` non-canonical.
        if len == 1 && data[0] <= 0x7f {
            return Err(RlpError::NonCanonical);
        }
        Ok((RlpItem::Bytes(data.to_vec()), end))
    } else if first <= 0xbf {
        // Long string: [0xb8, 0xbf] → length = big-endian of (first-0xb7) bytes.
        let len_of_len = (first - 0xb7) as usize;
        if len_of_len == 0 {
            return Err(RlpError::ZeroLength);
        }
        let lh_start: usize = 1;
        let lh_end = lh_start
            .checked_add(len_of_len)
            .ok_or(RlpError::UnexpectedEnd)?;
        if bytes.len() < lh_end {
            return Err(RlpError::UnexpectedEnd);
        }
        let len = read_big_endian(&bytes[lh_start..lh_end])?;
        // Canonical: long form yalnızca length >= 56 için. len_of_len minimal
        // olmalı (leading zero yok). Daha kısa (len < 56) → short form kullanılmalıydı.
        if len < 56 {
            return Err(RlpError::NonCanonical);
        }
        if has_leading_zero(&bytes[lh_start..lh_end]) {
            return Err(RlpError::NonCanonical);
        }
        let data_start = lh_end;
        let data_end = data_start.checked_add(len).ok_or(RlpError::UnexpectedEnd)?;
        if bytes.len() < data_end {
            return Err(RlpError::UnexpectedEnd);
        }
        Ok((
            RlpItem::Bytes(bytes[data_start..data_end].to_vec()),
            data_end,
        ))
    } else if first <= 0xf7 {
        // Short list: [0xc0, 0xf7] → payload 0..55 bytes.
        let len = (first - 0xc0) as usize;
        let payload_start: usize = 1;
        let payload_end = payload_start
            .checked_add(len)
            .ok_or(RlpError::UnexpectedEnd)?;
        if bytes.len() < payload_end {
            return Err(RlpError::UnexpectedEnd);
        }
        let (items, _consumed) = decode_list_items(&bytes[payload_start..payload_end], len)?;
        Ok((RlpItem::List(items), payload_end))
    } else {
        // Long list: [0xf8, 0xff].
        let len_of_len = (first - 0xf7) as usize;
        if len_of_len == 0 {
            return Err(RlpError::ZeroLength);
        }
        let lh_start: usize = 1;
        let lh_end = lh_start
            .checked_add(len_of_len)
            .ok_or(RlpError::UnexpectedEnd)?;
        if bytes.len() < lh_end {
            return Err(RlpError::UnexpectedEnd);
        }
        let len = read_big_endian(&bytes[lh_start..lh_end])?;
        if len < 56 {
            return Err(RlpError::NonCanonical);
        }
        if has_leading_zero(&bytes[lh_start..lh_end]) {
            return Err(RlpError::NonCanonical);
        }
        let payload_start = lh_end;
        let payload_end = payload_start
            .checked_add(len)
            .ok_or(RlpError::UnexpectedEnd)?;
        if bytes.len() < payload_end {
            return Err(RlpError::UnexpectedEnd);
        }
        let (items, _consumed) = decode_list_items(&bytes[payload_start..payload_end], len)?;
        Ok((RlpItem::List(items), payload_end))
    }
}

/// Decode all items within a payload slice of exactly `expected_len` bytes.
/// Trailing bytes within payload → TrailingBytes (strict list parsing).
fn decode_list_items(
    payload: &[u8],
    _expected_len: usize,
) -> Result<(Vec<RlpItem>, usize), RlpError> {
    let mut items = Vec::new();
    let mut pos = 0;
    while pos < payload.len() {
        let (item, consumed) = decode_item(&payload[pos..])?;
        pos += consumed;
        items.push(item);
    }
    Ok((items, pos))
}

fn read_big_endian(b: &[u8]) -> Result<usize, RlpError> {
    // Ethereum length-of-length pratikte küçük (≤8 byte); usize aşımı
    // malformed input → UnexpectedEnd (generic).
    let mut len = 0usize;
    for &byte in b {
        len = len
            .checked_mul(256)
            .and_then(|l| l.checked_add(byte as usize))
            .ok_or(RlpError::UnexpectedEnd)?;
    }
    Ok(len)
}

fn has_leading_zero(b: &[u8]) -> bool {
    b.len() > 1 && b[0] == 0x00
}

// ---------------------------------------------------------------------------
// TESTS (KAT — Ethereum well-known vectors + canonical negatif matris)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(hex_str: &str) -> Vec<u8> {
        hex::decode(hex_str).unwrap()
    }

    fn b(hex_str: &str) -> RlpItem {
        RlpItem::Bytes(hex::decode(hex_str).unwrap())
    }

    // ---- KAT: encode ----

    #[test]
    fn kat_encode_empty_string() {
        assert_eq!(encode(&RlpItem::Bytes(vec![])), vec![0x80]);
    }

    #[test]
    fn kat_encode_dog() {
        assert_eq!(encode(&b("646f67")), bytes("83646f67")); // "dog"
    }

    #[test]
    fn kat_encode_empty_list() {
        assert_eq!(encode(&RlpItem::List(vec![])), vec![0xc0]);
    }

    #[test]
    fn kat_encode_list_cat_dog() {
        let item = RlpItem::List(vec![b("636174"), b("646f67")]); // ["cat","dog"]
        assert_eq!(encode(&item), bytes("c88363617483646f67"));
    }

    #[test]
    fn kat_encode_single_byte_low() {
        assert_eq!(encode(&b("00")), vec![0x00]); // 0x00 → 0x00
        assert_eq!(encode(&b("0f")), vec![0x0f]); // 0x0f → 0x0f
        assert_eq!(encode(&b("7f")), vec![0x7f]); // 0x7f → 0x7f (boundary)
    }

    #[test]
    fn kat_encode_single_byte_high() {
        assert_eq!(encode(&b("80")), vec![0x81, 0x80]); // 0x80 → 0x81 0x80
    }

    #[test]
    fn kat_encode_0x0400() {
        assert_eq!(encode(&b("0400")), vec![0x82, 0x04, 0x00]);
    }

    #[test]
    fn kat_encode_long_string() {
        // 56-byte string (long form threshold). "Lorem ipsum dolor sit amet,
        // consectetur adipisicing elit" = 56 chars.
        let s = b"Lorem ipsum dolor sit amet, consectetur adipisicing elit";
        let enc = encode(&RlpItem::Bytes(s.to_vec()));
        assert_eq!(enc[0], 0xb8); // long string marker
        assert_eq!(enc[1], 56); // length-of-payload (1 byte, =56)
        assert_eq!(enc.len(), 2 + 56);
    }

    // ---- KAT: decode round-trip ----

    #[test]
    fn kat_decode_roundtrip_empty_string() {
        assert_eq!(decode(&[0x80]).unwrap(), RlpItem::Bytes(vec![]));
    }

    #[test]
    fn kat_decode_roundtrip_dog() {
        assert_eq!(decode(&bytes("83646f67")).unwrap(), b("646f67"));
    }

    #[test]
    fn kat_decode_roundtrip_empty_list() {
        assert_eq!(decode(&[0xc0]).unwrap(), RlpItem::List(vec![]));
    }

    #[test]
    fn kat_decode_roundtrip_list_cat_dog() {
        let item = RlpItem::List(vec![b("636174"), b("646f67")]);
        assert_eq!(decode(&bytes("c88363617483646f67")).unwrap(), item);
    }

    #[test]
    fn kat_decode_roundtrip_single_bytes() {
        assert_eq!(decode(&[0x00]).unwrap(), b("00"));
        assert_eq!(decode(&[0x0f]).unwrap(), b("0f"));
        assert_eq!(decode(&[0x7f]).unwrap(), b("7f"));
        assert_eq!(decode(&[0x81, 0x80]).unwrap(), b("80"));
    }

    #[test]
    fn kat_decode_roundtrip_long_string() {
        let s = b"Lorem ipsum dolor sit amet, consectetur adipisicing elit".to_vec();
        let item = RlpItem::Bytes(s);
        let enc = encode(&item);
        assert_eq!(decode(&enc).unwrap(), item);
    }

    #[test]
    fn kat_deeply_nested_list() {
        // [[], [[]], [[]]] — set theory rep
        let item = RlpItem::List(vec![
            RlpItem::List(vec![]),
            RlpItem::List(vec![RlpItem::List(vec![])]),
            RlpItem::List(vec![RlpItem::List(vec![])]),
        ]);
        let enc = encode(&item);
        assert_eq!(decode(&enc).unwrap(), item);
    }

    // ---- canonical (negatif matris — verifier güvenliği) ----

    #[test]
    fn canonical_rejects_non_canonical_single_byte() {
        // 0x00 must be 0x00, NOT 0x81 0x00.
        assert_eq!(decode(&[0x81, 0x00]).unwrap_err(), RlpError::NonCanonical);
        // 0x0f must be 0x0f, NOT 0x81 0x0f.
        assert_eq!(decode(&[0x81, 0x0f]).unwrap_err(), RlpError::NonCanonical);
    }

    #[test]
    fn canonical_rejects_long_form_short_payload() {
        // length 1 via long form (0xb8 prefix, len_of_len=1, len=0x01) → NonCanonical.
        assert_eq!(
            decode(&[0xb8, 0x01, 0xaa]).unwrap_err(),
            RlpError::NonCanonical
        );
    }

    #[test]
    fn canonical_rejects_leading_zero_length() {
        // long form length with leading zero: 0xb9 0x00 0x00 0x38 (=56 via 3 len bytes)
        assert_eq!(
            decode(&[0xb9, 0x00, 0x00, 0x38]).unwrap_err(),
            RlpError::NonCanonical
        );
    }

    // ---- structural errors ----

    #[test]
    fn decode_rejects_trailing_bytes() {
        // two items concatenated: 0x80 0x80 → TrailingBytes (single-item decode).
        assert_eq!(decode(&[0x80, 0x80]).unwrap_err(), RlpError::TrailingBytes);
    }

    #[test]
    fn decode_rejects_unexpected_end_short_string() {
        // claims length 3 but only 2 bytes follow
        assert_eq!(
            decode(&[0x83, 0xaa, 0xbb]).unwrap_err(),
            RlpError::UnexpectedEnd
        );
    }

    #[test]
    fn decode_rejects_unexpected_end_long_string() {
        // 0xb8 0x0a claims 10-byte payload, but empty
        assert_eq!(decode(&[0xb8, 0x0a]).unwrap_err(), RlpError::UnexpectedEnd);
    }

    #[test]
    fn decode_rejects_empty_input() {
        assert_eq!(decode(&[]).unwrap_err(), RlpError::UnexpectedEnd);
    }

    // ---- fuzz-like: random bytes don't panic ----

    #[test]
    fn fuzz_random_bytes_no_panic() {
        // Deterministik pseudo-rastgele; amaç: decode hiçbir girdide panic ETMESİN.
        let mut seed: u64 = 0x1234_5678_9abc_def0;
        let mut next = || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as u8
        };
        for _ in 0..5000 {
            let n = (next() as usize % 8) + 1;
            let buf: Vec<u8> = (0..n).map(|_| next()).collect();
            // Sonuç umursanmaz — yalnızca panic yokluğu test edilir.
            let _ = decode(&buf);
        }
    }
}
