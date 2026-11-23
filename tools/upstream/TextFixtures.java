import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.zip.*;
/** 上游TextStrokeExportTest/PointUtilTest的确定性输入构造，非输出验收替代。 */
public final class TextFixtures {
    private static void put(ZipOutputStream z,String path,String xml)throws Exception {z.putNextEntry(new ZipEntry(path));z.write(xml.getBytes(StandardCharsets.UTF_8));z.closeEntry();}
    private static void generate(Path out,Path font,String name,String attrs,String text,String coords,String boundary,boolean embedded)throws Exception {
        try(ZipOutputStream z=new ZipOutputStream(Files.newOutputStream(out.resolve(name+".ofd")))) {
            put(z,"OFD.xml","<OFD><DocBody><DocInfo><DocID>test</DocID></DocInfo><DocRoot>Doc_0/Document.xml</DocRoot></DocBody></OFD>");
            put(z,"Doc_0/Document.xml","<Document><CommonData><PublicRes>PublicRes.xml</PublicRes></CommonData><Pages><Page ID=\"1\" BaseLoc=\"Pages/P.xml\"/></Pages></Document>");
            put(z,"Doc_0/PublicRes.xml","<Res BaseLoc=\"Res\"><Fonts><Font ID=\"10\" FontName=\"sysfST\">"+(embedded?"<FontFile>font.ttf</FontFile>":"")+"</Font></Fonts><DrawParams><DrawParam ID=\"11\"><StrokeColor Value=\"255 0 0\"/></DrawParam></DrawParams></Res>");
            put(z,"Doc_0/Pages/P.xml","<Page><Area><PhysicalBox>0 0 "+(name.equals("ab")?"1000 1000":"211.5 140")+"</PhysicalBox></Area><Content><Layer ID=\"3\"><TextObject ID=\"2\" Boundary=\""+boundary+"\" Font=\"10\" Size=\""+(name.equals("vertical")?"3.175":"20")+"\" LineWidth=\"0.15\" "+attrs+">"+((name.equals("stroke_false")||name.equals("stroke_explicit"))?"<StrokeColor Value=\"255 0 0\"/>":"")+"<FillColor Value=\"0 0 0\"/><TextCode "+coords+">"+text+"</TextCode></TextObject></Layer></Content></Page>");
            if(embedded){z.putNextEntry(new ZipEntry("Doc_0/Res/font.ttf"));z.write(Files.readAllBytes(font));z.closeEntry();}
        }
    }
    public static void main(String[] args)throws Exception {
        Path out=Paths.get(args[0]),font=Paths.get(args[1]);Files.createDirectories(out);
        String[] names={"stroke_false","stroke_omitted","stroke_true","stroke_only","stroke_explicit"};
        String[] attrs={"Stroke=\"false\"","DrawParam=\"11\"","Stroke=\"true\" DrawParam=\"11\"","Stroke=\"true\" Fill=\"false\" DrawParam=\"11\"","Stroke=\"true\" DrawParam=\"11\""};
        for(int i=0;i<names.length;i++)generate(out,font,names[i],attrs[i],"/","X=\"0\" Y=\"20\"","10 10 30 30",true);
        generate(out,font,"ab","CTM=\"2 0 0 3 100 200\"","AB","X=\"1\" Y=\"2\" DeltaX=\"4\" DeltaY=\"5\"","10 20 200 300",false);
        generate(out,font,"vertical","CTM=\"0 1 -1 0 0.5 -2\"","下载次数：1","X=\"1.5\" Y=\"0\" DeltaX=\"g 5 3.18\"","207.5 30 20 20",false);
    }
}
