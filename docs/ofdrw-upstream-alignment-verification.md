# OFDRW增量对齐实际验证记录

日期：2026-10-14。Rust工作目录在dev，基线commit `12a0381bfa92a3736b84cadda95f4044c94f22e6`。本轮代码已提交推送；最终CI对应下文实际代码SHA，旧CI不用于本轮验收。
Java基线 `84e9eb260ba6fd351ecccf08a01a40305d4af635` / 2.4.0 / BC1.85。

## 当前状态

文字元数据、CTM逐字定位、PDF、SVG/PNG描边、固定基线工具及BC双向互操作已实现并完成本地验证。独立审查发现的三个Important问题已分别红→绿修复。**Rust1.88及stable的Linux/macOS/Windows六矩阵CI全部通过；两计划已ARCHIVED。**

## 实现与兼容性

- Reader新增TextRenderRun旁路数据和text_render_runs零基页面接口，保留CTM/TextCode/CGTransform/颜色继承/字体；不增加TextObject/OfdPage必填字段。
- 逐字原点使用完整CTM，步距不叠加平移；共享布局函数贯通真实PDF/SVG/PNG。
- PNG/SVG使用嵌入字体轮廓绘制，不用文字外接矩形充数；Fill/Stroke开关独立。
- 原简化writer把Size当pt、Font当名称的格式维持旧输出；标准数字Font资源引用采用mm绘制信息。
- SES V1/V4/V5验证器由直接成功改为版本/算法/摘要/证书公钥/SM2真实验签；签章保留原始印章DER、输出DER SM2签名，验签兼容历史64字节raw。
- PKCS#12读入支持BC的有界BER与分段OCTET STRING，修复KDF重复填充未截断错误并按UTF-16处理口令。
- 反向测试PFX由Rust构造，Java实际加载，验证口令MAC、私钥与证书对应。该工具只用于公开测试身份，私钥未加密，不提供生产导出API。

## 实际命令与结果

| 验证层 | 实际执行 | 结果 |
|---|---|---|
| 原基线 | reader/convert lib tests | 185+375通过 |
| Rust完整本地测试 | `cargo test --workspace` | 2878通过、0失败、17忽略；忽略项不计通过 |
| 原始OFD保真 | `cargo test -p easyofd --test roundtrip_diff` | 通过；不替代渲染验证 |
| 目标边界 | reader metadata、text coordinates、PDF matrices/missing glyph、SVG/PNG矩阵、BC互操作/PFX | 均有实际结果，详见日志 |
| 格式与静态检查 | `cargo fmt --all -- --check` / `cargo clippy --workspace --all-targets -- -D warnings` | 本地通过；并非跨平台证明 |
| 固定Java源码 | Git archive指定SHA到target，再Maven reactor构建converter/sign及依赖模块 | 构建成功；BC工具必须使用该classes及BC1.85 |
| 上游新增/受影响行为 | 反射执行PointUtilTest和TextStrokeExportTest真实方法，显式TempDir注入 | 9方法通过；不冒充完整Java Maven测试套件 |
| 实际渲染 | Rust导出SVG/PNG/PDF，rsvg-convert/pdftoppm渲染，独立查看竖排输出 | 竖排“下载次数：1”无字距挤压；AB和竖排矩阵有数值断言 |
| Java/Rust实际输出 | 编译固定修复AWTMaker/PointUtil，导出JavaSVG/PNG，reference_compare | 5描边/填充组合×两后端的红描边有无一致；DPI/抗锯齿不同，不声称逐像素相同 |
| Java→Rust密码学 | BC1.85真实SM2/SES样本，Rust正例/篡改/坏DER/错证书/未知算法/错版本/错ID/raw兼容 | 通过；固定源码classes重新生成的live样本另行验证 |
| Rust→Java密码学 | bc185_generate + verify_bc185_interop.sh | SM2、SES三版本、PFX加载/私钥证书匹配/错口令拒绝通过 |
| 验收工具反例 | Python基线契约检查 | 错SHA、缺字体、非空损坏PFX均拒绝；外部源目录不被修改 |

这里的完整workspace测试指本机默认feature集合，17项ignored及未覆盖目标不自动验收。

