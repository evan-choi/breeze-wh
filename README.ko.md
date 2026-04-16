<div align="center">

# Breeze

**Windows Hello가 얼굴을 인식한 뒤 "확인" 누르는 거, 이제 안 해도 됩니다.**

[![CI](https://github.com/evan-choi/breeze/actions/workflows/ci.yml/badge.svg)](https://github.com/evan-choi/breeze/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

[English](README.md) | [한국어](#작동-방식)

</div>

---

Windows Hello 얼굴 인식은 훌륭합니다 — 근데 매번 "확인" 버튼을 눌러야 합니다. 그 클릭은 그냥 불필요한 마찰입니다.

**Breeze**가 대신 눌러줍니다. Windows Hello 자격 증명 다이얼로그를 감시하고, 얼굴 인식이 성공하면 자동으로 확인합니다. Windows 서비스로 백그라운드에서 동작합니다.

## 작동 방식

```
Windows Hello가 얼굴 인식
        ↓
"확인" 버튼이 있는 자격 증명 다이얼로그 등장
        ↓
Breeze가 UI Automation API로 감지
        ↓
자동 확인 (얼굴 인식만 — PIN은 무시)
        ↓
클릭 없이 바로 통과
```

## 설치

```powershell
cargo install --git https://github.com/evan-choi/breeze
```

서비스 등록 및 시작 (관리자 권한 필요):

```powershell
breeze install
breeze start
```

끝입니다. Breeze가 백그라운드에서 알아서 동작합니다.

## 명령어

| 명령어 | 설명 |
|--------|------|
| `breeze install` | Windows 서비스 등록 |
| `breeze uninstall` | 서비스 중지 및 제거 |
| `breeze start` | 서비스 시작 |
| `breeze stop` | 서비스 중지 |
| `breeze status` | 서비스 상태 확인 |

## 삭제

```powershell
breeze uninstall
cargo uninstall breeze
```

## 구조

Breeze는 하나의 바이너리로 두 가지 모드를 실행합니다:

- **Service 모드** — Session 0에서 Windows 서비스로 실행. 사용자 로그온/로그오프를 감시하고, 유저 세션에 helper를 생성합니다. 크래시 시 지수 백오프로 자동 재시작합니다.

- **Helper 모드** — 유저 세션에서 관리자 권한으로 실행. UI Automation 포커스 이벤트를 구독해서 `Credential Dialog Xaml Host` 창을 감지합니다. 감지 시 UI 트리를 한 번에 스캔:
  - `PasswordField`이 있으면 → PIN 모드 → **무시**
  - `PasswordField` 없이 `OkButton`이 있으면 → 얼굴 인식 → **클릭**
  - `OkButton`이 아직 없으면 → `StructureChanged` 이벤트로 출현 대기

모든 감지는 언어 독립적인 `AutomationId`와 `ClassName`을 사용하므로, Windows 표시 언어에 관계없이 동작합니다.

## 설정

설정 파일: `C:\ProgramData\Breeze\config.toml` (설치 시 자동 생성)

```toml
enabled = true
debounce_ms = 2000
log_level = "info"
log_max_files = 7
```

로그 위치: `C:\ProgramData\Breeze\logs\`

## 요구 사항

- Windows 10 / 11
- Windows Hello 얼굴 인식 설정 완료
- Rust 1.85+ (소스에서 빌드 시)

## 라이선스

MIT OR Apache-2.0
