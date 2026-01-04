# tui_video

### 해당 프로젝트는 youtube 영상을 ascii 코드로 변경하여 TUI에 보여주는 프로젝트입니다.

## 실행 방법

### 의존 프로그램
#### ffmpeg
#### 🐧 Linux (Ubuntu/Debian)
- 가장 설정이 간편합니다. 패키지 관리자를 통해 dev 패키지를 설치하세요.

```Bash
sudo apt update
sudo apt install -y ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev libavdevice-dev
```

#### 🍎 macOS
- Homebrew를 사용하여 설치합니다. 설치 후 환경 변수 설정이 필요할 수 있습니다.

```Bash
brew install ffmpeg
```
- 컴파일 시 라이브러리를 찾지 못한다면 다음을 실행하세요:
```Bash
export PKG_CONFIG_PATH="/usr/local/opt/ffmpeg/lib/pkgconfig"
# 애플 실리콘(M1/M2/M3)의 경우:
export PKG_CONFIG_PATH="/opt/homebrew/opt/ffmpeg/lib/pkgconfig"
```

#### 🪟 Windows
- Windows는 설정이 조금 까다롭습니다. 가장 권장하는 방식은 vcpkg를 사용하는 것입니다.
- vcpkg를 통한 설치 (권장):
```PowerShell

git clone https://github.com/microsoft/vcpkg
.\vcpkg\bootstrap-vcpkg.bat
.\vcpkg\vcpkg install ffmpeg:x64-windows
```
- 환경 변수 설정: FFMPEG_DIR 환경 변수를 생성하고, FFmpeg이 설치된 경로(bin, include, lib 폴더가 있는 곳)를 지정해야 합니다.
- 직접 다운로드 시: gyan.dev에서 full shared 빌드를 다운로드하여 압축을 풀고, PATH 및 FFMPEG_DIR을 설정하세요.

#### 🚀 Troubleshooting
- "pkg-config not found": 시스템에 pkg-config가 설치되어 있는지 확인하세요. (Linux: apt install pkg-config, macOS: brew install pkg-config)
- "Library not found": 라이브러리 경로가 LD_LIBRARY_PATH(Linux) 또는 DYLD_LIBRARY_PATH(macOS)에 포함되어 있는지 확인하세요.
- https://ffmpeg.org/download.html 에서 ffmpeg-release-essentials.zip 다운로드

- 폴더 지정 (구조)

```sh
.
├── Cargo.lock
├── Cargo.toml
├── README.md
└── src
    ├── app.rs
    ├── ascii.rs
    ├── ffmpeg_fn.rs
    ├── main.rs
    ├── tui.rs
    └── video.rs
```

2. 빌드

```sh
cargo build --release
```

3. 실행
- 프로그램 실행

```sh
./target/release/main.exe
```

- youtube 링크 영상 입력
