#!/usr/bin/env python3

from __future__ import annotations

import argparse
import collections
import dataclasses
import itertools
import re
import sys
import textwrap
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any


GLOBAL_DECL_RE = re.compile(r"^\s*byte\s+(.+?);\s*$", re.MULTILINE)
PROCESS_RE = re.compile(r"process\s+([A-Za-z0-9_]+)\s*\{(.*?)\n\}", re.DOTALL)
STATE_RE = re.compile(r"state\s+(.+?);", re.DOTALL)
INIT_RE = re.compile(r"init\s+([A-Za-z0-9_]+)\s*;")
TRANS_RE = re.compile(r"trans\s+(.*?)\s*;\s*$", re.DOTALL)
TRANS_ITEM_RE = re.compile(
    r"([A-Za-z0-9_]+)\s*->\s*([A-Za-z0-9_]+)\s*\{(.*?)\}", re.DOTALL
)
VAR_TOKEN_RE = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\b")
PROCESS_INDEX_RE = re.compile(r"^(.*?)[_]?(\d+)$")

RESERVED_WORDS = {"and", "or", "not", "True", "False"}


class UnsupportedDve(RuntimeError):
    pass


def split_top_level(text: str, delimiter: str) -> list[str]:
    parts: list[str] = []
    current: list[str] = []
    depth = 0
    brace_depth = 0
    bracket_depth = 0
    for char in text:
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        elif char == "{":
            brace_depth += 1
        elif char == "}":
            brace_depth -= 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]":
            bracket_depth -= 1
        if (
            char == delimiter
            and depth == 0
            and brace_depth == 0
            and bracket_depth == 0
        ):
            piece = "".join(current).strip()
            if piece:
                parts.append(piece)
            current = []
            continue
        current.append(char)
    piece = "".join(current).strip()
    if piece:
        parts.append(piece)
    return parts


def normalize_space(text: str) -> str:
    return " ".join(text.replace("\n", " ").split())


@dataclasses.dataclass(frozen=True)
class VarRef:
    name: str
    index: str | None = None


@dataclasses.dataclass
class VarDecl:
    name: str
    size: int | None
    init: Any


@dataclasses.dataclass
class Transition:
    src: str
    dst: str
    guard: str | None
    effects: list[tuple[VarRef, str]]


@dataclasses.dataclass
class Process:
    name: str
    states: list[str]
    init_state: str
    locals: list[VarDecl]
    transitions: list[Transition]


@dataclasses.dataclass
class DveModel:
    name: str
    globals: list[VarDecl]
    processes: list[Process]


def parse_var_target(text: str) -> VarRef:
    text = text.strip()
    if "[" in text:
        name, rest = text.split("[", 1)
        index = rest.rstrip("]")
        return VarRef(name=name.strip(), index=index.strip())
    return VarRef(name=text)


def parse_decl_item(text: str) -> VarDecl:
    text = text.strip()
    name_and_dim, *init_parts = text.split("=", 1)
    init_text = init_parts[0].strip() if init_parts else None
    if "[" in name_and_dim:
        name, rest = name_and_dim.split("[", 1)
        size = int(rest.strip().rstrip("]"))
    else:
        name = name_and_dim.strip()
        size = None
    if init_text is None:
        init = [0] * size if size is not None else 0
    elif init_text.startswith("{"):
        values = [int(piece.strip()) for piece in init_text.strip("{} ").split(",") if piece.strip()]
        init = values
    else:
        init = int(init_text)
    return VarDecl(name=name.strip(), size=size, init=init)


def parse_decl_block(text: str) -> list[VarDecl]:
    decls: list[VarDecl] = []
    for match in GLOBAL_DECL_RE.finditer(text):
        for item in split_top_level(match.group(1), ","):
            decls.append(parse_decl_item(item))
    return decls


