<div align="center">

# Breeze

**Windows Hello 얼굴 인식 후 "확인" 클릭, 이제 필요 없습니다.**

[![CI](https://github.com/evan-choi/breeze-wh/actions/workflows/ci.yml/badge.svg)](https://github.com/evan-choi/breeze-wh/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/breeze-wh)](https://crates.io/crates/breeze-wh)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

[English](README.md) | [한국어](#어떻게-동작하나요)

</div>

---

Windows Hello 얼굴 인식은 잘 됩니다. 근데 인식이 끝나도 매번 "확인"을 눌러야 해요. 쓸데없는 한 클릭입니다.

**Breeze**는 그 클릭을 없애줍니다. Windows Hello 인증 창이 뜨면 얼굴 인식 성공 여부를 확인하고, 알아서 확인 버튼을 눌러줍니다. Windows 서비스로 백그라운드에서 조용히 돌아갑니다.

## 어떻게 동작하나요

```
Windows Hello가 얼굴을 인식함
        ↓
인증 다이얼로그에 "확인" 버튼이 나타남
        ↓
Breeze가 UI Automation API로 감지
        ↓
얼굴 인식일 때만 자동으로 확인 (PIN은 건드리지 않음)
        ↓
클릭 없이 바로 통과
```

## 설치

```powershell
cargo install breeze-wh
breeze-wh install
```

끝입니다.

## 명령어

- `breeze-wh install` — 서비스 등록, 데이터 디렉터리 권한 부여, 자동 시작
- `breeze-wh uninstall` — 서비스 중지 및 삭제
- `breeze-wh start` — 서비스 시작
- `breeze-wh stop` — 서비스 중지
- `breeze-wh status` — 현재 서비스 상태 출력
- `breeze-wh upgrade` — GitHub Releases에서 최신 `breeze-wh.exe`를 받아 교체 (서비스 상태 유지)
- `breeze-wh --version` — 설치된 버전 확인

`install` / `uninstall` / `start` / `stop` / `upgrade`는 관리자 권한이 필요하지만, UAC로 자동 상승되므로 관리자 셸을 따로 띄울 필요는 없습니다.

> **참고:** `upgrade`는 crates.io가 아니라 GitHub Release의 바이너리를 받아 갈아끼웁니다. 업그레이드 이후 cargo 레지스트리 메타데이터는 예전 버전으로 남아있을 수 있으니, 실제 설치된 버전은 `breeze-wh --version`으로 확인하세요.

## 구조

Breeze는 하나의 바이너리가 두 가지 모드로 동작합니다:

- **Service 모드** — Session 0에서 Windows 서비스로 실행됩니다. 사용자 로그온/로그오프를 감지하고, 유저 세션에 helper 프로세스를 띄웁니다. helper가 죽으면 자동으로 다시 띄웁니다.

- **Helper 모드** — 유저 세션에서 관리자 권한으로 실행됩니다. UI Automation 이벤트를 통해 `Credential Dialog Xaml Host` 창을 감지하고, UI 트리를 한 번에 스캔합니다:
  - `PasswordField`이 있으면 → PIN 모드 → **무시**
  - `PasswordField` 없이 `OkButton`이 있으면 → 얼굴 인식 → **클릭**
  - `OkButton`이 아직 안 떴으면 → `StructureChanged` 이벤트로 기다림

감지에 사용하는 `AutomationId`와 `ClassName`은 언어에 의존하지 않으므로, Windows 표시 언어가 뭐든 상관없이 동작합니다.

## 설정

설정 파일: `C:\ProgramData\Breeze-WH\config.toml` (설치 시 자동 생성)

```toml
enabled = true
debounce_ms = 2000
log_level = "info"
log_max_files = 7
```

로그: `C:\ProgramData\Breeze-WH\logs\`

## 요구 사항

- Windows 10 / 11
- Windows Hello 얼굴 인식이 설정되어 있어야 함
- [Rust](https://rustup.rs/) 툴체인 (cargo로 설치 시)

## 라이선스

MIT OR Apache-2.0
