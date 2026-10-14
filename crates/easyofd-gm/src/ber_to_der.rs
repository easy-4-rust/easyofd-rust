//! PKCS#12的有界BER定长化，支持BC的分段OCTET STRING。
/// 将单个BER对象转为定长编码，拒绝截断、非构造不定长和过深嵌套。
pub(crate) fn normalize(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() > 32 * 1024 * 1024 {
        return None;
    }
    let (out, end) = item(data, 0, 0)?;
    if end != data.len() {
        return None;
    }
    Some(out)
}
fn length(out: &mut Vec<u8>, n: usize) {
    if n < 128 {
        out.push(u8::try_from(n).expect("已限定短长度"));
    } else {
        let bytes = n.to_be_bytes();
        let first = bytes
            .iter()
            .position(|v| *v != 0)
            .unwrap_or(bytes.len() - 1);
        out.push(0x80 | u8::try_from(bytes.len() - first).expect("usize长度不超过8字节"));
        out.extend_from_slice(&bytes[first..]);
    }
}
fn item(data: &[u8], start: usize, depth: usize) -> Option<(Vec<u8>, usize)> {
    if depth > 32 {
        return None;
    }
    let tag = *data.get(start)?;
    if tag & 31 == 31 || tag == 0 {
        return None;
    }
    let l = *data.get(start + 1)?;
    let mut pos = start + 2;
    let indefinite = l == 128;
    let limit = if indefinite {
        if tag & 32 == 0 {
            return None;
        }
        None
    } else if l < 128 {
        Some(pos.checked_add(l as usize)?)
    } else {
        let n = (l & 127) as usize;
        if n == 0 || n > std::mem::size_of::<usize>() {
            return None;
        }
        let mut len = 0usize;
        for b in data.get(pos..pos + n)? {
            len = len.checked_mul(256)?.checked_add(*b as usize)?;
        }
        pos += n;
        Some(pos.checked_add(len)?)
    };
    if limit.is_some_and(|end| end > data.len()) {
        return None;
    }
    let mut content = Vec::new();
    if tag & 32 != 0 {
        loop {
            if indefinite && data.get(pos..pos + 2) == Some(&[0, 0]) {
                pos += 2;
                break;
            }
            if let Some(end) = limit {
                if pos == end {
                    break;
                }
                if pos > end {
                    return None;
                }
            }
            let (child, next) = item(data, pos, depth + 1)?;
            if tag == 0x24 {
                if child[0] != 4 {
                    return None;
                }
                let offset = if child[1] < 128 {
                    2
                } else {
                    2 + (child[1] & 127) as usize
                };
                content.extend_from_slice(child.get(offset..)?);
            } else {
                content.extend_from_slice(&child);
            }
            pos = next;
        }
    } else {
        let end = limit?;
        content.extend_from_slice(data.get(pos..end)?);
        pos = end;
    }
    let mut out = vec![if tag == 0x24 { 4 } else { tag }];
    length(&mut out, content.len());
    out.extend(content);
    Some((out, pos))
}
