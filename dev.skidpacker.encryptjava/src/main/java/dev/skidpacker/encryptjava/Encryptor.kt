package dev.skidpacker.encryptjava

import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.security.Key
import javax.crypto.Cipher
import javax.crypto.spec.SecretKeySpec

/*
This is the encryption program created in java before it is replicated in Rust.

Purpose of the program: A small program which intakes a File and outputs an
AES file based on a key specified and hardcoded in Java.
*/

class Encryptor(
    private val inputJar: File,
    private val outputJar: File,
    private val settings: Settings,
) {

    fun run() {
        val key = settings.encryptionKey
        val encryptedFile = File(outputJar.name + "encrypted")
        val decryptedFile = File(outputJar.name + "decrypted")
        try {
            encrypt(key, inputJar, encryptedFile) // Encrypt the file.
            decrypt(key, encryptedFile, decryptedFile) // Decrypt the file.
        } catch (e: Exception) {
            Console.info(e.message, Console.consoleNotificationType.ERROR)
        }
    }

    /**
     * Applies the encryption.
     * @param key - The 8 bit encryption key.
     * @param inputFile - The input file.
     * @param outputFile - The output file.
     * @throws Exception - Any errors whilst encryption will be thrown.
     */
    @Throws(Exception::class)
    fun encrypt(key: String, inputFile: File, outputFile: File) {
        executeAES(Cipher.ENCRYPT_MODE, key, inputFile, outputFile)
    }

    /**
     * Applies the decryption.
     * @param key - The 8 bit encryption key.
     * @param inputFile - The input file.
     * @param outputFile - The output file.
     * @throws Exception - Any errors whilst decrypting will be thrown.
     */
    @Throws(Exception::class)
    fun decrypt(key: String, inputFile: File, outputFile: File) {
        executeAES(Cipher.DECRYPT_MODE, key, inputFile, outputFile)
    }

    /**
     * The method which is responsible for performing the AES encryption on the file.
     *
     * @param cipherMode - The cipher mode.
     * @param key - The 8 bit encryption key.
     * @param inputFile - The input file.
     * @param outputFile - The output file.
     * @throws Exception - Displays an error if there is a problem while executing the AES encryption.
     */
    @Throws(Exception::class)
    private fun executeAES(cipherMode: Int, key: String, inputFile: File, outputFile: File) {
        try {
            val secretKeySpec: Key = SecretKeySpec(key.toByteArray(), "AES")
            val cipher = Cipher.getInstance("AES")
            cipher.init(cipherMode, secretKeySpec)
            val inputStream = FileInputStream(inputFile)
            val inputBytes = ByteArray(inputFile.length().toInt())
            inputStream.read(inputBytes)
            val outputBytes = cipher.doFinal(inputBytes)
            val fileOutputStream = FileOutputStream(outputFile)
            fileOutputStream.write(outputBytes)
            inputStream.close()
            fileOutputStream.close()
        } catch (exception: java.lang.Exception) {
            Console.info(exception.message, Console.consoleNotificationType.ERROR)
            throw Exception("[SKIDENCRYPT] Error encrypting/decrypting file!", exception)
        }
    }
}