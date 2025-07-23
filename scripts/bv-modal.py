import json
import os
import argparse
from typing import List, Dict, Tuple, Any, Callable


class KripkeStructure:
    """
    Represents a Kripke Structure loaded from a JSON file.
    This is the modal logic equivalent of the Model class in bv-fok.py.
    """

    def __init__(self, filepath: str):
        with open(filepath, "r") as f:
            data = json.load(f)

        self.filepath = filepath
        self.name = os.path.basename(filepath)

        self.domain = data.get("domain", [])
        if not self.domain or self.domain != list(range(len(self.domain))):
            raise ValueError(
                f"Domain in {self.name} must be a contiguous range from 0 to N-1."
            )
        self.domain_size = len(self.domain)

        self.propositions = set(data.get("propositions", []))
        self.transitions = {int(k): v for k, v in data.get(
            "transitions", {}).items()}
        self.labels = {int(k): set(v)
                       for k, v in data.get("labels", {}).items()}

    def get_neighbors(self, node: int) -> List[int]:
        """Get the successor nodes for a given node."""
        return self.transitions.get(node, [])

    def check_proposition_at_node(self, prop: str, node: int) -> bool:
        """Check if a proposition is true at a given node."""
        return prop in self.labels.get(node, set())


def generate_concatenated_bitvector(
    models: List[KripkeStructure], eval_func: Callable[[int, KripkeStructure], bool]
) -> Tuple[str, int, List[Dict[str, Any]]]:
    """
    Generates a single bitvector by concatenating the bitvectors from multiple models.
    This version is CORRECTED to produce LSB-first bitvectors matching SMT `extract` indexing.
    """
    full_bitstring_list = []
    current_total_bits = 0
    segment_info_list = []

    # The order of iteration is correct: Model 1, then Model 2, etc.
    # Within each model: Node 0, then Node 1, etc.
    for model in models:
        segment_len = model.domain_size
        model_segment_bits = []
        for node in model.domain:
            if eval_func(node, model):
                model_segment_bits.append("1")
            else:
                model_segment_bits.append("0")
        full_bitstring_list.extend(model_segment_bits)

        # The offset calculation for constraints is also correct based on this iteration order.
        start_bit = current_total_bits
        end_bit = current_total_bits + segment_len - 1
        segment_info_list.append(
            {
                "model_name": model.name,
                "smt_extract_high": end_bit,
                "smt_extract_low": start_bit,
                "domain_size": segment_len,
            }
        )
        current_total_bits += segment_len

    # CRUCIAL FIX: Reverse the list before joining.
    # This makes list index 0 (e.g., model1_node0) become the LSB (SMT bit index 0).
    return (
        "#b" + "".join(reversed(full_bitstring_list)),
        current_total_bits,
        segment_info_list,
    )


def generate_modal_operator_functions(
    models: List[KripkeStructure], total_bv_length: int
) -> List[str]:
    """
    Generates the SMT-LIB functions for modal operators 'box' (□) and 'diamond' (◇).
    This version is CORRECTED to consistently use LSB-first logic for all concatenations.
    """
    output = []

    let_bindings_str_list = []
    current_offset = 0
    for i, model in enumerate(models):
        high = current_offset + model.domain_size - 1
        low = current_offset
        let_bindings_str_list.append(
            f"(seg{i} ((_ extract {high} {low}) phi_val))")
        current_offset += model.domain_size

    # --- Unified Helper to build a bitvector from an LSB-first list ---
    # The list items [item0, item1, ...] correspond to indices [0, 1, ...].
    # This function builds (concat item_N-1 ... (concat item_1 item_0)).
    def build_bv_from_lsb_first_list(items: List[str]) -> str:
        if not items:
            return None
        expr = items[0]
        for item in items[1:]:
            expr = f"(concat {item} {expr})"
        return expr

    # --- Generate Box (□) Function ---
    output.append(
        f"(define-fun modal.box ((phi_val (_ BitVec {total_bv_length}))) (_ BitVec {total_bv_length})"
    )
    output.append("  (let (")
    output.extend([f"    {binding}" for binding in let_bindings_str_list])
    output.append("  ) ; end bindings")

    model_segments = []
    for i, model in enumerate(models):
        new_segment_bits = []
        for node in model.domain:
            neighbors = model.get_neighbors(node)
            if not neighbors:
                new_segment_bits.append("#b1")
            else:
                neighbor_checks = [
                    f"(b1_to_bool ((_ extract {n} {n}) seg{i}))" for n in neighbors
                ]
                new_segment_bits.append(
                    f"(bool_to_bv1 (and {' '.join(neighbor_checks)}))"
                )

        if not new_segment_bits:
            model_segments.append(f"(_ bv0 {model.domain_size})")
        else:
            model_segments.append(
                build_bv_from_lsb_first_list(new_segment_bits))

    output.append(
        f"    {build_bv_from_lsb_first_list(model_segments)} ; body of let")
    output.append("  ) ; end let")
    output.append(")")

    # --- Generate Diamond (◇) Function ---
    output.append(
        f"(define-fun modal.dia ((phi_val (_ BitVec {total_bv_length}))) (_ BitVec {total_bv_length})"
    )
    output.append("  (let (")
    output.extend([f"    {binding}" for binding in let_bindings_str_list])
    output.append("  ) ; end bindings")

    model_segments = []
    for i, model in enumerate(models):
        new_segment_bits = []
        for node in model.domain:
            neighbors = model.get_neighbors(node)
            if not neighbors:
                new_segment_bits.append("#b0")
            else:
                neighbor_checks = [
                    f"(b1_to_bool ((_ extract {n} {n}) seg{i}))" for n in neighbors
                ]
                new_segment_bits.append(
                    f"(bool_to_bv1 (or {' '.join(neighbor_checks)}))"
                )

        if not new_segment_bits:
            model_segments.append(f"(_ bv0 {model.domain_size})")
        else:
            model_segments.append(
                build_bv_from_lsb_first_list(new_segment_bits))

    output.append(
        f"    {build_bv_from_lsb_first_list(model_segments)} ; body of let")
    output.append("  ) ; end let")
    output.append(")")

    return output


