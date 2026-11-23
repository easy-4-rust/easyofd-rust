#!/usr/bin/env python3
"""独立以Python标准库按RFC7292生成公开PFX的MAC验收变体；不生成生产密钥。"""
import hashlib
import hmac
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / 'tests/fixtures/bc185/rust-public-test.p12'
OUTPUT = ROOT / 'tests/fixtures/pkcs12_integrity'

def tlv(tag, data):
    length = len(data)
    if length < 128:
        size = bytes([length])
    else:
        size_bytes = length.to_bytes((length.bit_length() + 7) // 8, 'big')
        size = bytes([128 + len(size_bytes)]) + size_bytes
    return bytes([tag]) + size + data

def fields(data):
    items = []
    pos = 0
    while pos < len(data):
        start = pos
        tag, size = data[pos:pos + 2]
        pos += 2
        if size >= 128:
            count = size & 127
            size = int.from_bytes(data[pos:pos + count], 'big')
            pos += count
        value = data[pos:pos + size]
        pos += size
        items.append((tag, value, data[start:pos]))
    return items

if __name__ == '__main__':
    OUTPUT.mkdir(parents=True, exist_ok=True)
    _, body, _ = fields(SOURCE.read_bytes())[0]
    version, content, old_mac = fields(body)
    authsafe = fields(fields(content[1])[1][1])[0][1]
    prefix = version[2] + content[2]
    (OUTPUT / 'no_mac.p12').write_bytes(tlv(48, prefix))
    salt = b'public-test-salt!'
    password = 'public-test'.encode('utf-16-be') + b'\0\0'
    identifiers = {'sha1': bytes.fromhex('06052b0e03021a')}
    for name, suffix in [('sha224', 4), ('sha256', 1), ('sha384', 2), ('sha512', 3), ('sha512_224', 5), ('sha512_256', 6)]:
        identifiers[name] = bytes.fromhex('06096086480165030402') + bytes([suffix])
    for name, oid in identifiers.items():
        digest = hashlib.new(name)
        block = digest.block_size
        seed = bytes([3]) * block
        for data in [salt, password]:
            count = ((len(data) + block - 1) // block) * block
            seed += (data * ((count + len(data) - 1) // len(data)))[:count]
        key = hashlib.new(name, seed).digest()
        mac = hmac.new(key, authsafe, name).digest()
        info = tlv(48, tlv(48, oid + b'\x05\x00') + tlv(4, mac))
        base = info + tlv(4, salt)
        (OUTPUT / (name + '.p12')).write_bytes(tlv(48, prefix + tlv(48, base)))  # 默认迭代1
        if name == 'sha1':
            for label, value in [('negative', b'\xff'), ('zero', b'\x00'), ('over_budget', (2_000_001).to_bytes(3, 'big'))]:
                (OUTPUT / (label + '.p12')).write_bytes(tlv(48, prefix + tlv(48, base + tlv(2, value))))
    import json
    manifest = {'source': str(SOURCE.relative_to(ROOT)), 'source_sha256': hashlib.sha256(SOURCE.read_bytes()).hexdigest(), 'generator': 'tools/upstream/pkcs12_mac_variants.py, Python hashlib/hmac, RFC7292 B.2 id=3', 'password': 'public-test', 'public_test_identity_only': True, 'files': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUTPUT.glob('*.p12'))}}
    (OUTPUT / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    # 合法MAC不代表密钥算法/标量合法；构造可独立验收的认证过坏密钥。
    sm2_oid = bytes.fromhex('06082a811ccf5501822d')
    p256_oid = bytes.fromhex('06082a8648ce3d030107')
    marker = bytes.fromhex('30250201010420')
    assert sm2_oid in authsafe and marker in authsafe
    offset = authsafe.index(marker) + len(marker)
    changes = {'wrong_curve': authsafe.replace(sm2_oid, p256_oid), 'zero_scalar': authsafe[:offset] + bytes(32) + authsafe[offset + 32:], 'invalid_scalar': authsafe[:offset] + bytes([255])*32 + authsafe[offset + 32:]}
    for label, data in changes.items():
        new_content = tlv(48, fields(content[1])[0][2] + tlv(160, tlv(4, data)))
        key = hashlib.sha1(bytes([3])*64 + (salt*4)[:64] + (password*4)[:64]).digest()
        digest = hmac.new(key, data, 'sha1').digest()
        info = tlv(48, tlv(48, identifiers['sha1'] + b'\x05\x00') + tlv(4, digest))
        encoded = tlv(48, version[2] + new_content + tlv(48, info + tlv(4, salt)))
        (OUTPUT / (label + '.p12')).write_bytes(encoded)
    manifest['files'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUTPUT.glob('*.p12'))}
    (OUTPUT / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    # 有效MAC包装的恶意PBKDF2 keyLength；无需实际派生巨量密钥。
    data_oid = bytes.fromhex('06092a864886f70d010701')
    pbes2_oid = bytes.fromhex('06092a864886f70d01050d')
    pbkdf2_oid = bytes.fromhex('06092a864886f70d01050c')
    aes_oid = bytes.fromhex('0609608648016503040102')
    shrouded_oid = bytes.fromhex('060b2a864886f70d010c0a0102')
    for label, length_bytes in [('negative', b'\xff'), ('zero', b'\0'), ('wrong_size', b'\x20'), ('huge', bytes.fromhex('00ffffffff'))]:
        kdf = tlv(48, pbkdf2_oid + tlv(48, tlv(4, b'salt') + tlv(2, b'\1') + tlv(2, length_bytes)))
        cipher = tlv(48, aes_oid + tlv(4, bytes(16)))
        encrypted = tlv(48, tlv(48, pbes2_oid + tlv(48, kdf + cipher)) + tlv(4, bytes(16)))
        bag = tlv(48, shrouded_oid + tlv(160, encrypted))
        content_info = tlv(48, data_oid + tlv(160, tlv(4, tlv(48, bag))))
        data = tlv(48, content_info)
        key = hashlib.sha1(bytes([3])*64 + (salt*4)[:64] + (password*4)[:64]).digest()
        digest = hmac.new(key, data, 'sha1').digest()
        info = tlv(48, tlv(48, identifiers['sha1'] + b'\x05\x00') + tlv(4, digest))
        auth_content = tlv(48, data_oid + tlv(160, tlv(4, data)))
        (OUTPUT / ('kdf_'+label+'.p12')).write_bytes(tlv(48, version[2]+auth_content+tlv(48, info+tlv(4,salt))))
    manifest['files'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(OUTPUT.glob('*.p12'))}
    (OUTPUT / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
