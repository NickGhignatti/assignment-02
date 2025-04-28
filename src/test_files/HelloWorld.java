import java.util.ArrayList;
import java.util.Set;

class Test {
    private Set mySet = new HashSet<Integer>();
    private HashSet mySet = new HashSet<Integer>();

    class Test2 {

        class Test3{
            void anotherTest() {
                System.out.println("Hello from Test3");
            }
        }

        void test() {
            System.out.println("Hello from Test2");
        }
    }
    
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}

class TestX {}