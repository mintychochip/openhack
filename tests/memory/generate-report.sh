#!/usr/bin/env bash
# generate-report.sh — Parse stats CSVs and produce console + HTML report
#
# Expected Behavior:
#     Reads the baseline CSV and full stats CSV produced by collect-stats.sh.
#     Computes per-container statistics: baseline RSS, peak RSS, post-load
#     RSS, memory limit, peak % of limit, leak delta, OOM kill status, CPU.
#     Auto-detects Docker vs K8s mode from container names.
#
# Args:
#     $1 — Baseline CSV path (default: /tmp/openhack-memory/baseline.csv)
#     $2 — Full stats CSV path (default: /tmp/openhack-memory/load.csv)
#     $3 — Output directory for report.html (default: same dir as baseline CSV)
#     $4 — Mode: "k8s" or "docker" (default: auto-detect from CSV data)
#
# Raises:
#     Exits 1 if required CSV files do not exist.
#
# Side Effects:
#     Creates report.html in the output directory.

set -euo pipefail

BASELINE_CSV="${1:-/tmp/openhack-memory/baseline.csv}"
LOAD_CSV="${2:-/tmp/openhack-memory/load.csv}"
OUTPUT_DIR="${3:-$(dirname "$BASELINE_CSV")}"
MODE="${4:-auto}"
REPORT_FILE="${OUTPUT_DIR}/report.html"

if [ ! -f "$BASELINE_CSV" ]; then
    echo "ERROR: Baseline CSV not found: $BASELINE_CSV" >&2
    exit 1
fi
if [ ! -f "$LOAD_CSV" ]; then
    echo "ERROR: Load CSV not found: $LOAD_CSV" >&2
    exit 1
fi

# Auto-detect mode from container names in the CSV
if [ "$MODE" = "auto" ]; then
    first_container=$(tail -n +2 "$LOAD_CSV" | head -1 | cut -d',' -f3)
    if echo "$first_container" | grep -q "^openhack-"; then
        MODE="docker"
    else
        MODE="k8s"
    fi
fi

echo "[report] Parsing stats... (mode: $MODE)"
echo "[report] Baseline: $BASELINE_CSV"
echo "[report] Load:     $LOAD_CSV"

# CSV format: timestamp,pod_name,container_name,mem_used_bytes,mem_limit_bytes,cpu_cores,cpu_limit_cores,mem_percent,oom_killed

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Get unique container names from load CSV (skip header)
tail -n +2 "$LOAD_CSV" | cut -d',' -f3 | sort -u > "$TMPDIR/containers"
tail -n +2 "$BASELINE_CSV" | cut -d',' -f3 | sort -u > "$TMPDIR/baseline_containers"

declare -A BASELINE_RSS BASELINE_LIMIT
while read -r container; do
    [ -z "$container" ] && continue
    vals=$(grep ",${container}," "$BASELINE_CSV" | head -6 | cut -d',' -f4)
    if [ -z "$vals" ]; then
        BASELINE_RSS[$container]=0
        BASELINE_LIMIT[$container]=0
        continue
    fi
    sum=0; count=0
    while read -r v; do
        v_num=$(echo "$v" | awk '{printf "%.0f", $1}')
        sum=$((sum + v_num))
        count=$((count + 1))
    done <<< "$vals"
    if [ $count -gt 0 ]; then
        BASELINE_RSS[$container]=$((sum / count))
    else
        BASELINE_RSS[$container]=0
    fi
    limit=$(grep ",${container}," "$BASELINE_CSV" | head -1 | cut -d',' -f5)
    BASELINE_LIMIT[$container]=$(echo "$limit" | awk '{printf "%.0f", $1}')
done < "$TMPDIR/baseline_containers"

declare -A PEAK_RSS POSTLOAD_RSS PEAK_CPU AVG_CPU MIN_CPU LEAK_DELTA OOM_KILLED

