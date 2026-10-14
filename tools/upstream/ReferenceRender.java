import java.nio.file.*;
import org.ofdrw.converter.export.ImageExporter;
import org.ofdrw.converter.export.SVGExporter;
/** 固定上游修复源码的真实输出验证。 */
public final class ReferenceRender {
    public static void main(String[] args)throws Exception {
        Path root=Paths.get(args[0]),out=Paths.get(args[1]);
        for(String name:new String[]{"stroke_false","stroke_omitted","stroke_true","stroke_explicit","stroke_only"}) {
            try(ImageExporter exporter=new ImageExporter(root.resolve(name+".ofd"),out.resolve(name+"-png"))){exporter.export();}
            try(SVGExporter exporter=new SVGExporter(root.resolve(name+".ofd"),out.resolve(name+"-svg"))){exporter.export();}
        }
    }
}
