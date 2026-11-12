# Karthu

App Android independente (Kotlin + Jetpack Compose), visual inspirado em chats minimalistas.

## Gerar o APK
1. Abra a pasta no Android Studio (Hedgehog ou mais novo) e aguarde o Gradle sincronizar.
2. Build > Build Bundle(s) / APK(s) > Build APK(s).
3. O arquivo sai em app/build/outputs/apk/debug/app-debug.apk.

## Ligar a um modelo
Qualquer servidor compatível com /v1/chat/completions: Ollama, llama.cpp, LM Studio, vLLM.
- Emulador: http://10.0.2.2:11434/v1
- Celular real: http://IP-DO-SEU-PC:11434/v1 (mesma rede Wi-Fi)
Escolha a URL e o modelo em ⚙️ Configurações.

## Recursos
- Respostas em streaming; anexa arquivos (texto, código, csv, json, docx) e gera novos a partir deles.
- Cria qualquer arquivo de texto, além de PDF e DOCX, salvos em Downloads/Karthu.
- Pesquisa opcional que usa apenas fontes citadas (SearXNG ou Wikipédia), sem inventar dados.

## Gerar o APK sem Android Studio (GitHub)
1. Crie um repositório no GitHub e envie todo o conteúdo desta pasta (incluindo .github).
2. Abra a aba Actions > Build APK > aguarde terminar (uns 5 min).
3. Baixe o artefato "Karthu-apk" e instale o app-debug.apk no celular.
