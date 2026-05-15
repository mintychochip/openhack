#!/usr/bin/env bash
# collect-stats.sh — Container memory/CPU stats collector (K8s or Docker)
#
# Expected Behavior:
#     Periodically samples container memory/CPU usage, writing timestamped
#     rows to a CSV. In k8s mode, uses kubectl top pods. In docker mode,
#     uses docker stats --no-stream. Terminates when a STOP signal file
#     is detected or on SIGTERM/SIGINT.
#
# Args:
#     $1 — Output CSV file path (required).
#     $2 — Sample interval in seconds (default: 5).
#     $3 — Stop signal file path (default: /tmp/openhack-stats-stop).
#     $4 — Mode: "k8s" or "docker" (default: auto-detect).
#     $5 — Kubernetes namespace when mode=k8s (default: openhack).
#
# Raises:
#     Exits 1 if output file path is not provided.
#
# Side Effects:
#     Creates and appends to the CSV file at $1.
#     Overwrites existing file with header on start.

set -euo pipefail

OUTPUT_CSV="${1:?Usage: collect-stats.sh <output.csv> [interval] [stop_file] [mode] [namespace]}"
INTERVAL="${2:-5}"
STOP_FILE="${3:-/tmp/openhack-stats-stop}"
MODE="${4:-auto}"
NAMESPACE="${5:-openhack}"

# Auto-detect mode
if [ "$MODE" = "auto" ]; then
    if command -v kubectl &>/dev/null && kubectl cluster-info &>/dev/null 2>&1; then
        MODE="k8s"
    elif command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
        MODE="docker"
    else
        echo "ERROR: Neither kubectl nor docker available." >&2
        exit 1
    fi
fi

rm -f "$STOP_FILE"

# Common CSV format: timestamp,container_name,container_name,mem_used_bytes,mem_limit_bytes,cpu_cores,cpu_limit_cores,mem_percent,oom_killed
# For docker mode: pod_name = container_name = docker container name
echo "timestamp,pod_name,container_name,mem_used_bytes,mem_limit_bytes,cpu_cores,cpu_limit_cores,mem_percent,oom_killed" > "$OUTPUT_CSV"

echo "[collect-stats] Writing to $OUTPUT_CSV every ${INTERVAL}s (mode: $MODE)"

cleanup() {
    local lines
    lines=$(tail -n +2 "$OUTPUT_CSV" | wc -l)
    echo "[collect-stats] Stopped after $lines samples"
    exit 0
}
trap cleanup SIGTERM SIGINT

# ── Docker mode ──
collect_docker() {
    while true; do
        if [ -f "$STOP_FILE" ]; then
            echo "[collect-stats] Stop signal detected"
            cleanup
        fi

        timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

        # Get list of openhack containers
        CONTAINERS=$(docker ps --filter "name=openhack" --format '{{.Names}}' 2>/dev/null || echo "")

        if [ -z "$CONTAINERS" ]; then
            echo "[collect-stats] WARNING: No openhack containers found"
            sleep "$INTERVAL"
            continue
        fi

        # docker stats for all openhack containers at once
        docker stats --no-stream --format \
            "{{.Name}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.CPUPerc}}" \
            $CONTAINERS 2>/dev/null | while IFS=$'\t' read -r name mem_usage mem_pct cpu_pct; do
            # Parse mem_usage: "25.5MiB / 64MiB"
            mem_used_str=$(echo "$mem_usage" | awk '{print $1}')
            mem_limit_str=$(echo "$mem_usage" | awk '{print $3}')

            # Convert to bytes
            mem_used_bytes=$(echo "$mem_used_str" | awk '{
                val=$1;
                if (val ~ /GiB/) { gsub(/GiB/,"",val); printf "%.0f", val*1073741824 }
                else if (val ~ /MiB/) { gsub(/MiB/,"",val); printf "%.0f", val*1048576 }
                else if (val ~ /KiB/) { gsub(/KiB/,"",val); printf "%.0f", val*1024 }
                else { gsub(/[^0-9.]/,"",val); printf "%.0f", val }
            }')

            mem_limit_bytes=$(echo "$mem_limit_str" | awk '{
                val=$1;
                if (val ~ /GiB/) { gsub(/GiB/,"",val); printf "%.0f", val*1073741824 }
                else if (val ~ /MiB/) { gsub(/MiB/,"",val); printf "%.0f", val*1048576 }
                else if (val ~ /KiB/) { gsub(/KiB/,"",val); printf "%.0f", val*1024 }
                else { gsub(/[^0-9.]/,"",val); printf "%.0f", val }
            }')

            mem_pct_clean=$(echo "$mem_pct" | tr -d '%')
            cpu_pct_clean=$(echo "$cpu_pct" | tr -d '%')

            # CPU as fraction of 100%
            cpu_cores=$(awk "BEGIN {printf \"%.4f\", $cpu_pct_clean/100}")
            cpu_limit_cores="1.0000"

            # OOM check from docker inspect
            oom=$(docker inspect --format '{{.State.OOMKilled}}' "$name" 2>/dev/null || echo "false")

            echo "${timestamp},${name},${name},${mem_used_bytes:-0},${mem_limit_bytes:-0},${cpu_cores:-0},${cpu_limit_cores},${mem_pct_clean:-0},${oom}" >> "$OUTPUT_CSV"
        done

        sleep "$INTERVAL"
    done
}

