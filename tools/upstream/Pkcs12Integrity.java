import java.io.ByteArrayInputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.security.Security;
import java.security.Signature;
import java.util.Objects;
import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.bouncycastle.pkcs.PKCS12PfxPdu;
import org.bouncycastle.pkcs.bc.BcPKCS12MacCalculatorBuilderProvider;
import org.bouncycastle.operator.bc.BcDefaultDigestProvider;

/** BC1.85独立加载RFC7292 MAC变体；仅使用公开测试身份。 */
public class Pkcs12Integrity {
    public static void main(String[] args) throws Exception {
        Security.addProvider(new BouncyCastleProvider());
        for (String name : new String[]{"sha1", "sha224", "sha256", "sha384", "sha512", "sha512_224", "sha512_256"}) {
            byte[] data = Files.readAllBytes(Path.of(args[0], name + ".p12"));
            PKCS12PfxPdu pfx = new PKCS12PfxPdu(data);
            BcPKCS12MacCalculatorBuilderProvider builder = new BcPKCS12MacCalculatorBuilderProvider(algorithm -> {
                String oid = algorithm.getAlgorithm().getId();
                if ("2.16.840.1.101.3.4.2.5".equals(oid)) {
                    return new org.bouncycastle.crypto.digests.SHA512tDigest(224);
                }
                if ("2.16.840.1.101.3.4.2.6".equals(oid)) {
                    return new org.bouncycastle.crypto.digests.SHA512tDigest(256);
                }
                return BcDefaultDigestProvider.INSTANCE.get(algorithm);
            });
            if (!pfx.isMacValid(builder, "public-test".toCharArray()) || pfx.isMacValid(builder, "wrong".toCharArray())) {
                throw new AssertionError("MAC正例或错误口令反例失败");
            }
            KeyStore store = KeyStore.getInstance("PKCS12", "BC");
            try {
                store.load(new ByteArrayInputStream(data), "public-test".toCharArray());
            } catch (java.io.IOException error) {
                if (error.getCause() instanceof java.security.NoSuchAlgorithmException) {
                    System.out.println(name + ": BC MAC verified, wrong password rejected; KeyStore algorithm unavailable");
                    continue;
                }
                throw error;
            }
            String alias = store.aliases().nextElement();
            java.security.PrivateKey key = (java.security.PrivateKey) store.getKey(alias, "public-test".toCharArray());
            if (Objects.isNull(key) || Objects.isNull(store.getCertificate(alias))) {
                throw new AssertionError("缺少公开测试密钥或证书");
            }
            Signature signature = Signature.getInstance("SM3withSM2", "BC");
            byte[] message = new byte[]{1, 2, 3};
            signature.initSign(key);
            signature.update(message);
            byte[] signed = signature.sign();
            signature.initVerify(store.getCertificate(alias));
            signature.update(message);
            if (!signature.verify(signed)) {
                throw new AssertionError("密钥/证书不匹配");
            }
            boolean rejected = false;
            try {
                store.load(new ByteArrayInputStream(data), "wrong".toCharArray());
            } catch (java.io.IOException expected) {
                rejected = true;
            }
            if (!rejected) {
                throw new AssertionError("MAC错误口令被接受");
            }
            System.out.println(name + ": loaded, key matched, wrong password rejected");
        }
    }
}
