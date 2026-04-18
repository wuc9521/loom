#!/usr/bin/env python3
"""Convert generated .fol separator instances into Delis FOK benchmarks."""

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
from collections import OrderedDict, defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


Atom = str
SExpr = Atom | list["SExpr"]


@dataclass(frozen=True)
class FunctionSig:
    name: str
    args: tuple[str, ...]
    ret: str


@dataclass(frozen=True)
class RelationSig:
    name: str
    args: tuple[str, ...]


@dataclass(frozen=True)
class Signature:
    sorts: tuple[str, ...]
    constants: tuple[FunctionSig, ...]
    functions: tuple[FunctionSig, ...]
    relations: tuple[RelationSig, ...]

    def key(self) -> tuple:
        return (self.sorts, self.constants, self.functions, self.relations)

    def digest(self) -> str:
        text = repr(self.key()).encode()
        return hashlib.sha1(text).hexdigest()[:8]


@dataclass
class FunctionInterp:
    values: OrderedDict[tuple[str, ...], str] = field(default_factory=OrderedDict)


@dataclass
class Model:
    label: str
    domains: OrderedDict[str, list[str]]
    constants: dict[str, str]
    functions: dict[str, FunctionInterp]
    relations: dict[str, set[tuple[str, ...]]]


@dataclass
class Instance:
    path: Path
    signature: Signature
    models: list[Model]


TOKEN_RE = re.compile(r"""\s*(;[^\n]*|[()]|"(?:\\.|[^"\\])*"|[^\s()]+)""")
SYMBOL_RE = re.compile(r"[^A-Za-z0-9+\-/*=%?!.$_~&^<>@]")


def tokenize(text: str) -> list[str]:
    tokens = []
    for match in TOKEN_RE.finditer(text):
        token = match.group(1)
        if token.startswith(";"):
            continue
        tokens.append(token)
    return tokens


def parse_sexprs(text: str) -> list[SExpr]:
    tokens = tokenize(text)
    pos = 0

    def parse_one() -> SExpr:
        nonlocal pos
        if pos >= len(tokens):
            raise ValueError("unexpected end of input")
        token = tokens[pos]
        pos += 1
        if token == "(":
            items = []
            while pos < len(tokens) and tokens[pos] != ")":
                items.append(parse_one())
            if pos >= len(tokens):
                raise ValueError("unclosed list")
            pos += 1
            return items
        if token == ")":
            raise ValueError("unmatched ')'")
        return token

    forms = []
    while pos < len(tokens):
        forms.append(parse_one())
    return forms


def expect_atom(value: SExpr) -> str:
    if not isinstance(value, str):
        raise ValueError(f"expected atom, got {value!r}")
    return value


def parse_instance(path: Path) -> Instance:
    forms = parse_sexprs(path.read_text())
    sorts: list[str] = []
    constants: list[FunctionSig] = []
    functions: list[FunctionSig] = []
    relations: list[RelationSig] = []
    models: list[Model] = []

    for form in forms:
        if not isinstance(form, list) or not form:
            continue
        head = expect_atom(form[0])
        if head == "sort":
            sorts.append(expect_atom(form[1]))
        elif head == "constant":
            constants.append(FunctionSig(expect_atom(form[1]), (), expect_atom(form[2])))
        elif head == "function":
            name = expect_atom(form[1])
            parts = tuple(expect_atom(x) for x in form[2:])
            if len(parts) < 1:
                raise ValueError(f"{path}: function {name} has no return sort")
            functions.append(FunctionSig(name, parts[:-1], parts[-1]))
        elif head == "relation":
            relations.append(RelationSig(expect_atom(form[1]), tuple(expect_atom(x) for x in form[2:])))
        elif head == "model":
            models.append(parse_model(path, form))

    return Instance(
        path=path,
        signature=Signature(tuple(sorts), tuple(constants), tuple(functions), tuple(relations)),
        models=models,
    )