def parse_transition_block(body: str) -> list[Transition]:
    match = TRANS_RE.search(body)
    if not match:
        return []
    block = match.group(1)
    items = TRANS_ITEM_RE.findall(block)
    transitions: list[Transition] = []
    for src, dst, inner in items:
        guard = None
        effects: list[tuple[VarRef, str]] = []
        for stmt in split_top_level(inner, ";"):
            stmt = normalize_space(stmt)
            if not stmt:
                continue
            if stmt.startswith("sync "):
                raise UnsupportedDve("synchronous/channel transitions are not supported yet")
            if stmt.startswith("guard "):
                guard = stmt[len("guard ") :].strip()
            elif stmt.startswith("effect "):
                assignments = split_top_level(stmt[len("effect ") :].strip(), ",")
                for assignment in assignments:
                    lhs, rhs = assignment.split("=", 1)
                    effects.append((parse_var_target(lhs), rhs.strip()))
            else:
                raise UnsupportedDve(f"unsupported transition statement: {stmt}")
        transitions.append(Transition(src=src, dst=dst, guard=guard, effects=effects))
    return transitions


def parse_dve(path: Path) -> DveModel:
    text = path.read_text()
    if "system async;" not in text:
        raise UnsupportedDve("only async systems are supported")
    if "channel " in text:
        raise UnsupportedDve("channel-based systems are not supported")
    globals_text = text.split("process ", 1)[0]
    globals_ = parse_decl_block(globals_text)
    processes: list[Process] = []
    for name, body in PROCESS_RE.findall(text):
        local_decls = parse_decl_block(body)
        state_match = STATE_RE.search(body)
        init_match = INIT_RE.search(body)
        if not state_match or not init_match:
            raise UnsupportedDve(f"missing state/init in process {name}")
        states = [piece.strip() for piece in state_match.group(1).split(",")]
        transitions = parse_transition_block(body)
        processes.append(
            Process(
                name=name,
                states=states,
                init_state=init_match.group(1),
                locals=local_decls,
                transitions=transitions,
            )
        )
    return DveModel(name=path.stem, globals=globals_, processes=processes)


@dataclasses.dataclass(frozen=True)
class ProcSnapshot:
    state: str
    locals: tuple[tuple[str, Any], ...]


@dataclasses.dataclass(frozen=True)
class Snapshot:
    globals: tuple[tuple[str, Any], ...]
    processes: tuple[ProcSnapshot, ...]


def freeze_value(value: Any) -> Any:
    if isinstance(value, list):
        return tuple(value)
    return value


def thaw_value(value: Any) -> Any:
    if isinstance(value, tuple):
        return list(value)
    return value


def snapshot_from_runtime(
    globals_state: dict[str, Any], proc_states: list[str], proc_locals: list[dict[str, Any]]
) -> Snapshot:
    return Snapshot(
        globals=tuple(sorted((name, freeze_value(value)) for name, value in globals_state.items())),
        processes=tuple(
            ProcSnapshot(
                state=proc_states[idx],
                locals=tuple(sorted((name, freeze_value(value)) for name, value in proc_locals[idx].items())),
            )
            for idx in range(len(proc_states))
        ),
    )


def runtime_from_snapshot(snapshot: Snapshot) -> tuple[dict[str, Any], list[str], list[dict[str, Any]]]:
    globals_state = {name: thaw_value(value) for name, value in snapshot.globals}
    proc_states = [proc.state for proc in snapshot.processes]
    proc_locals = [{name: thaw_value(value) for name, value in proc.locals} for proc in snapshot.processes]
    return globals_state, proc_states, proc_locals


def make_eval_env(globals_state: dict[str, Any], locals_state: dict[str, Any]) -> dict[str, Any]:
    env: dict[str, Any] = {}
    env.update(globals_state)
    env.update(locals_state)
    return env


def translate_expr(expr: str, env_names: set[str]) -> str:
    expr = normalize_space(expr)
    expr = expr.replace("&&", " and ").replace("||", " or ")
    expr = re.sub(r"!\s*(?!=)", " not ", expr)

    def replace_token(match: re.Match[str]) -> str:
        token = match.group(1)
        if token in RESERVED_WORDS:
            return token
        if token not in env_names:
            return token
        return f"env[{token!r}]"

    return VAR_TOKEN_RE.sub(replace_token, expr)


def eval_expr(expr: str, globals_state: dict[str, Any], locals_state: dict[str, Any]) -> Any:
    env = make_eval_env(globals_state, locals_state)
    translated = translate_expr(expr, set(env))
    return eval(translated, {"__builtins__": {}}, {"env": env})


