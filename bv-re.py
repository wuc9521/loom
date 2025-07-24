# File: bv-re.py
import os
import argparse
from typing import List, Tuple, Callable


class WordStructure:
    """
    Represents a word and its subword structure.
    This is the regular expression equivalent of KripkeStructure or Model.
    """

    def __init__(self, word: str, global_offset: int):
        self.word = word
        self.name = f"word('{word}')"
        self.length = len(word)
        self.alphabet = set(list(word))

        # Subwords are identified by (start, end) indices of a slice w[start:end]
        self.subwords: List[Tuple[int, int]] = []
        for start in range(self.length + 1):
            for end in range(start, self.length + 1):
                self.subwords.append((start, end))

        self.subword_to_local_idx = {
            subword: i for i, subword in enumerate(self.subwords)
        }
        self.num_subwords = len(self.subwords)

        self.global_offset = global_offset
        self.smt_extract_high = global_offset + self.num_subwords - 1
        self.smt_extract_low = global_offset

    def get_local_idx(self, start: int, end: int) -> int:
        return self.subword_to_local_idx.get((start, end))

    def __repr__(self):
        return f"<WordStructure word='{self.word}' subwords={self.num_subwords}>"


def generate_concatenated_bitvector(
    word_models: List[WordStructure],
    eval_func: Callable[[Tuple[int, int], WordStructure], bool],
) -> Tuple[str, int]:
    """
    Generates a single bitvector by concatenating results for all subwords of all words.
    """
    full_bitstring_list = []
    total_bits = 0

    for model in word_models:
        for subword in model.subwords:
            if eval_func(subword, model):
                full_bitstring_list.append("1")
            else:
                full_bitstring_list.append("0")
        total_bits += model.num_subwords

    # SMT-LIB bitvectors are indexed with LSB at index 0.
    # Our list is naturally constructed with index 0 being the first element.
    # Reversing makes our list index 0 the LSB.
    return "#b" + "".join(reversed(full_bitstring_list)), total_bits


# File: bv-re.py (opravená funkce)


def generate_operator_functions(
    word_models: List[WordStructure], total_bv_length: int
) -> List[str]:
    """
    Generates the SMT-LIB functions for regular expression operators.
    This is complex because concat and star have cross-bit dependencies.
    We must build the result bit by bit using `let` bindings.
    """
    output = []

    def build_bv_from_lsb_first_list(items: List[str]) -> str:
        if not items:
            return None
        expr = items[0]
        for item in items[1:]:
            expr = f"(concat {item} {expr})"
        return expr

    output.append(
        f"(define-fun re-concat ((phi1 (_ BitVec {total_bv_length})) (phi2 (_ BitVec {total_bv_length}))) (_ BitVec {total_bv_length})"
    )
    concat_bits = []
    for model in word_models:
        for start, end in model.subwords:
            split_checks = []
            for k in range(start, end + 1):
                idx1 = model.global_offset + model.get_local_idx(start, k)
                idx2 = model.global_offset + model.get_local_idx(k, end)
                check1 = f"(= #b1 ((_ extract {idx1} {idx1}) phi1))"
                check2 = f"(= #b1 ((_ extract {idx2} {idx2}) phi2))"
                split_checks.append(f"(and {check1} {check2})")

            if not split_checks:
                concat_bits.append("#b0")
            else:
                concat_bits.append(f"(bool_to_bv1 (or {' '.join(split_checks)}))")

    output.append(f"  {build_bv_from_lsb_first_list(concat_bits)}")
    output.append(")")
    output.append("")
    output.append(
        f"(define-fun re-star ((phi (_ BitVec {total_bv_length}))) (_ BitVec {total_bv_length})"
    )
    indent = "  "
    empty_str_bits = []
    for model in word_models:
        for start, end in model.subwords:
            if start == end:
                empty_str_bits.append("#b1")
            else:
                empty_str_bits.append("#b0")

    output.append(
        f"{indent}(let ((star_iter_0 {build_bv_from_lsb_first_list(empty_str_bits)}))"
    )
    indent += "  "
    max_len = max(m.length for m in word_models) if word_models else 0
    for i in range(max_len):
        prev_iter_name = f"star_iter_{i}"
        current_iter_name = f"star_iter_{i + 1}"

        one_step_bits = []
        for model in word_models:
            for start, end in model.subwords:
                split_checks = []
                for k in range(start + 1, end + 1):
                    idx_phi = model.global_offset + model.get_local_idx(start, k)
                    idx_star = model.global_offset + model.get_local_idx(k, end)
                    check_phi = f"(= #b1 ((_ extract {idx_phi} {idx_phi}) phi))"
                    check_star = (
                        f"(= #b1 ((_ extract {idx_star} {idx_star}) {prev_iter_name}))"
                    )
                    split_checks.append(f"(and {check_phi} {check_star})")

                if not split_checks:
                    one_step_bits.append("#b0")
                else:
                    one_step_bits.append(f"(bool_to_bv1 (or {' '.join(split_checks)}))")

        one_step_bv = build_bv_from_lsb_first_list(one_step_bits)

        output.append(
            f"{indent}(let (({current_iter_name} (bvor {prev_iter_name} {one_step_bv})))"
        )
        indent += "  "
    output.append(f"{indent}star_iter_{max_len}")
    for _ in range(max_len + 1):
        indent = indent[:-2]
        output.append(f"{indent})")

    output.append(")")
    output.append("")

    return output


