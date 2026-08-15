#!/bin/bash
# ofd_sample_gen.sh - 使用 ofdrw（Java）程序化生成多样化 OFD 样本
#
# 目的：通过 ofdrw 各模块 API 生成覆盖各种边界的 OFD 样本，
#       用于增强 easyofd-rust 的测试多样性。
#
# 前置条件：
#   - JDK 8+（java / javac）
#   - Maven 3.6+
#   - Git
#   - ~500MB 磁盘空间（ofdrw 克隆 + 构建产物）
#
# 用法：
#   bash scripts/ofd_sample_gen.sh
#
# 生成样本类别（至少 10 种）：
#   1. DocInfo 全字段
#   2. 多页文档（分页）
#   3. 复杂文字样式（粗体/斜体/下划线/颜色/字重）
#   4. 图片嵌入
#   5. Canvas 绘图（圆形/矩形/线条）
#   6. 多 VirtualPage（含背景层/正文层分层）
#   7. 自定义字体
#   8. 批注（印章注释 / 水印注释）
#   9. 浮动布局（左/中/右浮动）
#  10. VirtualPage 绝对定位
#  11. 段落分页溢出
#  12. 附件（Attachment）
#  13. 骑缝章签名（需 ofdrw-sign 资源）
#  14. 数字签名（需 ofdrw-sign 资源）
#
# 退出码：
#   0 - 成功
#   1 - 生成失败
#   2 - 前置条件不满足

set -euo pipefail

# ─── 配置 ─────────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CLONE_DIR="$PROJECT_ROOT/.ofdrw-gen"
OFRW_REPO="https://github.com/ofdrw/ofdrw.git"
OFRW_BRANCH="master"
SAMPLES_DIR="$CLONE_DIR/samples"
DEST_DIR="$PROJECT_ROOT/tests/fixtures/ofdrw_gen"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# ─── 前置检查 ─────────────────────────────────────────────────────────────────

check_prerequisites() {
    local missing=0

    if ! command -v java &> /dev/null; then
        log_error "Java 未找到，请安装 JDK 8+"
        missing=1
    else
        local java_ver
        java_ver=$(java -version 2>&1 | head -1)
        log_info "Java: $java_ver"
    fi

    if ! command -v javac &> /dev/null; then
        log_error "javac 未找到，请安装 JDK（含编译器）"
        missing=1
    fi

    if ! command -v mvn &> /dev/null; then
        log_error "Maven 未找到，请安装 Maven 3.6+"
        missing=1
    else
        local mvn_ver
        mvn_ver=$(mvn --version 2>&1 | head -1)
        log_info "Maven: $mvn_ver"
    fi

    if ! command -v git &> /dev/null; then
        log_error "Git 未找到"
        missing=1
    fi

    # 磁盘空间检查（至少 200MB 可用）
    local avail_kb
    avail_kb=$(df -k "$PROJECT_ROOT" | tail -1 | awk '{print $4}')
    if [ "${avail_kb:-0}" -lt 204800 ]; then
        log_warn "磁盘可用空间不足 200MB，构建可能失败"
    fi

    if [ $missing -eq 1 ]; then
        log_error "前置条件不满足，退出"
        exit 2
    fi

    log_info "所有前置条件满足"
}

# ─── 克隆/更新 ofdrw ────────────────────────────────────────────────────────

clone_or_update_ofdrw() {
    if [ -d "$CLONE_DIR/.git" ]; then
        log_info "ofdrw 克隆已存在，更新中..."
        cd "$CLONE_DIR"
        git pull --quiet --rebase 2>/dev/null || {
            log_warn "git pull 失败，使用现有版本"
        }
    else
        log_info "克隆 ofdrw 仓库（浅克隆）..."
        git clone --branch "$OFRW_BRANCH" --depth 1 "$OFRW_REPO" "$CLONE_DIR"
    fi
    log_info "ofdrw 仓库就绪：$CLONE_DIR"
}

# ─── Maven 构建 ──────────────────────────────────────────────────────────────

build_ofdrw() {
    log_info "构建 ofdrw（仅编译，跳过测试）..."
    cd "$CLONE_DIR"
    mvn -B -pl ofdrw-layout,ofdrw-sign,ofdrw-reader,ofdrw-gm,ofdrw-core,ofdrw-pkg,ofdrw-font,ofdrw-graphics2d -am install -DskipTests -q 2>&1 | tail -5
    log_info "ofdrw 构建完成"
}

