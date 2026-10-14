# PKCS#12完整性与资源预算验收样本
仅公开测试身份。由Python hashlib/hmac独立实现RFC7292 B.2 id=3派生，输入为Rust公开测试PFX；生成脚本为tools/upstream/pkcs12_mac_variants.py。
七个SHA1/SHA2 MAC算法样本、无MAC、无效迭代、合法MAC包装的错误曲线/标量与恶意keyLength反例。manifest包含每个文件SHA256。
密钥未加密，仅供测试，禁止生产使用。MAC有效不代表证书可信，也不代表该密钥格式/参数应被导入。
