# kis-supply-bot

한국투자증권 KIS Developers Open API를 이용한 기관/외국인 수급 기반 자동매매 후보 생성 봇입니다.

초기 버전은 국내주식 모의투자와 `DRY_RUN` 운영을 기본값으로 하며, 실제 주문 API는 호출하지 않습니다. API 키가 없어도 `mock` 모드에서 데이터 수집, 후보 생성, 장중 감시, Discord 메시지 포맷 확인이 가능하도록 설계했습니다.

## 주요 기능

- 장 마감 후 종목별 투자자매매동향, OHLCV, 재무/시총 mock 데이터 수집
- 최근 5거래일 기관/외국인 연속 순매수 기반 후보 생성
- 시가총액, 거래대금, 상승률, 관리/거래정지/ETF/ETN/스팩/우선주 필터
- supply/liquidity/trend/risk 점수화
- 다음 거래일 눌림목/돌파 조건 감시
- 조건 충족 시 `trade_signal`, `trade_order`에 `DRY_RUN` 기록
- Discord Webhook 한국어 보고
- SQLite 저장소 계층 분리
- 향후 실제 KIS HTTP 연동 및 MySQL 교체를 위한 trait 구조

## Ubuntu 26.x 설치

```bash
apt update
apt install -y build-essential pkg-config libssl-dev sqlite3 ca-certificates curl

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

git clone <repo_url>
cd kis-supply-bot

cp .env.example .env
cp config.yaml.example config.yaml
mkdir -p data

cargo build --release
```

## 설정

민감정보는 절대 코드에 하드코딩하지 않습니다. `.env` 또는 `config.yaml`에 입력하세요.

```bash
cp .env.example .env
cp config.yaml.example config.yaml
```

기본값은 안전하게 아래처럼 동작합니다.

- `TRADING_MODE=mock`
- `DRY_RUN=true`
- `enable_real_order=false`
- Discord Webhook URL이 없으면 경고 로그만 남기고 계속 실행
- KIS API Key가 없으면 mock client 사용

## CLI

```bash
cargo run -- collect-daily --date 2026-05-12
cargo run -- build-candidates --date 2026-05-12
cargo run -- monitor --date 2026-05-13
cargo run -- report --date 2026-05-12
cargo run -- backtest --from 2026-01-01 --to 2026-05-12
cargo run -- health-check
```

릴리즈 빌드 후:

```bash
./target/release/kis-supply-bot collect-daily --date 2026-05-12
./target/release/kis-supply-bot build-candidates --date 2026-05-12
./target/release/kis-supply-bot monitor --date 2026-05-13
```

## Mock Mode 예시

```bash
TRADING_MODE=mock DRY_RUN=true ./target/release/kis-supply-bot collect-daily --date 2026-05-12
TRADING_MODE=mock DRY_RUN=true ./target/release/kis-supply-bot build-candidates --date 2026-05-12
TRADING_MODE=mock DRY_RUN=true ./target/release/kis-supply-bot monitor --date 2026-05-13
```

## Health Check

```bash
./target/release/kis-supply-bot health-check
```

KIS 인증정보가 없으면 다음 취지의 경고를 출력합니다.

```text
KIS API credentials are missing. Running in mock data mode.
DRY_RUN is enabled. No real orders will be submitted.
```

## 실제 KIS API 연결 주의사항

- 한국투자증권 KIS Open API의 모의투자/실전투자 `base_url`은 다를 수 있습니다.
- 모의투자와 실전투자는 `app_key`/`app_secret`이 다를 수 있습니다.
- `TR_ID`는 API 종류와 실전/모의 여부에 따라 다를 수 있습니다.
- 호출 제한이 있으므로 rate limit을 적용해야 합니다.
- 기본적으로 API 호출 사이에 최소 100~300ms sleep을 두도록 HTTP client 확장 지점을 남겨 두었습니다.
- 429 또는 유량 제한 응답이 오면 exponential backoff를 적용해야 합니다.
- 장중 투자자매매동향은 가집계/추정 성격일 수 있으며 장 마감 후 확정 데이터와 다를 수 있습니다.
- 실제 주문 전에는 반드시 `DRY_RUN=true`로 충분히 검증해야 합니다.

## systemd 예시

### `/etc/systemd/system/kis-supply-bot-monitor.service`

```ini
[Unit]
Description=KIS Supply Bot Monitor
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/opt/kis-supply-bot
EnvironmentFile=/opt/kis-supply-bot/.env
ExecStart=/opt/kis-supply-bot/kis-supply-bot monitor
Restart=always
RestartSec=5
User=root

[Install]
WantedBy=multi-user.target
```

### `/etc/systemd/system/kis-supply-bot-collect-1610.service`

```ini
[Unit]
Description=KIS Supply Bot Collect Daily 16:10

[Service]
Type=oneshot
WorkingDirectory=/opt/kis-supply-bot
EnvironmentFile=/opt/kis-supply-bot/.env
ExecStart=/opt/kis-supply-bot/kis-supply-bot collect-daily
User=root
```

### `/etc/systemd/system/kis-supply-bot-collect-1610.timer`

```ini
[Unit]
Description=Run KIS Supply Bot Collect Daily at 16:10

[Timer]
OnCalendar=Mon..Fri 16:10:00
Persistent=true

[Install]
WantedBy=timers.target
```

### `/etc/systemd/system/kis-supply-bot-build-candidates.service`

```ini
[Unit]
Description=KIS Supply Bot Build Candidates

[Service]
Type=oneshot
WorkingDirectory=/opt/kis-supply-bot
EnvironmentFile=/opt/kis-supply-bot/.env
ExecStart=/opt/kis-supply-bot/kis-supply-bot build-candidates
User=root
```

### `/etc/systemd/system/kis-supply-bot-build-candidates.timer`

```ini
[Unit]
Description=Run KIS Supply Bot Build Candidates at 18:30

[Timer]
OnCalendar=Mon..Fri 18:30:00
Persistent=true

[Install]
WantedBy=timers.target
```

## 운영 안전장치

- 실제 주문은 구현하지 않았습니다.
- `RealOrderExecutor`, `KisOrderExecutor`, `submit_buy_order`, `submit_sell_order`, `cancel_order`, `fetch_order_status`는 TODO 확장 지점으로만 남겨 두었습니다.
- 실제 주문이 추가되더라도 아래 조건을 모두 만족해야만 호출되도록 해야 합니다.
- `TRADING_MODE=real`
- `DRY_RUN=false`
- `KIS_APP_KEY`, `KIS_APP_SECRET`, `KIS_ACCOUNT_NO` 존재
- `enable_real_order=true`

## 개발 메모

이 저장소의 현재 산출물은 다른 PC에서 pull 받아 실제 빌드/운영을 진행하기 위한 소스와 설명서입니다. 이번 작업 지시에 따라 이 PC에서는 `cargo build`, `cargo test`, `cargo run` 검증을 수행하지 않았습니다.