# ─── 写入 GenerateSamples.java ───────────────────────────────────────────────

write_generator_java() {
    log_info "写入 GenerateSamples.java..."
    cat > "$CLONE_DIR/GenerateSamples.java" << 'JAVA_EOF'
import org.ofdrw.layout.OFDDoc;
import org.ofdrw.layout.PageLayout;
import org.ofdrw.layout.VirtualPage;
import org.ofdrw.layout.element.Position;
import org.ofdrw.layout.element.AFloat;
import org.ofdrw.layout.element.Clear;
import org.ofdrw.layout.element.*;
import org.ofdrw.layout.element.canvas.Canvas;
import org.ofdrw.layout.edit.Annotation;
import org.ofdrw.layout.edit.Attachment;
import org.ofdrw.core.annotation.pageannot.AnnotType;
import org.ofdrw.core.basicStructure.pageObj.layer.Type;
import org.ofdrw.core.basicType.ST_Box;
import org.ofdrw.font.Font;
import org.ofdrw.font.FontName;
import org.ofdrw.reader.OFDReader;

import java.io.IOException;
import java.nio.file.*;

/**
 * OFD 样本生成器 —— 使用 ofdrw API 生成多样化边界样本。
 *
 * 每个 generate* 方法产出一个 .ofd 文件到 samples/ 目录。
 */
public class GenerateSamples {

    private final Path outDir;

    public GenerateSamples(Path outDir) throws IOException {
        this.outDir = outDir;
        Files.createDirectories(outDir);
    }

    // ─── 1. DocInfo 全字段 ───────────────────────────────────────────────────
    void generateDocInfoFull() throws IOException {
        Path p = outDir.resolve("gen_01_docinfo_full.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            // 通过 ofdDir 链路设置完整 DocInfo
            try {
                doc.getOfdDir().getOfd().getDocBody().getDocInfo()
                   .setDocID(java.util.UUID.randomUUID())
                   .setCreator("easyofd-rust OF-B generator")
                   .setCreationDate(java.time.LocalDate.now())
                   .setAuthor("OF-B 自动化")
                   .setSubject("DocInfo 全字段测试")
                   .setTile("DocInfo 测试文档");
            } catch (Exception e) {
                System.out.println("  [WARN] 设置 DocInfo 失败: " + e.getMessage());
            }

            Paragraph title = new Paragraph("DocInfo 全字段测试文档")
                .setFontSize(16d);
            title.setFloat(AFloat.center).setMargin(10d);
            doc.add(title);

            doc.add(new Paragraph("本文档包含完整的 DocInfo 元数据字段。"));
            doc.add(new Paragraph("DocID: UUID 随机生成"));
            doc.add(new Paragraph("Creator: easyofd-rust OF-B generator"));
            doc.add(new Paragraph("Subject: DocInfo 全字段测试"));
            doc.add(new Paragraph("Author: OF-B 自动化"));
        }
        System.out.println("  [1/14] gen_01_docinfo_full.ofd");
    }

    // ─── 2. 多页文档（分页） ─────────────────────────────────────────────────
    void generateMultiPage() throws IOException {
        Path p = outDir.resolve("gen_02_multi_page.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            for (int i = 1; i <= 5; i++) {
                doc.add(new Paragraph("第 " + i + " 页内容：这是多页文档的第 "
                    + i + " 页。").setFontSize(10d));
                doc.add(new Paragraph("页面 " + i + " 包含段落文本，用于测试分页逻辑。"));
                for (int j = 0; j < 30; j++) {
                    doc.add(new Paragraph("填充行 " + j
                        + "：ABCDEFGHIJKLMNOPQRSTUVWXYZ abcdefghijklmnopqrstuvwxyz 0123456789"));
                }
            }
        }
        System.out.println("  [2/14] gen_02_multi_page.ofd");
    }

    // ─── 3. 复杂文字样式 ────────────────────────────────────────────────────
    void generateComplexText() throws IOException {
        Path p = outDir.resolve("gen_03_complex_text.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            Paragraph p1 = new Paragraph();
            p1.add(new Span("粗体文字 ").setBold(true).setFontSize(12d));
            p1.add(new Span("斜体文字 ").setItalic(true).setFontSize(12d));
            p1.add(new Span("下划线文字 ").setUnderline(true).setFontSize(12d));
            p1.add(new Span("红色文字 ").setColor(255, 0, 0).setFontSize(12d));
            p1.add(new Span("蓝色粗体").setColor(0, 0, 255).setBold(true).setFontSize(12d));
            doc.add(p1);

            Paragraph p2 = new Paragraph();
            p2.add(new Span("字重 W100 ").setFontSize(8d)
                .setWeight(org.ofdrw.core.text.text.Weight.W_100));
            p2.add(new Span("字重 W500 ").setFontSize(8d)
                .setWeight(org.ofdrw.core.text.text.Weight.W_500));
            p2.add(new Span("字重 W900 ").setFontSize(8d)
                .setWeight(org.ofdrw.core.text.text.Weight.W_900));
            doc.add(p2);

            Paragraph p3 = new Paragraph();
            p3.add(new Span("大字号(20pt) ").setFontSize(20d));
            p3.add(new Span("中字号(12pt) ").setFontSize(12d));
            p3.add(new Span("小字号(6pt)").setFontSize(6d));
            doc.add(p3);

            // 中英文混排
            doc.add(new Paragraph("中文English混排テスト：Hello 世界！"));

            // 长段落首行缩进
            Paragraph longP = new Paragraph(
                "这是一段很长的中文文本，用于测试段落自动换行和首行缩进效果。"
                + "OFD 版式文档格式（Open Fixed-layout Document）是中国国家标准 "
                + "GB/T 33190-2016 定义的电子文件存储与交换格式。"
                + "它支持文字、图片、矢量图形、数字签名等多种内容类型。")
                .setFirstLineIndent(2);
            doc.add(longP);
        }
        System.out.println("  [3/14] gen_03_complex_text.ofd");
    }

    // ─── 4. 图片嵌入 ────────────────────────────────────────────────────────
    void generateWithImage() throws IOException {
        Path p = outDir.resolve("gen_04_with_image.ofd");
        // 创建一个简单的测试图片
        Path imgPath = outDir.resolve("_test_img.png");
        createTestPng(imgPath);

        try (OFDDoc doc = new OFDDoc(p)) {
            PageLayout layout = doc.getPageLayout();
            VirtualPage vp = new VirtualPage(layout);

            // VirtualPage 要求所有元素使用绝对定位
            Paragraph title = new Paragraph("图片嵌入测试").setFontSize(14d);
            title.setPosition(Position.Absolute).setX(50d).setY(20d).setWidth(120d);
            vp.add(title);

            Img img = new Img(60, 40, imgPath);
            double x = (layout.getWidth() - img.getWidth()) / 2;
            img.setPosition(Position.Absolute).setX(x).setY(80d);
            img.setBorder(1d);
            img.setPadding(2d);
            vp.add(img);

            Paragraph desc = new Paragraph("上方为嵌入的测试图片（PNG 格式）");
            desc.setPosition(Position.Absolute).setX(50d).setY(140d).setWidth(120d);
            vp.add(desc);

            doc.addVPage(vp);
        }
        Files.deleteIfExists(imgPath);
        System.out.println("  [4/14] gen_04_with_image.ofd");
    }

    // ─── 5. Canvas 绘图 ─────────────────────────────────────────────────────
    void generateCanvasDrawing() throws IOException {
        Path p = outDir.resolve("gen_05_canvas_drawing.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            doc.add(new Paragraph("Canvas 绘图测试").setFontSize(14d)
                .setFloat(AFloat.center).setMargin(5d));

            // 圆形
            Canvas circle = new Canvas(30d, 30d);
            circle.setDrawer(ctx -> {
                ctx.beginPath();
                ctx.arc(15, 15, 10, 0, 360);
                ctx.stroke();
            });
            doc.add(new Paragraph().add(new Span("圆形：")).setFontSize(10d));
            doc.add(circle);

            // 矩形填充
            Canvas rect = new Canvas(40d, 20d);
            rect.setDrawer(ctx -> {
                ctx.setFillColor(0, 100, 200);
                ctx.fillRect(5, 5, 30, 10);
                ctx.setStrokeColor(255, 0, 0);
                ctx.strokeRect(0, 0, 40, 20);
            });
            doc.add(new Paragraph().add(new Span("矩形（填充+描边）：")).setFontSize(10d));
            doc.add(rect);

            // 线条
            Canvas line = new Canvas(50d, 20d);
            line.setDrawer(ctx -> {
                ctx.beginPath();
                ctx.moveTo(0, 10);
                ctx.lineTo(50, 10);
                ctx.stroke();
                ctx.beginPath();
                ctx.moveTo(0, 0);
                ctx.lineTo(50, 20);
                ctx.stroke();
            });
            doc.add(new Paragraph().add(new Span("交叉线条：")).setFontSize(10d));
            doc.add(line);
        }
        System.out.println("  [5/14] gen_05_canvas_drawing.ofd");
    }

    // ─── 6. 多 VirtualPage（含分层） ────────────────────────────────────────
    void generateMultiVirtualPage() throws IOException {
        Path p = outDir.resolve("gen_06_multi_vpage.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            PageLayout layout = doc.getPageLayout();

            // 第一个 VirtualPage：含背景层和正文层
            VirtualPage vp1 = new VirtualPage(layout);
            // 背景层 Div
            Div bg = new Div(50d, 50d)
                .setBackgroundColor(220, 230, 240)
                .setLayer(Type.Background);
            bg.setPosition(Position.Absolute).setX(80d).setY(100d);
            vp1.add(bg);
            // 正文层文字
            Paragraph t1 = new Paragraph("VirtualPage 1：含背景层和正文层")
                .setFontSize(12d);
            t1.setPosition(Position.Absolute).setX(30d).setY(50d).setWidth(150d);
            t1.setLayer(Type.Body);
            vp1.add(t1);
            doc.addVPage(vp1);

            // 第二个 VirtualPage：绝对定位元素
            VirtualPage vp2 = new VirtualPage(layout);
            Paragraph t2 = new Paragraph("VirtualPage 2：多个绝对定位元素")
                .setFontSize(12d);
            t2.setPosition(Position.Absolute).setX(30d).setY(20d).setWidth(150d);
            vp2.add(t2);
            for (int i = 0; i < 5; i++) {
                Div box = new Div(30d, 20d)
                    .setBackgroundColor(100 + i * 30, 150, 200 - i * 20)
                    .setBorder(0.5d);
                box.setPosition(Position.Absolute)
                    .setX(30d + i * 35d).setY(60d);
                vp2.add(box);
            }
            doc.addVPage(vp2);
        }
        System.out.println("  [6/14] gen_06_multi_vpage.ofd");
    }

    // ─── 7. 自定义字体 ──────────────────────────────────────────────────────
    void generateCustomFont() throws IOException {
        Path p = outDir.resolve("gen_07_custom_font.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            // 使用内置字体名称
            Font serif = new Font("NotoSerif", "NotoSerif");
            Paragraph p1 = new Paragraph()
                .setDefaultFont(serif)
                .setFontSize(12d)
                .add("使用 NotoSerif 字体的文本");
            doc.add(p1);

            // 使用 FontName 枚举
            Font yahei = FontName.MSYahei.font();
            Paragraph p2 = new Paragraph()
                .setDefaultFont(yahei)
                .setFontSize(10d)
                .add("使用微软雅黑字体的文本");
            doc.add(p2);

            // 混合字体
            Paragraph p3 = new Paragraph();
            p3.add(new Span("Span-A ").setFontSize(10d));
            p3.add(new Span("Span-B ").setFontSize(12d));
            p3.add(new Span("Span-C ").setFontSize(14d));
            p3.add(new Span("Span-D ").setFontSize(8d));
            doc.add(p3);
        }
        System.out.println("  [7/14] gen_07_custom_font.ofd");
    }

    // ─── 8. 批注（印章注释 + 水印注释） ─────────────────────────────────────
    void generateAnnotations() throws IOException {
        // 先生成一个基础文档
        Path baseP = outDir.resolve("_base_for_annot.ofd");
        try (OFDDoc doc = new OFDDoc(baseP)) {
            doc.add(new Paragraph("批注测试基础文档").setFontSize(14d));
            doc.add(new Paragraph("此文档将被追加印章注释和水印注释。"));
            for (int i = 0; i < 20; i++) {
                doc.add(new Paragraph("正文内容行 " + i));
            }
        }

        // 创建测试印章图片
        Path stampImg = outDir.resolve("_stamp_img.png");
        createTestPng(stampImg);

        Path p = outDir.resolve("gen_08_annotations.ofd");
        try (OFDReader reader = new OFDReader(baseP);
             OFDDoc doc = new OFDDoc(reader, p)) {
            // 印章注释
            Annotation stamp = new Annotation(70d, 100d, 60d, 60d,
                AnnotType.Stamp, ctx -> {
                    ctx.setGlobalAlpha(0.7);
                    ctx.drawImage(stampImg, 0, 0, 40d, 40d);
                });
            doc.addAnnotation(1, stamp);

            // 水印注释
            ST_Box boundary = new ST_Box(50d, 50d, 80d, 30d);
            Annotation watermark = new Annotation(boundary,
                AnnotType.Watermark, ctx -> {
                    ctx.setGlobalAlpha(0.3);
                    ctx.drawImage(stampImg, 0, 0, 60d, 20d);
                });
            doc.addAnnotation(1, watermark);
        }
        Files.deleteIfExists(baseP);
        Files.deleteIfExists(stampImg);
        System.out.println("  [8/14] gen_08_annotations.ofd");
    }

    // ─── 9. 浮动布局 ────────────────────────────────────────────────────────
    void generateFloatLayout() throws IOException {
        Path p = outDir.resolve("gen_09_float_layout.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            doc.add(new Paragraph("浮动布局测试").setFontSize(14d)
                .setFloat(AFloat.center).setMargin(5d));

            Div left = new Div(40d, 20d)
                .setBackgroundColor(200, 100, 100)
                .setFloat(AFloat.left)
                .setMargin(3d).setBorder(0.5d).setPadding(3d);
            doc.add(left);

            Div center = new Div(40d, 20d)
                .setBackgroundColor(100, 200, 100)
                .setFloat(AFloat.center)
                .setMargin(3d).setBorder(0.5d).setPadding(3d);
            doc.add(center);

            Div right = new Div(40d, 20d)
                .setBackgroundColor(100, 100, 200)
                .setFloat(AFloat.right)
                .setMargin(3d).setBorder(0.5d).setPadding(3d);
            doc.add(right);

            doc.add(new Paragraph("浮动元素之后的段落文本。"));
        }
        System.out.println("  [9/14] gen_09_float_layout.ofd");
    }

    // ─── 10. VirtualPage 绝对定位 ───────────────────────────────────────────
    void generateVirtualPagePositioning() throws IOException {
        Path p = outDir.resolve("gen_10_virtual_page.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            PageLayout layout = doc.getPageLayout();
            VirtualPage vp = new VirtualPage(layout);

            // 绝对定位文字
            Paragraph title = new Paragraph("VirtualPage 绝对定位测试")
                .setFontSize(14d);
            title.setPosition(Position.Absolute)
                .setX(30d).setY(20d).setWidth(150d);
            vp.add(title);

            // 绝对定位 Div
            Div box = new Div(50d, 30d)
                .setBackgroundColor(50, 150, 200)
                .setBorder(1d);
            box.setPosition(Position.Absolute)
                .setX(80d).setY(80d);
            vp.add(box);

            // 绝对定位段落
            Paragraph note = new Paragraph("此段落位于页面右下角")
                .setFontSize(8d);
            note.setPosition(Position.Absolute)
                .setX(120d).setY(250d).setWidth(80d);
            vp.add(note);

            doc.addVPage(vp);
        }
        System.out.println("  [10/14] gen_10_virtual_page.ofd");
    }

    // ─── 11. 段落分页溢出 ───────────────────────────────────────────────────
    void generatePageOverflow() throws IOException {
        Path p = outDir.resolve("gen_11_page_overflow.ofd");
        try (OFDDoc doc = new OFDDoc(p)) {
            doc.setDefaultPageLayout(PageLayout.A4().setMargin(20d));

            doc.add(new Paragraph("段落分页溢出测试").setFontSize(14d)
                .setFloat(AFloat.center).setMargin(5d));

            // 大量 Div 触发分页
            for (int i = 0; i < 100; i++) {
                Div d = new Div(60d, 15d)
                    .setBackgroundColor(200 + (i % 50), 100, 100)
                    .setFloat(AFloat.center)
                    .setMargin(2d)
                    .setBorder(0.3d);
                doc.add(d);
            }
        }
        System.out.println("  [11/14] gen_11_page_overflow.ofd");
    }

    // ─── 12. 附件 ───────────────────────────────────────────────────────────
    void generateWithAttachment() throws IOException {
        Path p = outDir.resolve("gen_12_attachment.ofd");
        // 创建一个简单的文本附件
        Path attachFile = outDir.resolve("_attachment.txt");
        Files.write(attachFile, "这是一个附件内容。\nOF-B 测试生成。".getBytes());

        try (OFDDoc doc = new OFDDoc(p)) {
            doc.add(new Paragraph("附件测试文档").setFontSize(14d));
            doc.add(new Paragraph("本文档包含一个文本附件。"));
            doc.add(new Paragraph("附件文件名: _attachment.txt"));
            Attachment att = new Attachment("test_attachment", attachFile);
            doc.addAttachment(att);
        }
        Files.deleteIfExists(attachFile);
        System.out.println("  [12/14] gen_12_attachment.ofd");
    }

    // ─── 13. 骑缝章签名（条件生成） ─────────────────────────────────────────
    void generateRidingSeal() throws IOException {
        Path p12 = Paths.get("ofdrw-sign/src/test/resources/USER.p12");
        Path esl = Paths.get("ofdrw-sign/src/test/resources/UserV4.esl");
        if (!Files.exists(p12) || !Files.exists(esl)) {
            System.out.println("  [13/14] SKIP gen_13_riding_seal（签名资源不存在）");
            return;
        }

        try {
            java.security.PrivateKey prvKey = org.ofdrw.gm.cert.PKCS12Tools.ReadPrvKey(p12, "private", "777777");
            java.security.cert.Certificate signCert = org.ofdrw.gm.cert.PKCS12Tools.ReadUserCert(p12, "private", "777777");
            org.ofdrw.gm.ses.v4.SESeal seal = org.ofdrw.gm.ses.v4.SESeal.getInstance(Files.readAllBytes(esl));

            // 先生成多页基础文档
            Path baseP = outDir.resolve("_base_for_seal.ofd");
            try (OFDDoc doc = new OFDDoc(baseP)) {
                doc.setDefaultPageLayout(PageLayout.A4().setMargin(20d));
                for (int i = 0; i < 80; i++) {
                    doc.add(new Div(60d, 12d)
                        .setBackgroundColor(200, 220, 240)
                        .setFloat(AFloat.center)
                        .setMargin(2d).setBorder(0.3d));
                }
            }

            Path p = outDir.resolve("gen_13_riding_seal.ofd");
            try (org.ofdrw.reader.OFDReader reader = new org.ofdrw.reader.OFDReader(baseP);
                 org.ofdrw.sign.OFDSigner signer = new org.ofdrw.sign.OFDSigner(reader, p)) {
                org.ofdrw.sign.signContainer.SESV4Container container =
                    new org.ofdrw.sign.signContainer.SESV4Container(prvKey, seal, signCert);
                signer.setSignMode(org.ofdrw.sign.SignMode.WholeProtected);
                signer.setSignContainer(container);
                signer.addApPos(new org.ofdrw.sign.stamppos.RidingStampPos(
                    org.ofdrw.sign.stamppos.Side.Right, 40.0, 40, 40));
                signer.exeSign();
            }
            Files.deleteIfExists(baseP);
            System.out.println("  [13/14] gen_13_riding_seal.ofd");
        } catch (Exception e) {
            System.out.println("  [13/14] SKIP gen_13_riding_seal（签名失败: " + e.getMessage() + "）");
        }
    }

    // ─── 14. 数字签名（条件生成） ───────────────────────────────────────────
    void generateDigitalSign() throws IOException {
        Path p12 = Paths.get("ofdrw-sign/src/test/resources/USER.p12");
        Path esl = Paths.get("ofdrw-sign/src/test/resources/UserV4.esl");
        if (!Files.exists(p12) || !Files.exists(esl)) {
            System.out.println("  [14/14] SKIP gen_14_digital_sign（签名资源不存在）");
            return;
        }

        try {
            java.security.PrivateKey prvKey = org.ofdrw.gm.cert.PKCS12Tools.ReadPrvKey(p12, "private", "777777");
            java.security.cert.Certificate signCert = org.ofdrw.gm.cert.PKCS12Tools.ReadUserCert(p12, "private", "777777");
            org.ofdrw.gm.ses.v4.SESeal seal = org.ofdrw.gm.ses.v4.SESeal.getInstance(Files.readAllBytes(esl));

            // 生成基础文档
            Path baseP = outDir.resolve("_base_for_digital.ofd");
            try (OFDDoc doc = new OFDDoc(baseP)) {
                doc.add(new Paragraph("数字签名测试文档").setFontSize(14d));
                doc.add(new Paragraph("此文档包含 SES V4 数字签名。"));
                doc.add(new Paragraph("签名模式: WholeProtected"));
            }

            Path p = outDir.resolve("gen_14_digital_sign.ofd");
            try (org.ofdrw.reader.OFDReader reader = new org.ofdrw.reader.OFDReader(baseP);
                 org.ofdrw.sign.OFDSigner signer = new org.ofdrw.sign.OFDSigner(reader, p)) {
                org.ofdrw.sign.signContainer.SESV4Container container =
                    new org.ofdrw.sign.signContainer.SESV4Container(prvKey, seal, signCert);
                signer.setSignMode(org.ofdrw.sign.SignMode.WholeProtected);
                signer.setSignContainer(container);
                signer.addApPos(new org.ofdrw.sign.stamppos.NormalStampPos(1, 70d, 100d, 40d, 40d));
                signer.exeSign();
            }
            Files.deleteIfExists(baseP);
            System.out.println("  [14/14] gen_14_digital_sign.ofd");
        } catch (Exception e) {
            System.out.println("  [14/14] SKIP gen_14_digital_sign（签名失败: " + e.getMessage() + "）");
        }
    }

    // ─── 工具方法：创建测试 PNG ─────────────────────────────────────────────
    private void createTestPng(Path path) throws IOException {
        java.awt.image.BufferedImage img =
            new java.awt.image.BufferedImage(80, 60, java.awt.image.BufferedImage.TYPE_INT_RGB);
        java.awt.Graphics2D g = img.createGraphics();
        g.setColor(new java.awt.Color(100, 150, 255));
        g.fillRect(0, 0, 80, 60);
        g.setColor(new java.awt.Color(255, 50, 50));
        g.fillRect(20, 15, 40, 30);
        g.setColor(java.awt.Color.WHITE);
        g.setFont(new java.awt.Font("SansSerif", java.awt.Font.BOLD, 10));
        g.drawString("TEST", 28, 35);
        g.dispose();
        javax.imageio.ImageIO.write(img, "png", path.toFile());
    }

    // ─── 主入口 ──────────────────────────────────────────────────────────────
    public static void main(String[] args) throws Exception {
        Path outDir = Paths.get(args.length > 0 ? args[0] : "samples");
        System.out.println("OF-B 样本生成器 —— 输出目录: " + outDir.toAbsolutePath());

        GenerateSamples gen = new GenerateSamples(outDir);
        gen.generateDocInfoFull();
        gen.generateMultiPage();
        gen.generateComplexText();
        gen.generateWithImage();
        gen.generateCanvasDrawing();
        gen.generateMultiVirtualPage();
        gen.generateCustomFont();
        gen.generateAnnotations();
        gen.generateFloatLayout();
        gen.generateVirtualPagePositioning();
        gen.generatePageOverflow();
        gen.generateWithAttachment();
        gen.generateRidingSeal();
        gen.generateDigitalSign();

        long count = Files.list(outDir)
            .filter(fpath -> fpath.toString().endsWith(".ofd"))
            .count();
        System.out.println("\n生成完成！共 " + count + " 个 OFD 样本");
    }
}
JAVA_EOF
    log_info "GenerateSamples.java 写入完成"
}

