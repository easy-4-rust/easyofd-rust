# 生产就绪实施与验收计划

规格：[当前版本生产验收](../specs/2026-10-14-production-readiness.md)。状态：重新实施；Task1–5已验证结果保留，Task6尚未完成，current dev；已有提交/推送授权继续有效，不新建分支，不安装工具，不发布。

- [x] Task1 SES历史崩溃和长度边界：真实输入RED→checked arithmetic→GREEN，入Fuzz corpus。
- [x] Task2 PKCS#12 MAC与严格入口：真实PFX错口令/篡改RED→MAC校验/完整性严格接口→GREEN；迭代及输入预算。
- [x] Task3 当前版本安全验收：四目标Fuzz、audit/deny、真实双向互操作；失败修复与回归。
- [x] Task4 实际转换及并发负载烟测：记录范围/耗时/内存，业务SLA缺失不冒充压测验收。
- [x] Task5 独立审查、完整回归、当前SHA六矩阵CI/覆盖率、生产适用范围与未验收项记录。

最终本地证据：2893测试通过/17忽略，行覆盖率93.41%，四Fuzz各100000次，audit/deny经显式例外通过；独立审查两项Important已红→绿修复。Task5当前源码CI/受限Linux进程烟测均通过。

当前代码6ba781ab6302c550905dc743603a1c466f80c03b的CI/Coverage/Security/Fuzz均通过，见[最终验收报告](../../server-production-readiness.md)。生产集成须保持报告内严格入口/隔离预算；不扩大为正式认证或部署证明。

- [ ] Task6 关闭遗漏的验证器骨架：裸SM2/GB35275真实验签、Java互操作和反例；检查其他验证入口不存在伪成功，受影响完整CI重新验证。
