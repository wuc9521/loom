#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import json
import tarfile
from collections import defaultdict
from pathlib import Path
from statistics import median

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import LogLocator, NullFormatter


RESOLVED_STATUSES = {"solved", "infeasible"}
TIMEOUT_STATUSES = {"solve_timeout", "sygus_timeout"}
STATUS_ORDER = ["solved", "infeasible", "solve_timeout", "sygus_timeout", "error"]

GROUP_COLORS = [
    "#1F4E79",
    "#A23B32",
    "#2C6E49",
    "#8E6C8A",
    "#C0841A",
    "#6B7280",
    "#00798C",
    "#9A3412",
    "#5B8E7D",
    "#7C3AED",
    "#B45309",
    "#374151",
    "#0F766E",
    "#7F1D1D",
    "#4C6A92",
]

LEVEL_COLORS = {
    "FO0": "#1F4E79",
    "FO1": "#A23B32",
    "FO2": "#2C6E49",
    "FO3": "#8E6C8A",
    "FO1-LFP": "#1F4E79",
    "FO2-LFP": "#A23B32",
    "oracle": "#111827",
}

COMPARE_COLORS = {
    "Delis": "#1F4E79",
    "LearnCTL": "#A23B32",
    "Folseparators": "#2C6E49",
    "cvc5": "#A23B32",
    "sample2LTL": "#A23B32",
}

LEARNCTL_VARIANT_MAP = {
    "": "all",
    "-R": "all-R",
    "-E": "all-E",
    "-F": "full",
    "-F -R": "full-R",
    "-F -E": "full-E",
    "-U": "until",
    "-U -R": "until-R",
    "-U -E": "until-E",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render paper-style benchmark PDFs from delis TSV results."
    )
    parser.add_argument(
        "--results-dir",
        default="../bench-results",
        help="Directory containing TSV result files.",
    )
    parser.add_argument(
        "--out-dir",
        default="out",
        help="Directory for generated PDF figures.",
    )
    parser.add_argument(
        "--title",
        default="Delis Benchmark Results",
        help="Title prefix used in figure headers.",
    )
    parser.add_argument(
        "--learnctl-csv",
        default="../bench-results/ctl-learnctl-artifact.csv",
        help="LearnCTL benchmark CSV for CTL comparisons.",
    )
    parser.add_argument(
        "--folseparators-tsv",
        default="../bench-results/fo-folseparators-artifact.tsv",
        help="Folseparators benchmark TSV under bench-results.",
    )
    parser.add_argument(
        "--samples2ltl-csv",
        default="",
        help="Optional sample2LTL benchmark CSV for LTL/sample2LTL comparisons.",
    )
    return parser.parse_args()


def configure_style() -> None:
    plt.rcParams.update(
        {
            "font.family": "serif",
            "font.size": 11,
            "axes.titlesize": 13,
            "axes.labelsize": 11,
            "legend.fontsize": 9,
            "xtick.labelsize": 9,
            "ytick.labelsize": 9,
            "axes.spines.top": False,
            "axes.spines.right": False,
            "axes.linewidth": 0.8,
            "pdf.fonttype": 42,
            "ps.fonttype": 42,
        }
    )


def infer_family(path: str, group: str) -> str:
    parts = path.split("/")
    if len(parts) >= 3 and parts[0] == "benchmark":
        if parts[1] == "FO-LFP":
            return "FO-LFP"
        if parts[1] in {"CTL", "FO", "LTL", "Reg", "String"}:
            return parts[1]
    if group.startswith("fo-lfp-"):
        return "FO-LFP"
    if group.startswith("fo-"):
        return "FO"
    if group.startswith("ltl-"):
        return "LTL"
    if group.startswith("reg-"):
        return "Reg"
    if group.startswith("ctl-"):
        return "CTL"
    return "Other"


def normalize_row(tsv_name: str, row: dict[str, str]) -> dict[str, str]:
    normalized = dict(row)
    normalized["_tsv"] = tsv_name
    normalized.setdefault("group", Path(tsv_name).stem)

    if "path" not in normalized or not normalized.get("path"):
        if "case" in normalized and normalized["case"]:
            normalized["path"] = normalized["case"]
        else:
            normalized["path"] = normalized["group"]

    normalized.setdefault("status", "error")
    normalized.setdefault("total_ms", "0")
    normalized.setdefault("detail", normalized.get("error", ""))
    status = (normalized.get("status") or "").strip()
    error = (normalized.get("error") or "").strip().lower()
    if status == "timeout":
        normalized["status"] = "solve_timeout"
    elif status == "failed" and error == "timeout":
        normalized["status"] = "solve_timeout"
    normalized["_family"] = infer_family(normalized["path"], normalized["group"])
    return normalized