# ─── 编译并运行生成器 ────────────────────────────────────────────────────────

compile_and_run_generator() {
    log_info "编译 GenerateSamples.java..."
    cd "$CLONE_DIR"

    # 收集 classpath：ofdrw-layout + 传递依赖
    local layout_jar
    layout_jar=$(find ofdrw-layout/target -name "ofdrw-layout-*.jar" -not -name "*sources*" -not -name "*javadoc*" | head -1)
    local sign_jar
    sign_jar=$(find ofdrw-sign/target -name "ofdrw-sign-*.jar" -not -name "*sources*" -not -name "*javadoc*" | head -1)
    local gm_jar
    gm_jar=$(find ofdrw-gm/target -name "ofdrw-gm-*.jar" -not -name "*sources*" -not -name "*javadoc*" | head -1)
    local reader_jar
    reader_jar=$(find ofdrw-reader/target -name "ofdrw-reader-*.jar" -not -name "*sources*" -not -name "*javadoc*" | head -1)

    # 用 Maven dependency:build-classpath 获取传递依赖
    local cp
    cp=$(mvn -B -pl ofdrw-layout dependency:build-classpath -q -DincludeScope=runtime -Dmdep.outputFile=/dev/stdout 2>/dev/null || echo "")

    local full_cp="${layout_jar}:${sign_jar:-}:${gm_jar:-}:${reader_jar}:${cp}"

    # 编译
    javac -cp "$full_cp" GenerateSamples.java 2>&1 || {
        log_warn "首次编译失败，尝试降级（去掉签名功能）..."
        compile_and_run_generator_lite "$full_cp"
        return
    }

    # 运行
    log_info "运行生成器..."
    mkdir -p "$SAMPLES_DIR"
    java -Djava.awt.headless=true -cp "$full_cp:$CLONE_DIR" GenerateSamples "$SAMPLES_DIR" 2>&1 || {
        log_error "运行生成器失败"
        exit 1
    }
}

