# GTA VI Countdown

Timer em Rust e Slint para o lançamento brasileiro de Grand Theft Auto VI, previsto para **19 de novembro de 2026 à meia-noite, no horário de Brasília**. A janela abre em modo normal para ser movida até a segunda tela.

## Usar

- Pressione **F11** para alternar entre janela e tela cheia; **Esc** volta ao modo normal.
- O ícone de galeria à esquerda abre a seleção de wallpaper; a engrenagem à direita abre as opções do timer.
- Arraste o cartão do timer para posicioná-lo. A grade 3×3 aplica alinhamentos padrão.
- Na barra direita, ajuste cor, tamanho entre **50% e 200%**, opacidade entre **50% e 100%** e volume. A opacidade inicial de 50% mantém o visual atual; 100% deixa o fundo do cartão sólido.
- Wallpaper, cor, tamanho, opacidade, posição e áudio são salvos automaticamente em `%APPDATA%\GTA VI Countdown\settings.json`.
- A música toca em loop ao chegar a zero. As animações de celebração também entram em loop.

Mantenha a pasta `assets` ao lado do executável para que wallpapers e áudio funcionem sem internet. Para iniciar direto da pasta do projeto, execute `cargo run --release`.

O ícone `assets/icon.ico` é embutido no `.exe`, e `assets/icon.png` é usado na janela. Para redesenhar o ícone, execute `tools/generate_icon.ps1` antes de compilar.

## Compilar e empacotar no Windows x64

Instale Rust com a toolchain MSVC e execute no PowerShell:

```powershell
.\build.ps1
```

O script compila o executável em modo release e copia `gta-vi-countdown.exe` e `assets` para `dist`. Não há instalador.

## Arquivos de mídia

- `assets/wallpapers/01_rockstar_robbery.jpg`
- `assets/wallpapers/02_rockstar_jason_lucia.jpg`
- `assets/wallpapers/03_alphacoders_1343606.png`
- `assets/audio/love-is-a-long-road.wav` — áudio extraído localmente do MP4 informado; o MP4 de origem não faz parte da distribuição.
- `assets/icon.ico` e `assets/icon.png` — ícone original do app, com paleta inspirada no GTA VI.

Para substituir mídias, mantenha os nomes e formatos esperados. Wallpapers JPG/PNG ficam em `assets/wallpapers`; a faixa em loop deve ser WAV válido em `assets/audio/love-is-a-long-road.wav`. O programa não baixa arquivos durante a execução.
