import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.Security;
import java.security.cert.X509Certificate;
import java.io.ByteArrayInputStream;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.bouncycastle.cms.CMSSignedDataGenerator;
import org.bouncycastle.cms.CMSProcessableByteArray;
import org.bouncycastle.cms.jcajce.JcaSignerInfoGeneratorBuilder;
import org.bouncycastle.cert.jcajce.JcaCertStore;
import org.bouncycastle.operator.jcajce.JcaContentSignerBuilder;
import org.bouncycastle.operator.jcajce.JcaDigestCalculatorProviderBuilder;
import org.ofdrw.sign.signContainer.GBT35275DSContainer;
import org.ofdrw.sign.signContainer.GBT35275PKCS9DSContainer;
import org.ofdrw.gm.sm2strut.GBT35275Validate;
/** 公开身份的GB35275/标准CMS真实互操作，非生产凭据。 */
public class Gbt35275Interop {
    public static void main(String[] args) throws Exception {
        Security.addProvider(new BouncyCastleProvider());
        Path source=Path.of(args[1]);
        Path out=Path.of(args[2]);
        byte[] message=Files.readAllBytes(source.resolve("message.bin"));
        KeyStore store=KeyStore.getInstance("PKCS12","BC");
        store.load(new ByteArrayInputStream(Files.readAllBytes(source.resolve("rust-public-test.p12"))),"public-test".toCharArray());
        String alias=store.aliases().nextElement();
        PrivateKey key=(PrivateKey)store.getKey(alias,"public-test".toCharArray());
        X509Certificate cert=(X509Certificate)store.getCertificate(alias);
        Files.createDirectories(out);
        if ("generate".equals(args[0])) {
            Files.write(out.resolve("direct.der"),new GBT35275DSContainer(cert,key).sign(new ByteArrayInputStream(message),""));
            Files.write(out.resolve("pkcs9.der"),new GBT35275PKCS9DSContainer(cert,key).sign(new ByteArrayInputStream(message),""));
            CMSSignedDataGenerator generator=new CMSSignedDataGenerator();
            generator.addSignerInfoGenerator(new JcaSignerInfoGeneratorBuilder(new JcaDigestCalculatorProviderBuilder().setProvider("BC").build()).build(new JcaContentSignerBuilder("SM3withSM2").setProvider("BC").build(key),cert));
            generator.addCertificates(new JcaCertStore(java.util.Collections.singletonList(cert)));
            Files.write(out.resolve("cms.der"),generator.generate(new CMSProcessableByteArray(message),true).getEncoded("DER"));
        }
        for (String name:new String[]{"direct","pkcs9"}) {
            byte[] value=Files.readAllBytes(out.resolve(name+".der"));
            if (!GBT35275Validate.validate("SM3withSM2",message,value).result) {
                throw new AssertionError("GB35275验签失败:"+name);
            }
            if (GBT35275Validate.validate("SM3withSM2",new byte[]{1},value).result) {
                throw new AssertionError("篡改原文未拒绝");
            }
            System.out.println(name+": real verification and tamper rejection passed");
        }
    }
}
