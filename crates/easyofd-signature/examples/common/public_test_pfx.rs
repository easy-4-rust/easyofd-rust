//! 仅验收工具使用的公开测试PFX：无私钥加密，具有口令MAC，禁止生产使用。
use easyofd_gm::ses::{encode_length, encode_octet_string, encode_oid, encode_sequence};
use hmac::{Hmac, Mac};
use sha1::{Digest, Sha1};
fn seq(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    encode_sequence(data, &mut out);
    out
}
fn tlv(tag: u8, data: &[u8]) -> Vec<u8> {
    let mut out = vec![tag];
    encode_length(data.len(), &mut out);
    out.extend_from_slice(data);
    out
}
fn oid(parts: &[u32]) -> Vec<u8> {
    let mut out = Vec::new();
    encode_oid(parts, &mut out);
    out
}
fn octet(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    encode_octet_string(data, &mut out);
    out
}
fn content(data: &[u8]) -> Vec<u8> {
    let mut v = oid(&[1, 2, 840, 113_549, 1, 7, 1]);
    v.extend(tlv(0xa0, &octet(data)));
    seq(&v)
}
/// 使用公开测试标量与证书产生PFX，用于Java反向加载，不提供生产密钥导出API。
pub fn encode(secret: &[u8], cert: &[u8], password: &str) -> Vec<u8> {
    let mut ec = vec![2, 1, 1];
    ec.extend(octet(secret));
    let ec = seq(&ec);
    let mut algorithm = oid(&[1, 2, 840, 10_045, 2, 1]);
    algorithm.extend(oid(&[1, 2, 156, 10_197, 1, 301]));
    let mut info = vec![2, 1, 0];
    info.extend(seq(&algorithm));
    info.extend(octet(&ec));
    let info = seq(&info);
    let friendly: Vec<u8> = "public-test"
        .encode_utf16()
        .flat_map(u16::to_be_bytes)
        .collect();
    let mut attr = oid(&[1, 2, 840, 113_549, 1, 9, 20]);
    attr.extend(tlv(0x31, &tlv(0x1e, &friendly)));
    let friendly_attr = seq(&attr);
    let mut local = oid(&[1, 2, 840, 113_549, 1, 9, 21]);
    local.extend(tlv(0x31, &octet(b"public-test-key")));
    let local_attr = seq(&local);
    let mut members = [friendly_attr, local_attr];
    members.sort();
    let attrs = members.concat();
    // DER SET OF必须按编码顺序排序，两个属性的长度决定顺序。
    let attrs = tlv(0x31, &attrs);
    let mut keybag = oid(&[1, 2, 840, 113_549, 1, 12, 10, 1, 1]);
    keybag.extend(tlv(0xa0, &info));
    keybag.extend(&attrs);
    let mut cb = oid(&[1, 2, 840, 113_549, 1, 9, 22, 1]);
    cb.extend(tlv(0xa0, &octet(cert)));
    let mut certbag = oid(&[1, 2, 840, 113_549, 1, 12, 10, 1, 3]);
    certbag.extend(tlv(0xa0, &seq(&cb)));
    certbag.extend(attrs);
    let mut bags = seq(&keybag);
    bags.extend(seq(&certbag));
    let safe = seq(&content(&seq(&bags)));
    // RFC7292 SHA1 MAC KDF，固定公开测试盐与迭代1，仅需一个输出块。
    let salt = b"public-test-salt!";
    let password: Vec<u8> = password
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_be_bytes)
        .collect();
    let mut seed = vec![3; 64];
    seed.extend(salt.iter().cycle().take(64));
    seed.extend(password.iter().cycle().take(64));
    let key = Sha1::digest(&seed);
    let mut mac = Hmac::<Sha1>::new_from_slice(&key).expect("SHA1 MAC密钥长度合法");
    mac.update(&safe);
    let digest = mac.finalize().into_bytes();
    let mut digest_alg = oid(&[1, 3, 14, 3, 2, 26]);
    digest_alg.extend([5, 0]);
    let mut digest_info = seq(&digest_alg);
    digest_info.extend(octet(&digest));
    let mut macdata = seq(&digest_info);
    macdata.extend(octet(salt));
    macdata.extend([2, 1, 1]);
    let mut pfx = vec![2, 1, 3];
    pfx.extend(content(&safe));
    pfx.extend(seq(&macdata));
    seq(&pfx)
}
