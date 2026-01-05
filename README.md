# tui_video

### 해당 프로젝트는 youtube 영상을 ascii 코드로 변경하여 TUI에 보여주는 프로젝트입니다.

## 실행 방법

1. 필요한 바이너리 다운로드
- ffmpeg.exe 다운로드

https://ffmpeg.org/download.html 에서 ffmpeg-release-essentials.zip 다운로드

- 폴더 지정 (구조)

```aiignore
main/
├─ Cargo.toml
├─ tools/
│  ├─ ffmpeg/
│     └─ ffmpeg.exe
└─ src/
```

2. 빌드

```
cargo build --release
```

3. 실행
- 프로그램 실행

```aiignore
./target/release/main.exe
```

- youtube 링크 영상 입력