# ─── 降级编译（不含签名） ────────────────────────────────────────────────────

compile_and_run_generator_lite() {
    local base_cp="$1"
    cd "$CLONE_DIR"

    # 用 sed 去掉签名相关方法体，替换为空操作
    python3 -c "
import re
with open('GenerateSamples.java', 'r') as f:
    content = f.read()
# 替换 generateRidingSeal 方法体
content = re.sub(
    r'void generateRidingSeal\(\) throws IOException \{.*?^\s{4}\}',
    'void generateRidingSeal() throws IOException {\n        System.out.println(\"  [13/14] SKIP gen_13_riding_seal（降级模式）\");\n    }',
    content, flags=re.DOTALL | re.MULTILINE)
# 替换 generateDigitalSign 方法体
content = re.sub(
    r'void generateDigitalSign\(\) throws IOException \{.*?^\s{4}\}',
    'void generateDigitalSign() throws IOException {\n        System.out.println(\"  [14/14] SKIP gen_14_digital_sign（降级模式）\");\n    }',
    content, flags=re.DOTALL | re.MULTILINE)
with open('GenerateSamples.java', 'w') as f:
    f.write(content)
" 2>/dev/null || {
        # 如果 python3 不可用，用 awk 做简单替换
        log_warn "python3 不可用，跳过签名功能"
    }

    javac -cp "$base_cp" GenerateSamples.java 2>&1 || {
        log_error "降级编译也失败了"
        exit 1
    }

    mkdir -p "$SAMPLES_DIR"
    java -Djava.awt.headless=true -cp "$base_cp:$CLONE_DIR" GenerateSamples "$SAMPLES_DIR" 2>&1 || {
        log_error "降级运行也失败了"
        exit 1
    }
}