def assign_target(target: VarRef, value: Any, globals_state: dict[str, Any], locals_state: dict[str, Any]) -> None:
    if target.name in locals_state:
        owner = locals_state
    elif target.name in globals_state:
        owner = globals_state
    else:
        raise UnsupportedDve(f"assignment target {target.name} not found")
    if target.index is None:
        owner[target.name] = value
        return
    current = list(owner[target.name])
    idx_value = int(eval_expr(target.index, globals_state, locals_state))
    current[idx_value] = value
    owner[target.name] = current


def initial_runtime(model: DveModel) -> tuple[dict[str, Any], list[str], list[dict[str, Any]]]:
    globals_state = {
        decl.name: list(decl.init) if isinstance(decl.init, list) else decl.init for decl in model.globals
    }
    proc_states = [proc.init_state for proc in model.processes]
    proc_locals = []
    for proc in model.processes:
        proc_locals.append(
            {
                decl.name: list(decl.init) if isinstance(decl.init, list) else decl.init
                for decl in proc.locals
            }
        )
    return globals_state, proc_states, proc_locals


@dataclasses.dataclass
class Graph:
    states: list[Snapshot]
    edges: dict[int, set[int]]
    initial: int


def build_graph(model: DveModel, max_states: int) -> Graph:
    globals_state, proc_states, proc_locals = initial_runtime(model)
    initial = snapshot_from_runtime(globals_state, proc_states, proc_locals)
    queue = collections.deque([initial])
    states = [initial]
    index = {initial: 0}
    edges: dict[int, set[int]] = collections.defaultdict(set)

    while queue:
        current = queue.popleft()
        src = index[current]
        globals_state, proc_states, proc_locals = runtime_from_snapshot(current)
        for proc_idx, proc in enumerate(model.processes):
            if proc_idx >= len(proc_states):
                continue
            for transition in proc.transitions:
                if transition.src != proc_states[proc_idx]:
                    continue
                local_state = dict(proc_locals[proc_idx])
                if transition.guard:
                    try:
                        enabled = bool(eval_expr(transition.guard, globals_state, local_state))
                    except Exception as exc:
                        raise UnsupportedDve(
                            f"failed to evaluate guard {transition.guard!r} in {model.name}: {exc}"
                        ) from exc
                    if not enabled:
                        continue
                next_globals = {name: thaw_value(value) for name, value in globals_state.items()}
                next_proc_states = list(proc_states)
                next_proc_locals = [{name: thaw_value(value) for name, value in mapping.items()} for mapping in proc_locals]
                next_locals = dict(next_proc_locals[proc_idx])
                for target, expr in transition.effects:
                    try:
                        value = eval_expr(expr, next_globals, next_locals)
                    except Exception as exc:
                        raise UnsupportedDve(
                            f"failed to evaluate effect {expr!r} in {model.name}: {exc}"
                        ) from exc
                    assign_target(target, value, next_globals, next_locals)
                next_proc_states[proc_idx] = transition.dst
                next_proc_locals[proc_idx] = next_locals
                snapshot = snapshot_from_runtime(next_globals, next_proc_states, next_proc_locals)
                dst = index.get(snapshot)
                if dst is None:
                    if len(states) >= max_states:
                        continue
                    dst = len(states)
                    index[snapshot] = dst
                    states.append(snapshot)
                    queue.append(snapshot)
                edges[src].add(dst)
    return Graph(states=states, edges=edges, initial=0)


def process_group(name: str) -> tuple[str, int | None]:
    match = PROCESS_INDEX_RE.match(name)
    if not match:
        return name.lower(), None
    return match.group(1).lower(), int(match.group(2))


