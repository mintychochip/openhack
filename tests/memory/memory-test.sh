#!/usr/bin/env bash
# memory-test.sh — E2E memory usage test for all OpenHack services
#
# Expected Behavior:
#     Orchestrates the full E2E memory test. Auto-detects K8s or Docker
#     Compose as the runtime. In Docker mode: starts containers via
#     docker compose, hits services on localhost directly. In K8s mode:
#     deploys via Helm, sets up port-forwards.
#     Phases: deploy → health check → 30s baseline → load → 5min soak → report → cleanup
#
# Args:
#     --mode docker|k8s|auto   Runtime mode (default: auto)
#     --skip-deploy            Don't start services (assume running)
#     --skip-cleanup           Don't stop services after test
#     --no-k6                  Skip k6 load (baseline + idle only)
#     --interval N             Stats interval seconds (default: 5)
#     --soak N                 Soak duration seconds (default: 300)
#     --namespace N            K8s namespace (default: openhack)
#     --release N              Helm release name (default: openhack)
#
# Raises:
#     Exits 1 if neither K8s nor Docker is available.
#
# Side Effects:
#     Starts/stops Docker containers or Helm releases.
#     Creates /tmp/openhack-memory/ with baseline.csv, load.csv, report.html.
#     Runs k6 and collect-stats.sh as subprocesses.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORK_DIR="/tmp/openhack-memory"
STOP_FILE="/tmp/openhack-stats-stop"
BASELINE_CSV="${WORK_DIR}/baseline.csv"
LOAD_CSV="${WORK_DIR}/load.csv"
REPORT_DIR="${WORK_DIR}"

MODE="auto"
SKIP_DEPLOY=false
SKIP_CLEANUP=false
NO_K6=false
INTERVAL=5
SOAK_SECONDS=300
NAMESPACE="openhack"
RELEASE_NAME="openhack"

while [ $# -gt 0 ]; do
    case "$1" in
        --mode)        shift; MODE="${1:-auto}" ;;
        --skip-deploy)  SKIP_DEPLOY=true ;;
        --skip-cleanup) SKIP_CLEANUP=true ;;
        --no-k6)        NO_K6=true ;;
        --interval)     shift; INTERVAL="${1:-5}" ;;
        --soak)         shift; SOAK_SECONDS="${1:-300}" ;;
        --namespace)    shift; NAMESPACE="${1:-openhack}" ;;
        --release)      shift; RELEASE_NAME="${1:-openhack}" ;;
        --help|-h)
            echo "Usage: memory-test.sh [options]"
            echo ""
            echo "  --mode docker|k8s|auto  Runtime mode (default: auto-detect)"
            echo "  --skip-deploy            Don't start services (assume running)"
            echo "  --skip-cleanup           Don't stop services after test"
            echo "  --no-k6                  Skip k6 load (baseline + idle only)"
            echo "  --interval N             Stats interval seconds (default: 5)"
            echo "  --soak N                 Soak duration seconds (default: 300)"
            echo "  --namespace N            K8s namespace (default: openhack)"
            echo "  --release N              Helm release (default: openhack)"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
    shift
done

# ── Service definitions ──
# Docker container_name:port (also used as k8s component names)
DOCKER_SERVICES=(
    "openhack-postgres:5432"
    "openhack-pgbouncer:6432"
    "openhack-auth-svc:3001"
    "openhack-core-svc:3002"
    "openhack-judging-svc:3003"
    "openhack-leaderboard-svc:3004"
    "openhack-mail-svc:3005"
    "openhack-notify-svc:3006"
    "openhack-ai-svc:3007"
    "openhack-analytics-svc:3008"
    "openhack-sponsors-svc:3009"
    "openhack-media-svc:3010"
    "openhack-gateway-svc:8000"
)

K8S_SERVICES=(
    "auth-svc:3001"
    "core-svc:3002"
    "judging-svc:3003"
    "leaderboard-svc:3004"
    "mail-svc:3005"
    "gateway:8000"
    "notify-svc:3006"
    "ai-svc:3007"
    "analytics-svc:3008"
    "sponsors-svc:3009"
    "media-svc:3010"
)

# ── Helpers ──

