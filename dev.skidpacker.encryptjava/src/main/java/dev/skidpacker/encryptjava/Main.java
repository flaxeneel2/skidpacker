package dev.skidpacker.encryptjava;

import javax.crypto.BadPaddingException;
import javax.crypto.Cipher;
import javax.crypto.IllegalBlockSizeException;
import javax.crypto.NoSuchPaddingException;
import javax.crypto.spec.SecretKeySpec;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.security.InvalidKeyException;
import java.security.Key;
import java.security.NoSuchAlgorithmException;

/*
This is the encryption program created in java before it is replicated in Rust.

Purpose of the program: A small program which intakes a File and outputs an
AES file based on a key specified and hardcoded in Java.
 */

public class Main {

    // Ignore any bad code...
    public static void main(String[] args) {
        String key = "1111111111111111";
        File inputFile = new File("document.txt");
        File encryptedFile = new File("document.encrypted");
        File decryptedFile = new File("document.decrypted");

        try {
            encrypt(key, inputFile, encryptedFile); // Encrypt the file.
            decrypt(key, encryptedFile, decryptedFile); // Decrypt the file.
        } catch (Exception e) {
            Console.info(e.getMessage(), Console.consoleNotificationType.ERROR);
        }
    }

    /**
     * Applies the encryption.
     * @param key - The 8 bit encryption key.
     * @param inputFile - The input file.
     * @param outputFile - The output file.
     * @throws Exception - Any errors whilst encryption will be thrown.
     */
    public static void encrypt(String key, File inputFile, File outputFile) throws Exception {
        executeAES(Cipher.ENCRYPT_MODE, key, inputFile, outputFile);
    }

    /**
     * Applies the decryption.
     * @param key - The 8 bit encryption key.
     * @param inputFile - The input file.
     * @param outputFile - The output file.
     * @throws Exception - Any errors whilst decrypting will be thrown.
     */
    public static void decrypt(String key, File inputFile, File outputFile) throws Exception {
        executeAES(Cipher.DECRYPT_MODE, key, inputFile, outputFile);
    }

    /**
     * The method which is responsible for performing the AES encryption on the file.
     *
     * @param cipherMode
     * @param key
     * @param inputFile
     * @param outputFile
     * @throws Exception - Displays an error if there is a problem while executing the AES encryption.
     */
    private static void executeAES(int cipherMode, String key, File inputFile, File outputFile) throws Exception {
        try {
            Key secretKeySpec = new SecretKeySpec(key.getBytes(), "AES");
            Cipher cipher = Cipher.getInstance("AES");
            cipher.init(cipherMode, secretKeySpec);

            FileInputStream inputStream = new FileInputStream(inputFile);
            byte[] inputBytes = new byte[(int) inputFile.length()];
            inputStream.read(inputBytes);

            byte[] outputBytes = cipher.doFinal(inputBytes);

            FileOutputStream fileOutputStream = new FileOutputStream(outputFile);
            fileOutputStream.write(outputBytes);

            inputStream.close();
            fileOutputStream.close();

        } catch (NoSuchPaddingException | NoSuchAlgorithmException
                 | InvalidKeyException | BadPaddingException
                 | IllegalBlockSizeException | IOException exception) {
            Console.info(exception.getMessage(), Console.consoleNotificationType.ERROR);
            throw new Exception("[SKIDENCRYPT] Error encrypting/decrypting file!", exception);
        }
    }
}
