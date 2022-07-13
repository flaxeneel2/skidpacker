package dev.skidpacker.encryptjava;

/*
- Useless class with a bunch of console formatting.
 */
public class Console {

    private static String consoleName = "SKIDPACKER";

    /*
    Super insanely complex console logging mechanism.
    10/10
     */
    public static void info(String message, consoleNotificationType type) {
        System.out.println("[" + type.toString() + "] " + "[" + consoleName + "] " + message);
    }

    public enum consoleNotificationType {
        INFO,
        WARNING,
        ERROR,
    }
}
