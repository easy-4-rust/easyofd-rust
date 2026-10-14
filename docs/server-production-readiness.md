# 服务端库生产就绪验收

> 下列为代码9ba5d3f的历史验收记录。三项依赖例外正在由[无豁免依赖治理](dependency-remediation-verification.md)替代；新源码的最终CI完成前不沿用历史结论。
日期：2026-10-14。用途：服务端OFD读取、转换与密码学验签。当前状态：验收通过（限定已验证支持范围及服务集成契约），ARCHIVED。收尾遗漏的裸SM2/GB35275骨架已真实修复并完成新SHA完整门禁；不以历史归档或旧CI替代当前证据。
对应规格：[当前版本生产验收](superpowers/specs/2026-10-14-production-readiness.md)。

## 已实现安全边界
- SES DER超长长度/溢出/截断返回错误；历史Fuzz崩溃入库，不用catch_unwind掩盖解析缺陷。
- PFX有MAC时在密钥提取前验证原始AuthenticatedSafe；支持[RFC7292](https://www.rfc-editor.org/rfc/rfc7292.html)七种SHA1/SHA2。生产用Pkcs12Tools::read_private_key_verified，拒绝无MAC、错口令、篡改、错误曲线和无效SM2标量。
- PFX≤32MiB，口令≤4096字节，迭代≤200万（真实BC样本MAC120万）；AES128/SM4派生长度16字节，内部派生上限64字节；负/零/巨量长度拒绝。
- PNG文件转换和RasterRenderer::try_render_page先校验画布/图片尺寸：每边≤16384px、总像素≤3200万；文字有限正值、像素字号≤1024，实际字形位图≤4194304像素。
- 保留旧无MAC及不返回Result的受信render_page兼容入口，它们不作为外部输入的生产推荐入口。

## 服务集成契约
1. 使用独立worker进程处理不可信文件；任务超时30秒，进程地址空间/内存预算2GiB（按部署OS用cgroup/Job Object等实施），错误或进程退出返回失败，不无限重试同一坏输入。
2. OFD Reader显式配置PackageLimits：条目≤4096、单条目≤16MiB、解压总量≤64MiB、压缩比≤200。这些是服务示例选值，合法更大文档需按实际容量另行验收。
3. 初始并发≤4，拒绝超预算请求。库内像素预算不是整个服务的总内存硬限制。
4. 复杂多对多CGTransform、完整复杂字体塑形不在已支持范围；未知格式/缺字体不得对外宣称完整保真转换。真实业务样本和SLA未提供，本次使用公开固定样本验收。
5. 密码学验签结果只说明摘要/签名匹配；CA信任、有效期/吊销及印章授权由认证服务独立实施。此次用户确认未要求正式电子签章认证服务。

## 历史依赖风险裁决（已由新治理移除）
更新chacha20 0.10.1→0.10.2、wnaf0.14.0→0.14.1，移除已撤回版本。SM2要求正式0.14.0，Fuzz关键共享依赖锁定到生产版本。
保留三项明确例外：
- RUSTSEC-2026-0187：printpdf生成依赖lopdf0.31；当前外部PDF读取使用lopdf0.45。所查生成路径不调用旧load*。任何新增旧解析入口必须取消例外并重新验收。
- ttf-parser/proc-macro-error的unmaintained风险未消失，不等同于已知漏洞已修复；继续跟踪维护替代。
Clippy沿用只豁免assert_is_empty显示风格建议的CI规则，其他warning拒绝。

## 本地实际证据
证据目录：.superpowers/sdd/2026-10-14-production-readiness/。
- SES、MAC/密钥类型、DPI和审查两项资源漏洞均有真实红→绿日志。
- Python独立MAC变体，BC1.85底层MAC核验七算法/错口令；完整KeyStore加载与密钥证书匹配仅SHA1/SHA256，其他算法KeyStore不支持。
- 四线程28次输入/76个实际输出的烟测通过；最终耗时/RSS另以server-resource.log记录，不冒充业务SLA。
- 当前源码完整覆盖率、Fuzz、audit/deny、CI结果在门禁完成后填入下表。

| 门禁 | 结果 |
|---|---|
| workspace运行测试/行覆盖率≥90% | 运行测试2888通过/1忽略，doctest12通过/16忽略，合计2900通过/17忽略；本地行覆盖率93.37% |
| 四个Fuzz目标各100000次 | 当前SHA CI四目标均各100000次通过，本地受影响SES目标100000次通过 |
| audit/deny及上述例外 | 配置门禁通过；未过滤审计明确检出lopdf0.31例外及两个unmaintained提示，无撤回版本 |
| stable/MSRV1.88三平台CI | 六矩阵及fmt/bench共8 jobs成功 |
| Coverage CI（强制行覆盖率≥90%） | 通过 |
| Linux受限进程烟测（2GiB/30s） | 当前SHA通过，4线程28输入/76输出，1932ms |

未执行部署、发布或真实生产流量验收。上表CI证据均对应当前代码9ba5d3f；Task1–6所列工程门禁通过，规格/计划已归档。

独立审查：主审2 Important，加验签容器限定审查1 Important，合计3 Important/0 Critical；均已红→绿修复，无延期Minor。未过滤audit仍发现printpdf生成依赖的lopdf0.31旧解析漏洞；上述可达性裁决是范围限定风险接受，不宣称依赖本身已修复。

最终本地四线程烟测：28次输入、76个输出，耗时790ms，最大RSS约725MiB（macOS debug、本地公开样本）；该数字不是Linux/生产环境吞吐承诺。

## 最终提交与CI证据

代码提交：9ba5d3f161563278d96588b5b5dc343887cbad9f。[ci](https://github.com/easy-4-rust/easyofd-rust/actions/runs/38520794442), [coverage](https://github.com/easy-4-rust/easyofd-rust/actions/runs/38520794529), [security](https://github.com/easy-4-rust/easyofd-rust/actions/runs/38520794532), [fuzz](https://github.com/easy-4-rust/easyofd-rust/actions/runs/38520795694)全部success。最后归档提交仅改文档并使用skip ci；非文档tree与上述已验证SHA一致，不冒充归档提交独立重跑CI。

## 工程裁决与代价

- 用户确认范围为Rust服务端库；正式CA/吊销/印章授权及真实部署/SLA不在本次验收。代价：库验签成功不能用作认证结论。
- KDF上限选200万以容纳BC真实120万MAC参数。代价：仍需任务限流/超时，不能把迭代上限当总CPU服务预算。
- 保留旧受信API，生产选择严格接口。代价：误用旧入口不能获得完整性和像素预算保证。
- 通用PBKDF2内部长度上限64，AES128/SM4调用点限定16。代价：新增算法须更新对应契约并重验收。
- BC底层核验全部七MAC算法，KeyStore只声明SHA1/SHA256。代价：其余算法未验收完整KeyStore路径。
- 保留旧lopdf生成依赖不可达例外及两项维护例外。代价：必须持续跟踪，新增旧解析入口会使此裁决失效。
- 纯文档归档跳过重复CI。代价：文档提交无独立源码重跑，靠非文档tree一致性核验；本地日志保留以供复核。

收尾纠正：6ba781a的四工作流实际通过，但它们未覆盖两个骨架签名容器，不能证明完整生产验收。新增Task6并重新验证后才恢复最终结论。

## 验签支持范围与Task6

裸SM2容器真实校验SEC1公钥与DER/raw/ASCII HEX签名，默认SM2 ID为1234567812345678。GB35275容器读取Java直接摘要/PKCS9及BC标准CMS的SM2+SM3、版本1、Issuer/Serial签名者；所有签名者须与嵌入证书及显式DER证书pin匹配，并真实校验签名。此处trust_certs是签名者证书pin列表，不是CA链。

认证属性messageDigest必填且匹配原文SM3，各OID不得重复；标准CMS/GB contentType必填且匹配。仅历史Rust GB的直接content+嵌套SET编码保留缺失contentType兼容，不允许GB→CMS跨域重包装。旧Rust GB writer编码保留，未声明其反向Java互操作。签名输入≤32MiB、字段≤4096、签名者≤64。

原生verify_signature/verify_signature_multi仅验收已有Rust原生签名布局；容器层Java SES/GB/CMS互操作不代表所有外部OFD封装/C14N流程已端到端支持。OfdValidator目前是容器注册对象，不是完成的通用文档验证入口。cert::verify_chain中的兼容pass-through、crl::ocsp_check和timestamp占位入口不应被服务用作认证判断；本次不将这些功能算作已实现。

Task6红→绿：非空公钥/证书配置接受伪签名、Unicode/奇数HEX panic、合法GB属性签名跨CMS重包装。公开Java fixture正例及篡改/错误pin/重复属性负例通过；签名验证器已纳入SES Fuzz。限定审查1 Important/0 Critical，已修复内容类型绑定缺口。新SHA的完整门禁均通过；支持范围按本节及服务集成契约执行。

Task6本地最终：运行测试2888通过/1忽略，doctest12通过/16忽略，合计2900通过/17忽略；行覆盖率93.36710%。Clippy/fmt/diffcheck通过；新增验签路径SES Fuzz100000次通过。新SHA CI/Coverage/Security/Fuzz均通过。