def load_rows(results_dir: Path) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for tsv in sorted(results_dir.glob("*.tsv")):
        with tsv.open(newline="") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            for row in reader:
                rows.append(normalize_row(tsv.name, row))
    return rows


def seconds(row: dict[str, str]) -> float:
    solve_ms = row.get("solve_ms", "")
    if solve_ms not in ("", None):
        try:
            return float(solve_ms) / 1000.0
        except ValueError:
            pass
    return float(row["total_ms"]) / 1000.0


def grouped(rows: list[dict[str, str]], field: str) -> dict[str, list[dict[str, str]]]:
    out: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in rows:
        out[row[field]].append(row)
    return dict(sorted(out.items()))


def resolved_times(rows: list[dict[str, str]]) -> list[float]:
    return sorted(seconds(row) for row in rows if row["status"] in RESOLVED_STATUSES)


def cactus_times(
    rows: list[dict[str, str]],
) -> tuple[list[float], int, int]:
    solved = sorted(seconds(row) for row in rows if row["status"] in RESOLVED_STATUSES)
    timeouts = sorted(seconds(row) for row in rows if row["status"] in TIMEOUT_STATUSES)
    return solved + timeouts, len(solved), len(timeouts)


def status_counts(rows: list[dict[str, str]]) -> dict[str, int]:
    counts = {status: 0 for status in STATUS_ORDER}
    for row in rows:
        counts[row["status"]] = counts.get(row["status"], 0) + 1
    return counts


def suite_slug(group: str) -> str:
    return group.replace("/", "-")


def benchmark_leaf(row: dict[str, str]) -> str:
    return Path(row["path"]).name


def benchmark_stem(row: dict[str, str]) -> str:
    return Path(row["path"]).stem


def best_solved_seconds(candidates: list[dict[str, str]]) -> float | None:
    solved = [seconds(row) for row in candidates if row["status"] == "solved"]
    return min(solved) if solved else None


def percent_xs(resolved_count: int, total_count: int) -> list[float]:
    if resolved_count <= 0 or total_count <= 0:
        return []
    return [100.0 * idx / total_count for idx in range(1, resolved_count + 1)]


def draw_timeout_cutoff(
    ax: plt.Axes,
    cutoff_x: float | None,
) -> None:
    if cutoff_x is None:
        return
    ax.axvline(
        cutoff_x,
        color="#6B7280",
        linestyle="--",
        linewidth=1.0,
        alpha=0.9,
    )
    ax.text(
        cutoff_x,
        0.02,
        f"{cutoff_x:.1f}%",
        transform=ax.get_xaxis_transform(),
        rotation=90,
        ha="right",
        va="bottom",
        fontsize=8.5,
        color="#4B5563",
        backgroundcolor="white",
    )


def plot_row_value(row: dict[str, str]) -> float:
    return seconds(row)


def suite_order_key(row: dict[str, str]) -> tuple[int, float, str]:
    status = row["status"]
    if status in RESOLVED_STATUSES:
        return (0, plot_row_value(row), benchmark_leaf(row))
    if status in TIMEOUT_STATUSES:
        return (1, plot_row_value(row), benchmark_leaf(row))
    return (2, float("inf"), benchmark_leaf(row))


def sequence_points_from_rows(
    rows: list[dict[str, str]],
) -> tuple[list[float], list[float], int, int, float | None]:
    ordered_rows = sorted(rows, key=suite_order_key)
    total_count = len(ordered_rows)
    xs: list[float] = []
    ys: list[float] = []
    solved_count = 0
    timeout_count = 0
    cutoff_x: float | None = None
    for idx, row in enumerate(ordered_rows, start=1):
        status = row["status"]
        if status in RESOLVED_STATUSES:
            solved_count += 1
            xs.append(100.0 * idx / total_count)
            ys.append(plot_row_value(row))
        elif status in TIMEOUT_STATUSES:
            timeout_count += 1
            point_x = 100.0 * idx / total_count
            if cutoff_x is None:
                cutoff_x = point_x
            xs.append(point_x)
            ys.append(plot_row_value(row))
    return xs, ys, solved_count, timeout_count, cutoff_x


