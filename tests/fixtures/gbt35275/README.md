# GB35275 / CMS 公开互操作样本

由固定OFDRW提交84e9eb260ba6fd351ecccf08a01a40305d4af635及BC1.85生成。使用bc185目录的公开测试身份，签名原文为该目录message.bin。生成命令见tools/upstream/Gbt35275Interop.java；manifest记录哈希。Rust测试验证真实签名、篡改原文、坏签名、错误证书pin和GB→CMS重包装拒绝。

仅验收Java→Rust读取验证；旧Rust GB writer编码保留兼容，未声明反向Java互操作。未验收CA链、吊销、时间戳或正式签章认证。