wait_for_health() {
    local name="$1"
    local port="$2"
    local url="http://localhost:${port}/health"
    local max_retries=20
    local wait_sec=3
    local attempt=0

    printf "  Waiting for %s on localhost:%s ... " "$name" "$port"
    while [ $attempt -lt $max_retries ]; do
        if curl -sf "$url" > /dev/null 2>&1; then
            echo "OK"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep "$wait_sec"
        printf "."
    done
    echo "FAILED (timeout)"
    return 1
}

start_collector() {
    local csv_path="$1"
    bash "${SCRIPT_DIR}/collect-stats.sh" "$csv_path" "$INTERVAL" "$STOP_FILE" "$MODE" "$NAMESPACE" &
    COLLECTOR_PID=$!
    sleep 2
}

stop_collector() {
    touch "$STOP_FILE"
    sleep 2
    if [ -n "${COLLECTOR_PID:-}" ] && kill -0 "$COLLECTOR_PID" 2>/dev/null; then
        kill "$COLLECTOR_PID" 2>/dev/null || true
        wait "$COLLECTOR_PID" 2>/dev/null || true
    fi
    rm -f "$STOP_FILE"
}

# ── Auto-detect mode ──
if [ "$MODE" = "auto" ]; then
    if command -v kubectl &>/dev/null && kubectl cluster-info &>/dev/null 2>&1; then
        MODE="k8s"
        echo "[detect] Kubernetes cluster found → k8s mode"
    elif command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
        MODE="docker"
        echo "[detect] Docker available, no K8s → docker mode"
    else
        echo "ERROR: Neither kubectl nor docker is available." >&2
        exit 1
    fi
fi

echo ""
echo "========================================================================="
echo "         OPENHACK E2E MEMORY USAGE TEST ($MODE)"
echo "========================================================================="
echo "  Start:        $(date -u +'%Y-%m-%d %H:%M:%S UTC')"
echo "  Mode:         $MODE"
echo "  Work dir:     $WORK_DIR"
echo "  Interval:     ${INTERVAL}s"
echo "  Soak:         ${SOAK_SECONDS}s"
echo "  Skip deploy:  $SKIP_DEPLOY"
echo "  Skip cleanup: $SKIP_CLEANUP"
echo "  No k6:        $NO_K6"
echo "========================================================================="
echo ""

rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"

COLLECTOR_PID=""
PORT_FORWARD_PIDS=()