def sequence_points_from_entries(
    entries: list[dict[str, float | str | None]],
    total_count: int,
) -> tuple[list[float], list[float], int, int, float | None]:
    def entry_key(entry: dict[str, float | str | None]) -> tuple[int, float]:
        status = entry["status"]
        secs = entry["seconds"]
        value = float(secs) if secs is not None else float("inf")
        if status == "solved":
            return (0, value)
        if status in TIMEOUT_STATUSES:
            return (1, value)
        return (2, float("inf"))

    ordered = sorted(entries, key=entry_key)
    xs: list[float] = []
    ys: list[float] = []
    solved_count = 0
    timeout_count = 0
    cutoff_x: float | None = None
    shown_idx = 0
    for entry in ordered:
        status = entry["status"]
        secs = entry["seconds"]
        if secs is None or status not in RESOLVED_STATUSES.union(TIMEOUT_STATUSES):
            continue
        shown_idx += 1
        point_x = 100.0 * shown_idx / total_count
        if status in RESOLVED_STATUSES:
            solved_count += 1
        elif status in TIMEOUT_STATUSES:
            timeout_count += 1
            if cutoff_x is None:
                cutoff_x = point_x
        xs.append(point_x)
        ys.append(float(secs))
    return xs, ys, solved_count, timeout_count, cutoff_x


def sequence_points_from_entries_with_order(
    entries: list[dict[str, float | str | None]],
    total_count: int,
    resolved_order: tuple[str, ...],
) -> tuple[list[float], list[float], int, int, float | None]:
    resolved_set = set(resolved_order)

    def entry_key(entry: dict[str, float | str | None]) -> tuple[int, float]:
        status = str(entry["status"])
        secs = entry["seconds"]
        value = float(secs) if secs is not None else float("inf")
        if status in resolved_set:
            return (0, value)
        if status in TIMEOUT_STATUSES:
            return (1, value)
        return (2, float("inf"))

    ordered = sorted(entries, key=entry_key)
    xs: list[float] = []
    ys: list[float] = []
    resolved_count = 0
    timeout_count = 0
    cutoff_x: float | None = None
    shown_idx = 0
    for entry in ordered:
        status = str(entry["status"])
        secs = entry["seconds"]
        if secs is None or status not in resolved_set.union(TIMEOUT_STATUSES):
            continue
        shown_idx += 1
        point_x = 100.0 * shown_idx / total_count
        if status in resolved_set:
            resolved_count += 1
        elif status in TIMEOUT_STATUSES:
            timeout_count += 1
            if cutoff_x is None:
                cutoff_x = point_x
        xs.append(point_x)
        ys.append(float(secs))
    return xs, ys, resolved_count, timeout_count, cutoff_x


def prep_axes(ax: plt.Axes, title: str) -> None:
    ax.set_title(title, loc="left", fontweight="bold", pad=10)
    ax.set_xlabel("Benchmarks resolved (%)")
    ax.set_ylabel("Time (seconds)")
    ax.set_xlim(0, 100)
    ax.set_yscale("log")
    ax.yaxis.set_major_locator(LogLocator(base=10))
    ax.yaxis.set_minor_locator(LogLocator(base=10, subs=(0.2, 0.4, 0.6, 0.8)))
    ax.yaxis.set_minor_formatter(NullFormatter())
    ax.grid(which="major", color="#D1D5DB", linewidth=0.8)
    ax.grid(which="minor", color="#E5E7EB", linewidth=0.5, alpha=0.9)


def plot_single_suite_cactus(
    group: str,
    rows: list[dict[str, str]],
    out_path: Path,
    title: str,
    color: str,
) -> None:
    total_count = len(rows)
    xs, ys, solved_count, timeout_count, cutoff_x = sequence_points_from_rows(rows)
    fig, ax = plt.subplots(figsize=(8.6, 5.8), constrained_layout=True)
    prep_axes(ax, title)
    if ys:
        ax.step(
            xs,
            ys,
            where="post",
            linewidth=2.2,
            color=color,
            label=f"Delis (n={total_count}, solved={solved_count}, timeout={timeout_count})",
        )
        draw_timeout_cutoff(ax, cutoff_x)
        ax.legend(frameon=False, loc="upper left")
    subtitle = (
        f"solved={solved_count} / timeout={timeout_count} / "
        f"shown={len(ys)} / total={total_count}"
    )
    ax.text(
        0.01,
        0.98,
        subtitle,
        transform=ax.transAxes,
        ha="left",
        va="top",
        fontsize=9,
        color="#374151",
    )
    fig.savefig(out_path, format="pdf", bbox_inches="tight")
    plt.close(fig)


