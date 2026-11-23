# Karthus para Android

IA 100% local no celular: o app Android roda o **Karthus** (Rust) e o **llama.cpp** como programas nativos
e mostra a interface do Karthus numa WebView. Nada sai do aparelho (só o download dos modelos).

## Como gerar o APK (sem instalar nada no PC)
1. Crie um repositório no GitHub e envie tudo desta pasta (`rust/`, `android/`, `.github/`).
2. Abra a aba **Actions** > **Karthus APK** > **Run workflow**. Leva uns 15–25 minutos.
3. Ao terminar, baixe `Karthus.apk` em **Artifacts** (ou em **Releases**, se enviar uma tag `v6.4.0`).
4. No celular: abra o APK e permita "instalar apps desconhecidos". Android 9+ e processador ARM64.

## Primeiro uso
Abra o Karthus > menu lateral > **Criações/Instalação** > baixe um modelo pequeno no Catálogo de IAs
(ex.: 1B–4B, quantização Q4). Modelos grandes não cabem na memória da maioria dos celulares.

## Limites desta versão
- Só o chat (com Web, Fluxo e criação de arquivos). Voz, imagens e vídeo dependem de programas do Windows e ficam de fora.
- Velocidade: só CPU. Esperam-se poucos tokens por segundo em modelos de 3–4B.
- O APK sai assinado com a chave de depuração (serve para instalar direto; não é para a Play Store).