def derived_atoms(model: DveModel, graph: Graph) -> list[str]:
    atoms: set[str] = set()
    groups: dict[str, list[tuple[int | None, int]]] = collections.defaultdict(list)
    group_states: dict[str, set[str]] = collections.defaultdict(set)
    for proc_idx, proc in enumerate(model.processes):
        base, idx = process_group(proc.name)
        groups[base].append((idx, proc_idx))
        group_states[base].update(proc.states)
    for base, members in groups.items():
        first = sorted(members, key=lambda item: (item[0] is None, item[0] if item[0] is not None else 10**9))[0]
        first_idx = first[0] if first[0] is not None else 0
        proc_idx = first[1]
        for state in model.processes[proc_idx].states:
            atoms.add(f"{base}{first_idx}_{state.lower()}")
        for state in sorted(group_states[base]):
            atoms.add(f"some_{base}_{state.lower()}")

    observed_globals: dict[str, set[int]] = collections.defaultdict(set)
    for snapshot in graph.states:
        globals_state, _, _ = runtime_from_snapshot(snapshot)
        for name, value in globals_state.items():
            if isinstance(value, int):
                observed_globals[name].add(value)
    for name, values in observed_globals.items():
        if len(values) <= 4 and max(values) <= 3:
            for value in sorted(values):
                atoms.add(f"{name.lower()}_is_{value}")
    observed_arrays: dict[str, dict[int, set[int]]] = collections.defaultdict(lambda: collections.defaultdict(set))
    observed_locals: dict[tuple[str, int | None, str], set[int]] = collections.defaultdict(set)
    for snapshot in graph.states:
        globals_state, _, proc_locals = runtime_from_snapshot(snapshot)
        for name, value in globals_state.items():
            if isinstance(value, list) and len(value) <= 8 and all(isinstance(item, int) for item in value):
                for idx, item in enumerate(value):
                    observed_arrays[name][idx].add(item)
        for proc_idx, proc in enumerate(model.processes):
            base, idx = process_group(proc.name)
            for local_name, local_value in proc_locals[proc_idx].items():
                if isinstance(local_value, int):
                    observed_locals[(base, idx, local_name)].add(local_value)
    for name, cells in observed_arrays.items():
        for idx, values in cells.items():
            if len(values) <= 4 and max(values) <= 3:
                for value in sorted(values):
                    atoms.add(f"{name.lower()}_{idx}_is_{value}")
    for (base, idx, local_name), values in observed_locals.items():
        if len(values) <= 4 and max(values) <= 3:
            prefix = f"{base}{idx if idx is not None else 0}_{local_name.lower()}"
            for value in sorted(values):
                atoms.add(f"{prefix}_is_{value}")
    return sorted(atoms)


def atom_holds(atom: str, model: DveModel, snapshot: Snapshot) -> bool:
    globals_state, proc_states, proc_locals = runtime_from_snapshot(snapshot)
    if "_is_" in atom:
        name, value = atom.rsplit("_is_", 1)
        target = int(value)
        if name in globals_state and isinstance(globals_state[name], int):
            return globals_state[name] == target
        array_match = re.match(r"^(.*)_(\d+)$", name)
        if array_match:
            array_name = array_match.group(1)
            index = int(array_match.group(2))
            if array_name in globals_state and isinstance(globals_state[array_name], list):
                return globals_state[array_name][index] == target
        local_match = re.match(r"^(.*?)(\d+)_(.+)$", name)
        if local_match:
            base = local_match.group(1)
            wanted_idx = int(local_match.group(2))
            local_name = local_match.group(3)
            for proc_idx, proc in enumerate(model.processes):
                group, idx = process_group(proc.name)
                if group == base and idx == wanted_idx:
                    local_value = proc_locals[proc_idx].get(local_name)
                    return isinstance(local_value, int) and local_value == target
    if atom.startswith("some_"):
        _, rest = atom.split("some_", 1)
        base, state = rest.rsplit("_", 1)
        for proc_idx, proc in enumerate(model.processes):
            group, _ = process_group(proc.name)
            if group == base and proc_states[proc_idx].lower() == state:
                return True
        return False
    match = re.match(r"^(.*?)(\d+)_(.+)$", atom)
    if not match:
        return False
    base = match.group(1)
    wanted_idx = int(match.group(2))
    wanted_state = match.group(3)
    for proc_idx, proc in enumerate(model.processes):
        group, idx = process_group(proc.name)
        if group == base and idx == wanted_idx:
            return proc_states[proc_idx].lower() == wanted_state
    return False


class Formula:
    def render(self) -> str:
        raise NotImplementedError

    def eval(self, graph: Graph, model: DveModel, state_idx: int) -> bool:
        raise NotImplementedError

    def size(self) -> int:
        raise NotImplementedError

    def modal_depth(self) -> int:
        raise NotImplementedError

    def atoms(self) -> set[str]:
        raise NotImplementedError