def plot_parallel_levels(
    title: str,
    level_order: list[str],
    level_rows: dict[str, list[dict[str, str]]],
    out_path: Path,
) -> None:
    instance_map: dict[str, dict[str, dict[str, str]]] = defaultdict(dict)
    for level, rows in level_rows.items():
        for row in rows:
            instance_map[benchmark_leaf(row)][level] = row

    ordered_instances = []
    for leaf, per_level in instance_map.items():
        best = best_solved_seconds(list(per_level.values()))
        if best is not None:
            ordered_instances.append((leaf, best))
    ordered_instances.sort(key=lambda item: item[1])

    if not ordered_instances:
        return

    total_count = len(ordered_instances)
    xs = percent_xs(total_count, total_count)
    fig, ax = plt.subplots(figsize=(10.8, 6.6), constrained_layout=True)
    prep_axes(ax, title)
    oracle = [best for _, best in ordered_instances]
    ax.plot(
        xs,
        oracle,
        linewidth=2.6,
        color=LEVEL_COLORS["oracle"],
        linestyle="--",
        label=f"oracle (n={total_count})",
    )

    for level in level_order:
        ys = []
        solved = 0
        timed_out = 0
        for leaf, _ in ordered_instances:
            row = instance_map[leaf].get(level)
            if row and row["status"] == "solved":
                ys.append(seconds(row))
                solved += 1
            elif row and row["status"] in TIMEOUT_STATUSES:
                ys.append(seconds(row))
                timed_out += 1
            else:
                ys.append(float("nan"))
        ax.scatter(
            xs,
            ys,
            s=10,
            color=LEVEL_COLORS[level],
            label=f"{level} (solved={solved}/{total_count}, timeout={timed_out})",
            alpha=0.9,
        )

    ax.legend(frameon=False, ncol=min(len(level_order) + 1, 3), loc="upper left")
    ax.text(
        0.01,
        0.02,
        "x-order is sorted by oracle runtime min{levels}; only solved outcomes contribute to the oracle.",
        transform=ax.transAxes,
        ha="left",
        va="bottom",
        fontsize=8.5,
        color="#4B5563",
    )
    fig.savefig(out_path, format="pdf", bbox_inches="tight")
    plt.close(fig)


def plot_status_breakdown(
    rows: list[dict[str, str]],
    out_path: Path,
    title: str,
) -> None:
    by_group = grouped(rows, "group")
    groups = list(by_group.keys())
    counts = [status_counts(by_group[group]) for group in groups]
    totals = [len(by_group[group]) for group in groups]

    fig_height = max(5.5, 0.36 * len(groups) + 1.8)
    fig, ax = plt.subplots(figsize=(13.5, fig_height), constrained_layout=True)
    ax.set_title(title, loc="left", fontweight="bold", pad=10)
    ax.set_xlabel("Benchmarks (%)")
    ax.set_xlim(0, 100)
    ax.grid(axis="x", color="#D1D5DB", linewidth=0.8)
    ax.set_axisbelow(True)

    colors = {
        "solved": "#111827",
        "infeasible": "#6B7280",
        "solve_timeout": "#B45309",
        "sygus_timeout": "#D97706",
        "error": "#991B1B",
    }

    left = [0.0] * len(groups)
    y = list(range(len(groups)))
    for status in STATUS_ORDER:
        values = [
            (100.0 * entry.get(status, 0) / total) if total else 0.0
            for entry, total in zip(counts, totals)
        ]
        ax.barh(
            y,
            values,
            left=left,
            height=0.72,
            color=colors[status],
            edgecolor="white",
            linewidth=0.6,
            label=status.replace("_", " "),
        )
        left = [lhs + rhs for lhs, rhs in zip(left, values)]

    labels = [f"{group} (n={total})" for group, total in zip(groups, totals)]
    ax.set_yticks(y, labels)
    ax.invert_yaxis()
    ax.legend(frameon=False, ncol=5, loc="upper center", bbox_to_anchor=(0.5, 1.08))
    fig.savefig(out_path, format="pdf", bbox_inches="tight")
    plt.close(fig)