# ─── 复制样本到目标目录 ──────────────────────────────────────────────────────

copy_samples() {
    log_info "复制样本到 $DEST_DIR ..."
    mkdir -p "$DEST_DIR"

    local count=0
    for f in "$SAMPLES_DIR"/*.ofd; do
        if [ -f "$f" ]; then
            cp "$f" "$DEST_DIR/"
            count=$((count + 1))
        fi
    done

    if [ "$count" -eq 0 ]; then
        log_error "没有生成任何 .ofd 样本"
        exit 1
    fi

    log_info "已复制 $count 个样本到 $DEST_DIR"
}

# ─── 生成摘要 ────────────────────────────────────────────────────────────────

print_summary() {
    echo ""
    echo "════════════════════════════════════════════════════════════════"
    echo "  OF-B 样本生成完成"
    echo "════════════════════════════════════════════════════════════════"
    echo ""
    echo "  输出目录: $DEST_DIR"
    echo "  样本列表:"
    ls -la "$DEST_DIR"/*.ofd 2>/dev/null | awk '{print "    " $NF " (" $5 " bytes)"}'
    echo ""
    echo "  总计: $(ls "$DEST_DIR"/*.ofd 2>/dev/null | wc -l | tr -d ' ') 个样本"
    echo ""
    echo "════════════════════════════════════════════════════════════════"
}

# ─── 主流程 ──────────────────────────────────────────────────────────────────

main() {
    log_info "OF-B 样本生成流程启动"
    echo ""

    check_prerequisites
    clone_or_update_ofdrw
    build_ofdrw
    write_generator_java
    compile_and_run_generator
    copy_samples
    print_summary

    log_info "OF-B 样本生成流程完成"
}

main "$@"
