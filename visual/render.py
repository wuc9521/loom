#!/usr/bin/env python3

from __future__ import annotations

import argparse
import csv
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


def prep_axes(ax: plt.Axes, title: str) -> None:
    ax.set_title(title, loc="left", fontweight="bold", pad=10)
    ax.set_xlabel("Benchmarks resolved (sorted by time)")
    ax.set_ylabel("Time (seconds)")
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
    times = resolved_times(rows)
    fig, ax = plt.subplots(figsize=(8.6, 5.8), constrained_layout=True)
    prep_axes(ax, title)
    if times:
        xs = list(range(1, len(times) + 1))
        ax.plot(xs, times, linewidth=2.2, color=color)
    subtitle = f"resolved={len(times)} / total={len(rows)}"
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


def suite_slug(group: str) -> str:
    return group.replace("/", "-")


def benchmark_leaf(row: dict[str, str]) -> str:
    return Path(row["path"]).name


def best_solved_seconds(candidates: list[dict[str, str]]) -> float | None:
    solved = [seconds(row) for row in candidates if row["status"] == "solved"]
    return min(solved) if solved else None


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

    xs = list(range(1, len(ordered_instances) + 1))
    fig, ax = plt.subplots(figsize=(10.8, 6.6), constrained_layout=True)
    prep_axes(ax, title)
    oracle = [best for _, best in ordered_instances]
    ax.plot(xs, oracle, linewidth=2.6, color=LEVEL_COLORS["oracle"], linestyle="--", label="oracle")

    for level in level_order:
        ys = []
        for leaf, _ in ordered_instances:
            row = instance_map[leaf].get(level)
            ys.append(seconds(row) if row and row["status"] == "solved" else float("nan"))
        ax.plot(xs, ys, linewidth=2.0, color=LEVEL_COLORS[level], label=level)

    ax.legend(frameon=False, ncol=len(level_order) + 1, loc="upper left")
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

    fig_height = max(5.5, 0.36 * len(groups) + 1.8)
    fig, ax = plt.subplots(figsize=(13.5, fig_height), constrained_layout=True)
    ax.set_title(title, loc="left", fontweight="bold", pad=10)
    ax.set_xlabel("Benchmarks")
    ax.grid(axis="x", color="#D1D5DB", linewidth=0.8)
    ax.set_axisbelow(True)

    colors = {
        "solved": "#111827",
        "infeasible": "#6B7280",
        "solve_timeout": "#B45309",
        "sygus_timeout": "#D97706",
        "error": "#991B1B",
    }

    left = [0] * len(groups)
    y = list(range(len(groups)))
    for status in STATUS_ORDER:
        values = [entry.get(status, 0) for entry in counts]
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

    ax.set_yticks(y, groups)
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

    plot_status_breakdown(
        rows,
        out_dir / "status-breakdown.pdf",
        f"{args.title}: Outcome Breakdown by Benchmark Suite",
    )
    write_summary(rows, out_dir / "summary.csv")

    print(f"wrote PDFs to {out_dir}")


if __name__ == "__main__":
    main()
