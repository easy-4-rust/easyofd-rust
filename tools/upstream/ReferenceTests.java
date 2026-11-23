import java.nio.file.*;
import java.lang.reflect.*;
/** 在无JUnit引擎的参考构建中执行上游真实测试方法，显式注入TempDir。 */
public final class ReferenceTests {
    public static void main(String[] args)throws Exception {
        int count=0;
        for(String name:new String[]{"org.ofdrw.converter.utils.PointUtilTest","org.ofdrw.converter.export.TextStrokeExportTest"}) {
            Class<?> type=Class.forName(name);Constructor<?> ctor=type.getDeclaredConstructor();ctor.setAccessible(true);
            for(Method method:type.getDeclaredMethods()) {
                if(!method.isAnnotationPresent(org.junit.jupiter.api.Test.class))continue;
                Object test=ctor.newInstance();
                for(Field field:type.getDeclaredFields()) {
                    if(field.isAnnotationPresent(org.junit.jupiter.api.io.TempDir.class)){field.setAccessible(true);field.set(test,Files.createTempDirectory("ofdrw-reference-test-"));}
                }
                method.setAccessible(true);
                try{method.invoke(test);}catch(InvocationTargetException e){throw new AssertionError(name+"#"+method.getName(),e.getCause());}
                count++;System.out.println("PASS "+name+"#"+method.getName());
            }
        }
        if(count<5)throw new IllegalStateException("Expected upstream regression tests to run");
        System.out.println("Upstream real test methods passed: "+count);
    }
}