# ═══════════════════════════════════════════════
# DOCKER MODE
# ═══════════════════════════════════════════════
if [ "$MODE" = "docker" ]; then

    # Phase 1: Deploy
    if [ "$SKIP_DEPLOY" = false ]; then
        echo "[phase-1] Starting services via docker compose..."
        docker compose --profile full --profile redis up -d --build 2>&1 || {
            echo "ERROR: docker compose up failed." >&2
            exit 1
        }
        echo "[phase-1] Containers starting."
        echo ""
    else
        echo "[phase-1] Skipping docker compose up (--skip-deploy)"
        echo ""
    fi

    # Phase 2: Health checks
    echo "[phase-2] Checking service health endpoints..."
    HEALTH_FAILURES=0
    for svc in "${DOCKER_SERVICES[@]}"; do
        name="${svc%%:*}"
        port="${svc##*:}"
        if ! docker ps --format '{{.Names}}' | grep -q "^${name}$"; then
            echo "  SKIP $name (not running)"
            continue
        fi
        if ! wait_for_health "$name" "$port"; then
            HEALTH_FAILURES=$((HEALTH_FAILURES + 1))
        fi
    done
    echo ""
    [ $HEALTH_FAILURES -gt 0 ] && echo "WARNING: $HEALTH_FAILURES service(s) failed health checks."

    # Phase 3: Baseline
    echo "[phase-3] Collecting 30s idle baseline..."
    start_collector "$BASELINE_CSV"
    sleep 30
    stop_collector
    echo "[phase-3] Baseline done ($(tail -n +2 "$BASELINE_CSV" | wc -l) samples)"
    echo ""

    # Phase 4: Load
    echo "[phase-4] Starting load test + stats collection..."
    start_collector "$LOAD_CSV"

    if [ "$NO_K6" = false ]; then
        if command -v k6 &>/dev/null; then
            echo "[phase-4] Running k6..."
            GATEWAY_URL="http://localhost:8000" \
            AUTH_URL="http://localhost:3001" CORE_URL="http://localhost:3002" \
            JUDGING_URL="http://localhost:3003" LEADERBOARD_URL="http://localhost:3004" \
            MAIL_URL="http://localhost:3005" NOTIFY_URL="http://localhost:3006" \
            AI_URL="http://localhost:3007" ANALYTICS_URL="http://localhost:3008" \
            SPONSORS_URL="http://localhost:3009" MEDIA_URL="http://localhost:3010" \
            k6 run "${SCRIPT_DIR}/k6-memory-test.js" \
                --out "json=${WORK_DIR}/k6-results.json" 2>&1 || {
                echo "WARNING: k6 exited with error. Continuing."
            }
            echo "[phase-4] k6 complete."
        else
            echo "[phase-4] k6 not found. Using synthetic curl load..."
            for i in $(seq 1 100); do
                for svc in "${DOCKER_SERVICES[@]}"; do
                    port="${svc##*:}"
                    curl -sf "http://localhost:${port}/health" > /dev/null 2>&1 || true
                done
                sleep 0.5
            done
            echo "[phase-4] Synthetic load complete."
        fi
    else
        echo "[phase-4] Load phase skipped (--no-k6)"
    fi

    # Phase 5: Soak
    echo "[phase-5] Soak phase: ${SOAK_SECONDS}s idle..."
    SOAK_ITERS=$((SOAK_SECONDS / INTERVAL))
    for i in $(seq 1 $SOAK_ITERS); do
        sleep "$INTERVAL"
        printf "\r[phase-5] Soaking... %d/%ds" $((i * INTERVAL)) "$SOAK_SECONDS"
    done
    echo ""
    stop_collector
    echo "[phase-5] Soak done ($(tail -n +2 "$LOAD_CSV" | wc -l) samples)"
    echo ""

    # Phase 6: Report
    echo "[phase-6] Generating report..."
    bash "${SCRIPT_DIR}/generate-report.sh" "$BASELINE_CSV" "$LOAD_CSV" "$REPORT_DIR" "docker"
    echo ""

    # Phase 7: Cleanup
    if [ "$SKIP_CLEANUP" = false ]; then
        echo "[phase-7] Stopping services..."
        docker compose --profile full --profile redis down 2>&1 || true
        echo "[phase-7] Services stopped."
    else
        echo "[phase-7] Skipping cleanup (--skip-cleanup)"
    fi

