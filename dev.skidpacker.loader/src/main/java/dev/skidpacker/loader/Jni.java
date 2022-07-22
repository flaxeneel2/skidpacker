package dev.skidpacker.loader;

import com.sun.jna.Native;

import java.io.File;

public class Jni {

    static {
        try {
            File file = Native.extractFromResourcePath("loader_jni");
            System.load(file.getAbsolutePath());
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    public native static void init();
}
