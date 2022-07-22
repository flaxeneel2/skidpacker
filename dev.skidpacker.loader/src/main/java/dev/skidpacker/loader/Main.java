package dev.skidpacker.loader;

/*
The java loader program that a user would be expected to run in order for the native
module to work.
 */
public class Main {

    public static void main(String[] args) {
        Jni.init();
    }
}