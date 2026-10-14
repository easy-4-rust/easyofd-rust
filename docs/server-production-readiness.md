# 服务端库生产就绪验收
日期：2026-10-14。用途：服务端OFD读取、转换与密码学验签。当前状态：本地门禁核验中，当前源码CI待验证；不把旧阶段ARCHIVED视为当前验收。
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

## 依赖风险裁决
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
| workspace运行测试/行覆盖率≥90% | 运行测试2881通过/1忽略，doctest12通过/16忽略，合计2893通过/17忽略；行覆盖率93.41% |
| 四个Fuzz目标各100000次 | 最终代码本地4×100000次通过，CI复核待执行 |
| audit/deny及上述例外 | 配置门禁通过；未过滤审计明确检出lopdf0.31例外及两个unmaintained提示，无撤回版本 |
| stable/MSRV1.88三平台CI | 待当前源码CI |
| Coverage CI（强制行覆盖率≥90%） | 待当前源码CI |
| Linux受限进程烟测（2GiB/30s） | 待当前源码CI |

未执行部署、发布或真实生产流量验收。归档只在上述门禁有当前源码证据后进行。

独立审查：2 Important/0 Critical，已完成一次红→绿修复，无延期Minor。未过滤audit仍发现printpdf生成依赖的lopdf0.31旧解析漏洞；上述可达性裁决是范围限定风险接受，不宣称依赖本身已修复。

最终本地四线程烟测：28次输入、76个输出，耗时790ms，最大RSS约725MiB（macOS debug、本地公开样本）；该数字不是Linux/生产环境吞吐承诺。
