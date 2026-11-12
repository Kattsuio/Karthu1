package com.nova.app

import android.app.Application
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

val Cream = Color(0xFFFAF9F5)
val Accent = Color(0xFFC96442)
val Ink = Color(0xFF1F1E1D)
val Soft = Color(0xFFEFEDE6)

const val SYSTEM = "Você é a Karthu, um assistente rápido e objetivo. Responda no idioma do usuário. " +
    "Para criar um arquivo, escreva um bloco assim:\n```file:nome.ext\nconteúdo\n```\n" +
    "Você pode gerar qualquer formato de texto (txt, md, html, css, js, py, json, csv, xml, svg, etc.) " +
    "e também .pdf e .docx (escreva o texto simples). Ao receber arquivos anexados, use o conteúdo deles para criar novos. " +
    "Quando houver FONTES, use somente elas, cite como [1], [2] e diga claramente se a informação não estiver nas fontes. " +
    "Nunca invente dados."

class Msg(val role: String, text: String) {
    var text by mutableStateOf(text)
    val files = mutableStateListOf<Pair<String, Uri>>()
}

class VM(app: Application) : AndroidViewModel(app) {
    private val sp = app.getSharedPreferences("karthu", 0)
    var baseUrl by mutableStateOf(sp.getString("url", "http://10.0.2.2:11434/v1")!!)
    var model by mutableStateOf(sp.getString("model", "llama3.1")!!)
    var apiKey by mutableStateOf(sp.getString("key", "")!!)
    var searx by mutableStateOf(sp.getString("searx", "")!!)
    var webSearch by mutableStateOf(false)
    var busy by mutableStateOf(false)
    val messages = mutableStateListOf<Msg>()
    val attachments = mutableStateListOf<Pair<String, String>>()
    private val history = mutableListOf<Pair<String, String>>()

    fun saveSettings() = sp.edit().putString("url", baseUrl).putString("model", model)
        .putString("key", apiKey).putString("searx", searx).apply()

    fun newChat() { if (!busy) { messages.clear(); history.clear(); attachments.clear() } }

    fun attach(uri: Uri) {
        viewModelScope.launch(Dispatchers.IO) { attachments.add(Files.read(getApplication(), uri)) }
    }

    fun send(input: String) {
        if (busy || (input.isBlank() && attachments.isEmpty())) return
        val att = attachments.toList(); attachments.clear()
        messages.add(Msg("user", input + if (att.isNotEmpty()) "\n📎 " + att.joinToString { it.first } else ""))
        val reply = Msg("assistant", ""); messages.add(reply)
        busy = true
        viewModelScope.launch(Dispatchers.IO) {
            try {
                var content = input
                att.forEach { content += "\n\n[Arquivo anexado: ${it.first}]\n${it.second}" }
                if (webSearch && input.isNotBlank()) {
                    val src = Api.search(input, searx)
                    content += if (src.isEmpty()) "\n\nFONTES: nenhuma fonte encontrada. Avise o usuário que não achou fontes confiáveis."
                    else "\n\nFONTES:\n" + src.mapIndexed { i, s -> "[${i + 1}] ${s.title} (${s.url})\n${s.text}" }.joinToString("\n\n")
                }
                history.add("user" to content)
                val all = listOf("system" to SYSTEM) + history.takeLast(20)
                val sb = StringBuilder()
                Api.stream(baseUrl, apiKey, model, all) { sb.append(it); reply.text = sb.toString() }
                history.add("assistant" to sb.toString())
                Files.FILE_BLOCK.findAll(sb).forEach { m ->
                    val name = m.groupValues[1].trim()
                    Files.save(getApplication(), name, m.groupValues[2].trimEnd())?.let { reply.files.add(name to it) }
                }
            } catch (e: Exception) {
                reply.text = reply.text + "\n\n⚠️ " + (e.message ?: "falha na conexão") +
                    "\nConfira URL e modelo em ⚙️ Configurações."
            } finally { busy = false }
        }
    }
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(colorScheme = lightColorScheme(primary = Accent, background = Cream, surface = Cream)) { App() }
        }
    }
}

