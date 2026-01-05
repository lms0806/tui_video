# tui_video

### 해당 프로젝트는 YouTube 영상을 ASCII 코드로 변환하여 TUI에 표시하는 프로젝트입니다.

---

## 실행 방법

## 1. 필수 의존성 설치

### Windows

1. **ffmpeg 다운로드**
   [https://ffmpeg.org/download.html](https://ffmpeg.org/download.html) 에서 **ffmpeg-release-essentials.zip** 다운로드

2. **폴더 구조 설정**

```text
main/
├─ Cargo.toml
├─ tools/
│  ├─ ffmpeg/
│  │  └─ ffmpeg.exe
└─ src/
```

> ffmpeg.exe 경로는 코드에서 참조되므로 구조를 유지해야 합니다.

---

### macOS

macOS에서는 Homebrew를 이용해 ffmpeg를 설치하는 방식을 권장합니다.

1. **Homebrew 설치 (이미 설치되어 있다면 생략)**

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

2. **ffmpeg 설치**

```bash
brew install ffmpeg
```

3. **ffmpeg 경로 확인**

```bash
which ffmpeg
```

일반적으로 다음 경로 중 하나입니다:

* `/opt/homebrew/bin/ffmpeg` (Apple Silicon)
* `/usr/local/bin/ffmpeg` (Intel Mac)

> macOS에서는 ffmpeg 바이너리를 프로젝트 내부에 둘 필요 없이, 시스템 PATH에 등록된 ffmpeg를 사용합니다.

---

## 2. 빌드

### 공통 (Windows / macOS)

```bash
cargo build --release
```

---

## 3. 실행

### Windows

```bash
./target/release/main.exe
```

### macOS

```bash
./target/release/main
```

---

## 4. 사용 방법

1. 프로그램 실행
2. YouTube 영상 링크 입력
3. 영상이 ASCII 아트로 변환되어 TUI 화면에 출력

---

## 참고 사항

* macOS에서는 터미널 크기에 따라 출력 품질이 크게 달라집니다.
* 더 나은 결과를 위해 터미널을 전체 화면으로 사용하는 것을 권장합니다.
* ffmpeg 버전에 따라 디코딩 성능에 차이가 있을 수 있습니다.

---

## 지원 환경

* Windows 10 이상
* macOS 12 (Monterey) 이상
* Rust 1.92.0+

---

## 라이선스

개인 학습 및 실험용 프로젝트입니다.
