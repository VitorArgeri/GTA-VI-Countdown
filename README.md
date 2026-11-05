# GTA VI Countdown

Aplicativo portátil para Windows que mostra a contagem regressiva até **19 de novembro de 2026, à meia-noite no horário de Brasília**. A janela abre normalmente para você posicioná-la no monitor desejado; o papel de parede e o áudio funcionam sem internet.

## Instalar no Windows sem programar

O aplicativo não tem instalador. Você baixa um pacote ZIP, extrai a pasta e abre o executável. Não precisa instalar Rust, abrir o terminal ou estar conectado à internet para usar.

1. Baixe o [pacote pronto para Windows x64](https://github.com/VitorArgeri/GTA-VI-Countdown/raw/refs/heads/main/dist/GTA-VI-Countdown-Windows-x64.zip).
2. No Explorador de Arquivos, clique com o botão direito no ZIP e escolha **Extrair tudo**. Abra o conteúdo extraído; não execute o programa de dentro do ZIP.
3. Confira se `gta-vi-countdown.exe` e a pasta `assets` estão juntos na mesma pasta. Não mova o executável sozinho.
4. Clique duas vezes em `gta-vi-countdown.exe` para iniciar. Para criar um atalho, clique com o botão direito nele e escolha **Mostrar mais opções → Criar atalho**.

O botão **Code → Download ZIP** baixa o código-fonte. Para instalar sem programar, use o link do pacote pronto acima.

O Windows pode mostrar um aviso de editor desconhecido porque o aplicativo não possui assinatura digital. Só abra o executável se ele veio da página oficial do projeto ou de alguém em quem você confia.

## Usar o aplicativo

- Arraste o cartão para posicioná-lo. A grade 3×3 na barra direita oferece posições prontas.
- O ícone de galeria no canto inferior esquerdo abre a seleção de papel de parede. A engrenagem no canto inferior direito abre as opções.
- Na barra de opções, escolha uma cor, ajuste o tamanho de **50% a 200%** e a opacidade de **50% a 100%**. O padrão de 50% mantém o fundo no visual original; em 100%, o fundo do cartão fica sólido e os números continuam nítidos.
- **F11** alterna entre janela e tela cheia. **Esc** volta ao modo janela.
- Quando a contagem chegar a zero, a animação e a música entram em loop.
- Suas escolhas são salvas automaticamente em `%APPDATA%\GTA VI Countdown\settings.json`.

## Arquivos que precisam ficar juntos

O pacote portátil deve ter esta estrutura. Mantenha `assets` ao lado do executável:

```text
GTA-VI-Countdown/
├── gta-vi-countdown.exe
└── assets/
    ├── audio/
    │   └── love-is-a-long-road.wav
    ├── wallpapers/
    │   ├── 01_rockstar_robbery.jpg
    │   ├── 02_rockstar_jason_lucia.jpg
    │   └── 03_alphacoders_1343606.png
    ├── icon.ico
    └── icon.png
```

Se o app não abrir ou aparecer sem imagens, extraia o ZIP completo novamente e confirme que essa estrutura foi mantida. A música toca somente quando a contagem chega a zero.

## Criar o pacote a partir do código-fonte

Esta etapa é para quem quer gerar a versão portátil. Requer Windows x64, Rust com toolchain MSVC e as ferramentas de compilação C++ da Microsoft. Depois de baixar o código-fonte, abra o PowerShell na pasta do projeto e execute:

```powershell
.\build.ps1
```

O executável e os arquivos necessários serão preparados na pasta `dist`. A compilação usa um processo por vez para limitar o uso de memória. Não é necessário fazer isso quando você já tem o pacote portátil pronto.

## Mídias e preferências

Os papéis de parede ficam em `assets/wallpapers`; o áudio em loop deve ser WAV e manter o nome `love-is-a-long-road.wav` dentro de `assets/audio`. O aplicativo não baixa nem atualiza esses arquivos durante o uso.

Para redefinir as preferências, feche o aplicativo e apague `settings.json` no caminho `%APPDATA%\GTA VI Countdown`. Na próxima abertura, as opções voltarão aos valores padrão.
