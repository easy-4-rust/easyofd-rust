import java.nio.file.*;
import java.security.*;
import java.security.cert.Certificate;
import java.util.*;
import java.util.Objects;
import java.io.*;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.bouncycastle.jce.interfaces.ECPublicKey;
import org.bouncycastle.jce.interfaces.ECPrivateKey;
import org.bouncycastle.jcajce.spec.SM2ParameterSpec;
import org.ofdrw.core.signatures.SigType;
import org.ofdrw.sign.signContainer.*;
import org.ofdrw.sign.verify.container.*;
/** 使用公开上游测试身份进行真实BC1.85互操作，不用于生产。 */
public final class BC185Interop {
    private static final byte[] MESSAGE="BC1.85 OFDRW interoperability".getBytes(java.nio.charset.StandardCharsets.UTF_8);
    private static final String PASSWORD="public-test";
    public static void main(String[] args)throws Exception {
        Security.addProvider(new BouncyCastleProvider());
        if(!Security.getProvider("BC").getVersionStr().equals("1.85"))throw new IllegalStateException("BC must be 1.85");
        Path source=Paths.get(args[1]),out=Paths.get(args[2]);Files.createDirectories(out);
        if(args[0].equals("generate"))generate(source,out);else verify(out);
    }
    private static void generate(Path source,Path out)throws Exception {
        Path res=source.resolve("ofdrw-sign/src/test/resources");
        KeyStore ks=KeyStore.getInstance("PKCS12","BC");
        try(InputStream in=Files.newInputStream(res.resolve("USER.p12"))){ks.load(in,"777777".toCharArray());}
        String alias=ks.aliases().nextElement();PrivateKey key=(PrivateKey)ks.getKey(alias,"777777".toCharArray());Certificate cert=ks.getCertificate(alias);
        Files.write(out.resolve("cert.der"),cert.getEncoded());
        Files.write(out.resolve("public.sec1"),((ECPublicKey)cert.getPublicKey()).getQ().getEncoded(false));
        byte[] scalar=((ECPrivateKey)key).getD().toByteArray();byte[] secret=new byte[32];System.arraycopy(scalar,Math.max(0,scalar.length-32),secret,Math.max(0,32-scalar.length),Math.min(32,scalar.length));
        Files.write(out.resolve("test-secret.bin"),secret);Files.write(out.resolve("message.bin"),MESSAGE);
        KeyStore generated=KeyStore.getInstance("PKCS12","BC");generated.load(null,null);generated.setKeyEntry("public-test",key,PASSWORD.toCharArray(),new Certificate[]{cert});
        try(OutputStream os=Files.newOutputStream(out.resolve("public-test.p12"))){generated.store(os,PASSWORD.toCharArray());}
        Signature signature=Signature.getInstance("SM3WithSM2","BC");signature.initSign(key);signature.update(MESSAGE);Files.write(out.resolve("sm2.der"),signature.sign());
        byte[] s1=Files.readAllBytes(res.resolve("UserV1.esl"));byte[] s4=Files.readAllBytes(res.resolve("UserV4.esl"));byte[] s5=Files.readAllBytes(res.resolve("UserV5.esl"));
        Files.write(out.resolve("seal-v1.der"),s1);Files.write(out.resolve("seal-v4.der"),s4);Files.write(out.resolve("seal-v5.der"),s5);
        Files.write(out.resolve("ses-v1.der"),new SESV1Container(key,org.ofdrw.gm.ses.v1.SESeal.getInstance(s1),cert).sign(new ByteArrayInputStream(MESSAGE),"interop"));
        Files.write(out.resolve("ses-v4.der"),new SESV4Container(key,org.ofdrw.gm.ses.v4.SESeal.getInstance(s4),cert).sign(new ByteArrayInputStream(MESSAGE),"interop"));
        Files.write(out.resolve("ses-v5.der"),new SESV5Container(key,org.ofdrw.gm.ses.v5.SESeal.getInstance(s5),cert).sign(new ByteArrayInputStream(MESSAGE),"interop"));
        verify(out);
        System.out.println("BC 1.85 Java fixtures generated and cryptographically verified");
    }
    private static void verify(Path out)throws Exception {
        byte[] message=Files.readAllBytes(out.resolve("message.bin"));
        Certificate cert=java.security.cert.CertificateFactory.getInstance("X.509","BC").generateCertificate(new ByteArrayInputStream(Files.readAllBytes(out.resolve("cert.der"))));
        KeyStore pfx=KeyStore.getInstance("PKCS12","BC");
        try(InputStream in=Files.newInputStream(out.resolve("public-test.p12"))){pfx.load(in,PASSWORD.toCharArray());}
        String alias=pfx.aliases().nextElement();
        Certificate[] chain=pfx.getCertificateChain(alias);
        if(Objects.isNull(chain)||chain.length==0||!Arrays.equals(chain[0].getEncoded(),cert.getEncoded()))throw new IllegalStateException("PFX certificate does not match signing certificate");
        Signature keyCheck=Signature.getInstance("SM3WithSM2","BC");keyCheck.initSign((PrivateKey)pfx.getKey(alias,PASSWORD.toCharArray()));keyCheck.update(message);byte[] challenge=keyCheck.sign();
        keyCheck.initVerify(cert);keyCheck.update(message);if(!keyCheck.verify(challenge))throw new IllegalStateException("PFX private key/certificate mismatch");
        boolean badPasswordRejected=false;
        try(InputStream in=Files.newInputStream(out.resolve("public-test.p12"))){KeyStore bad=KeyStore.getInstance("PKCS12","BC");bad.load(in,"wrong-password".toCharArray());}catch(Exception expected){badPasswordRejected=true;}
        if(!badPasswordRejected)throw new IllegalStateException("PFX wrong password accepted");
        System.out.println("PFX loaded; private key matches certificate; wrong password rejected");
        Signature signature=Signature.getInstance("SM3WithSM2","BC");signature.initVerify(cert);signature.update(message);
        if(!signature.verify(Files.readAllBytes(out.resolve("sm2.der"))))throw new IllegalStateException("SM2 verify failed");
        for(int v:new int[]{1,4,5}) {
            org.ofdrw.sign.verify.SignedDataValidateContainer verifier=v==1?new SESV1ValidateContainer():v==4?new SESV4ValidateContainer():new SESV5ValidateContainer();
            byte[] value=Files.readAllBytes(out.resolve("ses-v"+v+".der"));verifier.validate(SigType.Seal,"SM3WithSM2",message,value);
            byte[] bad=message.clone();bad[0]^=1;boolean rejected=false;
            try{verifier.validate(SigType.Seal,"SM3WithSM2",bad,value);}catch(Exception e){rejected=true;}
            if(!rejected)throw new IllegalStateException("SES tamper accepted: "+v);
            System.out.println("SES V"+v+" verified; tamper rejected");
        }
        signature=Signature.getInstance("SM3WithSM2","BC");signature.setParameter(new SM2ParameterSpec("wrong-id".getBytes()));signature.initVerify(cert);signature.update(message);
        if(signature.verify(Files.readAllBytes(out.resolve("sm2.der"))))throw new IllegalStateException("Wrong ID accepted");
    }
}