while read -r container; do
    [ -z "$container" ] && continue
    mem_vals=$(grep ",${container}," "$LOAD_CSV" | cut -d',' -f4 | awk '{printf "%.0f\n", $1}')
    cpu_vals=$(grep ",${container}," "$LOAD_CSV" | cut -d',' -f6 | awk '{printf "%.4f\n", $1}')

    if [ -z "$mem_vals" ]; then
        PEAK_RSS[$container]=0
        POSTLOAD_RSS[$container]=0
        PEAK_CPU[$container]="0.0000"
        AVG_CPU[$container]="0.0000"
        MIN_CPU[$container]="0.0000"
        LEAK_DELTA[$container]=0
        OOM_KILLED[$container]="unknown"
        continue
    fi

    peak=$(echo "$mem_vals" | sort -n | tail -1)
    PEAK_RSS[$container]=${peak:-0}

    post_rows=$(grep ",${container}," "$LOAD_CSV" | tail -6 | cut -d',' -f4 | awk '{printf "%.0f\n", $1}')
    psum=0; pcount=0
    while read -r v; do
        psum=$((psum + $(echo "$v" | awk '{printf "%.0f", $1}')))
        pcount=$((pcount + 1))
    done <<< "$post_rows"
    if [ $pcount -gt 0 ]; then
        POSTLOAD_RSS[$container]=$((psum / pcount))
    else
        POSTLOAD_RSS[$container]=0
    fi

    bl=${BASELINE_RSS[$container]:-0}
    LEAK_DELTA[$container]=$((${POSTLOAD_RSS[$container]} - bl))

    min_cpu=$(echo "$cpu_vals" | sort -n | head -1)
    max_cpu=$(echo "$cpu_vals" | sort -n | tail -1)
    avg_cpu=$(echo "$cpu_vals" | awk '{sum+=$1; c++} END {if(c>0) printf "%.4f", sum/c; else print "0.0000"}')

    MIN_CPU[$container]=${min_cpu:-0.0000}
    PEAK_CPU[$container]=${max_cpu:-0.0000}
    AVG_CPU[$container]=${avg_cpu:-0.0000}

    oom_any=$(grep ",${container}," "$LOAD_CSV" | cut -d',' -f9 | grep -c "true" 2>/dev/null || true)
    oom_count=$(echo "$oom_any" | tr -d '[:space:]')
    if [ "${oom_count:-0}" -gt 0 ] 2>/dev/null; then
        OOM_KILLED[$container]="true"
    else
        OOM_KILLED[$container]="false"
    fi
done < "$TMPDIR/containers"

format_bytes() {
    local bytes="${1:-0}"
    bytes=$(echo "$bytes" | awk '{printf "%.0f", $1}')
    if [ "$bytes" -ge 1073741824 ] 2>/dev/null; then
        awk "BEGIN {printf \"%.1fGiB\", $bytes/1073741824}"
    elif [ "$bytes" -ge 1048576 ] 2>/dev/null; then
        awk "BEGIN {printf \"%.1fMiB\", $bytes/1048576}"
    elif [ "$bytes" -ge 1024 ] 2>/dev/null; then
        awk "BEGIN {printf \"%.1fKiB\", $bytes/1024}"
    else
        echo "${bytes}B"
    fi
}

format_cpu() {
    awk "BEGIN {printf \"%.2fm\", $1*1000}"
}

MODE_LABEL="$MODE"
[ "$MODE" = "docker" ] && MODE_LABEL="Docker Compose"

echo ""
echo "========================================================================="
echo "         OPENHACK MEMORY USAGE E2E TEST REPORT ($MODE_LABEL)"
echo "========================================================================="
echo ""

printf "%-28s %10s %10s %10s %10s %8s %10s %8s %10s\n" \
    "CONTAINER" "BASELINE" "PEAK" "POST-LOAD" "LIMIT" "PEAK%" "LEAK" "OOM" "PEAK_CPU"
printf "%-28s %10s %10s %10s %10s %8s %10s %8s %10s\n" \
    "----------------------------" "--------" "--------" "--------" "--------" "------" "--------" "--------" "--------"

WARN_COUNT=0
CRIT_COUNT=0
LEAK_COUNT=0