def generate_sygus_script(
    true_model_paths: List[str], false_model_paths: List[str]
) -> str:
    """Generates the full SyGuS-SMT script for modal logic formula synthesis."""
    output = []
    cmt = "; "

    output.append("(set-logic BV)")
    output.append("")

    # Load models
    all_model_paths = true_model_paths + false_model_paths
    if not all_model_paths:
        return f"{cmt}Error: No models provided."

    loaded_models = [KripkeStructure(p) for p in all_model_paths]
    all_propositions = sorted(
        list(set.union(*(m.propositions for m in loaded_models))))
    num_true_models = len(true_model_paths)

    output.append(f"{cmt}--- Helper Functions ---")
    output.append(
        "(define-fun bool_to_bv1 ((b Bool)) (_ BitVec 1) (ite b #b1 #b0))")
    # A version for SMTLib that doesn't natively support bool-to-bv conversion on bitvectors
    output.append("(define-fun b1_to_bool ((b (_ BitVec 1))) Bool (= b #b1))")
    output.append("\n")

    # Generate atomic propositions as bitvectors
    output.append(f"{cmt}--- Atomic Propositions ---")
    final_bv_length = 0
    grammar_terms = []

    for prop in all_propositions:
        def evaluator(
            node, model, p=prop): return model.check_proposition_at_node(p, node)
        bv_str, length, _ = generate_concatenated_bitvector(
            loaded_models, evaluator)

        if final_bv_length == 0:
            final_bv_length = length
            output.append(
                f"{cmt}Concatenated bitvector length: {final_bv_length}")
        elif final_bv_length != length:
            raise RuntimeError(
                "Bitvector length mismatch between propositions!")

        output.append(
            f"(define-fun {prop} () (_ BitVec {final_bv_length}) {bv_str})")
        grammar_terms.append(prop)

    # Generate bitwise logical operations
    output.extend(
        [
            f"\n{cmt}--- Logical Operations ---",
            f"(define-fun modal.and ((x (_ BitVec {final_bv_length})) (y (_ BitVec {final_bv_length}))) (_ BitVec {final_bv_length}) (bvand x y))",
            f"(define-fun modal.or ((x (_ BitVec {final_bv_length})) (y (_ BitVec {final_bv_length}))) (_ BitVec {final_bv_length}) (bvor x y))",
            f"(define-fun modal.not ((x (_ BitVec {final_bv_length}))) (_ BitVec {final_bv_length}) (bvnot x))",
            "",
        ]
    )

    # Generate modal operator functions
    output.append(f"{cmt}--- Modal Operators ---")
    modal_op_lines = generate_modal_operator_functions(
        loaded_models, final_bv_length)
    output.extend(modal_op_lines)

    # Generate synthesis task
    grammar_terms_str = "\n      ".join(grammar_terms)
    output.extend(
        [
            f"\n{cmt}--- Synthesis Task ---",
            f"(synth-fun separator () (_ BitVec {final_bv_length})",
            f"  ((Start (_ BitVec {final_bv_length})))",
            f"  ((Start (_ BitVec {final_bv_length})",
            "    ( (modal.and Start Start)",
            "      (modal.or Start Start)",
            "      (modal.not Start)",
            "      (modal.box Start)",
            "      (modal.dia Start)",
            f"      {grammar_terms_str}",
            "    )",
            "  ))",
            ")",
            "",
        ]
    )

    # Generate constraints
    current_offset = 0
    for i, model in enumerate(loaded_models):
        # A formula is TRUE for a model if it holds at the initial state (node 0).
        is_true_model = i < num_true_models
        target_bit = "1" if is_true_model else "0"

        # The bit we care about is the one for node 0 of this model's segment
        bit_to_check_index = current_offset
        output.append(
            f"(constraint (= ((_ extract {bit_to_check_index} {bit_to_check_index}) separator) #b{target_bit})) {cmt}Model {model.name}"
        )
        current_offset += model.domain_size

    output.append("\n(check-synth)")
    return "\n".join(output)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Generate SyGuS SMT-LIB for modal logic formulas from Kripke structure JSON files."
    )
    parser.add_argument(
        "-t",
        "--true-models",
        nargs="+",
        required=True,
        metavar="FILE",
        help="Path(s) to JSON Kripke models where formula should be TRUE.",
    )
    parser.add_argument(
        "-f",
        "--false-models",
        nargs="+",
        required=False,
        default=[],
        metavar="FILE",
        help="Path(s) to JSON Kripke models where formula should be FALSE.",
    )
    args = parser.parse_args()

    if not args.true_models:
        print("Error: At least one true model (-t) must be provided.")
        exit(1)

    smt_script_content = generate_sygus_script(
        args.true_models,
        args.false_models,
    )

    print(smt_script_content)