@Composable
fun App(vm: VM = viewModel()) {
    val ctx = LocalContext.current
    var text by remember { mutableStateOf("") }
    var showSettings by remember { mutableStateOf(false) }
    val list = rememberLazyListState()
    val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris -> uris.forEach { vm.attach(it) } }
    LaunchedEffect(vm.messages.size, vm.messages.lastOrNull()?.text) {
        if (vm.messages.isNotEmpty()) list.scrollToItem(vm.messages.size - 1)
    }

    Column(Modifier.fillMaxSize().background(Cream).statusBarsPadding().navigationBarsPadding().imePadding()) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("Karthu", fontFamily = FontFamily.Serif, fontSize = 24.sp, color = Ink, modifier = Modifier.weight(1f))
            TextButton(onClick = { vm.newChat() }) { Text("Novo chat", color = Accent) }
            IconButton(onClick = { showSettings = true }) { Icon(Icons.Default.Settings, "Configurações", tint = Ink) }
        }
        Box(Modifier.weight(1f).fillMaxWidth()) {
            if (vm.messages.isEmpty()) {
                Column(Modifier.align(Alignment.Center).padding(32.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("✺", fontSize = 40.sp, color = Accent)
                    Spacer(Modifier.height(12.dp))
                    Text("Como posso ajudar?", fontFamily = FontFamily.Serif, fontSize = 28.sp, color = Ink)
                }
            }
            LazyColumn(state = list, contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                items(vm.messages) { m -> Bubble(m) { uri, name ->
                    ctx.startActivity(Intent(Intent.ACTION_VIEW).setDataAndType(uri, Files.mime(name))
                        .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK))
                } }
            }
        }
        if (vm.attachments.isNotEmpty()) {
            Row(Modifier.padding(horizontal = 16.dp)) {
                vm.attachments.forEach { a ->
                    AssistChip(onClick = { vm.attachments.remove(a) }, label = { Text("📎 ${a.first} ✕", maxLines = 1) },
                        modifier = Modifier.padding(end = 6.dp))
                }
            }
        }
        Surface(Modifier.fillMaxWidth().padding(12.dp), shape = RoundedCornerShape(24.dp), color = Color.White, shadowElevation = 2.dp) {
            Column(Modifier.padding(horizontal = 8.dp, vertical = 4.dp)) {
                TextField(
                    value = text, onValueChange = { text = it }, modifier = Modifier.fillMaxWidth(),
                    placeholder = { Text("Converse com a Karthu") }, maxLines = 5,
                    colors = TextFieldDefaults.colors(focusedContainerColor = Color.Transparent, unfocusedContainerColor = Color.Transparent,
                        focusedIndicatorColor = Color.Transparent, unfocusedIndicatorColor = Color.Transparent)
                )
                Row(verticalAlignment = Alignment.CenterVertically) {
                    IconButton(onClick = { picker.launch(arrayOf("*/*")) }) { Icon(Icons.Default.Add, "Anexar", tint = Ink) }
                    FilterChip(selected = vm.webSearch, onClick = { vm.webSearch = !vm.webSearch },
                        label = { Text("Pesquisar") }, leadingIcon = { Icon(Icons.Default.Search, null, Modifier.size(16.dp)) })
                    Spacer(Modifier.weight(1f))
                    FilledIconButton(onClick = { vm.send(text); text = "" }, enabled = !vm.busy,
                        colors = IconButtonDefaults.filledIconButtonColors(containerColor = Accent)) {
                        Icon(Icons.AutoMirrored.Filled.Send, "Enviar", tint = Color.White)
                    }
                }
            }
        }
    }

    if (showSettings) SettingsDialog(vm) { showSettings = false }
}

@Composable
fun Bubble(m: Msg, open: (Uri, String) -> Unit) {
    val shown = m.text.replace(Files.FILE_BLOCK) { "📄 ${it.groupValues[1].trim()}" }.ifBlank { "…" }
    if (m.role == "user") {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
            Surface(shape = RoundedCornerShape(18.dp), color = Soft) {
                SelectionContainer { Text(shown, Modifier.padding(12.dp), color = Ink) }
            }
        }
    } else {
        Column {
            SelectionContainer { Text(shown, color = Ink, fontSize = 16.sp, lineHeight = 24.sp) }
            m.files.forEach { (name, uri) ->
                Surface(Modifier.padding(top = 8.dp).clickable { open(uri, name) }, shape = RoundedCornerShape(12.dp), color = Soft) {
                    Text("📄 $name — toque para abrir (Downloads/Karthu)", Modifier.padding(10.dp), fontSize = 13.sp, color = Ink)
                }
            }
        }
    }
}

@Composable
fun SettingsDialog(vm: VM, close: () -> Unit) {
    AlertDialog(
        onDismissRequest = close,
        confirmButton = { TextButton(onClick = { vm.saveSettings(); close() }) { Text("Salvar", color = Accent) } },
        title = { Text("Configurações", fontFamily = FontFamily.Serif) },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(vm.baseUrl, { vm.baseUrl = it }, label = { Text("URL do servidor (/v1)") }, singleLine = true)
                OutlinedTextField(vm.model, { vm.model = it }, label = { Text("Modelo") }, singleLine = true)
                OutlinedTextField(vm.apiKey, { vm.apiKey = it }, label = { Text("Chave (opcional)") }, singleLine = true)
                OutlinedTextField(vm.searx, { vm.searx = it }, label = { Text("SearXNG (opcional)") }, singleLine = true)
                Text("Sem SearXNG, a pesquisa usa a Wikipédia.", fontSize = 12.sp)
            }
        }
    )
}