while read -r container; do
    [ -z "$container" ] && continue
    bl=${BASELINE_RSS[$container]:-0}
    pk=${PEAK_RSS[$container]:-0}
    pl=${POSTLOAD_RSS[$container]:-0}
    lim=${BASELINE_LIMIT[$container]:-0}
    leak=${LEAK_DELTA[$container]:-0}
    oom=${OOM_KILLED[$container]:-unknown}
    pcpu=${PEAK_CPU[$container]:-0}

    if [ "$lim" -gt 0 ] 2>/dev/null; then
        pct=$(awk "BEGIN {printf \"%.1f\", ($pk/$lim)*100}")
    else
        pct="N/A"
    fi

    bl_fmt=$(format_bytes "$bl")
    pk_fmt=$(format_bytes "$pk")
    pl_fmt=$(format_bytes "$pl")
    lim_fmt=$(format_bytes "$lim")
    leak_fmt=$(format_bytes "${leak#-}")
    pcpu_fmt=$(format_cpu "$pcpu")

    flags=""
    pct_num=$(echo "$pct" | awk '{printf "%.0f", $1}')
    if [ "$pct_num" -ge 95 ] 2>/dev/null; then
        flags="${flags}CRIT "
        CRIT_COUNT=$((CRIT_COUNT + 1))
    elif [ "$pct_num" -ge 80 ] 2>/dev/null; then
        flags="${flags}WARN "
        WARN_COUNT=$((WARN_COUNT + 1))
    fi

    bl_abs=$(echo "$bl" | awk '{printf "%.0f", $1}')
    if [ "$bl_abs" -gt 0 ] && [ "$leak" -gt 0 ] 2>/dev/null; then
        leak_pct=$(awk "BEGIN {printf \"%.0f\", ($leak/$bl_abs)*100}")
        if [ "$leak_pct" -ge 20 ]; then
            flags="${flags}LEAK"
            LEAK_COUNT=$((LEAK_COUNT + 1))
        fi
    fi

    printf "%-28s %10s %10s %10s %10s %7s%% %10s %8s %10s %s\n" \
        "$container" "$bl_fmt" "$pk_fmt" "$pl_fmt" "$lim_fmt" "$pct" "$leak_fmt" "$oom" "$pcpu_fmt" "$flags"
done < "$TMPDIR/containers"

echo ""
echo "========================================================================="
echo "  WARN: peak > 80% of limit  |  CRIT: peak > 95% of limit  |  LEAK: post-load > baseline + 20%"
echo "========================================================================="
echo ""
echo "  WARN count:  $WARN_COUNT"
echo "  CRIT count:  $CRIT_COUNT"
echo "  LEAK count:  $LEAK_COUNT"
echo ""

# ── HTML report ──
BADGE_CLASS="k8s-badge"
BADGE_LABEL="Kubernetes"
if [ "$MODE" = "docker" ]; then
    BADGE_CLASS="docker-badge"
    BADGE_LABEL="Docker Compose"
fi

