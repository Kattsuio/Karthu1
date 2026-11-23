package app.karthus

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import java.io.File

/** Roda o Karthus (Rust) e, quando preciso, o llama.cpp, como programas nativos dentro do app. */
class MotorService : Service() {
    private var proc: Process? = null
    private var wake: PowerManager.WakeLock? = null

    override fun onBind(i: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        val nm = getSystemService(NotificationManager::class.java)
        if (Build.VERSION.SDK_INT >= 26) nm.createNotificationChannel(NotificationChannel("motor", "Motor do Karthus", NotificationManager.IMPORTANCE_LOW))
        val n = Notification.Builder(this, "motor").setContentTitle("Karthus").setContentText("IA rodando neste celular").setSmallIcon(android.R.drawable.stat_notify_more).setOngoing(true).build()
        if (Build.VERSION.SDK_INT >= 34) startForeground(1, n, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE) else startForeground(1, n)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        iniciar()
        return START_STICKY
    }

    @Synchronized private fun iniciar() {
        if (proc?.isAlive == true) return
        val nl = applicationInfo.nativeLibraryDir
        val raiz = getExternalFilesDir(null) ?: filesDir
        val home = File(raiz, "karthus").apply { mkdirs() }
        val pb = ProcessBuilder("$nl/libkarthus.so")
        val env = pb.environment()
        env["HIRELOD_HOME"] = home.path
        env["HIRELOD_PORTA"] = PORTA.toString()
        env["HIRELOD_LLAMA_SERVER"] = "$nl/libllama-server.so"
        env["HOME"] = home.path
        env["TMPDIR"] = cacheDir.path
        env["LD_LIBRARY_PATH"] = nl
        pb.redirectErrorStream(true)
        pb.redirectOutput(File(home, "saida-karthus.log"))
        proc = pb.start()
        wake = (getSystemService(POWER_SERVICE) as PowerManager).newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "karthus:motor").apply { acquire() }
    }

    override fun onDestroy() {
        proc?.destroyForcibly()
        try { wake?.release() } catch (_: Exception) {}
        super.onDestroy()
    }

    companion object { const val PORTA = 8765 }
}
