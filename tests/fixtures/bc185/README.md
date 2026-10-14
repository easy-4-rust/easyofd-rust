# BC1.85公开测试身份互操作基线

来源是固定OFDRW2.4.0提交的`ofdrw-sign/src/test/resources/USER.p12`和UserV1/V4/V5.esl。
这是上游已公开的测试身份，**禁止生产使用**。本目录PFX为BC1.85重打包的测试文件，口令`public-test`；不包含生产凭据。

manifest记录文件哈希、SM2用户ID、来源和版本。Java生成的真实签名由Rust验证；RustSES/SM2及测试PFX由Java实际验签/加载，反向PFX由Rust构造而非复制Java夹具。反向PFX为公开测试身份的无私钥加密+口令MAC结构，仅存在target验收输出。

原始Java证书仍按签名验证提取公钥，测试不等于证书有效期、信任链或电子印章授权认证。`Pkcs12Tools::read_private_key`只承诺现有私钥提取语义，不宣称完整KeyStore/MAC验证。

重建参考源码：`OFDRW_SOURCE_DIR=<固定检出> bash scripts/build_ofdrw_reference.sh`。
生成Java样本：`OFDRW_SOURCE_DIR=<固定检出> bash scripts/generate_bc185_fixtures.sh <临时输出目录>`。
反向工具：`cargo run -p easyofd-signature --example bc185_generate -- target/bc185/rust`，再运行`OFDRW_SOURCE_DIR=<固定检出> bash scripts/verify_bc185_interop.sh target/bc185/rust`。
