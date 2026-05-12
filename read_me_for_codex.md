# Codex 작업 전달 메모

## 이번 작업 범위

- Rust CLI 프로젝트 `kis-supply-bot`를 생성했습니다.
- KIS 실제 주문은 구현하지 않고, mock 데이터와 `DRY_RUN` 주문 기록까지만 구현했습니다.
- API 키가 없어도 mock mode로 동작하도록 구성했습니다.
- SQLite migration은 프로그램 시작 시 자동 적용되도록 작성했습니다.
- `.env.example`, `config.yaml.example`, README, systemd 예시 파일을 포함했습니다.

## 이 PC에서 하지 않은 것

요청 원문에 따라 이 PC에서는 아래 명령을 실행하지 않았습니다.

```bash
cargo build
cargo test
cargo run
```

다른 Ubuntu 26.x PC에서 pull 받은 뒤 README 순서대로 빌드와 mock mode 검증을 진행하세요.

## 새 PC 준비

```bash
apt update
apt install -y build-essential pkg-config libssl-dev sqlite3 ca-certificates curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cp .env.example .env
cp config.yaml.example config.yaml
mkdir -p data
cargo build --release
```

## 운영 전 필수 확인

- `.env`의 `TRADING_MODE=mock`, `DRY_RUN=true` 기본값 유지
- 실제 주문을 붙이기 전 `KIS_ENABLE_REAL_ORDER=false` 유지
- Discord 보고가 필요하면 `DISCORD_WEBHOOK_URL` 설정
- KIS 실제 API endpoint와 `TR_ID`는 공식 문서 기준으로 `HttpKisClient`에 보강 필요

## 권장 검증 순서

```bash
./target/release/kis-supply-bot health-check
TRADING_MODE=mock DRY_RUN=true ./target/release/kis-supply-bot collect-daily --date 2026-05-12
TRADING_MODE=mock DRY_RUN=true ./target/release/kis-supply-bot build-candidates --date 2026-05-12
TRADING_MODE=mock DRY_RUN=true ./target/release/kis-supply-bot monitor --date 2026-05-13
```

## 향후 실제 API 연결 TODO

- KIS 공식 문서 기준으로 endpoint, query parameter, response DTO 확정
- 모의투자/실전투자별 `TR_ID` 분리
- 100~300ms rate limit과 429 exponential backoff 구체화
- 실제 주문은 `TRADING_MODE=real`, `DRY_RUN=false`, 인증정보 존재, `enable_real_order=true`를 모두 만족할 때만 허용