def parse_model(path: Path, form: list[SExpr]) -> Model:
    label = expect_atom(form[1])
    raw_domain = form[2]
    if not isinstance(raw_domain, list):
        raise ValueError(f"{path}: model domain is not a list")
    domains: OrderedDict[str, list[str]] = OrderedDict()
    constants: dict[str, str] = {}
    functions: dict[str, FunctionInterp] = defaultdict(FunctionInterp)
    relations: dict[str, set[tuple[str, ...]]] = defaultdict(set)

    for entry in raw_domain:
        if not isinstance(entry, list) or len(entry) != 2:
            raise ValueError(f"{path}: bad domain entry {entry!r}")
        elem = expect_atom(entry[0])
        sort = expect_atom(entry[1])
        domains.setdefault(sort, []).append(elem)

    for item in form[3:]:
        if not isinstance(item, list) or not item:
            raise ValueError(f"{path}: bad model fact {item!r}")
        head = expect_atom(item[0])
        if head == "=":
            lhs = item[1]
            rhs = expect_atom(item[2])
            if isinstance(lhs, list):
                fname = expect_atom(lhs[0])
                args = tuple(expect_atom(x) for x in lhs[1:])
                functions[fname].values[args] = rhs
            else:
                constants[expect_atom(lhs)] = rhs
        else:
            relations[head].add(tuple(expect_atom(x) for x in item[1:]))

    return Model(label, domains, constants, dict(functions), dict(relations))


def symbol(value: str) -> str:
    value = SYMBOL_RE.sub("_", value)
    if not value or value[0].isdigit():
        value = f"s_{value}"
    return value


def term_sort(sort_name: str) -> str:
    return f"Term.{symbol(sort_name)}"


def var_ctor(sort_name: str, index: int) -> str:
    return f"fol.var.x{index}.{symbol(sort_name)}"


def var_field(sort_name: str, index: int) -> str:
    return f"x{index}.{symbol(sort_name)}"


def const_ctor(name: str) -> str:
    return f"fol.term.{symbol(name)}"


def func_ctor(name: str) -> str:
    return f"fol.term.{symbol(name)}"


def rel_ctor(name: str) -> str:
    return f"fol.atom.{symbol(name)}"


def eq_ctor(sort_name: str) -> str:
    return f"fol.eq.{symbol(sort_name)}"


def quant_ctor(kind: str, sort_name: str, index: int) -> str:
    return f"fol.{kind}.x{index}.{symbol(sort_name)}"


def term_sem(sort_name: str) -> str:
    return f"{term_sort(sort_name)}.Sem"


def model_name(index: int, label: str) -> str:
    prefix = "P" if label == "+" else "N"
    return f"{prefix}{index}"


def sexp(items: Iterable[str]) -> str:
    return "(" + " ".join(items) + ")"


def ctor_name(decl: str) -> str:
    return decl.strip().removeprefix("(").split()[0].removesuffix(")")