@dataclasses.dataclass(frozen=True)
class Atom(Formula):
    name: str

    def render(self) -> str:
        return self.name

    def eval(self, graph: Graph, model: DveModel, state_idx: int) -> bool:
        return atom_holds(self.name, model, graph.states[state_idx])

    def size(self) -> int:
        return 1

    def modal_depth(self) -> int:
        return 0

    def atoms(self) -> set[str]:
        return {self.name}


@dataclasses.dataclass(frozen=True)
class Not(Formula):
    inner: Formula

    def render(self) -> str:
        return f"(modal.not {self.inner.render()})"

    def eval(self, graph: Graph, model: DveModel, state_idx: int) -> bool:
        return not self.inner.eval(graph, model, state_idx)

    def size(self) -> int:
        return 1 + self.inner.size()

    def modal_depth(self) -> int:
        return self.inner.modal_depth()

    def atoms(self) -> set[str]:
        return self.inner.atoms()


@dataclasses.dataclass(frozen=True)
class Box(Formula):
    inner: Formula

    def render(self) -> str:
        return f"(modal.box {self.inner.render()})"

    def eval(self, graph: Graph, model: DveModel, state_idx: int) -> bool:
        successors = graph.edges.get(state_idx, set())
        return all(self.inner.eval(graph, model, succ) for succ in successors)

    def size(self) -> int:
        return 1 + self.inner.size()

    def modal_depth(self) -> int:
        return 1 + self.inner.modal_depth()

    def atoms(self) -> set[str]:
        return self.inner.atoms()


@dataclasses.dataclass(frozen=True)
class Dia(Formula):
    inner: Formula

    def render(self) -> str:
        return f"(modal.dia {self.inner.render()})"

    def eval(self, graph: Graph, model: DveModel, state_idx: int) -> bool:
        successors = graph.edges.get(state_idx, set())
        return any(self.inner.eval(graph, model, succ) for succ in successors)

    def size(self) -> int:
        return 1 + self.inner.size()

    def modal_depth(self) -> int:
        return 1 + self.inner.modal_depth()

    def atoms(self) -> set[str]:
        return self.inner.atoms()


@dataclasses.dataclass(frozen=True)
class BinOp(Formula):
    op: str
    lhs: Formula
    rhs: Formula

    def render(self) -> str:
        return f"({self.op} {self.lhs.render()} {self.rhs.render()})"

    def eval(self, graph: Graph, model: DveModel, state_idx: int) -> bool:
        if self.op == "modal.and":
            return self.lhs.eval(graph, model, state_idx) and self.rhs.eval(graph, model, state_idx)
        if self.op == "modal.or":
            return self.lhs.eval(graph, model, state_idx) or self.rhs.eval(graph, model, state_idx)
        raise ValueError(self.op)

    def size(self) -> int:
        return 1 + self.lhs.size() + self.rhs.size()

    def modal_depth(self) -> int:
        return max(self.lhs.modal_depth(), self.rhs.modal_depth())

    def atoms(self) -> set[str]:
        return self.lhs.atoms() | self.rhs.atoms()


@dataclasses.dataclass
class InstanceData:
    family: str
    instance_id: str
    graph: Graph
    model: DveModel
    labels: dict[str, str]

    @property
    def model_name(self) -> str:
        return f"{self.family}_{self.instance_id}"


def project_graph(graph: Graph, radius: int) -> Graph:
    keep: dict[int, int] = {graph.initial: 0}
    queue = collections.deque([(graph.initial, 0)])
    while queue:
        src, depth = queue.popleft()
        if depth >= radius:
            continue
        for dst in sorted(graph.edges.get(src, set())):
            if dst not in keep:
                keep[dst] = len(keep)
                queue.append((dst, depth + 1))
    inverse = {new: old for old, new in keep.items()}
    states = [graph.states[inverse[idx]] for idx in range(len(inverse))]
    edges: dict[int, set[int]] = collections.defaultdict(set)
    for old_src, new_src in keep.items():
        for old_dst in graph.edges.get(old_src, set()):
            new_dst = keep.get(old_dst)
            if new_dst is not None:
                edges[new_src].add(new_dst)
    return Graph(states=states, edges=edges, initial=0)