# ── K8s mode ──
collect_k8s() {
    while true; do
        if [ -f "$STOP_FILE" ]; then
            echo "[collect-stats] Stop signal detected"
            cleanup
        fi

        timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

        TOP_OUTPUT=$(kubectl top pods -n "$NAMESPACE" --containers --no-headers 2>/dev/null || echo "")

        if [ -z "$TOP_OUTPUT" ]; then
            echo "[collect-stats] WARNING: kubectl top pods returned empty"
            sleep "$INTERVAL"
            continue
        fi

        declare -A usage_lookup
        while IFS= read -r line; do
            pod=$(echo "$line" | awk '{print $1}')
            container=$(echo "$line" | awk '{print $2}')
            cpu=$(echo "$line" | awk '{print $3}')
            mem=$(echo "$line" | awk '{print $4}')
            key="${pod}/${container}"

            mem_bytes=$(echo "$mem" | awk '{
                val=$1;
                if (val ~ /Gi/) { gsub(/Gi/,"",val); printf "%.0f", val*1073741824 }
                else if (val ~ /Mi/) { gsub(/Mi/,"",val); printf "%.0f", val*1048576 }
                else if (val ~ /Ki/) { gsub(/Ki/,"",val); printf "%.0f", val*1024 }
                else { gsub(/[a-zA-Z]/,"",val); printf "%.0f", val }
            }')

            cpu_cores=$(echo "$cpu" | awk '{
                val=$1;
                if (val ~ /m$/) { gsub(/m$/,"",val); printf "%.4f", val/1000 }
                else { printf "%.4f", val }
            }')

            usage_lookup["$key"]="${cpu_cores} ${mem_bytes}"
        done <<< "$TOP_OUTPUT"

        PODS_JSON=$(kubectl get pods -n "$NAMESPACE" -o json 2>/dev/null || echo '{"items":[]}')

        while IFS= read -r line; do
            pod=$(echo "$line" | awk -F'\t' '{print $1}')
            container=$(echo "$line" | awk -F'\t' '{print $2}')
            mem_limit=$(echo "$line" | awk -F'\t' '{print $3}')
            cpu_limit=$(echo "$line" | awk -F'\t' '{print $4}')
            oom=$(echo "$line" | awk -F'\t' '{print $5}')

            mem_limit_bytes=$(echo "$mem_limit" | awk '{
                val=$1;
                if (val ~ /Gi/) { gsub(/Gi/,"",val); printf "%.0f", val*1073741824 }
                else if (val ~ /Mi/) { gsub(/Mi/,"",val); printf "%.0f", val*1048576 }
                else if (val ~ /Ki/) { gsub(/Ki/,"",val); printf "%.0f", val*1024 }
                else { gsub(/[a-zA-Z]/,"",val); printf "%.0f", val }
            }')

            cpu_limit_cores=$(echo "$cpu_limit" | awk '{
                val=$1;
                if (val ~ /m$/) { gsub(/m$/,"",val); printf "%.4f", val/1000 }
                else { printf "%.4f", val }
            }')

            key="${pod}/${container}"
            if [ -n "${usage_lookup[$key]+x}" ]; then
                usage_parts=(${usage_lookup[$key]})
                cpu_used="${usage_parts[0]}"
                mem_used="${usage_parts[1]}"
            else
                cpu_used="0"
                mem_used="0"
            fi

            if [ "$mem_limit_bytes" -gt 0 ] 2>/dev/null; then
                mem_pct=$(awk "BEGIN {printf \"%.1f\", ($mem_used/$mem_limit_bytes)*100}")
            else
                mem_pct="0.0"
            fi

            echo "${timestamp},${pod},${container},${mem_used:-0},${mem_limit_bytes:-0},${cpu_used:-0},${cpu_limit_cores:-0},${mem_pct},${oom}" >> "$OUTPUT_CSV"
        done < <(echo "$PODS_JSON" | python3 -c "
import json, sys
data = json.load(sys.stdin)
for pod in data.get('items', []):
    pname = pod['metadata']['name']
    statuses = pod.get('status', {}).get('containerStatuses', []) or []
    oom_map = {}
    for cs in statuses:
        cname = cs.get('name', '')
        last_state = cs.get('lastState', {}) or {}
        terminated = last_state.get('terminated', {}) or {}
        oom_map[cname] = 'true' if terminated.get('reason') == 'OOMKilled' else 'false'
    for spec_c in pod.get('spec', {}).get('containers', []):
        cname = spec_c.get('name', '')
        limits = spec_c.get('resources', {}).get('limits', {}) or {}
        mem_limit = limits.get('memory', '0')
        cpu_limit = limits.get('cpu', '0')
        oom = oom_map.get(cname, 'false')
        print(f'{pname}\t{cname}\t{mem_limit}\t{cpu_limit}\t{oom}')
" 2>/dev/null)

        unset usage_lookup
        sleep "$INTERVAL"
    done
}

# ── Run selected mode ──
case "$MODE" in
    docker) collect_docker ;;
    k8s)    collect_k8s ;;
    *)      echo "ERROR: Unknown mode '$MODE'. Use 'docker', 'k8s', or 'auto'." >&2; exit 1 ;;
esac