{
    cat <<'HTML_HEAD'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>OpenHack Memory Usage E2E Report</title>
<style>
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 2rem; background: #0d1117; color: #c9d1d9; }
h1 { color: #58a6ff; border-bottom: 1px solid #30363d; padding-bottom: 0.5rem; }
h2 { color: #79c0ff; margin-top: 2rem; }
table { border-collapse: collapse; width: 100%; margin: 1rem 0; }
th, td { padding: 8px 12px; text-align: left; border: 1px solid #30363d; }
th { background: #161b22; color: #58a6ff; font-weight: 600; }
tr:nth-child(even) { background: #161b22; }
.warn { background: #3d2e00 !important; color: #d29922; }
.crit { background: #3d1e00 !important; color: #f85149; }
.leak { background: #1a1e40 !important; color: #a371f7; }
.summary { display: grid; grid-template-columns: repeat(4, 1fr); gap: 1rem; margin: 1rem 0; }
.card { background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 1rem; text-align: center; }
.card h3 { margin: 0; color: #8b949e; font-size: 0.8rem; text-transform: uppercase; }
.card .value { font-size: 2rem; font-weight: bold; margin: 0.5rem 0; }
.card.warn .value { color: #d29922; }
.card.crit .value { color: #f85149; }
.card.leak .value { color: #a371f7; }
.card.ok .value { color: #3fb950; }
.timestamp { color: #8b949e; font-size: 0.9rem; }
.k8s-badge { display: inline-block; background: #326ce5; color: white; padding: 2px 8px; border-radius: 4px; font-size: 0.75rem; margin-left: 0.5rem; }
.docker-badge { display: inline-block; background: #2496ed; color: white; padding: 2px 8px; border-radius: 4px; font-size: 0.75rem; margin-left: 0.5rem; }
</style>
</head>
<body>
HTML_HEAD

    echo "<h1>OpenHack Memory Usage E2E Report <span class='${BADGE_CLASS}'>${BADGE_LABEL}</span></h1>"
    echo "<p class='timestamp'>Generated: $(date -u +'%Y-%m-%d %H:%M:%S UTC')</p>"

    echo "<div class='summary'>"
    echo "<div class='card ok'><h3>Containers Tested</h3><div class='value'>$(wc -l < "$TMPDIR/containers")</div></div>"
    echo "<div class='card $([ $WARN_COUNT -gt 0 ] && echo warn || echo ok)'><h3>Warnings</h3><div class='value'>$WARN_COUNT</div></div>"
    echo "<div class='card $([ $CRIT_COUNT -gt 0 ] && echo crit || echo ok)'><h3>Critical</h3><div class='value'>$CRIT_COUNT</div></div>"
    echo "<div class='card $([ $LEAK_COUNT -gt 0 ] && echo leak || echo ok)'><h3>Leak Suspects</h3><div class='value'>$LEAK_COUNT</div></div>"
    echo "</div>"

    echo "<h2>Per-Container Memory &amp; CPU</h2>"
    echo "<table>"
    echo "<tr><th>Container</th><th>Baseline</th><th>Peak</th><th>Post-Load</th><th>Limit</th><th>Peak %</th><th>Leak</th><th>OOM Killed</th><th>CPU Peak</th><th>Flags</th></tr>"

    while read -r container; do
        [ -z "$container" ] && continue
        bl=${BASELINE_RSS[$container]:-0}
        pk=${PEAK_RSS[$container]:-0}
        pl=${POSTLOAD_RSS[$container]:-0}
        lim=${BASELINE_LIMIT[$container]:-0}
        leak=${LEAK_DELTA[$container]:-0}
        oom=${OOM_KILLED[$container]:-unknown}
        pcpu=${PEAK_CPU[$container]:-0}

        if [ "$lim" -gt 0 ] 2>/dev/null; then
            pct=$(awk "BEGIN {printf \"%.1f\", ($pk/$lim)*100}")
        else
            pct="N/A"
        fi

        bl_fmt=$(format_bytes "$bl")
        pk_fmt=$(format_bytes "$pk")
        pl_fmt=$(format_bytes "$pl")
        lim_fmt=$(format_bytes "$lim")
        leak_fmt=$(format_bytes "${leak#-}")
        pcpu_fmt=$(format_cpu "$pcpu")

        row_class=""
        pct_num=$(echo "$pct" | awk '{printf "%.0f", $1}')
        flags_str=""
        if [ "$pct_num" -ge 95 ] 2>/dev/null; then
            row_class="crit"; flags_str="CRIT"
        elif [ "$pct_num" -ge 80 ] 2>/dev/null; then
            row_class="warn"; flags_str="WARN"
        fi

        bl_abs=$(echo "$bl" | awk '{printf "%.0f", $1}')
        if [ "$bl_abs" -gt 0 ] && [ "$leak" -gt 0 ] 2>/dev/null; then
            leak_pct=$(awk "BEGIN {printf \"%.0f\", ($leak/$bl_abs)*100}")
            if [ "$leak_pct" -ge 20 ]; then
                [ -n "$flags_str" ] && flags_str="$flags_str, "
                flags_str="${flags_str}LEAK"
                [ -z "$row_class" ] && row_class="leak"
            fi
        fi

        echo "<tr class='$row_class'><td>$container</td><td>$bl_fmt</td><td>$pk_fmt</td><td>$pl_fmt</td><td>$lim_fmt</td><td>${pct}%</td><td>$leak_fmt</td><td>$oom</td><td>$pcpu_fmt</td><td>$flags_str</td></tr>"
    done < "$TMPDIR/containers"
    echo "</table>"

    echo "<h2>Flags Legend</h2>"
    echo "<ul>"
    echo "<li><strong>WARN</strong> — Peak memory exceeded 80% of container limit</li>"
    echo "<li><strong>CRIT</strong> — Peak memory exceeded 95% of container limit (OOM imminent)</li>"
    echo "<li><strong>LEAK</strong> — Post-load memory is &gt;20% above baseline (potential leak)</li>"
    echo "</ul>"

    echo "<h2>CPU Stats</h2>"
    echo "<table><tr><th>Container</th><th>CPU Min</th><th>CPU Avg</th><th>CPU Peak</th></tr>"
    while read -r container; do
        [ -z "$container" ] && continue
        echo "<tr><td>$container</td><td>$(format_cpu "${MIN_CPU[$container]:-0}")</td><td>$(format_cpu "${AVG_CPU[$container]:-0}")</td><td>$(format_cpu "${PEAK_CPU[$container]:-0}")</td></tr>"
    done < "$TMPDIR/containers"
    echo "</table>"

    echo "<p class='timestamp'>Data sources: $BASELINE_CSV, $LOAD_CSV</p>"
    echo "</body></html>"
} > "$REPORT_FILE"

echo "[report] HTML report written to: $REPORT_FILE"
echo "[report] Done."
