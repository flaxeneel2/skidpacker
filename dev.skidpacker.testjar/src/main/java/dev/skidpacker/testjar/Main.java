package dev.skidpacker.testjar;

/*
 * The purpose of this program is to act as a sample jar which
 * test that the classes are delivered correctly with the loader.
 */
public class Main {

    static ThreadedClass threadTest = new ThreadedClass();

    public static void main(String[] args) {
        Console.info(
                "Test jar has executed successfully!",
                Console.consoleNotificationType.INFO
        );
        // Execute thread test.
        threadTest.start();
    }

    // Threaded class test which displays a pane to show that the loader has worked.
    public static class ThreadedClass extends Thread {
        public void run() {
            JPane.showDisplay();
        }
    }
}