def write_summary(rows: list[dict[str, str]], out_path: Path) -> None:
    by_group = grouped(rows, "group")
    with out_path.open("w", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(
            [
                "group",
                "family",
                "total",
                "resolved",
                "solved",
                "infeasible",
                "timeouts",
                "errors",
                "median_resolved_seconds",
            ]
        )
        for group, group_rows in by_group.items():
            times = resolved_times(group_rows)
            counts = status_counts(group_rows)
            writer.writerow(
                [
                    group,
                    group_rows[0]["_family"],
                    len(group_rows),
                    len(times),
                    counts["solved"],
                    counts["infeasible"],
                    counts["solve_timeout"] + counts["sygus_timeout"],
                    counts["error"],
                    f"{median(times):.6f}" if times else "",
                ]
            )


def learnctl_variant(parameters: str) -> str | None:
    return LEARNCTL_VARIANT_MAP.get(parameters.strip())


def learnctl_filename(row: dict[str, str]) -> str | None:
    variant = learnctl_variant(row.get("Parameters", ""))
    if variant is None:
        return None
    positive = row["Positive sample"].strip()
    mutants = []
    for token in row["Negative sample"].split():
        token = token.strip()
        if not token:
            continue
        mutants.append(token.split("/")[-1].replace("-", "_"))
    if not positive or not mutants:
        return None
    return f"{positive}--{'__'.join(mutants)}--{variant}.sl"


def parse_learnctl_seconds(value: str) -> tuple[str, float | None]:
    text = (value or "").strip()
    if not text:
        return ("error", None)
    if text.isdigit():
        return ("solved", float(text) / 1000.0)
    if text.startswith("timed out at "):
        parts = text.split()
        try:
            return ("solve_timeout", float(parts[3]))
        except (IndexError, ValueError):
            return ("solve_timeout", None)
    return ("error", None)


def load_learnctl_series(csv_path: Path) -> dict[str, dict[str, float | str | None]]:
    if not csv_path.exists():
        return {}
    series: dict[str, dict[str, float | str | None]] = {}
    with csv_path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            filename = learnctl_filename(row)
            if not filename:
                continue
            status, secs = parse_learnctl_seconds(row.get("Time (ms)", ""))
            current = series.get(filename)
            candidate = {"status": status, "seconds": secs}
            if current is None:
                series[filename] = candidate
                continue
            current_secs = current["seconds"]
            if status == "solved" and (
                current["status"] != "solved"
                or current_secs is None
                or (secs is not None and secs < current_secs)
            ):
                series[filename] = candidate
    return series


def parse_samples2ltl_seconds(value: str) -> tuple[str, float | None]:
    text = (value or "").strip()
    if not text or text == "/":
        return ("solve_timeout", None)
    try:
        return ("solved", float(text))
    except ValueError:
        return ("error", None)


def load_samples2ltl_series(csv_path: Path) -> dict[str, dict[str, float | str | None]]:
    if not csv_path.exists():
        return {}
    series: dict[str, dict[str, float | str | None]] = {}
    with csv_path.open(newline="") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            trace_name = (row.get("name of the test file") or "").strip()
            if not trace_name:
                continue
            filename = f"{Path(trace_name).stem}.sl"
            status, secs = parse_samples2ltl_seconds(row.get("dt solving time", ""))
            candidate = {"status": status, "seconds": secs}
            current = series.get(filename)
            if current is None:
                series[filename] = candidate
                continue
            current_secs = current["seconds"]
            if status == "solved" and (
                current["status"] != "solved"
                or current_secs is None
                or (secs is not None and secs < current_secs)
            ):
                series[filename] = candidate
    return series


def load_folseparators_rows(path: Path) -> list[dict]:
    if not path.exists():
        return []
    if path.suffix == ".json":
        return json.loads(path.read_text())
    if path.suffixes[-2:] == [".tar", ".gz"] or path.suffix == ".tgz":
        preferred = [
            "data/original-submission/pldi-cvc4.json",
            "data/pldi20-learning.json",
        ]
        with tarfile.open(path, "r:gz") as outer:
            manifest = json.load(outer.extractfile("manifest.json"))
            layers = manifest[0]["Layers"]
            for target in preferred:
                payload = None
                for layer_name in layers:
                    layer = tarfile.open(
                        fileobj=outer.extractfile(layer_name), mode="r:"
                    )
                    try:
                        member = layer.getmember(target)
                    except KeyError:
                        continue
                    payload = layer.extractfile(member).read()
                    break
                if payload is not None:
                    return json.loads(payload)
        return []
    return []


def best_series_update(
    series: dict[str, dict[str, float | str | None]],
    key: str,
    candidate: dict[str, float | str | None],
) -> None:
    current = series.get(key)
    if current is None:
        series[key] = candidate
        return
    if candidate["status"] == "solved":
        current_secs = current["seconds"]
        cand_secs = candidate["seconds"]
        if (
            current["status"] != "solved"
            or current_secs is None
            or (cand_secs is not None and cand_secs < current_secs)
        ):
            series[key] = candidate


def load_tsv_series(
    path: Path,
    key_field: str = "path",
) -> dict[str, dict[str, float | str | None]]:
    if not path.exists():
        return {}
    series: dict[str, dict[str, float | str | None]] = {}
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        for raw in reader:
            row = normalize_row(path.name, raw)
            key = Path(row.get(key_field, row["path"])).name
            status = row["status"]
            secs = (
                seconds(row)
                if status in RESOLVED_STATUSES or status in TIMEOUT_STATUSES
                else None
            )
            best_series_update(series, key, {"status": status, "seconds": secs})
    return series


def load_folseparators_series_from_tsv(
    path: Path,
) -> dict[str, dict[str, float | str | None]]:
    if not path.exists():
        return {}
    series: dict[str, dict[str, float | str | None]] = {}
    with path.open(newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        for raw in reader:
            row = normalize_row(path.name, raw)
            key = Path(row["path"]).name
            status = row["status"]
            secs = (
                seconds(row)
                if status in RESOLVED_STATUSES or status in TIMEOUT_STATUSES
                else None
            )
            best_series_update(series, key, {"status": status, "seconds": secs})
    return series


def delis_series(
    rows: list[dict[str, str]],
) -> dict[str, dict[str, float | str | None]]:
    series: dict[str, dict[str, float | str | None]] = {}
    for row in rows:
        leaf = benchmark_leaf(row)
        secs = (
            seconds(row)
            if row["status"] in RESOLVED_STATUSES or row["status"] in TIMEOUT_STATUSES
            else None
        )
        candidate = {"status": row["status"], "seconds": secs}
        current = series.get(leaf)
        if current is None:
            series[leaf] = candidate
        elif row["status"] == "solved":
            current_secs = current["seconds"]
            if (
                current["status"] != "solved"
                or current_secs is None
                or secs < current_secs
            ):
                series[leaf] = candidate
    return series


def delis_series_by_stem(
    rows: list[dict[str, str]],
) -> dict[str, dict[str, float | str | None]]:
    series: dict[str, dict[str, float | str | None]] = {}
    for row in rows:
        key = benchmark_stem(row)
        secs = (
            seconds(row)
            if row["status"] in RESOLVED_STATUSES or row["status"] in TIMEOUT_STATUSES
            else None
        )
        best_series_update(series, key, {"status": row["status"], "seconds": secs})
    return series


def comparable_order(
    series_map: dict[str, dict[str, dict[str, float | str | None]]],
) -> list[str]:
    ordered = []
    for key in set().union(*[set(series.keys()) for series in series_map.values()]):
        solved = [
            float(entry["seconds"])
            for entry in (series.get(key) for series in series_map.values())
            if entry and entry["status"] == "solved" and entry["seconds"] is not None
        ]
        if solved:
            ordered.append((key, min(solved)))
    ordered.sort(key=lambda item: item[1])
    return [key for key, _ in ordered]


def comparison_keys(
    series_map: dict[str, dict[str, dict[str, float | str | None]]],
    mode: str,
) -> list[str]:
    key_sets = [set(series.keys()) for series in series_map.values() if series]
    if not key_sets:
        return []
    if mode == "intersection":
        return sorted(set.intersection(*key_sets))
    return sorted(set.union(*key_sets))


def plot_comparison_cactus(
    title: str,
    out_path: Path,
    series_map: dict[str, dict[str, dict[str, float | str | None]]],
    note: str = "",
    universe: str = "union",
    resolved_label: str = "solved",
) -> None:
    keys = comparison_keys(series_map, universe)
    if not keys:
        return
    filtered_series_map = {
        name: {key: value for key, value in series.items() if key in keys}
        for name, series in series_map.items()
    }
    ordered_keys = comparable_order(filtered_series_map)
    if not ordered_keys:
        return

    total_count = len(keys)
    if total_count == 0:
        return

    fig, ax = plt.subplots(figsize=(9.4, 6.0), constrained_layout=True)
    prep_axes(ax, title)

    for name, series in filtered_series_map.items():
        entries = [series[key] for key in ordered_keys if key in series]
        xs, ys, solved_count, timeout_count, cutoff_x = sequence_points_from_entries(
            entries, total_count
        )
        if not ys:
            continue
        ax.step(
            xs,
            ys,
            where="post",
            linewidth=2.2,
            color=COMPARE_COLORS.get(name, "#374151"),
            label=(
                f"{name} ({resolved_label}={solved_count}/{total_count}, timeout={timeout_count})"
            ),
        )
        draw_timeout_cutoff(ax, cutoff_x)

    if note:
        ax.text(
            0.01,
            0.02,
            note,
            transform=ax.transAxes,
            ha="left",
            va="bottom",
            fontsize=8.5,
            color="#4B5563",
        )
    ax.legend(frameon=False, loc="upper left")
    fig.savefig(out_path, format="pdf", bbox_inches="tight")
    plt.close(fig)


def plot_fo_comparison_cactus(
    title: str,
    out_path: Path,
    series_map: dict[str, dict[str, dict[str, float | str | None]]],
    universe: str = "intersection",
) -> None:
    keys = comparison_keys(series_map, universe)
    if not keys:
        return
    filtered_series_map = {
        name: {key: value for key, value in series.items() if key in keys}
        for name, series in series_map.items()
    }

    total_count = len(keys)
    if total_count == 0:
        return

    fig, ax = plt.subplots(figsize=(9.2, 5.6), constrained_layout=True)
    prep_axes(ax, title)

    resolved_order = ("solved", "infeasible")
    for name, series in filtered_series_map.items():
        entries = list(series.values())
        xs, ys, finished_count, timeout_count, cutoff_x = (
            sequence_points_from_entries_with_order(
                entries, total_count, resolved_order
            )
        )
        if not ys:
            continue
        ax.step(
            xs,
            ys,
            where="post",
            linewidth=2.3,
            color=COMPARE_COLORS.get(name, "#374151"),
            label=f"{name} ({finished_count}/{total_count})",
        )
        draw_timeout_cutoff(ax, cutoff_x)

    ax.legend(frameon=False, loc="upper left")
    fig.savefig(out_path, format="pdf", bbox_inches="tight")
    plt.close(fig)


def plot_fo_outcome_breakdown(
    level_rows: dict[str, list[dict[str, str]]],
    out_path: Path,
    title: str,
) -> None:
    levels = [level for level in ["FO0", "FO1", "FO2", "FO3"] if level_rows.get(level)]
    if not levels:
        return

    solved = []
    infeasible = []
    timeout = []
    totals = []
    for level in levels:
        rows = level_rows[level]
        counts = status_counts(rows)
        totals.append(len(rows))
        solved.append(counts["solved"])
        infeasible.append(counts["infeasible"])
        timeout.append(counts["solve_timeout"] + counts["sygus_timeout"])

    fig, ax = plt.subplots(figsize=(7.0, 4.4), constrained_layout=True)
    x = list(range(len(levels)))
    width = 0.62
    ax.bar(x, solved, width=width, color="#1F4E79", label="solved")
    ax.bar(
        x,
        infeasible,
        width=width,
        bottom=solved,
        color="#9CA3AF",
        label="infeasible",
    )
    ax.bar(
        x,
        timeout,
        width=width,
        bottom=[a + b for a, b in zip(solved, infeasible)],
        color="#B45309",
        label="timeout",
    )
    ax.set_title(title, loc="left", fontweight="bold", pad=10)
    ax.set_ylabel("Benchmarks")
    ax.set_xticks(x, levels)
    ax.grid(axis="y", color="#D1D5DB", linewidth=0.8)
    ax.set_axisbelow(True)
    ax.legend(frameon=False, ncol=3, loc="upper left")
    fig.savefig(out_path, format="pdf", bbox_inches="tight")
    plt.close(fig)


def main() -> None:
    args = parse_args()
    configure_style()

    root = Path(__file__).resolve().parent
    results_dir = (root / args.results_dir).resolve()
    out_dir = (root / args.out_dir).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    rows = load_rows(results_dir)
    if not rows:
        raise SystemExit(f"no TSV rows found under {results_dir}")

    by_group = grouped(rows, "group")
    for idx, (group, group_rows) in enumerate(by_group.items()):
        plot_single_suite_cactus(
            group,
            group_rows,
            out_dir / f"{suite_slug(group)}.pdf",
            f"{args.title}: {group}",
            GROUP_COLORS[idx % len(GROUP_COLORS)],
        )

    fo_levels = {
        "FO0": by_group.get("fo-folseparators-fo0", []),
        "FO1": by_group.get("fo-folseparators-fo1", []),
        "FO2": by_group.get("fo-folseparators-fo2", []),
        "FO3": by_group.get("fo-folseparators-fo3", []),
    }
    if any(fo_levels.values()):
        plot_parallel_levels(
            f"{args.title}: FO Folseparators Parallel View",
            ["FO0", "FO1", "FO2", "FO3"],
            fo_levels,
            out_dir / "fo-folseparators-parallel.pdf",
        )
        plot_fo_outcome_breakdown(
            fo_levels,
            out_dir / "fo-folseparators-outcomes.pdf",
            f"{args.title}: FO Folseparators Outcomes",
        )

    fo_lfp_levels = {
        "FO1-LFP": by_group.get("fo-lfp-folseparators-fo1-lfp", []),
        "FO2-LFP": by_group.get("fo-lfp-folseparators-fo2-lfp", []),
    }
    if any(fo_lfp_levels.values()):
        plot_parallel_levels(
            f"{args.title}: FO-LFP Folseparators Parallel View",
            ["FO1-LFP", "FO2-LFP"],
            fo_lfp_levels,
            out_dir / "fo-lfp-folseparators-parallel.pdf",
        )

    learnctl_series = (
        load_learnctl_series(Path(args.learnctl_csv)) if args.learnctl_csv else {}
    )
    delis_ctl_rows = by_group.get("ctl-learnctl", [])
    if delis_ctl_rows and learnctl_series:
        plot_comparison_cactus(
            f"{args.title}: CTL LearnCTL Comparison",
            out_dir / "ctl-learnctl-comparison.pdf",
            {
                "Delis": delis_series(delis_ctl_rows),
                "LearnCTL": learnctl_series,
            },
        )

    samples2ltl_series = (
        load_samples2ltl_series(Path(args.samples2ltl_csv))
        if args.samples2ltl_csv
        else {}
    )
    delis_samples2ltl_rows = by_group.get("ltl-sample2ltl", [])
    if delis_samples2ltl_rows and samples2ltl_series:
        plot_comparison_cactus(
            f"{args.title}: LTL sample2LTL Comparison",
            out_dir / "ltl-sample2ltl-comparison.pdf",
            {
                "Delis": delis_series(delis_samples2ltl_rows),
                "sample2LTL": samples2ltl_series,
            },
        )

    reg_compare = [
        ("reg-alpharegex", "reg-alpharegex-cvc5.tsv"),
        ("reg-deepregex", "reg-deepregex-cvc5.tsv"),
        ("reg-so", "reg-so-cvc5.tsv"),
    ]
    for group, baseline_tsv in reg_compare:
        group_rows = by_group.get(group, [])
        baseline_series = load_tsv_series(results_dir / baseline_tsv)
        if not group_rows or not baseline_series:
            continue
        plot_comparison_cactus(
            f"{args.title}: {group} vs cvc5",
            out_dir / f"{suite_slug(group)}-cvc5-comparison.pdf",
            {
                "Delis": delis_series(group_rows),
                "cvc5": baseline_series,
            },
        )

    folseparators_series = load_folseparators_series_from_tsv(
        (root / args.folseparators_tsv).resolve()
    )
    for group in [
        "fo-folseparators-fo0",
        "fo-folseparators-fo1",
        "fo-folseparators-fo2",
        "fo-folseparators-fo3",
    ]:
        group_rows = by_group.get(group, [])
        if not group_rows or not folseparators_series:
            continue
        plot_fo_comparison_cactus(
            f"{args.title}: {group} vs folseparators",
            out_dir / f"{suite_slug(group)}-comparison.pdf",
            {
                "Delis": delis_series_by_stem(group_rows),
                "Folseparators": folseparators_series,
            },
            universe="intersection",
        )

    fo2_rows = by_group.get("fo-folseparators-fo2", [])
    if fo2_rows and folseparators_series:
        plot_fo_comparison_cactus(
            f"{args.title}: FO2 vs folseparators",
            out_dir / "fo-folseparators-main-comparison.pdf",
            {
                "Delis": delis_series_by_stem(fo2_rows),
                "Folseparators": folseparators_series,
            },
            universe="intersection",
        )

    plot_status_breakdown(
        rows,
        out_dir / "status-breakdown.pdf",
        f"{args.title}: Outcome Breakdown by Benchmark Suite",
    )
    write_summary(rows, out_dir / "summary.csv")

    print(f"wrote PDFs to {out_dir}")


if __name__ == "__main__":
    main()
