package dev.skidpacker.encryptjava

import java.io.File

object Main {
    @JvmStatic
    fun main(args: Array<String>) {
        val configName = args.getOrNull(0)
            ?: error("Missing first argument: Config file")

        val inputLocation = args.getOrNull(1) ?: error("Missing second argument: Input jar")
        val outputLocation = args.getOrNull(2) ?: error("Missing third argument: Output jar")

        val jar = File(inputLocation)
        val output = File(outputLocation)

        val configFile = File(configName)
        if (!configFile.exists()) {
            error("Config file not found at '${configFile.absolutePath}'")
        }

        val settings = Settings.fromJson(configFile.readText())
        val encryptor = Encryptor(jar, output, settings)
        encryptor.run()
    }
}