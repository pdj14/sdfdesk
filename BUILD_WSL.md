# WSL을 이용한 네이티브 라이브러리 빌드 가이드

WSL(Windows Subsystem for Linux)을 설치하고 Ubuntu 배포판을 다운로드했습니다.
하지만 초기 설정(사용자 이름 및 비밀번호 생성)은 사용자가 직접 완료해야 합니다.

## 1. WSL 설정 완료하기

1.  새로 열린 **Ubuntu** 터미널 창이 있다면, 그곳에서 사용자 이름과 비밀번호를 설정하세요.
2.  창이 없다면, 시작 메뉴에서 **Ubuntu**를 찾아 실행하세요.
3.  화면의 지시에 따라 UNIX 사용자 계정을 생성하세요.

## 2. 빌드 스크립트 실행하기

WSL 설정이 완료되면, 다음 명령어를 Ubuntu 터미널에서 실행하여 빌드를 진행하세요.
이 스크립트는 필요한 의존성을 설치하고 `librustdesk.so`를 빌드합니다.

```bash
# 프로젝트 디렉토리로 이동 (Windows의 C 드라이브는 /mnt/c 에 마운트됨)
cd /mnt/c/workspace/rustdesk/sdfdesk

# 스크립트 실행 (chmod 오류 시 bash로 직접 실행)
bash setup_wsl_build.sh
```

## 3. 빌드 확인

빌드가 성공하면 `java_apk/app/src/main/jniLibs/arm64-v8a/librustdesk.so` 파일이 생성됩니다.
이후 다시 Windows에서 APK를 빌드하면 됩니다.

```powershell
cd java_apk
.\gradlew assembleDebug
```