def generate_sygus_script(true_words: List[str], false_words: List[str]) -> str:
    """Generates the full SyGuS-SMT script for regular expression synthesis."""
    output = []
    cmt = "; "

    output.append("(set-logic BV)")
    output.append("")

    all_words = true_words + false_words
    if not all_words:
        return f"{cmt}Error: No words provided."

    # alphabet is the union of all characters that have appeared in the input.
    alphabet = sorted(list(set.union(*(set(w) for w in all_words))))

    # Create word models and calculate total bitvector length
    word_models: List[WordStructure] = []
    current_offset = 0
    for word in all_words:
        model = WordStructure(word, current_offset)
        word_models.append(model)
        current_offset += model.num_subwords
    total_bv_length = current_offset

    output.append(f"{cmt}--- Word Structure & Bitvector Info ---")
    for i, model in enumerate(word_models):
        label = "TRUE" if i < len(true_words) else "FALSE"
        output.append(
            f"{cmt}Model {i + 1} ({label}): word='{model.word}', len={model.length}, subwords={model.num_subwords}, bv_indices={model.smt_extract_low}-{model.smt_extract_high}"
        )
    output.append(f"{cmt}Total concatenated bitvector length: {total_bv_length}")
    output.append("")

    output.append(f"{cmt}--- Helper Functions ---")
    output.append("(define-fun bool_to_bv1 ((b Bool)) (_ BitVec 1) (ite b #b1 #b0))")
    output.append("")

    # Generate atomic propositions (single characters)
    output.append(f"{cmt}--- Atomic Expressions (Alphabet) ---")
    grammar_terms = []
    for char in alphabet:
        # An evaluator that is true iff the subword is exactly this character
        def evaluator(subword, model, c=char) -> bool:
            return model.word[subword[0] : subword[1]] == c

        bv_str, length = generate_concatenated_bitvector(word_models, evaluator)
        if total_bv_length != length:
            raise RuntimeError("Bitvector length mismatch!")
        output.append(
            f"(define-fun {char} () (_ BitVec {total_bv_length}) {bv_str}) {cmt} for character '{char}'"
        )
        grammar_terms.append(char)
    output.append("")

    # Generate bitwise logical operations (for union, intersection)
    output.extend(
        [
            f"{cmt}--- Logical Operations (Union, Intersection, etc.) ---",
            f"(define-fun re-union ((x (_ BitVec {total_bv_length})) (y (_ BitVec {total_bv_length}))) (_ BitVec {total_bv_length}) (bvor x y))",
            f"(define-fun re-neg ((x (_ BitVec {total_bv_length}))) (_ BitVec {total_bv_length}) (bvnot x))",
            "",
        ]
    )

    # Generate complex operator functions (concat, star)
    output.append(f"{cmt}--- Structural Operations (Concat, Star) ---")
    op_lines = generate_operator_functions(word_models, total_bv_length)
    output.extend(op_lines)

    # Generate synthesis task
    grammar_terms_str = "\n      ".join(grammar_terms)
    output.extend(
        [
            f"\n{cmt}--- Synthesis Task ---",
            f"(synth-fun separator () (_ BitVec {total_bv_length})",
            f"  ((Start (_ BitVec {total_bv_length})))",
            f"  ((Start (_ BitVec {total_bv_length})",
            "    ( (re-union Start Start)",
            "      (re-concat Start Start)",
            "      (re-star Start)",
            "      (re-neg Start)",
            f"      {grammar_terms_str}",
            "    )",
            "  ))",
            ")",
            "",
        ]
    )

    # Generate constraints
    output.append(f"{cmt}--- Constraints ---")
    for i, model in enumerate(word_models):
        is_true_example = i < len(true_words)
        target_bit = "1" if is_true_example else "0"

        # The bit we care about is the one for the *entire word*
        # The entire word w is the subword w[0:len(w)]
        full_word_subword = (0, model.length)
        bit_to_check_idx = model.global_offset + model.get_local_idx(
            full_word_subword[0], full_word_subword[1]
        )

        output.append(
            f"(constraint (= ((_ extract {bit_to_check_idx} {bit_to_check_idx}) separator) #b{target_bit})) {cmt}Model {model.name}"
        )

    output.append("\n(check-synth)")
    return "\n".join(output)


def read_words_from_file(filepath: str) -> List[str]:
    """Reads one word per line from a file."""
    if not os.path.exists(filepath):
        print(f"Warning: File not found: {filepath}")
        return []
    with open(filepath, "r") as f:
        # Filter out empty lines
        return [line.strip() for line in f if line.strip()]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Generate SyGuS SMT-LIB for regular expression synthesis."
    )
    parser.add_argument(
        "-t",
        "--true-models",
        nargs="+",
        required=True,
        metavar="FILE",
        help="Path(s) to text files containing words the regex should match (one word per line).",
    )
    parser.add_argument(
        "-f",
        "--false-models",
        nargs="+",
        required=False,
        default=[],
        metavar="FILE",
        help="Path(s) to text files containing words the regex should NOT match.",
    )
    args = parser.parse_args()

    true_words = []
    for path in args.true_models:
        true_words.extend(read_words_from_file(path))

    false_words = []
    for path in args.false_models:
        false_words.extend(read_words_from_file(path))

    if not true_words:
        print(
            "Error: At least one positive example word must be provided and files must not be empty."
        )
        exit(1)

    # Check for empty word, which this formulation doesn't handle well
    if any(w == "" for w in true_words + false_words):
        print(
            "Warning: Empty strings are not fully supported in this formulation and may lead to unexpected behavior."
        )

    smt_script_content = generate_sygus_script(
        true_words,
        false_words,
    )

    print(smt_script_content)