def build_theory(name: str, sig: Signature, num_vars: int) -> tuple[str, list[str]]:
    lines: list[str] = ["(set-logic ALL)"]
    provided: list[str] = []

    for sort_name in sig.sorts:
        lines.append(f"(declare-sort {sort_name} 0)")
        provided.append(sort_name)
    for fn in [*sig.constants, *sig.functions]:
        lines.append(f"(declare-fun {fn.name} ({' '.join(fn.args)}) {fn.ret})")
        provided.append(fn.name)
    for rel in sig.relations:
        lines.append(f"(declare-fun {rel.name} ({' '.join(rel.args)}) Bool)")
        provided.append(rel.name)

    sort_decls = ["(Formula 0)", *[f"({term_sort(s)} 0)" for s in sig.sorts]]
    formula_ctors = [
        "(fol.true)",
        "(fol.false)",
        "(fol.not Formula)",
        "(fol.and Formula Formula)",
        "(fol.or Formula Formula)",
    ]
    for sort_name in sig.sorts:
        for index in range(num_vars):
            formula_ctors.append(f"({quant_ctor('forall', sort_name, index)} Formula)")
            formula_ctors.append(f"({quant_ctor('exists', sort_name, index)} Formula)")
        formula_ctors.append(f"({eq_ctor(sort_name)} {term_sort(sort_name)} {term_sort(sort_name)})")
    for rel in sig.relations:
        formula_ctors.append(f"({rel_ctor(rel.name)} {' '.join(term_sort(s) for s in rel.args)})")

    term_groups: list[list[str]] = []
    for sort_name in sig.sorts:
        ctors = [f"({var_ctor(sort_name, i)})" for i in range(num_vars)]
        for const in sig.constants:
            if const.ret == sort_name:
                ctors.append(f"({const_ctor(const.name)})")
        for fn in sig.functions:
            if fn.ret == sort_name:
                ctors.append(f"({func_ctor(fn.name)} {' '.join(term_sort(s) for s in fn.args)})")
        term_groups.append(ctors)

    lines.append("")
    lines.append("(declare-term-types")
    lines.append(f" ({' '.join(sort_decls)})")
    all_groups = [formula_ctors, *term_groups]
    lines.append(" (" + "\n  ".join("(" + "\n   ".join(group) + ")" for group in all_groups) + "))")

    fields = ["(model Model)"]
    for sort_name in sig.sorts:
        for index in range(num_vars):
            fields.append(f"({var_field(sort_name, index)} {sort_name})")
    lines.append("")
    lines.append(f"(define-finite FoAsp (record {' '.join(fields)}))")
    provided.extend(["Formula", "FoAsp"])

    for sort_name in sig.sorts:
        lines.append("")
        lines.append("(define-semantics")
        lines.append(f"  (({term_sem(sort_name)} (term {term_sort(sort_name)}) (asp FoAsp)))")
        lines.append(f"  :returns {sort_name}")
        lines.append("  ((match term")
        for index in range(num_vars):
            lines.append(f"     (({var_ctor(sort_name, index)}) ({var_field(sort_name, index)} asp))")
        for const in sig.constants:
            if const.ret == sort_name:
                lines.append(f"     (({const_ctor(const.name)}) {const.name})")
        for fn in sig.functions:
            if fn.ret == sort_name:
                args = [f"a{i}" for i in range(len(fn.args))]
                app = " ".join([fn.name, *[f"({term_sem(arg_sort)} a{i} asp)" for i, arg_sort in enumerate(fn.args)]])
                lines.append(f"     (({func_ctor(fn.name)} {' '.join(args)}) ({app}))")
        lines.append("  )))")
        provided.extend([term_sort(sort_name), term_sem(sort_name)])

    lines.append("")
    lines.append("(define-semantics")
    lines.append("  ((FO.Sem (phi Formula) (asp FoAsp)))")
    lines.append("  ((match phi")
    lines.append("     ((fol.true) true)")
    lines.append("     ((fol.false) false)")
    lines.append("     ((fol.not p) (not (FO.Sem p asp)))")
    lines.append("     ((fol.and p q) (and (FO.Sem p asp) (FO.Sem q asp)))")
    lines.append("     ((fol.or p q) (or (FO.Sem p asp) (FO.Sem q asp)))")
    for sort_name in sig.sorts:
        for index in range(num_vars):
            replacement = aspect_args(sig.sorts, num_vars, sort_name, index, "v")
            lines.append(f"     (({quant_ctor('forall', sort_name, index)} p)")
            lines.append(f"      (forall ((v (domain ({sort_name} (model asp)))))")
            lines.append(f"        (FO.Sem p (FoAsp (model asp) {replacement}))))")
            lines.append(f"     (({quant_ctor('exists', sort_name, index)} p)")
            lines.append(f"      (exists ((v (domain ({sort_name} (model asp)))))")
            lines.append(f"        (FO.Sem p (FoAsp (model asp) {replacement}))))")
        lines.append(f"     (({eq_ctor(sort_name)} lhs rhs)")
        lines.append(f"      (= ({term_sem(sort_name)} lhs asp) ({term_sem(sort_name)} rhs asp)))")
    for rel in sig.relations:
        args = [f"a{i}" for i in range(len(rel.args))]
        call_args = [f"({term_sem(arg_sort)} a{i} asp)" for i, arg_sort in enumerate(rel.args)]
        lines.append(f"     (({rel_ctor(rel.name)} {' '.join(args)}) ({' '.join([rel.name, *call_args])}))")
    lines.append("  )))")
    provided.append("FO.Sem")

    for ctor in formula_ctors:
        provided.append(ctor_name(ctor))
    for group in term_groups:
        for ctor in group:
            provided.append(ctor_name(ctor))

    provide = f"(provide {' '.join(OrderedDict.fromkeys(provided))})"
    return provide + "\n" + "\n".join(lines) + "\n", list(OrderedDict.fromkeys(provided))


def aspect_args(sorts: tuple[str, ...], num_vars: int, target_sort: str, target_index: int, value: str) -> str:
    args = []
    for sort_name in sorts:
        for index in range(num_vars):
            if sort_name == target_sort and index == target_index:
                args.append(value)
            else:
                args.append(f"({var_field(sort_name, index)} asp)")
    return " ".join(args)


def default_elem(model: Model, sort_name: str, path: Path) -> str:
    elems = model.domains.get(sort_name, [])
    if not elems:
        raise ValueError(f"{path}: model {model.label} has no element for sort {sort_name}")
    return elems[0]


