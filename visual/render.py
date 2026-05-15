#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
import json
import math
import tarfile
from collections import Counter, defaultdict
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
        default="/Users/wuchentian/Codes/C++/learnctl/bench/bench_results_Naturalificas-MacBook.local.csv",
        help="LearnCTL benchmark CSV for CTL comparisons.",
    )
    parser.add_argument(
        "--folseparators-json",
        default="/Users/wuchentian/Downloads/artifact-upload/560.tgz",
        help="Folseparators benchmark source: results JSON or ACM artifact .tgz.",
    )
    parser.add_argument(
        "--samples2ltl-csv",
        default="/Users/wuchentian/Codes/Python/samples2LTL/experiments/dtSubset3Strategy2/statsSubset3Strategy2.csv",
        help="sample2LTL benchmark CSV for LTL/sample2LTL comparisons.",
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


def load_rows(results_dir: Path) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for tsv in sorted(results_dir.glob("*.tsv")):
        with tsv.open(newline="") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            for row in reader:
                row["_tsv"] = tsv.name
                row["_family"] = infer_family(row["path"], row["group"])
                rows.append(row)
    return rows


def seconds(row: dict[str, str]) -> float:
    return float(row["total_ms"]) / 1000.0


def grouped(rows: list[dict[str, str]], field: str) -> dict[str, list[dict[str, str]]]:
    out: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in rows:
        out[row[field]].append(row)
    return dict(sorted(out.items()))


def resolved_times(rows: list[dict[str, str]]) -> list[float]:
    return sorted(seconds(row) for row in rows if row["status"] in RESOLVED_STATUSES)


def status_counts(rows: list[dict[str, str]]) -> dict[str, int]:
    counts = {status: 0 for status in STATUS_ORDER}
    for row in rows:
        counts[row["status"]] = counts.get(row["status"], 0) + 1
    return counts


def suite_slug(group: str) -> str:
    return group.replace("/", "-")


def benchmark_leaf(row: dict[str, str]) -> str:
    return Path(row["path"]).name


def best_solved_seconds(candidates: list[dict[str, str]]) -> float | None:
    solved = [seconds(row) for row in candidates if row["status"] == "solved"]
    return min(solved) if solved else None


def percent_xs(resolved_count: int, total_count: int) -> list[float]:
    if resolved_count <= 0 or total_count <= 0:
        return []
    return [100.0 * idx / total_count for idx in range(1, resolved_count + 1)]


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
    times = resolved_times(rows)
    fig, ax = plt.subplots(figsize=(8.6, 5.8), constrained_layout=True)
    prep_axes(ax, title)
    if times:
        ax.plot(
            percent_xs(len(times), total_count),
            times,
            linewidth=2.2,
            color=color,
            label=f"Delis (n={total_count}, resolved={len(times)})",
        )
        ax.legend(frameon=False, loc="upper left")
    subtitle = f"resolved={len(times)} / total={total_count}"
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
        for leaf, _ in ordered_instances:
            row = instance_map[leaf].get(level)
            ok = row and row["status"] == "solved"
            ys.append(seconds(row) if ok else float("nan"))
            solved += int(bool(ok))
        ax.scatter(
            xs,
            ys,
            s=10,
            color=LEVEL_COLORS[level],
            label=f"{level} (resolved={solved}/{total_count})",
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


def load_folseparators_series(path: Path) -> dict[str, dict[str, float | str | None]]:
    rows = load_folseparators_rows(path)
    if not rows:
        return {}
    series: dict[str, dict[str, float | str | None]] = {}
    for row in rows:
        filename = f"{row['base']}-{row['conjecture']}.sl"
        if row.get("success") and isinstance(row.get("stats"), dict):
            secs = float(row["stats"].get("total_time", 0.0))
            candidate = {"status": "solved", "seconds": secs}
        elif row.get("killed"):
            candidate = {
                "status": "solve_timeout",
                "seconds": float(row.get("timeout", 0)),
            }
        else:
            candidate = {"status": "error", "seconds": None}
        current = series.get(filename)
        if current is None:
            series[filename] = candidate
        elif candidate["status"] == "solved":
            current_secs = current["seconds"]
            cand_secs = candidate["seconds"]
            if (
                current["status"] != "solved"
                or current_secs is None
                or (cand_secs is not None and cand_secs < current_secs)
            ):
                series[filename] = candidate
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


def plot_comparison_cactus(
    title: str,
    out_path: Path,
    series_map: dict[str, dict[str, dict[str, float | str | None]]],
    note: str = "",
) -> None:
    ordered_keys = comparable_order(series_map)
    if not ordered_keys:
        return

    total_count = len(
        set().union(*[set(series.keys()) for series in series_map.values()])
    )
    if total_count == 0:
        return

    fig, ax = plt.subplots(figsize=(9.4, 6.0), constrained_layout=True)
    prep_axes(ax, title)

    for name, series in series_map.items():
        solved_times = sorted(
            float(entry["seconds"])
            for entry in series.values()
            if entry["status"] == "solved" and entry["seconds"] is not None
        )
        if not solved_times:
            continue
        ax.plot(
            percent_xs(len(solved_times), total_count),
            solved_times,
            linewidth=2.2,
            color=COMPARE_COLORS.get(name, "#374151"),
            label=f"{name} (resolved={len(solved_times)}/{total_count})",
        )

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

    learnctl_series = load_learnctl_series(Path(args.learnctl_csv))
    delis_ctl_rows = by_group.get("ctl-learnctl-generated", [])
    if delis_ctl_rows and learnctl_series:
        plot_comparison_cactus(
            f"{args.title}: CTL LearnCTL Comparison",
            out_dir / "ctl-learnctl-generated-comparison.pdf",
            {
                "Delis": delis_series(delis_ctl_rows),
                "LearnCTL": learnctl_series,
            },
            note="x-axis is percentage of the currently known union of benchmark cases; missing/incomplete runs are skipped until present.",
        )

    samples2ltl_series = load_samples2ltl_series(Path(args.samples2ltl_csv))
    delis_samples2ltl_rows = by_group.get("ltl-sample2ltl", [])
    if delis_samples2ltl_rows and samples2ltl_series:
        plot_comparison_cactus(
            f"{args.title}: LTL sample2LTL Comparison",
            out_dir / "ltl-sample2ltl-comparison.pdf",
            {
                "Delis": delis_series(delis_samples2ltl_rows),
                "sample2LTL": samples2ltl_series,
            },
            note="comparison uses the matched 5to10Traces decision-tree benchmark CSV; x-axis is percentage over the currently known union.",
        )

    folseparators_series = load_folseparators_series(Path(args.folseparators_json))
    for group in [
        "fo-folseparators-fo0",
        "fo-folseparators-fo1",
        "fo-folseparators-fo2",
        "fo-folseparators-fo3",
    ]:
        group_rows = by_group.get(group, [])
        if not group_rows or not folseparators_series:
            continue
        plot_comparison_cactus(
            f"{args.title}: {group} vs folseparators",
            out_dir / f"{suite_slug(group)}-comparison.pdf",
            {
                "Delis": delis_series(group_rows),
                "Folseparators": folseparators_series,
            },
            note="x-axis is percentage of the currently known union of benchmark cases; folseparators filenames are matched by normalized benchmark stem.",
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