# ═══════════════════════════════════════════════
# K8S MODE
# ═══════════════════════════════════════════════
elif [ "$MODE" = "k8s" ]; then

    if ! command -v kubectl &>/dev/null || ! kubectl cluster-info &>/dev/null 2>&1; then
        echo "ERROR: kubectl cannot connect to a cluster." >&2
        exit 1
    fi

    # Phase 1: Deploy
    if [ "$SKIP_DEPLOY" = false ]; then
        if ! command -v helm &>/dev/null; then
            echo "ERROR: helm is required for k8s deploy." >&2
            exit 1
        fi
        echo "[phase-1] Deploying via Helm..."
        kubectl create namespace "$NAMESPACE" 2>/dev/null || true
        helm upgrade --install "$RELEASE_NAME" "${PROJECT_ROOT}/deploy/helm/openhack" \
            --namespace "$NAMESPACE" \
            -f "${PROJECT_ROOT}/deploy/helm/openhack/values.yaml" \
            --wait --timeout 5m 2>&1 || {
            echo "ERROR: Helm install failed." >&2
            exit 1
        }
        echo "[phase-1] Helm deploy complete."
        echo ""
    else
        echo "[phase-1] Skipping Helm deploy (--skip-deploy)"
        echo ""
    fi

    # Phase 2: Wait for pods
    echo "[phase-2] Waiting for pods Ready..."
    timeout=300; elapsed=0
    while [ $elapsed -lt $timeout ]; do
        not_ready=$(kubectl get pods -n "$NAMESPACE" -o json 2>/dev/null | python3 -c "
import json,sys
d=json.load(sys.stdin)
c=0
for p in d.get('items',[]):
    if p.get('status',{}).get('phase','') in ('Succeeded','Failed'): continue
    cs=[c for c in p.get('status',{}).get('conditions',[]) if c.get('type')=='Ready']
    if not cs or cs[0].get('status')!='True': c+=1
print(c)" 2>/dev/null || echo "999")
        [ "$not_ready" = "0" ] && break
        printf "\r[phase-2] %d pod(s) not ready (%ds/%ds)   " "$not_ready" "$elapsed" "$timeout"
        sleep 5; elapsed=$((elapsed + 5))
    done
    echo ""

    # Phase 3: Port-forwards
    echo "[phase-3] Setting up port-forwards..."
    for svc in "${K8S_SERVICES[@]}"; do
        name="${svc%%:*}"; port="${svc##*:}"
        svc_name="${RELEASE_NAME}-${name}"
        kubectl port-forward -n "$NAMESPACE" "svc/${svc_name}" "${port}:${port}" > /dev/null 2>&1 &
        PORT_FORWARD_PIDS+=($!)
        echo "  $svc_name -> localhost:$port"
    done
    sleep 3
    echo ""

    # Phase 4: Health checks
    echo "[phase-4] Health checks..."
    HEALTH_FAILURES=0
    for svc in "${K8S_SERVICES[@]}"; do
        name="${svc%%:*}"; port="${svc##*:}"
        wait_for_health "$name" "$port" || HEALTH_FAILURES=$((HEALTH_FAILURES + 1))
    done
    echo ""

    # Phase 5: Baseline
    echo "[phase-5] Collecting 30s idle baseline..."
    start_collector "$BASELINE_CSV"
    sleep 30
    stop_collector
    echo "[phase-5] Baseline done ($(tail -n +2 "$BASELINE_CSV" | wc -l) samples)"
    echo ""

    # Phase 6: Load
    echo "[phase-6] Starting load test + stats collection..."
    start_collector "$LOAD_CSV"
    if [ "$NO_K6" = false ]; then
        if command -v k6 &>/dev/null; then
            echo "[phase-6] Running k6..."
            GATEWAY_URL="http://localhost:8000" \
            AUTH_URL="http://localhost:3001" CORE_URL="http://localhost:3002" \
            JUDGING_URL="http://localhost:3003" LEADERBOARD_URL="http://localhost:3004" \
            MAIL_URL="http://localhost:3005" NOTIFY_URL="http://localhost:3006" \
            AI_URL="http://localhost:3007" ANALYTICS_URL="http://localhost:3008" \
            SPONSORS_URL="http://localhost:3009" MEDIA_URL="http://localhost:3010" \
            k6 run "${SCRIPT_DIR}/k6-memory-test.js" \
                --out "json=${WORK_DIR}/k6-results.json" 2>&1 || true
            echo "[phase-6] k6 complete."
        else
            echo "[phase-6] k6 not found. Using synthetic curl load..."
            for i in $(seq 1 100); do
                for svc in "${K8S_SERVICES[@]}"; do
                    curl -sf "http://localhost:${svc##*:}/health" > /dev/null 2>&1 || true
                done
                sleep 0.5
            done
        fi
    fi

    # Phase 7: Soak
    echo "[phase-7] Soak phase: ${SOAK_SECONDS}s idle..."
    SOAK_ITERS=$((SOAK_SECONDS / INTERVAL))
    for i in $(seq 1 $SOAK_ITERS); do
        sleep "$INTERVAL"
        printf "\r[phase-7] Soaking... %d/%ds" $((i * INTERVAL)) "$SOAK_SECONDS"
    done
    echo ""
    stop_collector
    echo "[phase-7] Soak done."
    echo ""

    # Phase 8: Report
    echo "[phase-8] Generating report..."
    bash "${SCRIPT_DIR}/generate-report.sh" "$BASELINE_CSV" "$LOAD_CSV" "$REPORT_DIR" "k8s"
    echo ""

    # Phase 9: Cleanup
    echo "[phase-9] Cleaning up..."
    for pid in "${PORT_FORWARD_PIDS[@]:-}"; do kill "$pid" 2>/dev/null || true; done
    if [ "$SKIP_CLEANUP" = false ]; then
        helm uninstall "$RELEASE_NAME" -n "$NAMESPACE" 2>&1 || true
        echo "[phase-9] Helm release uninstalled."
    else
        echo "[phase-9] Skipping cleanup (--skip-cleanup)"
    fi
fi

echo ""
echo "========================================================================="
echo "  Test complete."
echo "  Report:  file://${REPORT_DIR}/report.html"
echo "  Data:    ${BASELINE_CSV}, ${LOAD_CSV}"
echo "========================================================================="
echo ""
