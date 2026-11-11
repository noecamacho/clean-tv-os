# Emulador Android x86 / Android TV (Linux e Windows)

Este guia orienta a instalação e execução do emulador Android x86/x86_64 tanto no **Linux** (foco em distribuições modernas como Fedora/Ubuntu) quanto no **Windows** (foco em Windows 10/11 com aceleração por hardware), cobrindo o uso via **Android Studio** (interface gráfica) e via **Linha de Comando** (`cmdline-tools`).

O foco principal para testes do Lume TV Launcher é a imagem de **Android TV** (API 28 / Android 9 e API 30 / Android 11).

---

## 1. Linux

### 1.1 Pré-requisitos e Aceleração KVM
No Linux, o emulador utiliza o KVM (Kernel-based Virtual Machine) para executar com aceleração nativa de hardware:

1. Verifique se o módulo KVM está ativo:
   ```bash
   ls -l /dev/kvm
   ```
2. Caso seu usuário não tenha permissão de acesso ao `/dev/kvm`, adicione-o ao grupo responsável:
   ```bash
   sudo usermod -aG kvm $USER
   # No Fedora ou distros onde o nó pertence ao grupo nobody/kvm, faça logout e login novamente.
   ```
3. Instale as bibliotecas gráficas e de sistema necessárias:
   - **Fedora / RHEL:**
     ```bash
     sudo dnf install -y pulseaudio-libs mesa-libGL libX11 libXcomposite libXcursor libXdamage libXext libXfixes libXi libXrender libXtst java-17-openjdk-devel
     ```
   - **Ubuntu / Debian:**
     ```bash
     sudo apt update && sudo apt install -y qemu-kvm libvirt-daemon-system libvirt-clients bridge-utils libglu1 libpulse0 openjdk-17-jdk
     ```

---

### 1.2 Opção A: Via Android Studio (GUI)
1. Instale o Android Studio (pode ser via Flatpak no Linux):
   ```bash
   flatpak install flathub com.google.AndroidStudio
   ```
2. Abra o Android Studio e acerte o assistente inicial (que baixa o SDK padrão).
3. Abra **Device Manager** (Virtual Device Manager).
4. Clique em **Create Device**:
   - Categoria: **TV**.
   - Perfil: **Television (1080p)** ou **Television (720p)**.
5. Na tela de seleção de imagem (**x86 Images**):
   - Baixe a versão desejada, por exemplo: `Android 9.0 (Android TV)` (API 28 x86) ou `Android 11.0 (Android TV)` (API 30 x86).
6. Conclua a criação e clique no botão **Play** para iniciar o emulador.

---

### 1.3 Opção B: Via Linha de Comando (`cmdline-tools`)
Para quem prefere uma instalação mínima sem a IDE:

1. **Defina o diretório base e variáveis no seu shell (`~/.zshenv`, `~/.zshrc` ou `~/.bashrc`):**
   ```bash
   export ANDROID_HOME=$HOME/Android
   export ANDROID_SDK_ROOT=$ANDROID_HOME
   export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator
   ```
   Recarregue o shell: `source ~/.zshenv`.

2. **Baixe as Command-Line Tools:**
   ```bash
   mkdir -p ~/Android/cmdline-tools
   cd /tmp
   # Baixe o pacote oficial mais recente do site do Android Developer
   curl -O https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip
   unzip commandlinetools-linux-*_latest.zip
   mkdir -p ~/Android/cmdline-tools/latest
   mv cmdline-tools/* ~/Android/cmdline-tools/latest/
   rm -rf cmdline-tools commandlinetools-linux-*
   ```

3. **Aceite as licenças e instale a imagem:**
   ```bash
   sdkmanager --licenses
   # Instala o emulador, as ferramentas ADB/fastboot e a imagem x86 do Android TV API 28
   sdkmanager "platform-tools" "emulator" "system-images;android-28;android-tv;x86"
   ```

4. **Crie o dispositivo virtual (AVD):**
   ```bash
   avdmanager create avd -n tv_api28 -k "system-images;android-28;android-tv;x86" --device "tv_1080p"
   ```

