package app.karthus

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.content.ContentValues
import android.content.Intent
import android.graphics.Color
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.provider.MediaStore
import android.view.Gravity
import android.view.ViewGroup
import android.webkit.*
import android.widget.FrameLayout
import android.widget.TextView
import android.widget.Toast
import java.net.HttpURLConnection
import java.net.URL
import kotlin.concurrent.thread

class MainActivity : Activity() {
    private lateinit var web: WebView
    private lateinit var msg: TextView
    private var seletor: ValueCallback<Array<Uri>>? = null
    private val base = "http://127.0.0.1:${MotorService.PORTA}"

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(b: Bundle?) {
        super.onCreate(b)
        if (Build.VERSION.SDK_INT >= 33) requestPermissions(arrayOf(Manifest.permission.POST_NOTIFICATIONS), 1)
        val raiz = FrameLayout(this)
        raiz.setBackgroundColor(Color.parseColor("#0D0D0D"))
        web = WebView(this)
        web.setBackgroundColor(Color.parseColor("#0D0D0D"))
        web.visibility = android.view.View.INVISIBLE
        with(web.settings) {
            javaScriptEnabled = true
            domStorageEnabled = true
            mediaPlaybackRequiresUserGesture = false
            allowFileAccess = false
        }
        web.webViewClient = object : WebViewClient() {
            override fun shouldOverrideUrlLoading(v: WebView, r: WebResourceRequest): Boolean {
                val u = r.url.toString()
                if (u.startsWith(base)) return false
                startActivity(Intent(Intent.ACTION_VIEW, r.url)); return true
            }
        }
        web.webChromeClient = object : WebChromeClient() {
            override fun onShowFileChooser(w: WebView, cb: ValueCallback<Array<Uri>>, p: FileChooserParams): Boolean {
                seletor?.onReceiveValue(null); seletor = cb
                return try { startActivityForResult(p.createIntent(), 77); true } catch (e: Exception) { seletor = null; false }
            }
        }
        web.setDownloadListener { url, _, disp, mime, _ -> baixar(url, URLUtil.guessFileName(url, disp, mime), mime) }
        msg = TextView(this).apply { setTextColor(Color.WHITE); textSize = 16f; gravity = Gravity.CENTER; text = "Iniciando o Karthus…" }
        raiz.addView(msg, FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT))
        raiz.addView(web, FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT))
        setContentView(raiz)
        val si = Intent(this, MotorService::class.java)
        if (Build.VERSION.SDK_INT >= 26) startForegroundService(si) else startService(si)
        esperarMotor()
    }

    private fun esperarMotor() = thread {
        val t0 = System.currentTimeMillis()
        while (System.currentTimeMillis() - t0 < 60000) {
            try {
                val c = URL("$base/carregando").openConnection() as HttpURLConnection
                c.connectTimeout = 800; c.readTimeout = 1500
                if (c.responseCode == 200) { runOnUiThread { web.visibility = android.view.View.VISIBLE; msg.visibility = android.view.View.GONE; web.loadUrl("$base/") }; return@thread }
            } catch (_: Exception) {}
            Thread.sleep(400)
        }
        runOnUiThread { msg.text = "O motor do Karthus não iniciou.\nFeche o app e abra de novo." }
    }

    private fun baixar(url: String, nome: String, mime: String?) = thread {
        try {
            val c = URL(url).openConnection() as HttpURLConnection
            val v = ContentValues().apply {
                put(MediaStore.Downloads.DISPLAY_NAME, nome)
                put(MediaStore.Downloads.MIME_TYPE, mime ?: "application/octet-stream")
                put(MediaStore.Downloads.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS + "/Karthus")
            }
            val dest = contentResolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, v) ?: throw Exception("sem destino")
            contentResolver.openOutputStream(dest)!!.use { o -> c.inputStream.use { it.copyTo(o) } }
            runOnUiThread { Toast.makeText(this, "Salvo em Downloads/Karthus: $nome", Toast.LENGTH_LONG).show() }
        } catch (e: Exception) {
            runOnUiThread { Toast.makeText(this, "Não consegui baixar: ${e.message}", Toast.LENGTH_LONG).show() }
        }
    }

    @Deprecated("Deprecated in Java")
    override fun onActivityResult(req: Int, res: Int, data: Intent?) {
        if (req == 77) { seletor?.onReceiveValue(WebChromeClient.FileChooserParams.parseResult(res, data)); seletor = null }
        else super.onActivityResult(req, res, data)
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() { if (web.canGoBack()) web.goBack() else moveTaskToBack(true) }
}