def formula_signature(formula: Formula, instances: list[InstanceData]) -> tuple[bool, ...]:
    return tuple(formula.eval(instance.graph, instance.model, instance.graph.initial) for instance in instances)


def enumerate_formulas(
    instances: list[InstanceData], atoms: list[str], max_size: int
) -> dict[tuple[bool, ...], Formula]:
    by_size: dict[int, list[Formula]] = collections.defaultdict(list)
    seen: dict[tuple[bool, ...], Formula] = {}
    for atom in atoms:
        formula = Atom(atom)
        signature = formula_signature(formula, instances)
        if signature not in seen:
            seen[signature] = formula
            by_size[1].append(formula)
    for size in range(2, max_size + 1):
        candidates: list[Formula] = []
        for inner in itertools.chain.from_iterable(by_size[s] for s in range(1, size)):
            if inner.size() + 1 != size:
                continue
            candidates.extend([Not(inner), Box(inner), Dia(inner)])
        for left_size in range(1, size):
            right_size = size - 1 - left_size
            if right_size < 1:
                continue
            for lhs in by_size[left_size]:
                for rhs in by_size[right_size]:
                    if lhs.render() > rhs.render():
                        continue
                    candidates.append(BinOp("modal.and", lhs, rhs))
                    candidates.append(BinOp("modal.or", lhs, rhs))
        for formula in candidates:
            signature = formula_signature(formula, instances)
            if signature in seen:
                continue
            seen[signature] = formula
            by_size[size].append(formula)
    return seen


def search_formula(
    instances: list[InstanceData], atoms: list[str], target: tuple[bool, ...], max_size: int
) -> Formula | None:
    return enumerate_formulas(instances, atoms, max_size).get(target)


def choose_synthetic_formula(
    instances: list[InstanceData], atoms: list[str], max_size: int
) -> tuple[Formula, tuple[bool, ...]] | None:
    candidates = enumerate_formulas(instances, atoms, max_size)
    best: tuple[Formula, tuple[bool, ...]] | None = None
    best_score = (-1, -1, 0)
    total = len(instances)
    for signature, formula in candidates.items():
        positives = sum(signature)
        negatives = total - positives
        if positives == 0 or negatives == 0:
            continue
        score = (min(positives, negatives), total, -formula.size())
        if score > best_score:
            best = (formula, signature)
            best_score = score
    return best


def load_results(path: Path) -> tuple[dict[str, dict[str, str]], dict[str, str]]:
    root = ET.parse(path).getroot()
    labels: dict[str, dict[str, str]] = {}
    property_desc: dict[str, str] = {}
    for instance in root.findall("instance"):
        instance_id = instance.attrib["id"]
        labels[instance_id] = {}
        for item in instance:
            prop_id = item.attrib.get("prop-id")
            answer = item.attrib.get("answer")
            if prop_id and answer:
                labels[instance_id][prop_id] = answer
    return labels, property_desc


def load_property_metadata(path: Path) -> dict[str, str]:
    root = ET.parse(path).getroot()
    metadata: dict[str, str] = {}
    for item in root.findall("property"):
        prop_id = item.attrib["id"]
        formula = normalize_space((item.findtext("formula") or "").strip())
        description = normalize_space((item.findtext("description") or "").strip())
        metadata[prop_id] = description or formula or f"property {prop_id}"
    return metadata


def collect_supported_instances(family_dir: Path, max_states: int, max_instance_id: int) -> list[InstanceData]:
    family = family_dir.name
    results_path = family_dir / "generated_files" / "results.xml"
    xml_path = family_dir / f"{family}.xml"
    labels_by_instance, _ = load_results(results_path)
    property_metadata = load_property_metadata(xml_path)
    instances: list[InstanceData] = []
    for instance_id in sorted(labels_by_instance, key=lambda item: int(item)):
        if int(instance_id) > max_instance_id:
            continue
        dve_path = family_dir / "generated_files" / f"{family}.{instance_id}.dve"
        if not dve_path.exists():
            continue
        try:
            model = parse_dve(dve_path)
            graph = build_graph(model, max_states=max_states)
        except UnsupportedDve:
            continue
        labels = {
            prop_id: answer
            for prop_id, answer in labels_by_instance[instance_id].items()
            if answer in {"yes", "no"} and prop_id in property_metadata
        }
        if labels:
            instances.append(
                InstanceData(
                    family=family,
                    instance_id=instance_id,
                    graph=graph,
                    model=model,
                    labels=labels,
                )
            )
    return instances


