package dev.skidpacker.encryptjava

import com.google.gson.*
import java.lang.reflect.ParameterizedType
import java.lang.reflect.Type

class Settings {

    val encryptionKey: String = ""

    companion object {
        private val gson = GsonBuilder()
                .setLenient()
                .setPrettyPrinting()
                .registerTypeAdapter(Regex::class.java, RegexSerializer)
                .registerTypeAdapter(Pair::class.java, PairSerializer)
                .create()

        fun fromJson(json: String): Settings {
            return gson.fromJson(json, Settings::class.java)
        }

        fun toJson(settings: Settings): String {
            return gson.toJson(settings)
        }
    }
}

object RegexSerializer : JsonSerializer<Regex>, JsonDeserializer<Regex> {
    override fun serialize(regex: Regex, ty: Type, ctx: JsonSerializationContext): JsonElement {
        return JsonPrimitive(regex.pattern)
    }

    override fun deserialize(json: JsonElement, ty: Type, ctx: JsonDeserializationContext): Regex {
        return Regex(json.asString)
    }
}

object PairSerializer : JsonSerializer<Pair<Any?, Any?>>, JsonDeserializer<Pair<Any?, Any?>> {
    override fun serialize(pair: Pair<Any?, Any?>, ty: Type, ctx: JsonSerializationContext): JsonElement {
        return JsonArray().also {
            it.add(ctx.serialize(pair.first))
            it.add(ctx.serialize(pair.second))
        }
    }

    override fun deserialize(json: JsonElement, ty: Type, ctx: JsonDeserializationContext): Pair<Any?, Any?> {
        val arr = json.asJsonArray
        val pty = ty as ParameterizedType
        return ctx.deserialize<Any?>(arr[0], pty.actualTypeArguments[0]) to ctx.deserialize<Any?>(
                arr[1],
                pty.actualTypeArguments[1]
        )
    }
}