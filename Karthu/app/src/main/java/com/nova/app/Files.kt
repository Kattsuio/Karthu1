package com.nova.app

import android.content.ContentValues
import android.content.Context
import android.graphics.Paint
import android.graphics.pdf.PdfDocument
import android.net.Uri
import android.provider.MediaStore
import android.provider.OpenableColumns
import android.webkit.MimeTypeMap
import java.io.ByteArrayOutputStream
import java.util.zip.ZipEntry
import java.util.zip.ZipInputStream
import java.util.zip.ZipOutputStream

object Files {
    val FILE_BLOCK = Regex("```file:([^\\n]+)\\n(.*?)```", RegexOption.DOT_MATCHES_ALL)

    fun mime(name: String) = MimeTypeMap.getSingleton()
        .getMimeTypeFromExtension(name.substringAfterLast('.', "").lowercase()) ?: "application/octet-stream"

    private fun esc(s: String) = s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")

    private fun pdf(text: String): ByteArray {
        val doc = PdfDocument(); val p = Paint().apply { textSize = 11f }
        val lines = text.lines().flatMap { l ->
            if (l.isEmpty()) listOf("") else l.chunked(90)
        }
        var i = 0; var n = 1
        do {
            val page = doc.startPage(PdfDocument.PageInfo.Builder(595, 842, n++).create())
            var y = 50f
            while (i < lines.size && y < 800f) { page.canvas.drawText(lines[i++], 40f, y, p); y += 15f }
            doc.finishPage(page)
        } while (i < lines.size)
        val o = ByteArrayOutputStream(); doc.writeTo(o); doc.close(); return o.toByteArray()
    }

    private fun docx(text: String): ByteArray {
        val bos = ByteArrayOutputStream()
        ZipOutputStream(bos).use { z ->
            fun put(n: String, s: String) { z.putNextEntry(ZipEntry(n)); z.write(s.toByteArray()); z.closeEntry() }
            put("[Content_Types].xml", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>")
            put("_rels/.rels", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>")
            val paras = text.lines().joinToString("") { "<w:p><w:r><w:t xml:space=\"preserve\">${esc(it)}</w:t></w:r></w:p>" }
            put("word/document.xml", "<?xml version=\"1.0\" encoding=\"UTF-8\"?><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>$paras</w:body></w:document>")
        }
        return bos.toByteArray()
    }

    fun render(name: String, content: String): ByteArray = when (name.substringAfterLast('.', "").lowercase()) {
        "pdf" -> pdf(content)
        "docx" -> docx(content)
        else -> content.toByteArray()
    }

    fun save(ctx: Context, name: String, content: String): Uri? {
        val safe = name.trim().replace(Regex("[^A-Za-z0-9._-]"), "_")
        val v = ContentValues().apply {
            put(MediaStore.Downloads.DISPLAY_NAME, safe)
            put(MediaStore.Downloads.MIME_TYPE, mime(safe))
            put(MediaStore.Downloads.RELATIVE_PATH, "Download/Karthu")
        }
        val uri = ctx.contentResolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, v) ?: return null
        ctx.contentResolver.openOutputStream(uri)?.use { it.write(render(safe, content)) }
        return uri
    }

    /** Lê um anexo como texto (texto puro, código, csv, json, docx...). */
    fun read(ctx: Context, uri: Uri): Pair<String, String> {
        var name = "arquivo"
        ctx.contentResolver.query(uri, null, null, null, null)?.use {
            if (it.moveToFirst()) { val i = it.getColumnIndex(OpenableColumns.DISPLAY_NAME); if (i >= 0) name = it.getString(i) }
        }
        val bytes = ctx.contentResolver.openInputStream(uri)?.use { it.readBytes() } ?: ByteArray(0)
        val text = when {
            name.endsWith(".docx", true) -> docxText(bytes)
            bytes.take(2000).any { it.toInt() == 0 } -> "[formato binário não suportado para leitura]"
            else -> String(bytes)
        }
        return name to text.take(60000)
    }

    private fun docxText(b: ByteArray): String {
        ZipInputStream(b.inputStream()).use { z ->
            var e = z.nextEntry
            while (e != null) {
                if (e.name == "word/document.xml") {
                    return String(z.readBytes()).replace("</w:p>", "\n").replace(Regex("<[^>]+>"), "")
                }
                e = z.nextEntry
            }
        }
        return ""
    }
}
