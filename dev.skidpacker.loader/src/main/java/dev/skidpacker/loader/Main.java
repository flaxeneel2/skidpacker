package dev.skidpacker.loader;

/*
The java loader program that a user would be expected to run in order for the native
module to work.
 */
public class Main {

    // The assumed init function in the native module.
    public static native void init();

    public static void main(String[] args) {
        init();
    }

    static {
        /*
        - The library will be called libskidpacker.
         */
        System.loadLibrary("resources/skidpacker");
    }
}