def build_model(path: Path, sig: Signature, model: Model, name: str) -> str:
    lines = [f"(define-model {name}"]
    for sort_name in sig.sorts:
        elems = model.domains.get(sort_name, [])
        if not elems:
            raise ValueError(f"{path}: model {name} has no universe for sort {sort_name}")
        lines.append(f"  (define-univ {sort_name} ({' '.join(elems)}))")
    for const in sig.constants:
        value = model.constants.get(const.name, default_elem(model, const.ret, path))
        lines.append(f"  (interp-fun {const.name} ({value}))")
    for fn in sig.functions:
        interp = model.functions.get(fn.name, FunctionInterp())
        lines.append(f"  (interp-fun {fn.name} (match (")
        for args, value in interp.values.items():
            lines.append(f"    ({sexp(args)} {value})")
        wild = tuple("_" for _ in fn.args)
        lines.append(f"    ({sexp(wild)} {default_elem(model, fn.ret, path)})")
        lines.append("  )))")
    for rel in sig.relations:
        true_tuples = sorted(model.relations.get(rel.name, set()))
        if rel.args:
            lines.append(f"  (interp-fun {rel.name} (match (")
            for args in true_tuples:
                lines.append(f"    ({sexp(args)} true)")
            wild = tuple("_" for _ in rel.args)
            lines.append(f"    ({sexp(wild)} false)")
            lines.append("  )))")
        else:
            lines.append(f"  (interp-fun {rel.name} ({'true' if () in true_tuples else 'false'}))")
    lines.append(")")
    return "\n".join(lines)


def build_instance(inst: Instance, theory_rel: str, num_vars: int) -> str:
    lines = [f"(require \"{theory_rel}\")", ""]
    names = []
    for index, model in enumerate(inst.models):
        name = model_name(index, model.label)
        names.append((name, model.label))
        lines.append(build_model(inst.path, inst.signature, model, name))
        lines.append("")
    lines.append("(synth-fun phi () Formula)")
    lines.append("")
    for model, (name, label) in zip(inst.models, names):
        seed_args = " ".join(default_elem(model, sort_name, inst.path) for sort_name in inst.signature.sorts for _ in range(num_vars))
        expr = f"(FO.Sem phi (FoAsp {name}{(' ' + seed_args) if seed_args else ''}))"
        if label == "+":
            lines.append(f"(constraint {expr})")
        elif label == "-":
            lines.append(f"(constraint (not {expr}))")
        else:
            raise ValueError(f"{inst.path}: unknown model label {label}")
    lines.append("")
    lines.append("(check-synth)")
    return "\n".join(lines) + "\n"


def cluster_name(instances: list[Instance], sig: Signature) -> str:
    prefixes = [inst.path.stem.split("-", 1)[0] for inst in instances]
    base = prefixes[0] if all(p == prefixes[0] for p in prefixes) else "shared"
    return f"{symbol(base)}-{sig.digest()}"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("--out", type=Path, default=Path("benchmark/fok/generated-fol"))
    parser.add_argument("--vars", type=int, default=2)
    parser.add_argument("--clean", action="store_true")
    args = parser.parse_args()

    instances = [parse_instance(path) for path in sorted(args.source.glob("*.fol"))]
    clusters: dict[tuple, list[Instance]] = defaultdict(list)
    sigs: dict[tuple, Signature] = {}
    for inst in instances:
        key = inst.signature.key()
        clusters[key].append(inst)
        sigs[key] = inst.signature

    if args.clean and args.out.exists():
        shutil.rmtree(args.out)
    theories = args.out / "theories"
    theories.mkdir(parents=True, exist_ok=True)

    cluster_files: dict[tuple, str] = {}
    for key, grouped in sorted(clusters.items(), key=lambda item: cluster_name(item[1], sigs[item[0]])):
        sig = sigs[key]
        name = cluster_name(grouped, sig)
        cluster_files[key] = f"theories/{name}.sl"
        text, _ = build_theory(name, sig, args.vars)
        (theories / f"{name}.sl").write_text(text)

    for inst in instances:
        rel = cluster_files[inst.signature.key()]
        (args.out / f"{inst.path.stem}.sl").write_text(build_instance(inst, rel, args.vars))

    print(f"converted {len(instances)} instances into {args.out}")
    print(f"wrote {len(clusters)} shared theory file(s)")


if __name__ == "__main__":
    main()
