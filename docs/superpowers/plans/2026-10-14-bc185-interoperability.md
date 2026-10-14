# BC 1.85 双向互操作验证 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 判定 OFDRW 的 BC 1.84→1.85 升级是否要求修正 Rust 密码学行为，并保存双向证据。

**Architecture:** 从固定 Java 提交生成测试专用签章/证书/PKCS#12样本，Rust验证Java产物，Java验证Rust产物。每种格式与方向单独验收；无差异时只交付互操作基线，不强加生产代码或依赖变更。

**Tech Stack:** OFDRW 2.4.0@84e9eb26、BC1.85，Rust既有sm2/sm3/sm4/x509-cert，easyofd-gm/signature/crypto。

**Spec:** [2026-10-14-ofdrw-upstream-alignment.md](../specs/2026-10-14-ofdrw-upstream-alignment.md)，R6–R7。

状态：ARCHIVED；两任务及最小修复完成。Java1.85/Rust双向验证、本地回归及代码`65b21bf14ebaa55956addfbd9097f12a12872227`六矩阵CI通过，已提交推送。

## Global Constraints

- MSRV1.88/edition2024，保持现有密码学API与历史格式契约；不照搬BC版本为Rust依赖升级。
- 固定Java SHA/BC依赖解析结果，记录证书、测试私钥来源与样本哈希；仅使用公开测试身份，禁止保存生产凭据。
- 两个方向、SES V1/V4/V5分别报告，读DER成功不代表密码验签成功。
- 无差异不修改生产逻辑；发现明确行为差异才新增可复现测试与最小修复任务，不临时扩大功能范围。

## Review Focus

1. SES版本误判或只解析不验签：每版本有正确签名和篡改反例（任务1/2）。
2. SM2用户ID或签名编码差异：记录参数、错误ID/证书不得通过（任务1/2）。
3. PKCS#12口令/证书链差异：正确口令读写，错误口令及不匹配证书明确失败（任务1/2）。
4. DER损坏、未知算法和缺字段：明确拒绝，不panic或静默接受（任务1）。
5. Java未运行或未实际使用BC1.85：单独标为未验证，不能以Rust自签自验代替（任务2）。

### Task 1：Java→Rust 固定样本验证

**Files:** Create `tests/fixtures/bc185/manifest.json`、README.md及公开测试样本，`scripts/generate_bc185_fixtures.sh`、`.ofdrw-gen/BC185Interop.java`、`crates/easyofd-signature/tests/bc185_interop.rs`、`crates/easyofd-gm/tests/bc185_pkcs12.rs`。

**Interfaces:** 脚本显式接受已存在 `OFDRW_SOURCE_DIR`，校验固定SHA及解析出的BC1.85，生成签名/证书/PKCS#12与JSON清单，不修改外部仓库、不安装工具。测试消费清单中每个格式/算法/SM2用户ID/证书/签名路径与SHA256。

- [x] Step 1：建立SM2WithSM3、SES V1/V4/V5、PKCS#12/证书矩阵测试；正例成功，篡改、错误证书/用户ID/口令、坏DER/未知算法失败。先记录夹具缺失，不能称为密码行为红灯。
- [x] Step 2：以固定Java版本生成真实样本后运行 `cargo test -p easyofd-signature --test bc185_interop` 与 `cargo test -p easyofd-gm --test bc185_pkcs12`；逐项记录成功/可复现行为差异/环境阻塞。
- [x] Step 3：补齐生成器与夹具溯源；如有行为差异，先更新本计划添加具名复现和最小修复步骤，再修正对应真实对象文件，避免无依据升级依赖。
- [x] Step 4：目标测试及 `cargo test -p easyofd-gm -p easyofd-signature -p easyofd-crypto` 通过；记录使用的实际BC解析版本及测试参数。
- [x] Step 5：审查每格式均有正/反例及密码验签证据，保存Java→Rust结果；若全通过明确“此方向无需生产修复”，不推断另一方向通过。

### Task 2：Rust→Java 验证及兼容性结论

**Files:** Create `crates/easyofd-signature/examples/bc185_generate.rs`、`scripts/verify_bc185_interop.sh`、`docs/bc185-interoperability.md`；Extend `.ofdrw-gen/BC185Interop.java` 和 manifest.json。

**Interfaces:** Rust生成器调用既有真实签章/PKCS#12接口输出临时产物；Java验证器消费同格式清单，返回每项验证结果及非零失败码。输出不自动替换已存基线。

- [x] Step 1：给验证脚本编写契约检查：篡改、未运行Java、版本非1.85、格式缺失均不得输出全通过；准备与Task1相同格式和反例矩阵。
- [x] Step 2：运行 `cargo run -p easyofd-signature --example bc185_generate -- <输出目录>`，再执行 `OFDRW_SOURCE_DIR=<固定源码路径> bash scripts/verify_bc185_interop.sh <输出目录>`；记录尚缺反向验证器/行为差异，不虚构现成命令已通过。
- [x] Step 3：实现Java反向验签/PKCS#12读取与结果清单；参数、编码、证书链行为与Task1对应。只修复实际失败且已确认契约的差异。
- [x] Step 4：全矩阵通过，运行受影响Rust测试、fmt/clippy，并保存Java实际输出/退出码；缺JDK/样本/算法能力保留具体未验证项。
- [x] Step 5：发布仓库内验证报告，按方向/SES版本/PKCS#12列证据；无差异写明无需业务API新增，有差异标注修复及回归。仅全部门禁满足才归档计划。

## 当前证据

实际证据见 [2026-10-14验证报告](../../ofdrw-upstream-alignment-verification.md)。真实Java→Rust和Rust→Java均执行SM2/SES V1/V4/V5验签与反例；BC1.85 PFX读入、错误口令、损坏数据、私钥与证书对应验证通过。反向PFX由Rust验收工具生成，含口令MAC而不加密公开测试私钥，不提供生产导出API。

接口裁决：验签保留原始TBS，签章保留原始印章DER以兼容未知扩展；SES新输出采用DER SM2签名，读取仍支持历史raw编码；BC BER与KDF填充边界缺口经实测修复。证书信任链/吊销和Pkcs12Tools完整MAC语义不作完成声明。

生成器在 `tools/upstream/BC185Interop.java`，不放入被Git忽略的 `.ofdrw-gen`；其classpath优先且必须使用固定SHA源码实际构建的classes。公开测试身份来源和文件SHA256保存在manifest，`test-secret.bin`仅存在忽略的target目录。

## 实施发现与最小修复任务（2026-10-14）

真实BC1.85样本与源码发现：SES V1/V4/V5验证容器直接返回成功，Rust签章容器输出64字节raw SM2签名而Java使用DER。此为既有互操作缺口，不能归因于BC1.85新增回归。

- [x] 验证容器对真实签章和消息/签名篡改运行红灯，再分别接入版本、摘要、证书、公钥与密码验签。
- [x] Rust→Java实测raw/DER编码差异的失败，再改SES签章输出DER，同时验证旧raw编码在Rust读取端仍兼容。
- [x] 三版本双向实际验签、篡改/坏DER/错误ID等反例全部通过；不得用结构解析或空验证充数。

整体门禁：[CI通过](https://github.com/easy-4-rust/easyofd-rust/actions/runs/38510929976)，Coverage通过。证书信任、吊销与生产KeyStore完整MAC校验仍为已声明的非目标。
