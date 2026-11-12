package com.nova.app

import org.json.JSONArray
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

data class Source(val title: String, val url: String, val text: String)

object Api {
    /** Chat em streaming para qualquer servidor compatível com /chat/completions (Ollama, llama.cpp, vLLM, LM Studio...). */
    fun stream(baseUrl: String, key: String, model: String, msgs: List<Pair<String, String>>, onDelta: (String) -> Unit) {
        val c = URL(baseUrl.trimEnd('/') + "/chat/completions").openConnection() as HttpURLConnection
        c.requestMethod = "POST"; c.connectTimeout = 15000; c.readTimeout = 180000; c.doOutput = true
        c.setRequestProperty("Content-Type", "application/json")
        if (key.isNotBlank()) c.setRequestProperty("Authorization", "Bearer $key")
        val arr = JSONArray()
        msgs.forEach { arr.put(JSONObject().put("role", it.first).put("content", it.second)) }
        val body = JSONObject().put("model", model).put("stream", true).put("messages", arr)
        c.outputStream.use { it.write(body.toString().toByteArray()) }
        if (c.responseCode !in 200..299) {
            throw Exception("Erro ${c.responseCode}: " + (c.errorStream?.bufferedReader()?.readText() ?: ""))
        }
        c.inputStream.bufferedReader().useLines { lines ->
            for (l in lines) {
                if (!l.startsWith("data:")) continue
                val d = l.removePrefix("data:").trim()
                if (d == "[DONE]") break
                try {
                    val ch = JSONObject(d).getJSONArray("choices")
                    if (ch.length() > 0) {
                        val t = ch.getJSONObject(0).optJSONObject("delta")?.optString("content", "")
                        if (!t.isNullOrEmpty()) onDelta(t)
                    }
                } catch (_: Exception) {}
            }
        }
    }

    private fun get(u: String): String {
        val c = URL(u).openConnection() as HttpURLConnection
        c.connectTimeout = 10000; c.readTimeout = 15000
        c.setRequestProperty("User-Agent", "Karthu/1.0")
        return c.inputStream.bufferedReader().readText()
    }

    /** Pesquisa só em fontes verificáveis: SearXNG (se configurado) ou Wikipédia. */
    fun search(q: String, searx: String): List<Source> = try {
        val e = URLEncoder.encode(q, "UTF-8")
        if (searx.isNotBlank()) {
            val r = JSONObject(get(searx.trimEnd('/') + "/search?q=$e&format=json")).getJSONArray("results")
            (0 until minOf(5, r.length())).map {
                val o = r.getJSONObject(it)
                Source(o.optString("title"), o.optString("url"), o.optString("content"))
            }
        } else {
            val u = "https://pt.wikipedia.org/w/api.php?action=query&generator=search&gsrsearch=$e&gsrlimit=3" +
                "&prop=extracts&exintro=1&explaintext=1&exchars=900&format=json"
            val pages = JSONObject(get(u)).optJSONObject("query")?.optJSONObject("pages")
            val out = ArrayList<Source>()
            if (pages != null) {
                val it = pages.keys()
                while (it.hasNext()) {
                    val o = pages.getJSONObject(it.next())
                    val t = o.optString("title")
                    out.add(Source(t, "https://pt.wikipedia.org/wiki/" + URLEncoder.encode(t.replace(' ', '_'), "UTF-8"), o.optString("extract")))
                }
            }
            out
        }
    } catch (_: Exception) { emptyList() }
}
