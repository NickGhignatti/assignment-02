import java.util.ArrayList;
import java.util.HashSet;
import java.util.Set;

class Test {
    private Set mySet = new HashSet<Integer>();
    private Integer mySetaa = 0;

    class Test2 {
        private Boolean isRunning = true;

        class Test3{
            private Integer count = 0;

            void anotherTest() {
                System.out.println("Hello from Test3");
            }
        }

        void test() {
            System.out.println("Hello from Test2");
        }
    }

    class Test4 {
        
    }
    
    public static void main(String[] args) {
        System.out.println("Hello, World!");
    }
}

class TestX {}