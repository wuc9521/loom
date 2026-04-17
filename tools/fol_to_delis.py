#!/usr/bin/env python3
"""Translate generated folseparators .fol examples to Delis model declarations.

The generated .fol files contain a compact signature and labeled finite models.
This script deliberately does not add any Delis language compatibility for that
format; it emits normal Delis `(declare-sort ...)`, `(declare-fun ...)`, and
`(define-model ...)` commands.
"""

from __future__ import annotations

import argparse
import itertools
import sys
from dataclasses import dataclass, field
from pathlib import Path


SExpr = str | list["SExpr"]


@dataclass
class Signature:
    sorts: list[str] = field(default_factory=list)
    constants: dict[str, str] = field(default_factory=dict)
    funcs: dict[str, tuple[list[str], str]] = field(default_factory=dict)


@dataclass
class FolModel:
    label: str
    elems_by_sort: dict[str, list[str]]
    constants: dict[str, str] = field(default_factory=dict)
    facts: dict[str, set[tuple[str, ...]]] = field(default_factory=dict)
    funcs: dict[str, dict[tuple[str, ...], str]] = field(default_factory=dict)


def tokenize(src: str) -> list[str]:
    tokens: list[str] = []
    current: list[str] = []
    in_comment = False
    for ch in src:
        if in_comment:
            if ch == "\n":
                in_comment = False
            continue
        if ch == ";":
            in_comment = True
            continue
        if ch in "()":
            if current:
                tokens.append("".join(current))
                current.clear()
            tokens.append(ch)
        elif ch.isspace():
            if current:
                tokens.append("".join(current))
                current.clear()
        else:
            current.append(ch)
    if current:
        tokens.append("".join(current))
    return tokens


def parse_many(tokens: list[str]) -> list[SExpr]:
    pos = 0

    def parse_one() -> SExpr:
        nonlocal pos
        if pos >= len(tokens):
            raise ValueError("unexpected end of input")
        tok = tokens[pos]
        pos += 1
        if tok == "(":
            out: list[SExpr] = []
            while pos < len(tokens) and tokens[pos] != ")":
                out.append(parse_one())
            if pos >= len(tokens):
                raise ValueError("unclosed list")
            pos += 1
            return out
        if tok == ")":
            raise ValueError("unexpected )")
        return tok

    exprs: list[SExpr] = []
    while pos < len(tokens):
        exprs.append(parse_one())
    return exprs


def atom(expr: SExpr) -> str:
    if not isinstance(expr, str):
        raise ValueError(f"expected atom, got {expr}")
    return expr


def list_expr(expr: SExpr) -> list[SExpr]:
    if not isinstance(expr, list):
        raise ValueError(f"expected list, got {expr}")
    return expr


def parse_signature_and_models(exprs: list[SExpr]) -> tuple[Signature, list[FolModel]]:
    sig = Signature()
    models: list[FolModel] = []
    relation_arities: dict[str, list[str]] = {}

    for expr in exprs:
        if not isinstance(expr, list) or not expr:
            continue
        head = atom(expr[0])
        if head == "sort":
            sig.sorts.append(atom(expr[1]))
        elif head == "constant":
            sig.constants[atom(expr[1])] = atom(expr[2])
        elif head == "relation":
            name = atom(expr[1])
            args = [atom(item) for item in expr[2:]]
            relation_arities[name] = args
            sig.funcs[name] = (args, "Bool")
        elif head == "function":
            name = atom(expr[1])
            sorts = [atom(item) for item in expr[2:]]
            if len(sorts) < 2:
                raise ValueError(f"function {name} needs at least domain and return sort")
            sig.funcs[name] = (sorts[:-1], sorts[-1])
        elif head == "model":
            models.append(parse_model(expr, sig, relation_arities))

    return sig, models


def parse_model(
    expr: list[SExpr], sig: Signature, relation_arities: dict[str, list[str]]
) -> FolModel:
    label = atom(expr[1])
    elem_decls = list_expr(expr[2])
    elems_by_sort: dict[str, list[str]] = {}
    for decl in elem_decls:
        elem, sort = [atom(item) for item in list_expr(decl)]
        elems_by_sort.setdefault(sort, []).append(elem)

    model = FolModel(label=label, elems_by_sort=elems_by_sort)
    for name in relation_arities:
        model.facts[name] = set()
    for name in sig.funcs:
        if name not in relation_arities:
            model.funcs[name] = {}

    for raw_fact in expr[3:]:
        fact = list_expr(raw_fact)
        head = atom(fact[0])
        if head == "=":
            lhs = fact[1]
            rhs = atom(fact[2])
            if isinstance(lhs, str):
                model.constants[lhs] = rhs
            else:
                app = list_expr(lhs)
                fname = atom(app[0])
                args = tuple(atom(arg) for arg in app[1:])
                model.funcs.setdefault(fname, {})[args] = rhs
        else:
            args = tuple(atom(arg) for arg in fact[1:])
            model.facts.setdefault(head, set()).add(args)

    return model


def emit(sig: Signature, models: list[FolModel], *, require_total: bool) -> str:
    lines: list[str] = ["(set-logic ALL)"]
    for sort in sig.sorts:
        lines.append(f"(declare-sort {sort} 0)")
    for name, sort in sig.constants.items():
        lines.append(f"(declare-fun {name} () {sort})")
    for name, (args, ret) in sig.funcs.items():
        lines.append(f"(declare-fun {name} ({' '.join(args)}) {ret})")

    for idx, model in enumerate(models):
        name = f"M{idx}_{'pos' if model.label == '+' else 'neg'}"
        lines.append("")
        lines.append(f"(define-model {name}")
        for sort in sig.sorts:
            elems = model.elems_by_sort.get(sort, [])
            lines.append(f"  (define-univ {sort} ({' '.join(elems)}))")
        for const, sort in sig.constants.items():
            if const not in model.constants:
                raise ValueError(f"model {name} is missing constant {const}")
            lines.append(f"  (interp-fun {const} ({model.constants[const]}))")
        for fname, (arg_sorts, ret) in sig.funcs.items():
            if ret == "Bool":
                true_rows = sorted(model.facts.get(fname, set()))
                if not arg_sorts:
                    value = "true" if () in model.facts.get(fname, set()) else "false"
                    lines.append(f"  (interp-fun {fname} ({value}))")
                else:
                    lines.append(f"  (interp-fun {fname} (match (")
                    for row in true_rows:
                        lines.append(f"    (({' '.join(row)}) true)")
                    lines.append(f"    (({' '.join('_' for _ in arg_sorts)}) false)")
                    lines.append("  )))")
                continue

            table = model.funcs.get(fname, {})
            if require_total:
                expected = set(itertools.product(*(model.elems_by_sort[s] for s in arg_sorts)))
                missing = sorted(expected.difference(table))
                if missing:
                    preview = ", ".join(f"({', '.join(row)})" for row in missing[:5])
                    raise ValueError(f"model {name} function {fname} is missing rows: {preview}")
            lines.append(f"  (interp-fun {fname} (match (")
            for row, value in sorted(table.items()):
                lines.append(f"    (({' '.join(row)}) {value})")
            lines.append("  )))")
        lines.append(")")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("-o", "--output", type=Path)
    parser.add_argument(
        "--allow-partial-functions",
        action="store_true",
        help="emit partial function tables instead of rejecting missing finite rows",
    )
    args = parser.parse_args()

    exprs = parse_many(tokenize(args.input.read_text()))
    sig, models = parse_signature_and_models(exprs)
    output = emit(sig, models, require_total=not args.allow_partial_functions)
    if args.output:
        args.output.write_text(output)
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