5. **Inicie o emulador:**
   ```bash
   emulator -avd tv_api28 -gpu host
   ```

---

## 2. Windows

### 2.1 Pré-requisitos e Aceleração de Hardware
No Windows 10/11 x64, o emulador usa a **Plataforma do Hipervisor do Windows (WHPX)** ou o **AEHD (Android Emulator Hypervisor Driver)**.

1. **Habilitar Virtualização na BIOS/UEFI:**
   Certifique-se de que a opção **Intel VT-x** ou **AMD-V / SVM** esteja habilitada na placa-mãe.
2. **Habilitar Recursos do Windows:**
   - Abra o menu Iniciar, digite `Ativar ou desativar recursos do Windows`.
   - Marque:
     - **Plataforma de Máquina Virtual** (Virtual Machine Platform).
     - **Plataforma do Hipervisor do Windows** (Windows Hypervisor Platform).
   - Clique em OK e reinicie o computador se solicitado.

---

### 2.2 Opção A: Via Android Studio (GUI)
1. Baixe o instalador oficial do [Android Studio](https://developer.android.com/studio).
2. Execute o instalador e garanta que o componente **Android Virtual Device** esteja marcado.
3. Abra o Android Studio e vá em **Tools > Device Manager**.
4. Clique em **Create Device**:
   - Categoria: **TV**.
   - Perfil de hardware: **Television (1080p)**.
5. Selecione a aba de imagens **x86 Images**:
   - Baixe a imagem desejada (ex: `Pie / Android 9.0 (Android TV)` API 28 ou `Android 11.0 (Android TV)` API 30).
6. Finalize o assistente e inicie o AVD clicando no ícone de Play.

---

### 2.3 Opção B: Via Linha de Comando (PowerShell)
1. **Configurar variáveis de ambiente:**
   - Escolha um diretório base, por exemplo `C:\Android`.
   - No PowerShell (Executar como Administrador):
     ```powershell
     [System.Environment]::SetEnvironmentVariable("ANDROID_HOME", "C:\Android", "User")
     [System.Environment]::SetEnvironmentVariable("ANDROID_SDK_ROOT", "C:\Android", "User")
     $oldPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
     $newPath = "$oldPath;C:\Android\cmdline-tools\latest\bin;C:\Android\platform-tools;C:\Android\emulator"
     [System.Environment]::SetEnvironmentVariable("Path", $newPath, "User")
     ```
   - Feche e reabra o terminal.

2. **Baixar e descompactar as ferramentas:**
   - Baixe o pacote `commandlinetools-win-*_latest.zip` do portal de desenvolvedores Android.
   - Crie a pasta `C:\Android\cmdline-tools\latest` e extraia os arquivos contidos em `cmdline-tools` para dentro de `latest`.

3. **Aceitar licenças e instalar componentes:**
   ```powershell
   sdkmanager --licenses
   sdkmanager "platform-tools" "emulator" "system-images;android-28;android-tv;x86"
   ```

4. **Criar e iniciar o AVD:**
   ```powershell
   avdmanager create avd -n tv_api28 -k "system-images;android-28;android-tv;x86" --device "tv_1080p"
   emulator -avd tv_api28 -gpu host
   ```

---

## 3. Testando o Lume no Emulador

Assim que o emulador estiver rodando (em qualquer SO), ele responderá via `adb`:

```bash
# 1. Verificar se o emulador foi detectado
adb devices

# 2. Instalar o APK gerado do Lume
adb install dist/lume-<versao>-release-debugsigned.apk

# 3. Definir o Lume como launcher padrão
adb shell cmd package set-home-activity app.lume.tv/.LumeActivity

# 4. Simular teclas de controle remoto da TV via adb
adb shell input keyevent 19 # DPAD_UP
adb shell input keyevent 20 # DPAD_DOWN
adb shell input keyevent 21 # DPAD_LEFT
adb shell input keyevent 22 # DPAD_RIGHT
adb shell input keyevent 23 # DPAD_CENTER (OK)
adb shell input keyevent 4  # BACK
adb shell input keyevent 3  # HOME
adb shell input keyevent 82 # KEYCODE_MENU (Menu de contexto do Lume)
```
