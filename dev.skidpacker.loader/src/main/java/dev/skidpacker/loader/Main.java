package dev.skidpacker.loader;

import com.sun.jna.Native;

import java.io.File;

/*
The java loader program that a user would be expected to run in order for the native
module to work.
 */
public class Main {

    // The init function in the native module.
    public static native void init();

    public static void main(String[] args) {
        init();
    }

    static {
        try {
            File file = Native.extractFromResourcePath("skidpacker");
            System.load(file.getAbsolutePath());
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

}