## 独立审查和修复证据

审查为一次独立只读整体验证，覆盖已修改及新文件。

1. 缺资源图片引起content_index错位：新测试先失败（1≠0），按既有媒体解析规则跳过不存在资源后通过。
2. 嵌入PDF字体缺字被printpdf跳过：真实🦀输入先成功返回导致反例失败，新增glyph覆盖验证后明确返回错误，测试通过。
3. 反向PFX仅复制且未读取：替换为非空垃圾仍通过的红灯已记录；Rust实际构造PFX、Java加载/口令/密钥对应验证后垃圾拒绝测试通过。

红绿和完整输出在 `.superpowers/sdd/2026-10-14-ofdrw-text-rendering-alignment/` 与 `.superpowers/sdd/2026-10-14-bc185-interoperability/`。实现已提交推送；本地红绿日志保留。实际产物位于target/upstream-output、target/upstream-reference、target/bc185。

## 工程裁决与审查保留边界

| 裁决 | 理由 | 判断错误的代价/尚未证明的内容 |
|---|---|---|
| 增加字体名称/数据旁路字段 | 实际嵌入字体绘制必须具备资源信息，保留旧公开结构 | 资源解析边界仍需更多真实样本覆盖 |
| 绘制元数据错误延迟到转换接口报告 | 严格新解析不能阻断既有读取与无损回写 | 可读取的文档可能不能转换，错误会明确返回 |
| 保留旧pt字号/Font名称路径 | 不破坏既有writer输出契约 | 格式识别有明确范围，非标准混合格式需新增样本 |
| 保留原始印章/TBS并兼容raw签名读取 | 未建模扩展不能被重编码丢弃 | 不代表印章授权、证书信任或所有结构一致性认证 |
| 使用固定Git源码实际构建Java模块 | 消除只凭Maven版本号不能证明指定SHA的歧义 | 工具必须先构建参考模块，不能默默回退旧jar |
| Rust反向PFX写出限于公开测试工具 | 证明真实反向结构兼容，不新增生产KeyStore API | 不能用于生产密钥导出；现有Pkcs12Tools完整MAC/证书链校验仍非本次范围 |
| CGTransform当前1:1映射，复杂多对多显式拒绝 | 不冒充完整复杂塑形引擎 | 此类合法复杂文档仍不能转换，未作完整字体能力声明 |
| 像素完全一致、复杂字体塑形不验收 | 当前目标是上游修复的可观察行为 | 输出视觉差异的其他来源尚未全面比较 |
| CI/MSRV必须使用本轮代码 | 本机Rust1.98.1不能代替MSRV证据；最终六矩阵实际通过 | 不代表矩阵以外的平台或编译器验收 |

未发现需延后的Minor代码问题；审查保留的上述边界不通过旧报告或删减验收标准关闭。

## 交付状态

代码已提交推送，远端已整合#36/#37。两计划完成并ARCHIVED；本次不发布版本。归档记录为纯文档提交，使用`[skip ci]`避免重复运行未改变的源码测试；其源码、夹具、脚本、Cargo和工作流与下述已通过CI的代码提交完全一致。

CI首轮 `dc8ee27`：Rust1.88三平台均通过，stable测试通过后被Rust1.99新增`assert_is_empty`风格检查阻断。CI仅显式豁免该断言显示风格检查，其余`-D warnings`保留；不批量重写历史测试。该豁免不证明断言诊断显示质量，重新运行的CI已全部通过。

最终代码提交：`65b21bf14ebaa55956addfbd9097f12a12872227`。[CI](https://github.com/easy-4-rust/easyofd-rust/actions/runs/38510929976)：8/8 jobs成功（6平台/工具链矩阵、格式、bench）；[Coverage](https://github.com/easy-4-rust/easyofd-rust/actions/runs/38510929917)：成功。整合远端依赖后的本地完整测试仍为2878通过/0失败/17忽略。

补充裁决：CI仅豁免`assert_is_empty`风格建议，保留其他`-D warnings`；代价是断言失败的显示风格不由该lint验收。归档只改文档并跳过重复CI，代码验收对应上述SHA；代价是归档commit本身没有独立重跑源码测试，靠非文档tree一致性核验。