def choose_property(instances: list[InstanceData], min_examples: int) -> tuple[str, tuple[bool, ...]] | None:
    prop_ids = sorted({prop_id for instance in instances for prop_id in instance.labels})
    best: tuple[str, tuple[bool, ...]] | None = None
    best_score = (-1, -1)
    for prop_id in prop_ids:
        labeled = [instance for instance in instances if prop_id in instance.labels]
        if len(labeled) < min_examples:
            continue
        target = tuple(instance.labels[prop_id] == "yes" for instance in labeled)
        positives = sum(target)
        negatives = len(target) - positives
        if positives == 0 or negatives == 0:
            continue
        score = (min(positives, negatives), len(target))
        if score > best_score:
            best = (prop_id, target)
            best_score = score
    return best


def render_delis(
    family: str,
    instances: list[InstanceData],
    label_name: str,
    target_formula: Formula,
    property_description: str,
    atoms: list[str],
    labels: tuple[bool, ...],
) -> str:
    atom_decls = "\n".join(f"(declare-fun {atom} (U) Bool)" for atom in atoms)
    constructors = " ".join(f"({atom})" for atom in atoms)
    models_text = []
    constraints = []
    for instance, label in zip(instances, labels):
        node_names = [f"${idx}" for idx in range(len(instance.graph.states))]
        models_text.append(f"(define-model {instance.model_name}")
        models_text.append(f"    (define-univ U ({' '.join(node_names)}))")
        models_text.append("    (interp-fun R (match (")
        for src in range(len(instance.graph.states)):
            for dst in sorted(instance.graph.edges.get(src, set())):
                models_text.append(f"        ((${src} ${dst}) true)")
        models_text.append("        ((_ _) false)")
        models_text.append("    )))")
        for atom in atoms:
            models_text.append(f"    (interp-fun {atom} (match (")
            for idx, snapshot in enumerate(instance.graph.states):
                if atom_holds(atom, instance.model, snapshot):
                    models_text.append("        (($" + str(idx) + ") true)")
            models_text.append("        ((_) false)")
            models_text.append("    )))")
        models_text.append("    (interp-fun I ($0))")
        models_text.append(")")
        body = f"(Modal.Sem phi (ModalAsp {instance.model_name} $0))"
        constraints.append(f"(constraint {body})" if label else f"(constraint (not {body}))")
    rendered_models = "\n".join(models_text)
    rendered_constraints = "\n".join(constraints)
    semantics_atoms = "\n".join(f"     (({atom}) ({atom} (node asp)))" for atom in atoms)
    return (
        f";; Generated from BEEM family `{family}`, label source {label_name}: {property_description}\n"
        f";; Found modal-K separator: {target_formula.render()}\n\n"
        "(set-logic ALL)\n"
        "(declare-sort U 0)\n"
        "(declare-fun I () U)\n"
        "(declare-fun R (U U) Bool)\n"
        f"{atom_decls}\n\n"
        f"{rendered_models}\n\n"
        "(declare-term-types\n"
        " ((Modal 0))\n"
        " (((modal.not Modal)\n"
        "   (modal.and Modal Modal)\n"
        "   (modal.or Modal Modal)\n"
        "   (modal.box Modal)\n"
        "   (modal.dia Modal)\n"
        f"   {constructors})))\n\n"
        "(define-finite ModalAsp\n"
        "  (record (model Model) (node U))\n"
        "  :where true)\n\n"
        "(define-semantics\n"
        "  ((Modal.Sem (phi Modal) (asp ModalAsp)))\n"
        "  ((match phi\n"
        f"{semantics_atoms}\n"
        "     ((modal.not x) (not (Modal.Sem x asp)))\n"
        "     ((modal.and x y) (and (Modal.Sem x asp) (Modal.Sem y asp)))\n"
        "     ((modal.or x y) (or (Modal.Sem x asp) (Modal.Sem y asp)))\n"
        "     ((modal.box x)\n"
        "      (forall ((next (domain (U (model asp)))))\n"
        "        (or (not (R (node asp) next))\n"
        "            (Modal.Sem x (ModalAsp (model asp) next)))))\n"
        "     ((modal.dia x)\n"
        "      (exists ((next (domain (U (model asp)))))\n"
        "        (and (R (node asp) next)\n"
        "             (Modal.Sem x (ModalAsp (model asp) next))))))))\n\n"
        "(synth-fun phi () Modal)\n"
        f"{rendered_constraints}\n\n"
        "(check-synth)\n"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Convert BEEM DVE instances into Delis modal-K benchmarks.")
    parser.add_argument("--beem-root", type=Path, required=True)
    parser.add_argument("--family", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--max-states", type=int, default=2000)
    parser.add_argument("--max-instance-id", type=int, default=5)
    parser.add_argument("--min-examples", type=int, default=3)
    parser.add_argument("--max-formula-size", type=int, default=4)
    parser.add_argument(
        "--atom-scope",
        choices=["shared", "formula"],
        default="formula",
        help="Keep either all shared atoms across instances or only the atoms used by the separator.",
    )
    parser.add_argument(
        "--project-radius",
        choices=["formula", "full"],
        default="formula",
        help="Export the full reachable graph or only the initial neighborhood needed by the separator depth.",
    )
    parser.add_argument(
        "--label-mode",
        choices=["auto", "beem", "synthetic"],
        default="auto",
        help="Use a BEEM property if separable, otherwise fall back to a synthetic modal-K split.",
    )
    args = parser.parse_args()

    family_dir = args.beem_root / "original-benchmark" / args.family
    if not family_dir.exists():
        print(f"family not found: {family_dir}", file=sys.stderr)
        return 1

    instances = collect_supported_instances(
        family_dir, max_states=args.max_states, max_instance_id=args.max_instance_id
    )
    if not instances:
        print("no supported instances found", file=sys.stderr)
        return 1

    property_choice = choose_property(instances, min_examples=args.min_examples)
    label_name: str
    label_description: str
    labels: tuple[bool, ...]
    formula: Formula | None = None
    if property_choice is not None and args.label_mode in {"auto", "beem"}:
        target_prop, _ = property_choice
        labeled_instances = [instance for instance in instances if target_prop in instance.labels]
    else:
        target_prop = ""
        labeled_instances = instances
    property_metadata = load_property_metadata(family_dir / f"{args.family}.xml")
    atoms = sorted(
        set.intersection(*(set(derived_atoms(instance.model, instance.graph)) for instance in labeled_instances))
    )
    if not atoms:
        print("no shared atoms derived across instances", file=sys.stderr)
        return 1
    if target_prop:
        labels = tuple(instance.labels[target_prop] == "yes" for instance in labeled_instances)
        formula = search_formula(labeled_instances, atoms, labels, max_size=args.max_formula_size)
        if formula is not None:
            label_name = f"BEEM property {target_prop}"
            label_description = property_metadata.get(target_prop, f"property {target_prop}")
            instances = labeled_instances
    if formula is None and args.label_mode in {"auto", "synthetic"}:
        synthetic = choose_synthetic_formula(labeled_instances, atoms, max_size=args.max_formula_size)
        if synthetic is not None:
            formula, labels = synthetic
            label_name = "synthetic modal-K split"
            label_description = "balanced separator mined from the shared modal atom vocabulary"
            instances = labeled_instances
    if formula is None:
        print("no usable modal-K labeling found within the search bound", file=sys.stderr)
        return 1

    if args.atom_scope == "formula":
        atoms = sorted(formula.atoms())

    if args.project_radius == "formula":
        radius = formula.modal_depth()
        instances = [
            dataclasses.replace(instance, graph=project_graph(instance.graph, radius))
            for instance in instances
        ]

    output = render_delis(
        family=args.family,
        instances=instances,
        label_name=label_name,
        target_formula=formula,
        property_description=label_description,
        atoms=atoms,
        labels=labels,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(output)
    positives = sum(labels)
    negatives = len(labels) - positives
    print(
        f"generated {args.output} using {label_name} with formula {formula.render()} "
        f"({positives} positive / {negatives} negative)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
