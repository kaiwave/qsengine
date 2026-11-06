"""Text drawing for `print(qc)`: one wire per qubit (q0 on top), one column per operation.

    q0: ──H──●─
             │
    q1: ─────X─

`●` marks a control, `×` the two ends of a swap, `M→c0` a measurement into
classical bit 0, `|0⟩` a reset, and `X if c1=1` a gate inside an `if_test`.

shortcut: columns are never packed or folded, so a long circuit draws as one very wide line; add folding if learners hit it.
"""

# Method name -> symbol drawn on the target wire.
SYMBOLS = {
    "x": "X", "y": "Y", "z": "Z", "h": "H", "s": "S", "t": "T", "sdg": "S†", "tdg": "T†",
    "rx": "RX", "ry": "RY", "rz": "RZ", "p": "P", "u": "U", "unitary": "U",
    "cx": "X", "cz": "Z", "cp": "P", "crz": "RZ", "ccx": "X",
}


def _label(kind, name, params, condition, clbit):
    if kind == "measure":
        return f"M→c{clbit}"
    if kind == "reset":
        return "|0⟩"
    text = "×" if kind == "swap" else SYMBOLS[name]
    if params:
        text += "(" + ",".join(f"{a:.2f}" for a in params) + ")"
    if condition is not None:
        mask, value = condition
        bits = [b for b in range(mask.bit_length()) if mask >> b & 1]
        text += " if " + ",".join(f"c{b}={value >> b & 1}" for b in bits)
    return text


def draw(circuit) -> str:
    n = circuit.num_qubits
    name_w = len(f"q{max(n - 1, 0)}")
    # Even rows are qubit wires, odd rows the gaps between them.
    rows = [f"{f'q{r // 2}':>{name_w}}: ─" if r % 2 == 0 else " " * (name_w + 3) for r in range(max(2 * n - 1, 0))]

    for kind, name, params, controls, targets, condition, clbit in circuit._ops():
        label = _label(kind, name, params, condition, clbit)
        # A swap draws × on both qubits (its condition, if any, on the first); everything else its label on the target.
        marks = {targets[1]: "×", targets[0]: label} if kind == "swap" else {targets[0]: label}
        w = max(len(m) for m in marks.values()) + 2
        mid = w // 2
        qubits = controls + targets
        lo, hi = 2 * min(qubits), 2 * max(qubits)  # rows that carry the vertical line

        def at(ch, fill):
            return fill * mid + ch + fill * (w - mid - 1)

        for r in range(len(rows)):
            q, inside = r // 2, lo <= r <= hi
            if r % 2 == 0 and q in marks:
                cell = ("─" + marks[q]).ljust(w, "─")
            elif r % 2 == 0 and q in controls:
                cell = at("●", "─")
            elif r % 2 == 0:
                cell = at("┼", "─") if inside else "─" * w
            else:
                cell = at("│", " ") if inside else " " * w
            rows[r] += cell
    return "\n".join(row.rstrip() for row in rows)
